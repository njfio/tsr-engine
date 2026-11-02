#!/usr/bin/env python3
"""Prepare an offline envelope or decode a saved completion; never dispatch.

Read literal constants with ast, not import/execution. No credentials, environment
reads, sockets, subprocesses, ledger writes, or provider calls.
"""
import argparse
import ast
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def strict_json(raw):
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError('duplicate JSON field')
            result[key] = value
        return result
    def bad_constant(value):
        raise ValueError('nonfinite JSON number')
    return json.loads(raw, object_pairs_hook=unique, parse_constant=bad_constant)


def bounded_json(path, limit):
    with path.open('rb') as source:
        raw = source.read(limit + 1)
    if len(raw) > limit:
        raise ValueError('JSON input exceeds byte bound')
    return strict_json(raw)


def decode(request_path, completion_path):
    """Extract a bounded typed response. Rust still validates scope and authority."""
    return decode_response(bounded_json(request_path, 32768), bounded_json(completion_path, 65536))


def decode_response(request, completion):
    if request.get('protocol') != 'tsr-proposal@v1':
        raise ValueError('unsupported proposal protocol')
    choices = completion.get('choices')
    if completion.get('error') or not isinstance(choices, list) or len(choices) != 1:
        raise ValueError('expected one successful completion choice')
    choice = choices[0]
    if choice.get('finish_reason') == 'length':
        raise ValueError('output_truncated')
    if choice.get('finish_reason') != 'stop':
        raise ValueError('incomplete or non-text completion')
    message = choice.get('message', {})
    if message.get('role') != 'assistant' or message.get('refusal') or message.get('tool_calls') or message.get('function_call'):
        raise ValueError('refusal or tool response is not a proposal')
    content = message.get('content')
    if not isinstance(content, str) or len(content.encode()) > 8192:
        raise ValueError('proposal content must be a string within 8192 bytes')
    response = strict_json(content)
    fields = {'protocol', 'state_root', 'proposals'}
    if request.get('next_attention') is not None:
        fields.add('attention_binding')
    if not isinstance(response, dict) or set(response) != fields:
        raise ValueError('unexpected proposal envelope fields')
    if response['protocol'] != request['protocol'] or response['state_root'] != request['state_root']:
        raise ValueError('stale or foreign proposal response')
    if request.get('next_attention') is not None and response.get('attention_binding') != request['next_attention']['binding']:
        raise ValueError('stale or foreign attention binding')
    if not isinstance(response['proposals'], list) or not 1 <= len(response['proposals']) <= 4:
        raise ValueError('proposal count outside bounds')
    return response


def envelope(path):
    return envelope_request(bounded_json(path, 32768))


def envelope_request(request):
    if len(json.dumps(request, ensure_ascii=False, separators=(',', ':')).encode()) > 32768:
        raise ValueError('request exceeds 32768 bytes')
    if request.get('protocol') != 'tsr-proposal@v1':
        raise ValueError('unsupported proposal protocol')
    constants = {}
    tree = ast.parse((ROOT / 'scripts/m3-cheaper-broker.py').read_text())
    for node in tree.body:
        if isinstance(node, ast.Assign) and len(node.targets) == 1 and isinstance(node.targets[0], ast.Name):
            name = node.targets[0].id
            if name in ('HOST', 'MODEL', 'MAX_OUTPUT', 'MAX_WIRE_BODY'):
                constants[name] = ast.literal_eval(node.value)
    body = {
        'model': constants['MODEL'], 'stream': False, 'min_discount_percent': 0,
        'max_tokens': min(2048, constants['MAX_OUTPUT']),
        'response_format': {'type': 'json_object'},
        'messages': [
            {'role': 'system', 'content': request['instructions']},
            {'role': 'user', 'content': json.dumps(request)},
        ],
    }
    if len(json.dumps(body).encode()) > constants['MAX_WIRE_BODY']:
        raise ValueError('existing transport wire bound exceeded')
    return {'dispatch_enabled': False, 'endpoint': constants['HOST'] + 'chat/completions',
            'body': body, 'response_decoder': 'engine-provider-envelope.py --request REQUEST --completion COMPLETION; pass decoded JSON to engine propose --response',
            'limitation': 'Existing paid broker has task-specific scope/codec checks. This envelope is not a dispatch adapter.'}

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--request', type=Path, required=True)
    parser.add_argument('--completion', type=Path, help='Decode a saved chat completion; performs no dispatch')
    args = parser.parse_args()
    try:
        result = decode(args.request, args.completion) if args.completion else envelope(args.request)
        print(json.dumps(result, separators=(',', ':')))
    except (ValueError, KeyError, TypeError, AttributeError, OSError):
        parser.exit(2, 'Invalid or unsupported request/completion; no proposal emitted.\n')
