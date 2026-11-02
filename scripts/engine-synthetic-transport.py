"""Bounded synthetic-only transport; caller owns admission, durable intent and schema.

No retries or action application. Optional bounded local failure quarantine is
separate from receipts/model context. Unknown usage
or billing never implies a refund. invoke is foreground/main-thread only so its
wall deadline covers both connection and response reading.
"""
import decimal
import hashlib
import importlib.util
import json
import math
from pathlib import Path
import signal
import socket
import ssl
import uuid
import threading
import time
import urllib.error
import urllib.request

_capacity_spec=importlib.util.spec_from_file_location('request_capacity',Path(__file__).with_name('engine-request-capacity.py'))
capacity=importlib.util.module_from_spec(_capacity_spec);_capacity_spec.loader.exec_module(capacity)

MODEL = 'deepseek-v4.1-flash'
SHADOW_MODELS = frozenset((MODEL, 'glm-5.3-flash', 'qwen3.8-flash', 'deepseek-v4.1-flash-1017', 'gpt-6-luna'))
ENDPOINT = 'https://api.cheaperinference.com/v1/chat/completions'
RESPONSE_BOUND = 1024 * 1024
loader = importlib.util.spec_from_file_location('synthetic_broker', Path(__file__).with_name('m3-cheaper-broker.py'))
broker = importlib.util.module_from_spec(loader)
loader.loader.exec_module(broker)
evidence_loader = importlib.util.spec_from_file_location('response_evidence', Path(__file__).with_name('engine-response-evidence.py'))
evidence_module = importlib.util.module_from_spec(evidence_loader)
evidence_loader.loader.exec_module(evidence_module)
FailureEvidence = evidence_module.FailureEvidence


def strict(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ValueError('duplicate_key')
            result[key] = value
        return result
    def constant(_):
        raise ValueError('nonfinite')
    def finite(text):
        value = float(text)
        if not math.isfinite(value):
            raise ValueError('nonfinite')
        return value
    return json.loads(raw, object_pairs_hook=pairs, parse_constant=constant, parse_float=finite)


def validate(value, schema, _depth=0):
    """Fail-closed bounded subset used by synthetic response contracts."""
    if _depth > 32 or not isinstance(schema, dict):
        raise ValueError('schema_depth_or_shape')
    allowed = {'type','properties','required','additionalProperties','items','minItems','maxItems',
               'uniqueItems','minLength','maxLength','minimum','maximum','const','enum','anyOf'}
    if set(schema) - allowed:
        raise ValueError('unsupported_schema_keyword')
    if 'anyOf' in schema:
        options = schema['anyOf']
        if not isinstance(options, list) or not 1 <= len(options) <= 64:
            raise ValueError('schema_anyof')
        if not any(_matches(value, option, _depth + 1) for option in options):
            raise ValueError('schema_anyof')
    if 'const' in schema and (type(value) != type(schema['const']) or value != schema['const']):
        raise ValueError('schema_const')
    if 'enum' in schema and not any(type(value) == type(x) and value == x for x in schema['enum']):
        raise ValueError('schema_enum')
    kind = schema.get('type')
    if kind == 'object':
        if not isinstance(value, dict) or len(value) > 128 or not set(schema.get('required', [])) <= value.keys():
            raise ValueError('schema_object')
        properties = schema.get('properties', {})
        if schema.get('additionalProperties') is not False or set(value) - properties.keys():
            raise ValueError('schema_extra')
        for key, item in value.items():
            validate(item, properties[key], _depth + 1)
    elif kind == 'array':
        if not isinstance(value, list) or not schema.get('minItems',0) <= len(value) <= min(128,schema.get('maxItems',128)):
            raise ValueError('schema_array')
        if schema.get('uniqueItems') and len({json.dumps(x,sort_keys=True) for x in value}) != len(value):
            raise ValueError('schema_duplicate')
        for item in value:
            validate(item, schema['items'], _depth + 1)
    elif kind == 'string':
        if not isinstance(value, str) or not schema.get('minLength',0) <= len(value) <= schema.get('maxLength',512) or len(value.encode()) > 512:
            raise ValueError('schema_text')
    elif kind in ('number','integer'):
        if type(value) not in ((int,) if kind == 'integer' else (int,float)) or not schema.get('minimum',-1e308) <= value <= schema.get('maximum',1e308) or not math.isfinite(value):
            raise ValueError('schema_number')
    elif kind == 'boolean':
        if type(value) is not bool:
            raise ValueError('schema_boolean')
    elif kind == 'null':
        if value is not None:
            raise ValueError('schema_null')
    elif kind is not None or not any(k in schema for k in ('anyOf','const','enum')):
        raise ValueError('unsupported_schema_type')


def _matches(value, schema, depth):
    try:
        validate(value, schema, depth)
        return True
    except (ValueError, TypeError, KeyError, OverflowError):
        return False


def _alarm(*_):
    raise TimeoutError()


def error_diagnostic(error, stage):
    """Fixed categories and numeric codes only; never arbitrary exception messages."""
    reason = error.reason if isinstance(error, urllib.error.URLError) else error
    if isinstance(reason, socket.gaierror): kind = 'dns'
    elif isinstance(reason, PermissionError): kind = 'permission'
    elif isinstance(reason, TimeoutError): kind = 'timeout'
    elif isinstance(reason, ssl.SSLError): kind = 'tls'
    elif isinstance(reason, ConnectionError): kind = 'connection'
    elif isinstance(reason, OSError): kind = 'os_error'
    elif isinstance(reason, (ValueError, TypeError, KeyError)): kind = 'parse_or_contract'
    else: kind = 'other'
    number = getattr(reason, 'errno', None)
    return {'contract':'synthetic-transport-diagnostic@v1','stage':stage,'category':kind,
            'errno':number if type(number) is int and -65536 <= number <= 65536 else None}


def response_identity(receipt, headers):
    """Keep provider UUID headers even if reading/parsing the body later fails."""
    ids = []
    for name in ('x-ci-request-id','X-Cheaper-Inference-Request-Id','x-request-id'):
        value = headers.get(name) if headers is not None else None
        if not isinstance(value,str) or len(value) != 36: continue
        try: parsed = str(uuid.UUID(value))
        except ValueError: continue
        if parsed == value.lower() and parsed not in ids: ids.append(parsed)
    if ids:
        receipt['response_request_ids'] = ids
        receipt['response_identity_status'] = 'single' if len(ids) == 1 else 'conflicting'


def persist_response_identity(receipt, sink):
    """Persist only authenticated response metadata before reading its body.

    This is identity evidence, never a bill, response, semantic judgment or retry
    authorization. Missing/conflicting IDs remain explicit and fail closed.
    """
    if sink is not None:
        sink({'version':'response-header-identity@v1',
              'request_body_sha256':receipt['request_body_sha256'],
              'request_body_bytes':receipt['request_body_bytes'],
              'http_status':receipt.get('http_status'),
              'response_request_ids':list(receipt.get('response_request_ids',[])),
              'identity_status':receipt.get('response_identity_status','missing')})


def recovery_disposition(receipt, value):
    """Diagnosis only. Never schedules a call or repeats any effect."""
    if receipt['status']!='settled' or receipt['response_status'] in ('uncertain',):
        state='reconcile_exact_request'
    elif value is None:
        state='known_terminal_invalid_response'
    else:
        state='native_validation_required'
    return {'state':state,'automatic_retry':False,'reuse_request_identity':False,
            'retry_requires_fresh_admission':True,'repeat_execution':False}


def serialize_request(body, payload_limit):
    return json.dumps(body, allow_nan=False, ensure_ascii=(payload_limit==16384)).encode()


def invoke(body, timeout=240, *, expected_model=MODEL, payload_limit=16384, failure_evidence=None, completion_limit=8192, on_response_identity=None):
    """Return {value, receipt}; caller must independently admit expected_model.

    The default remains DeepSeek. Body content cannot choose its own model authority.
    This finite alternative set is for separately admitted synthetic shadow runs.
    """
    limit_key = 'max_completion_tokens' if expected_model == 'gpt-6-luna' else 'max_tokens'
    other_limit = 'max_tokens' if limit_key == 'max_completion_tokens' else 'max_completion_tokens'
    if (not isinstance(expected_model,str) or expected_model not in SHADOW_MODELS or not isinstance(body,dict) or body.get('model') != expected_model or body.get('stream') is not False
            or type(timeout) not in (int,float) or not 0 < timeout <= 240
            or other_limit in body or type(body.get(limit_key)) is not int or not 1 <= body[limit_key] <= 32768):
        raise ValueError('invalid_request_contract')
    # An explicit caller-side continuation admission may select this finite profile.
    # Neither the request body nor a provider response can enlarge it.
    if (type(completion_limit) is not int or completion_limit not in (8192,32768) or type(payload_limit) is not int or payload_limit not in (16384,32768,capacity.PAYLOAD_BYTES)
            or (payload_limit>=32768 and (expected_model!='gpt-6-luna' or body.get(limit_key)!=completion_limit))):
        raise ValueError('invalid_payload_profile')
    if failure_evidence is not None and not isinstance(failure_evidence,FailureEvidence):raise ValueError('invalid_evidence_sink')
    if on_response_identity is not None and not callable(on_response_identity):raise ValueError('invalid_identity_sink')
    if 'response_format' in body and not isinstance(body['response_format'],dict):raise ValueError('invalid_response_format')
    raw = serialize_request(body, payload_limit)
    if len(raw) > payload_limit:
        raise ValueError('payload_bound')
    if threading.current_thread() is not threading.main_thread() or signal.getitimer(signal.ITIMER_REAL)[0] != 0:
        raise ValueError('exclusive_foreground_deadline_required')
    receipt = {'status':'unknown','response_status':'uncertain','structural_status':'invalid_or_unknown',
               'usd':None,'usage_status':'unknown','usage':{},'body_sha256':None}
    value = None
    response_raw = content_raw = None
    parsed = None
    key = None
    request_hash=hashlib.sha256(raw).hexdigest()
    receipt['request_body_sha256']=request_hash
    receipt['request_body_bytes']=len(raw)
    receipt['response_format_requested']=body.get('response_format',{}).get('type')
    start = time.monotonic()
    previous = signal.getsignal(signal.SIGALRM)
    stage = 'deadline_setup'
    try:
        signal.signal(signal.SIGALRM, _alarm)
        signal.setitimer(signal.ITIMER_REAL, timeout)
        stage = 'credential'
        key = broker.credential()
        stage = 'request_construction'
        request = urllib.request.Request(ENDPOINT,data=raw,method='POST',headers={
            'Content-Type':'application/json','Authorization':'Bearer '+key,'x-ci-model-lock':'1'})
        stage = 'open'
        with urllib.request.build_opener(urllib.request.ProxyHandler({}),broker.NoRedirect()).open(request,timeout=timeout) as response:
            receipt['http_status'] = response.status
            response_identity(receipt, getattr(response,'headers',None))
            stage = 'identity_persistence'
            persist_response_identity(receipt, on_response_identity)
            stage = 'read'
            raw = response.read(RESPONSE_BOUND + 1)
            response_raw = raw
        receipt['response_status'] = 'received'
        receipt['body_sha256'] = hashlib.sha256(raw).hexdigest()
        receipt['response_bytes'] = len(raw)
        if len(raw) > RESPONSE_BOUND:
            raise ValueError('response_bound')
        stage = 'parse'
        parsed = strict(raw)
        if not isinstance(parsed,dict):
            raise ValueError('response_shape')
        receipt['request_id'] = parsed.get('request_id') if isinstance(parsed.get('request_id'),str) and len(parsed['request_id']) <= 256 and not evidence_module.private_bytes(parsed['request_id'].encode(),key) else None
        receipt['returned_model'] = parsed.get('model') if parsed.get('model') == expected_model else 'unexpected_or_missing'
        usage = parsed.get('usage',{})
        usage = usage if isinstance(usage,dict) else {}
        receipt['usage'] = {k:usage.get(k) if type(usage.get(k)) is int and usage[k] >= 0 else None for k in ('prompt_tokens','completion_tokens','total_tokens')}
        for name, fields in [('prompt_tokens_details',('cached_tokens','cache_write_tokens')),('completion_tokens_details',('reasoning_tokens','accepted_prediction_tokens','rejected_prediction_tokens'))]:
            detail = usage.get(name,{})
            receipt['usage'][name] = {k:detail[k] for k in fields if isinstance(detail,dict) and type(detail.get(k)) is int and detail[k] >= 0}
        counts = receipt['usage']
        if all(counts[k] is not None for k in ('prompt_tokens','completion_tokens','total_tokens')):
            details_fit = (all(v <= counts['prompt_tokens'] for v in counts['prompt_tokens_details'].values())
                           and all(v <= counts['completion_tokens'] for v in counts['completion_tokens_details'].values()))
            if counts['total_tokens'] == counts['prompt_tokens'] + counts['completion_tokens'] and details_fit:
                receipt['usage_status'] = 'reported'
            else:
                receipt['usage_status'] = 'inconsistent'
        billing = parsed.get('billing',{})
        if isinstance(billing,dict) and billing.get('status') == 'settled' and billing.get('currency') == 'USD':
            try:
                cost = decimal.Decimal(str(billing.get('billed_cost_usd')))
                if cost.is_finite() and cost >= 0:
                    receipt.update(status='settled',usd=str(cost))
            except decimal.InvalidOperation:
                pass
        choices = parsed.get('choices')
        if not isinstance(choices,list) or len(choices) != 1 or not isinstance(choices[0],dict):
            raise ValueError('choice_shape')
        choice = choices[0]
        reason = choice.get('finish_reason')
        receipt['finish_reason'] = reason if reason in ('stop','length','tool_calls','content_filter','function_call') else 'unknown'
        if parsed.get('model') != expected_model:
            raise ValueError('model_mismatch')
        candidate_message=choice.get('message')
        if isinstance(candidate_message,dict) and isinstance(candidate_message.get('content'),str):
            content_raw=candidate_message['content'].encode('utf-8')
        if reason != 'stop':
            raise ValueError('incomplete_output')
        message = choice.get('message')
        if not isinstance(message,dict) or message.get('tool_calls') or message.get('function_call') or not isinstance(message.get('content'),str):
            raise ValueError('message_shape')
        content_raw = message['content'].encode('utf-8')
        stage = 'content_parse'
        value = strict(message['content'])
        if key and key in json.dumps(value,ensure_ascii=False):raise ValueError('credential_echo')
        receipt['structural_status'] = 'typed_json'
    except urllib.error.HTTPError as error:
        try:
            response_identity(receipt, error.headers)
            receipt['http_status'] = error.code
            persist_response_identity(receipt, on_response_identity)
            receipt['diagnostic'] = {'contract':'synthetic-transport-diagnostic@v1','stage':stage,'category':'http_error','errno':None}
            receipt.update(response_status='received_http_error',http_status=error.code,error_code='http_error')
            try:
                response_raw=error.read(RESPONSE_BOUND+1)
                receipt['body_sha256']=hashlib.sha256(response_raw).hexdigest()
                receipt['response_bytes']=len(response_raw)
                if len(response_raw)>RESPONSE_BOUND:raise ValueError('response_bound')
                parsed=strict(response_raw)
                detail=parsed.get('error',{}) if isinstance(parsed,dict) else {}
                if error.code in (400,422) and isinstance(detail,dict) and detail.get('param')=='response_format':
                    receipt['structured_output_status']='rejected_by_provider'
            except Exception:pass
        finally:
            error.close()
    except Exception as error:
        receipt['diagnostic'] = error_diagnostic(error, stage)
        # Never store provider text, credentials, reasoning or arbitrary exception messages.
        known = {'response_bound','response_shape','choice_shape','model_mismatch','incomplete_output','message_shape','duplicate_key','nonfinite','credential_echo'}
        receipt['error_code'] = ('malformed_json' if isinstance(error,(json.JSONDecodeError,UnicodeDecodeError)) else
            str(error) if isinstance(error,ValueError) and str(error) in known else ('deadline' if isinstance(error,TimeoutError) else 'transport_or_parse_error'))
        if stage in ('parse','content_parse'):receipt['parse_location']='content' if stage=='content_parse' else 'wire'
        if isinstance(error,json.JSONDecodeError):receipt['parse_position']={'offset':error.pos,'line':error.lineno,'column':error.colno}
        value = None
    finally:
        signal.setitimer(signal.ITIMER_REAL,0)
        signal.signal(signal.SIGALRM, previous)
        receipt['elapsed_seconds'] = time.monotonic() - start
        if failure_evidence is not None:
            try:
                failure_evidence.capture(response_raw,content_raw,key,evidence_module.safe_wire(parsed) and not (key and key in json.dumps(parsed,ensure_ascii=False)))
                if value is None:receipt['failure_evidence']=failure_evidence.retain(stage)
            except Exception:
                receipt['failure_evidence']={'status':'retention_failed'}
        key=None
    receipt['recovery']=recovery_disposition(receipt,value)
    return {'value':value,'receipt':receipt}
