"""Deadline regression tests for final fixed12 controller, without workloads."""
import tempfile
from pathlib import Path
from unittest.mock import patch
import unittest
import test_m3_fixed12 as original
c=original.prior.load('m3-fixed12-v3-controller')
class Tests(original.Tests):
    controller = c
class Deadlines(unittest.TestCase):
    def test_slow_predispatch_guard_prevents_launch(self):
        now=[0]
        class Slow(original.Fake):
            def guard(self,initial=False):
                if not initial:now[0]=45001
        with tempfile.TemporaryDirectory() as t,patch('time.monotonic',side_effect=lambda:now[0]):
            ops=Slow()
            with self.assertRaisesRegex(ValueError,'headroom before dispatch'):c.coordinate(original.manifest(),Path(t)/'state',ops)
            self.assertEqual(ops.runs,[])
    def test_slow_export_prevents_completed_decision(self):
        now=[0]
        class Slow(original.Fake):
            def export_and_verify(self,*a,**kw):
                x=super().export_and_verify(*a,**kw);now[0]=46800;return x
        with tempfile.TemporaryDirectory() as t,patch('time.monotonic',side_effect=lambda:now[0]):
            ops=Slow();state=Path(t)/'state'
            with self.assertRaisesRegex(ValueError,'after export'):c.coordinate(original.manifest(),state,ops)
            self.assertEqual(len(ops.runs),1)
            self.assertFalse((state/'0/decision.json').exists())
if __name__=='__main__':unittest.main()
