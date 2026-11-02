"""Exact-edit pilot budget: immutable history, $5 incremental, $100 lifetime, 240 calls."""
from decimal import Decimal as D
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('historical_budget', ROOT/'scripts/m3_smoke_budget.py')
history = importlib.util.module_from_spec(spec)
spec.loader.exec_module(history)
BASELINE = ROOT / 'docs/m3-handle-live-budget-baseline.json'


def check(raw, baseline, dispatch=True):
    if history.freeze(raw[:baseline['bytes']]) != baseline:
        raise ValueError('historical ledger changed')
    used = history.total(raw)  # Unknown or outstanding reservations fail closed.
    incremental = used - D(baseline['settled_usd'])
    reserve = D('2') if dispatch else D('0')
    if used + reserve > D('100') or incremental + reserve > D('5'):
        raise ValueError('exact-edit budget cap')
    calls = sum(json.loads(line)['kind'] == 'reserve' for line in raw[baseline['bytes']:].splitlines())
    if calls > 240 or dispatch and calls >= 240:
        raise ValueError('exact-edit request cap')
    return used


def locked_check(log):
    log.seek(0)
    return check(log.read().encode('utf8'), json.loads(BASELINE.read_text()))
