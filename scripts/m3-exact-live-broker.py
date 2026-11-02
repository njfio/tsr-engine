#!/usr/bin/env python3
"""Frozen exact-edit pilot entry point; no operations on import. No retries."""
import argparse
import importlib.util
import json
from pathlib import Path
import sys


def load(name):
    spec = importlib.util.spec_from_file_location(name, Path(__file__).with_name(name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


edit = load('m3-edit-broker')
budget = load('m3-exact-budget')
edit.transport.smoke_budget = budget.locked_check


def encode(request):
    if request.get('task') != '0c5a91a6aaa862c279aed1e1602a74eda7c42c4fc0f72247de513fa948f50b70':
        raise edit.BrokerFailure('pilot_scope')
    if type(request.get('remaining_action_calls')) is not int or not 1 <= request['remaining_action_calls'] <= 20:
        raise edit.BrokerFailure('pilot_call_scope')
    return edit.encode(request)


edit.transport.encode = encode

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--ledger', required=True)
    parser.add_argument('--approved-live', action='store_true')
    args = parser.parse_args()
    try:
        if not args.approved_live:
            raise edit.BrokerFailure('authorization_missing')
        raw = sys.stdin.buffer.read(edit.transport.MAX_INPUT + 1)
        if len(raw) > edit.transport.MAX_INPUT:
            raise edit.BrokerFailure('request_limit')
        request = json.loads(raw)
        encode(request)  # Scope validation before credentials or catalog access.
        print(json.dumps(edit.transport.complete_with_diagnostics(request, args.ledger)))
    except Exception:
        print('broker unavailable; check controller diagnostics and aggregate cost ledger', file=sys.stderr)
        sys.exit(2)
