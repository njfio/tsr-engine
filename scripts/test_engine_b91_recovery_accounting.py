"""Recovery accounting fixtures only; never invokes a provider or target worker.

The recovery runner enforces its 3-call/2-group admission. These checks establish
that its campaign seam preserves the shared history, identity, and global stops.
"""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('recovery_campaign', Path(__file__).with_name('engine-b91-campaign.py'))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class RecoveryAccounting(unittest.TestCase):
    def setUp(self):
        tmp = tempfile.TemporaryDirectory()
        self.addCleanup(tmp.cleanup)
        self.path = Path(tmp.name) / 'ledger.jsonl'
        self.manifest = m.manifest('a' * 64, self.path, fixture=True)
        self.ledger = m.Ledger(self.path, self.manifest, m.digest(self.manifest), create=True)
        self.parent = m.digest({'trial': 'fixture-parent', 'admission': 'old'})
        self.child = m.digest({'trial': 'fixture-recovery', 'admission': 'new'})
        self.ledger.enroll(self.parent, 'b' * 64)
        self.body = {'model': m.MODEL, 'stream': False, 'max_completion_tokens': 8192, 'messages': []}
        for i in range(12):
            self.call(self.parent, str(i), '0.004991' if i == 0 else '0')
        for i in range(8):
            self.ledger.run_execution(self.parent, str(i), 'b' * 64, lambda: {'fixture_only': True})
        self.prefix = self.path.read_bytes()
        self.ledger.enroll(self.child, 'c' * 64)

    def call(self, trial, request, cost):
        return self.ledger.run_call(trial, request, self.body, 240,
            lambda *_: {'receipt': {'status': 'settled', 'usd': cost}}, payload_limit=32768)

    def restart(self):
        self.ledger = m.Ledger(self.path, self.manifest, m.digest(self.manifest))

    def test_three_calls_two_groups_preserve_parent_and_restart(self):
        for i in range(3):
            self.call(self.child, 'recovery-' + str(i), '0.000100')
            self.restart()
        for i in range(2):
            self.ledger.run_execution(self.child, 'recovery-' + str(i), 'c' * 64, lambda: {'fixture_only': True})
            self.restart()
        status = self.ledger.status()
        self.assertEqual((status['calls'], status['execution_groups'], status['known_cost_microusd']), (15, 10, 5291))
        self.assertTrue(self.path.read_bytes().startswith(self.prefix))
        with self.assertRaisesRegex(ValueError, 'already reserved'):
            self.call(self.child, 'recovery-0', '0')

    def test_new_identity_and_unchanged_model_budget(self):
        self.assertNotEqual(self.parent, self.child)
        # Even an identical local stage label is a separate campaign allocation.
        self.call(self.child, '0', '0')
        self.assertEqual(self.ledger.status()['calls'], 13)
        with self.assertRaises(ValueError):
            self.ledger.enroll(self.child, 'd' * 64)
        body = dict(self.body, model='other')
        with self.assertRaises(ValueError):
            self.ledger.run_call(self.child, 'bad', body, 240, lambda *_: self.fail('no callback'))
        self.assertEqual(self.ledger.status()['calls'], 13)

    def test_unknown_new_call_blocks_all_trials_after_restart(self):
        self.call(self.child, 'unknown', None)
        self.restart()
        status = self.ledger.status()
        self.assertEqual(status['calls'], 13)
        self.assertEqual(status['committed_microusd'], 54991)
        with self.assertRaises(ValueError):
            self.call(self.parent, 'new', '0')
        with self.assertRaises(ValueError):
            self.ledger.run_execution(self.child, 'new', 'c' * 64, lambda: self.fail('no worker'))

    def test_tampered_history_is_not_a_recovery_reset(self):
        rows = self.path.read_text().splitlines()
        row = m.strict(rows[2]); row['event']['reserved_microusd'] = 0
        rows[2] = m.canonical(row).decode()
        self.path.write_text('\n'.join(rows) + '\n')
        with self.assertRaises(ValueError):
            self.restart()


if __name__ == '__main__':
    unittest.main()
