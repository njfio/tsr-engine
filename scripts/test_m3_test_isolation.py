"""Offline order/repetition coverage for controller and dedicated budget inputs."""
import hashlib
import itertools
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest

from m3_test_budget_inputs import isolated_budget_input


class Isolation(unittest.TestCase):
    def test_controller_import_orders_and_repeated_suites(self):
        modules = ['test_m3_fixed12' + suffix for suffix in ('', '_v3', '_v5', '_v6')]
        code = '''
import importlib, io, sys, unittest
modules = [importlib.import_module(name) for name in sys.argv[1:]]
base = importlib.import_module('test_m3_fixed12')
assert base.c.__file__.endswith('m3-fixed12-controller.py')
assert len({m.Tests.controller.__file__ for m in modules}) == 4
for iteration in range(2):
    suite = unittest.TestSuite(unittest.defaultTestLoader.loadTestsFromModule(m) for m in modules)
    result = unittest.TextTestRunner(stream=io.StringIO()).run(suite)
    assert result.testsRun == 29, result.testsRun
    assert result.wasSuccessful(), (result.failures, result.errors)
'''
        env = {**os.environ, 'PYTHONPATH': str(Path(__file__).parent.resolve())}
        for order in itertools.permutations(modules):
            with self.subTest(order=order):
                result = subprocess.run([sys.executable, '-c', code, *order],
                                        capture_output=True, text=True, env=env, timeout=30)
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_dedicated_inputs_repeat_without_operational_ledger(self):
        for case, amount in [('smoke', '0.165094'), ('progress', '0.208627')]:
            snapshots = []
            for _ in range(2):
                with isolated_budget_input(case) as (broker, ledger):
                    raw = ledger.read_bytes()
                    snapshots.append(raw)
                    self.assertFalse(ledger.stat().st_mode & 0o222)
                    self.assertFalse((ledger.parent / 'target').exists())
                    baseline_path, = (ledger.parent / 'docs').iterdir()
                    baseline = json.loads(baseline_path.read_text())
                    self.assertEqual(baseline, {'bytes': len(raw),
                        'sha256': hashlib.sha256(raw).hexdigest(), 'settled_usd': amount})
                    self.assertEqual(broker.read_bytes(), Path(__file__).with_name(broker.name).read_bytes())
            self.assertEqual(*snapshots)


if __name__ == '__main__':
    unittest.main()
