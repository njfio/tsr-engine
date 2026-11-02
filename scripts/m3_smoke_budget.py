"""Offline-verifiable reservation budget for the fresh M3 smoke comparison.

The baseline pins exact historical bytes. No ledger correction or reconciliation is
performed here. Unknown charges fail closed; settlement remains a separate action.
"""
import hashlib
import json
from decimal import Decimal as D

INCREMENTAL_CAP = D('10')
AGGREGATE_CAP = D('100')
RESERVATION = D('2')


def total(raw):
    if raw and not raw.endswith(b'\n'):
        raise ValueError('partial ledger row')
    reserves, settled = {}, {}
    for line in raw.splitlines():
        event = json.loads(line)
        kind, ident = event['kind'], event['id']
        cost = D(event['usd'])
        if not cost.is_finite() or cost < 0:
            raise ValueError('invalid cost')
        if kind == 'reserve' and ident not in reserves:
            reserves[ident] = cost
        elif kind == 'settled' and ident in reserves and ident not in settled and cost <= reserves[ident]:
            settled[ident] = cost
        else:
            raise ValueError('invalid cost event')
    if reserves.keys() != settled.keys():
        raise ValueError('unreconciled charge')
    return sum(settled.values(), D(0))


def freeze(raw):
    return {'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'settled_usd': str(total(raw))}


def check(raw, baseline):
    prefix = raw[:baseline['bytes']]
    if freeze(prefix) != baseline:
        raise ValueError('historical ledger changed')
    used = total(raw)
    if used + RESERVATION > AGGREGATE_CAP:
        raise ValueError('aggregate cap')
    if used - D(baseline['settled_usd']) + RESERVATION > INCREMENTAL_CAP:
        raise ValueError('incremental cap')
    return used
