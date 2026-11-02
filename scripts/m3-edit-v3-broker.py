"""Offline-developed exact-edit@v3 contract; not wired into any frozen live controller.

Uses action-specific wire fields; normalized engine actions retain their semantics. New schemas and bounded diagnostics require a
new reviewed baseline before live use. Importing this module performs no I/O beyond
loading trusted source files and never loads credentials or contacts a provider.
"""
import importlib.util
import json
from pathlib import Path
import re

spec=importlib.util.spec_from_file_location('frozen_edit',Path(__file__).with_name('m3-edit-broker.py'))
frozen=importlib.util.module_from_spec(spec);spec.loader.exec_module(frozen)
transport=frozen.transport
ACTION_FIELDS={'inspect':('path',),'test':('test_id',),'edit':('path','expected_digest','old_text','new_text'),'stop':()}
CONTRACT={action:('action',*fields,*(('forecast',) if action in ('test','edit') else ())) for action,fields in ACTION_FIELDS.items()}
FIELDS=('action','path','test_id','expected_digest','old_text','new_text','forecast')
ACTION_CATEGORIES=frozenset((*ACTION_FIELDS,'missing','invalid','unparsed'))
JSON_TYPES=frozenset(('missing','null','boolean','number','string','array','object','other'))
ARGUMENTS='/choices/0/message/tool_calls/0/function/arguments'
SAFE_PATHS=frozenset(('/', '/choices','/choices/0','/choices/0/message',
    '/choices/0/message/tool_calls','/choices/0/message/tool_calls/0',
    '/choices/0/message/tool_calls/0/type','/choices/0/message/tool_calls/0/function',
    '/choices/0/message/tool_calls/0/function/name',ARGUMENTS,
    '/arguments','/arguments/*','/usage','/usage/total_tokens',
    '/choices/0/finish_reason','/choices/0/message/refusal',
    *('/arguments/'+key for key in FIELDS)))
REASONS=frozenset(('type','required','unknown_field','duplicate_key','enum','must_be_empty',
    'range','must_be_nonempty','digest_pattern','must_change','invalid_json','depth_or_size',
    'encoding','cardinality','scope','refusal','truncated'))
CODES=frozenset(('choice_count','tool_shape','provider_refusal','output_truncated','proposal_limit',
    'proposal_fields','proposal_type','duplicate_field','forecast_type','forecast_missing',
    'smoke_scope','edit_precondition','usage_missing','json_invalid'))
MISSING=object()


def kind(value):
    if value is MISSING:return 'missing'
    if value is None:return 'null'
    return {bool:'boolean',int:'number',float:'number',str:'string',list:'array',dict:'object'}.get(type(value),'other')


class ValidationFailure(transport.BrokerFailure):
    def __init__(self,code,path,expected_type,actual_type,reason):
        # Every logged token belongs to a finite controller-owned vocabulary.
        if code not in CODES or path not in SAFE_PATHS or expected_type not in JSON_TYPES|{'number_or_null'} or actual_type not in JSON_TYPES or reason not in REASONS:
            raise ValueError('invalid controller diagnostic specification')
        super().__init__(code)
        self.detail=dict(version=1,path=path,expected_type=expected_type,actual_type=actual_type,reason=reason)


def fail(code,path,expected,value,reason):
    raise ValidationFailure(code,path,expected,kind(value),reason)


def schema(request):
    path=transport.allowed_path(request)
    variants=[]
    for action,fields in CONTRACT.items():
        props={key:{'type':'string'} for key in fields}
        props['action']={'type':'string','enum':[action]}
        if 'path' in props:props['path']['enum']=[path]
        if 'test_id' in props:props['test_id']['enum']=request['allowed_test_ids']
        if 'forecast' in props:props['forecast']={'type':'number','minimum':0,'maximum':1}
        if action=='edit':
            props['expected_digest'].update(minLength=64,maxLength=64,pattern='^[0-9a-f]{64}$')
            props['old_text']['minLength']=1
        variants.append(dict(type='object',additionalProperties=False,properties=props,required=list(fields)))
    return dict(type='object',oneOf=variants)


def examples(request):
    values=[]
    for action,fields in CONTRACT.items():
        value={key:'' for key in fields};value['action']=action
        if 'path' in value:value['path']=transport.allowed_path(request)
        if 'test_id' in value:value['test_id']=request['allowed_test_ids'][0]
        if 'forecast' in value:value['forecast']=0.5
        if action=='edit':value.update(expected_digest='a'*64,old_text='EXAMPLE_OLD_SPAN',new_text='EXAMPLE_NEW_SPAN')
        values.append(value)
    return values


def encode(request):
    body=frozen.encode(request)
    instruction=body['messages'][0]['content'].replace('exact-edit@v1.','exact-edit@v3.').replace('Use empty strings for irrelevant fields. ', '')
    instruction+=(' Return only the fields applicable to the chosen action. Every listed field is required; no extra fields or null values. '
        'test_id selects a command from allowed_test_ids; never invent or omit it for test. '
        'Do not emit id or test_id for inspect/edit/stop. Forecast must be a finite number in [0,1] for test/edit only. '
        'Action fields: '+json.dumps(CONTRACT)+'. Allowed test_id values: '+json.dumps(request['allowed_test_ids'])+'. '
        'Examples below illustrate shape only; do not replay them. Edit digest and spans are inert placeholders: '
        'copy the actual inspected full-file digest and exact source span before editing.\n'+json.dumps(examples(request),separators=(',',':')))
    body['messages'][0]['content']=instruction
    body['tools'][0]['function']['parameters']=schema(request)
    return body


def unique_object(pairs):
    result={}
    for key,value in pairs:
        if key in result:
            fail('duplicate_field','/arguments/*','object',result,'duplicate_key')
        result[key]=value
    return result


def decode_inner(response,request,meta):
    allowed=transport.allowed_path(request)
    if type(response) is not dict:fail('choice_count','/','object',response,'type')
    choices=response.get('choices',MISSING)
    if type(choices) is not list:fail('choice_count','/choices','array',choices,'type')
    if len(choices)!=1:fail('choice_count','/choices','array',choices,'cardinality')
    choice=choices[0]
    if type(choice) is not dict:fail('tool_shape','/choices/0','object',choice,'type')
    message=choice.get('message',MISSING)
    if type(message) is not dict:fail('tool_shape','/choices/0/message','object',message,'type')
    if message.get('refusal') or choice.get('finish_reason')=='content_filter':
        fail('provider_refusal','/choices/0/message/refusal','null',message.get('refusal',MISSING),'refusal')
    if choice.get('finish_reason')=='length':
        fail('output_truncated','/choices/0/finish_reason','string',choice['finish_reason'],'truncated')
    calls=message.get('tool_calls',MISSING)
    if type(calls) is not list:fail('tool_shape','/choices/0/message/tool_calls','array',calls,'type')
    if len(calls)!=1:fail('tool_shape','/choices/0/message/tool_calls','array',calls,'cardinality')
    call=calls[0]
    if type(call) is not dict:fail('tool_shape','/choices/0/message/tool_calls/0','object',call,'type')
    if call.get('type')!='function':fail('tool_shape','/choices/0/message/tool_calls/0/type','string',call.get('type',MISSING),'enum')
    function=call.get('function',MISSING)
    if type(function) is not dict:fail('tool_shape','/choices/0/message/tool_calls/0/function','object',function,'type')
    if function.get('name')!='propose':fail('tool_shape','/choices/0/message/tool_calls/0/function/name','string',function.get('name',MISSING),'enum')
    raw=function.get('arguments',MISSING)
    if type(raw) is not str:fail('tool_shape',ARGUMENTS,'string',raw,'type')
    try:raw_bytes=raw.encode('utf8')
    except UnicodeError:fail('json_invalid',ARGUMENTS,'string',raw,'encoding')
    if len(raw_bytes)>65536:fail('proposal_limit',ARGUMENTS,'string',raw,'depth_or_size')
    try:args=json.loads(raw,object_pairs_hook=unique_object)
    except ValidationFailure:raise
    except RecursionError:fail('json_invalid',ARGUMENTS,'string',raw,'depth_or_size')
    except ValueError:fail('json_invalid',ARGUMENTS,'string',raw,'invalid_json')
    if type(args) is not dict:fail('proposal_fields','/arguments','object',args,'type')
    action=args.get('action',MISSING)
    meta['action_category']=action if type(action) is str and action in ACTION_FIELDS else ('missing' if action is MISSING else 'invalid')
    if action is MISSING:fail('proposal_fields','/arguments/action','string',action,'required')
    if type(action) is not str:fail('proposal_type','/arguments/action','string',action,'type')
    if action not in ACTION_FIELDS:fail('proposal_type','/arguments/action','string',action,'enum')
    for key in CONTRACT[action]:
        if key not in args:fail('proposal_fields','/arguments/'+key,'number' if key=='forecast' else 'string',MISSING,'required')
    if set(args)!=set(CONTRACT[action]):fail('proposal_fields','/arguments/*','object',args,'unknown_field')
    for key in CONTRACT[action]:
        if key!='forecast' and type(args[key]) is not str:fail('proposal_type','/arguments/'+key,'string',args[key],'type')
    p=args.get('forecast')
    if action in ('test','edit'):
        if type(p) not in (int,float):fail('forecast_type','/arguments/forecast','number',p,'type')
        if not 0<=p<=1:fail('forecast_type','/arguments/forecast','number',p,'range')
    if action in ('inspect','edit') and args['path']!=allowed:fail('smoke_scope','/arguments/path','string',args['path'],'scope')
    if action=='test' and args['test_id'] not in request['allowed_test_ids']:fail('smoke_scope','/arguments/test_id','string',args['test_id'],'scope')
    if action=='edit':
        if not re.fullmatch('[0-9a-f]{64}',args['expected_digest']):fail('edit_precondition','/arguments/expected_digest','string',args['expected_digest'],'digest_pattern')
        if not args['old_text']:fail('edit_precondition','/arguments/old_text','string',args['old_text'],'must_be_nonempty')
        if args['old_text']==args['new_text']:fail('edit_precondition','/arguments/new_text','string',args['new_text'],'must_change')
    usage=response.get('usage',MISSING)
    if type(usage) is not dict:fail('usage_missing','/usage','object',usage,'type')
    tokens=usage.get('total_tokens',MISSING)
    if type(tokens) is not int:fail('usage_missing','/usage/total_tokens','number',tokens,'type')
    if tokens<0:fail('usage_missing','/usage/total_tokens','number',tokens,'range')
    return dict(reply=dict(request=request['id'],proposal={'action':action,**{('id' if k=='test_id' else k):args[k] for k in ACTION_FIELDS[action]}},forecast=p),billed_tokens=tokens)


def decode(response,request):
    meta={'action_category':'unparsed'}
    try:return decode_inner(response,request,meta)
    except ValidationFailure as error:
        error.detail['action_category']=meta['action_category']
        raise

original_run=transport.run

def run(request,ledger,trace=None):
    try:
        result=original_run(request,ledger,trace)
        if trace is not None:trace['proposal_action']=result['reply']['proposal']['action']
        return result
    except ValidationFailure as error:
        if trace is not None:
            trace['proposal_validation']=error.detail.copy()
            trace['proposal_action']=error.detail['action_category']
        raise

transport.encode=encode
transport.decode=decode
transport.run=run
