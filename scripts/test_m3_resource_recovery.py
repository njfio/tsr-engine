import importlib.util
from pathlib import Path
import unittest
spec = importlib.util.spec_from_file_location('recovery', Path(__file__).with_name('m3-resource-recovery.py'))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)

class ResourceChecks(unittest.TestCase):
    def test_no_launch_below_threshold_or_without_live_guard(self):
        for free, halted, alive in [(m.THRESHOLD-1, False, True), (m.THRESHOLD, True, True), (m.THRESHOLD, False, False)]:
            called=[]
            result=m.gated_launch(lambda: m.permitted(free, halted, alive), lambda: called.append(True))
            self.assertEqual(called, [])
            self.assertEqual(result['status'], 'cancelled-before-launch')

    def test_healthy_guard_allows_launch(self):
        self.assertEqual(m.gated_launch(lambda: m.permitted(m.THRESHOLD, False, True), lambda: 'started'), 'started')

    def test_nonzero_child_cannot_mark_campaign_completed(self):
        self.assertEqual(m.terminal_status([dict(status='completed', exit=1, errors=[])], False), 'halted')

    def test_outer_must_match_identity_and_stopped_state(self):
        self.assertFalse(m.valid_outer('S', m.OUTER_STARTED, m.OUTER_COMMAND))
        self.assertFalse(m.valid_outer('T', 'different', m.OUTER_COMMAND))
        self.assertFalse(m.valid_outer('T', m.OUTER_STARTED, 'different'))
        self.assertTrue(m.valid_outer('Ts', m.OUTER_STARTED, m.OUTER_COMMAND))
