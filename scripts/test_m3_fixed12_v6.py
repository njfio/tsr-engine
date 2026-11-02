"""Deadline regression tests for final fixed12 controller, without workloads."""
import tempfile
from pathlib import Path
from unittest.mock import patch
import unittest
import test_m3_fixed12 as original
c=original.prior.load('m3-fixed12-v6-controller')
class Tests(original.Tests):
    controller = c
    def test_blocked_and_call_budget_counts_distinct(self):
        x=dict(result=dict(outcome='rejected',oracle=None,system='rules'),diagnostics=[dict(code='ok',proposal_action='test')],proposal_kinds=[],proposed_count=0,closure_reasons=['stop:blocked:required inspections and failing test must precede patching'],stop_reasons=['blocked'],terminal_event_kinds=['stop','task_closed'])
        self.assertEqual(c.classify(x,{}),'rejected')
        with self.assertRaises(ValueError):c.classify(x|dict(proposed_count=1),{})
        x.update(closure_reasons=['stop:budget:generator_calls'],stop_reasons=['budget:generator_calls'],proposed_count=1)
        self.assertEqual(c.classify(x,{}),'rejected')
        with self.assertRaises(ValueError):c.classify(x|dict(stop_reasons=[]),{})
    def test_zero_response_budget_and_unknown_reason(self):
        x=dict(result=dict(outcome='rejected',oracle=None),diagnostics=[],proposal_kinds=[],proposed_count=0,closure_reasons=['stop:budget:request_bytes'],stop_reasons=['budget:request_bytes'],terminal_event_kinds=['stop','task_closed'])
        self.assertEqual(c.bind_accounting(b'',b'',[]),[])
        self.assertEqual(c.classify(x,{}),'rejected')
        with self.assertRaises(ValueError):c.bind_accounting(b'',b'x',[])
        with self.assertRaises(ValueError):c.classify(x|dict(closure_reasons=['stop:budget:unknown'],stop_reasons=['budget:unknown']),{})

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
class Denials(unittest.TestCase):
    def test_denial_binding_and_submission(self):
        from copy import deepcopy
        result=dict(system='rules',outcome='rejected',denials=[dict(candidate='a',reason='policy')])
        kinds=[dict(kind='proposed',candidate=dict(id='a',kind='patch')),dict(kind='denied',candidate='a',reason='policy')]
        def check(r,k):c.verify_terminal_actions(r,[(str(i),dict(kind=e)) for i,e in enumerate(k)],'oracle:false')
        check(result,kinds)
        for changes in [dict(denials=[]),dict(system='record'),dict(denials=[dict(candidate='b',reason='policy')]),dict(denials=[dict(candidate='a',reason='other')])]:
            with self.assertRaises(ValueError):check(result|changes,kinds)
        with self.assertRaises(ValueError):check(result,kinds+[dict(kind='executed',candidate='a')])
        with self.assertRaises(ValueError):check(result,kinds+[dict(kind='proposed',candidate=dict(id='b',kind='stop'))])
        with self.assertRaises(ValueError):check(result|dict(denials=[]),kinds[:1])
        submitted=[dict(kind='proposed',candidate=dict(id='z',kind='stop')),dict(kind='committed',candidate='z')]
        check(result|dict(denials=[]),submitted)
        with self.assertRaises(ValueError):check(result|dict(denials=[]),submitted+[dict(kind='stop',reason='budget:wall_clock')])
if __name__=='__main__':unittest.main()
