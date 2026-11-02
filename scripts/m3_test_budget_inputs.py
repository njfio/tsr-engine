"""Dedicated synthetic test inputs; never read or rewrite operational cost history."""
from contextlib import contextmanager
import hashlib
import json
from pathlib import Path
import tempfile


# Fixed test amounts retain the historical assertions, not historical event claims.
CASES = {
    'smoke': ('m3-smoke-broker.py', 'm3-smoke-budget-baseline.json', '0.165094'),
    'progress': ('m3-progress-validation-broker.py', 'm3-progress-validation-baseline.json', '0.208627'),
}


@contextmanager
def isolated_budget_input(case):
    broker_name, baseline_name, amount = CASES[case]
    raw = (json.dumps({'kind': 'reserve', 'id': 'synthetic-test-only', 'usd': '2'}) + '\n'
           + json.dumps({'kind': 'settled', 'id': 'synthetic-test-only', 'usd': amount}) + '\n').encode()
    baseline = {'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest(), 'settled_usd': amount}
    with tempfile.TemporaryDirectory(prefix='tsr-budget-test-') as directory:
        root = Path(directory)
        (root / 'scripts').mkdir()
        (root / 'docs').mkdir()
        # Exercise the unchanged broker and its normal relative baseline lookup in -I.
        # There is no credentials file, operational ledger or provider invocation.
        for name in (broker_name, 'm3_smoke_budget.py'):
            (root / 'scripts' / name).write_bytes(Path(__file__).with_name(name).read_bytes())
        (root / 'docs' / baseline_name).write_text(json.dumps(baseline))
        ledger = root / 'synthetic-cost.jsonl'
        ledger.write_bytes(raw)
        ledger.chmod(0o444)
        yield root / 'scripts' / broker_name, ledger
