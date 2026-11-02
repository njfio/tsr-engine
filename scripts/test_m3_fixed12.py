"""Offline fixed-24 lifecycle, policy, resource and accounting checks."""
import json
from pathlib import Path
import tempfile
import unittest
import test_m3_two_model as prior
c=prior.load('m3-fixed12-controller')
def manifest():return dict(global_wall_seconds=46800,arms=[dict(index=i,task='task'+str(i//2),pair_index=i//2,mode=('record','rules')[i%2],model='deepseek-v4.1-flash') for i in range(24)])
class Fake(prior.Fake):
    def export_and_verify(self,arm,process,dest):
        x=super().export_and_verify(arm,process,dest);x['result']['system']=arm['mode'];return x
class Tests(unittest.TestCase):
    controller = c
    def test_24_ordinary_failures_continue_once(self):
        with tempfile.TemporaryDirectory() as t:
            ops=Fake();state=Path(t)/'state';rows=self.controller.coordinate(manifest(),state,ops)
            self.assertEqual(len(rows),24)
            with self.assertRaises(ValueError):self.controller.coordinate(manifest(),state,ops)
            self.assertEqual(len(ops.runs),24)
    def test_uncertain_and_identity_halt(self):
        for failure in ('billing','containment','evidence','usage_missing','transport_or_internal','identity'):
            with self.subTest(failure=failure),tempfile.TemporaryDirectory() as t:
                ops=Fake(failure)
                with self.assertRaises(ValueError):self.controller.coordinate(manifest(),Path(t)/'state',ops)
                self.assertEqual(len(ops.runs),1)
    def test_blocked_and_call_budget_counts_distinct(self):
        x=dict(result=dict(outcome='rejected',oracle=None),diagnostics=[dict(code='ok')],proposal_kinds=['stop'],proposed_count=1,closure_reasons=['stop:blocked:required inspections and failing test must precede patching'])
        self.assertEqual(self.controller.classify(x,{}),'rejected')
        x['closure_reasons']=['stop:budget:generator_calls']
        with self.assertRaises(ValueError):self.controller.classify(x,{})
        x['proposed_count']=2;self.assertEqual(self.controller.classify(x,{}),'rejected')
    def test_predispatch_budget_no_charge_and_unknown_halt(self):
        d=dict(code='token_reservation_bound',status='failed',settled=False,stage='encode')
        self.assertEqual(self.controller.bind_accounting(b'',b'',[d]),[])
        self.assertEqual(self.controller.classify(dict(result=dict(outcome='unavailable'),diagnostics=[d],proposed_count=0),{}),'budget-unavailable')
        with self.assertRaises(ValueError):self.controller.bind_accounting(b'',b'',[d|dict(code='transport_or_internal')])
    def test_480_cap(self):
        raw=b''
        for i in range(480):raw+=(json.dumps(dict(kind='reserve',id=str(i),usd='2'))+'\n'+json.dumps(dict(kind='settled',id=str(i),usd='.01'))+'\n').encode()
        base=self.controller.budget.history.freeze(b'');self.controller.budget.check(raw,base,False)
        with self.assertRaises(ValueError):self.controller.budget.check(raw,base,True)
if __name__=='__main__':unittest.main()
