"""Private bounded completion diagnostics and explicit settled-decode recovery validation."""
import json
import os


def save_response(path, response, secret):
    choices = response.get('choices', [])
    choice = choices[0] if isinstance(choices, list) and len(choices) == 1 and isinstance(choices[0], dict) else {}
    message = choice.get('message', {})
    if not isinstance(message, dict):
        message = {}
    safe = {'finish_reason': choice.get('finish_reason'), 'usage': response.get('usage'),
            'content': message.get('content'), 'refusal': message.get('refusal'),
            'reasoning_content_bytes': len(str(message.get('reasoning_content', '')).encode())}
    raw = json.dumps(safe, ensure_ascii=False)
    if secret:
        raw = raw.replace(json.dumps(secret, ensure_ascii=False)[1:-1], '[REDACTED_CREDENTIAL]')
        raw = raw.replace(secret, '[REDACTED_CREDENTIAL]')
    bounded = raw.encode()[:16384].decode('utf8', errors='ignore')
    payload = json.dumps({'diagnostic_preview': bounded, 'truncated': len(raw.encode()) > 16384}).encode()
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, 'wb') as file:
        file.write(payload); file.flush(); os.fsync(file.fileno())


def validate_recovery(events, ledger_events, diagnostics, reservation_id, request, max_steps):
    if any(e['kind'] == 'recovery' for e in events):
        raise ValueError('recovery_already_recorded')
    starts = [e for e in events if e['kind'] == 'started']
    failures = [e for e in events if e['kind'] == 'failed']
    if len(starts) != 1 or len(failures) != 1 or max_steps != 2:
        raise ValueError('recovery_requires_one_failed_call_and_one_remaining_step')
    start, failure = starts[0], failures[0]
    if (failure['step_id'] != start['step_id'] or failure['reason'] not in
            ('transport_or_decode_failure', 'output_truncated', 'response_decode_invalid')
            or request['state_root'] != start['state_root'] or request['pending'] is not None
            or any(e['kind'] in ('decoded', 'committed', 'execution_started', 'completed') for e in events)):
        raise ValueError('recovery_state_changed_or_not_decode_failure')
    settlement = [e for e in ledger_events if e['kind'] == 'settled' and e['id'] == reservation_id]
    diagnostic = [d for d in diagnostics if d.get('reservation_id') == reservation_id]
    if (len(settlement) != 1 or len(diagnostic) != 1 or diagnostic[0].get('stage') != 'decode'
            or diagnostic[0].get('settled') is not True or diagnostic[0].get('status') != 'failed'):
        raise ValueError('verified_settled_decode_receipt_required')
    tokens = settlement[0].get('usage', {}).get('total_tokens')
    if type(tokens) is not int or tokens < 0:
        raise ValueError('verified_usage_required')
    return {'failed_step_id': start['step_id'], 'reservation_id': reservation_id,
            'billed_tokens': tokens, 'completion_tokens': 4096,
            'reason': 'explicit_owner_recovery_of_settled_decode_failure'}
