import importlib.util
from pathlib import Path
import unittest
spec = importlib.util.spec_from_file_location('recovery', Path(__file__).with_name('m3-supplement-daemon-continuation.py'))
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

    def test_default_invocation_refuses_launch(self):
        import subprocess
        import sys
        before = m.STATE.exists()
        result = subprocess.run([sys.executable, str(Path(m.__file__))], capture_output=True, text=True)
        self.assertEqual(result.returncode, 2)
        self.assertEqual(m.STATE.exists(), before)

    def test_supplement_daemon_selects_pinned_nested_module_engine(self):
        from types import SimpleNamespace
        from unittest.mock import patch
        queue = SimpleNamespace(ENGINE='old-engine')
        plan = dict(engine='target/m3-qualification-engine-module-v2', engine_sha256='expected')
        with patch.object(m, 'digest', return_value='expected'):
            m.configure_engine(queue, plan)
        self.assertEqual(queue.ENGINE, str(m.ROOT / plan['engine']))
        queue.ENGINE = 'old-engine'
        with patch.object(m, 'digest', return_value='wrong'):
            with self.assertRaises(RuntimeError):
                m.configure_engine(queue, plan)
        self.assertEqual(queue.ENGINE, 'old-engine')

    def test_initial_allocation_is_additional_to_live_guard(self):
        self.assertFalse(m.initial_headroom(256 * 1024**3 - 1))
        self.assertTrue(m.initial_headroom(256 * 1024**3))

    def test_module_v1_cannot_be_substituted(self):
        from types import SimpleNamespace
        queue = SimpleNamespace(ENGINE='unchanged')
        with self.assertRaises(RuntimeError):
            m.configure_engine(queue, dict(engine='target/m3-qualification-engine-module-v1', engine_sha256='irrelevant'))
        self.assertEqual(queue.ENGINE, 'unchanged')
