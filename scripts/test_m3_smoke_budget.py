"""M3-SMOKE-01: immutable baseline and incremental reservation limits, offline."""
import importlib.util
import json
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location('budget', Path(__file__).with_name('m3_smoke_budget.py'))
b = importlib.util.module_from_spec(spec)
spec.loader.exec_module(b)

def rows(*events):
    return ''.join(json.dumps(e, sort_keys=True) + '\n' for e in events).encode()

def pair(ident, cost):
    return ({'kind':'reserve','id':ident,'usd':'2'}, {'kind':'settled','id':ident,'usd':cost})

class Budget(unittest.TestCase):
    def setUp(self):
        self.base = rows(*pair('historic', '.165094'))
        self.pin = b.freeze(self.base)
    def test_M3_SMOKE_01_incremental_boundary_counts_reservations(self):
        spent = rows(*(e for i in range(4) for e in pair(str(i), '2')))
        b.check(self.base + spent, self.pin)
        with self.assertRaises(ValueError):
            b.check(self.base + spent + rows(*pair('extra', '.000001')), self.pin)
        with self.assertRaises(ValueError):
            b.check(self.base + rows({'kind':'reserve','id':'unknown','usd':'2'}), self.pin)
    def test_baseline_must_be_exact_reconciled_prefix(self):
        for data in [b'', self.base.replace(b'.165094', b'.165093'), self.base[:-1]]:
            with self.assertRaises(ValueError): b.check(data, self.pin)
        with self.assertRaises(ValueError):
            b.freeze(rows({'kind':'reserve','id':'pending','usd':'2'}))
    def test_invalid_costs_and_duplicate_or_oversized_settlements(self):
        for cost in ['-1', 'NaN', 'Infinity', '2.01']:
            with self.assertRaises(ValueError): b.freeze(rows(*pair('x', cost)))
        with self.assertRaises(ValueError): b.freeze(rows(*pair('x', '1'), *pair('x', '1')))
    def test_aggregate_limit_independent_from_incremental(self):
        data = rows(*(e for i in range(50) for e in pair(str(i), '1.99')))
        with self.assertRaises(ValueError): b.check(data, b.freeze(data))

if __name__ == '__main__': unittest.main()

class Adapter(unittest.TestCase):
    def test_M3_SMOKE_02_raw_baseline_bytes_and_precredential_gate(self):
        import tempfile
        from unittest.mock import patch
        spec = importlib.util.spec_from_file_location('adapter', Path(__file__).with_name('m3-smoke-broker.py'))
        adapter = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(adapter)
        root = Path(__file__).resolve().parents[1]
        original = (root/'target/m3-provider-cost.jsonl').read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            ledger = Path(directory)/'ledger.jsonl'
            ledger.write_bytes(original.replace(b'\n', b'\r\n'))
            with patch.object(adapter, 'credential', side_effect=AssertionError('credential reached')) as key:
                with self.assertRaises(ValueError): adapter.run({}, ledger)
                key.assert_not_called()
