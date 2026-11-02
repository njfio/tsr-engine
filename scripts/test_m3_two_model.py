"""Offline comparison control tests. No credentials or network."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

def load(name):
    s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+'.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
c=load('m3-two-model-controller')
def manifest():return dict(arms=[dict(index=i,model=model,task='synthetic',mode='record') for i,model in enumerate(['deepseek-v4.1-flash','glm-5.3-flash']*3)])
class Fake:
    def __init__(self,failure=None):self.raw=b'';self.runs=[];self.failure=failure
    def ledger(self):return self.raw
    def guard(self,initial=False):
        c.budget.history.total(self.raw)
        if self.runs and self.failure=='containment':raise ValueError('containment')
    def run(self,arm,state):
        self.runs.append(arm);i=str(arm['index'])
        events=[dict(kind='reserve',id=i,usd='2',model=arm['model'])]
        if self.failure!='billing':events.append(dict(kind='settled',id=i,usd='.001'))
        self.raw+=b''.join((json.dumps(e)+'\n').encode() for e in events)
        return dict(started='2026-10-17T00:00:00+00:00',finished='2026-10-17T00:00:01+00:00')
    def export_and_verify(self,arm,process,dest):
        if self.failure=='evidence':raise ValueError('evidence')
        d=dict(code=self.failure if self.failure in ('usage_missing','transport_or_internal') else 'proposal_fields',stage='decode',status='failed',settled=True,reservation_id=str(arm['index']))
        result=dict(task=arm['task'],system='record',outcome='unavailable',oracle=None)
        if self.failure=='identity':result['task']='other'
        return dict(result=result,diagnostics=[d],proposed_count=0,proposal_kinds=[],visible_test_exit_codes=[],manifest_sha256='fake')
class Tests(unittest.TestCase):
    def test_all_six_recorded_schema_failures_advance_no_retry(self):
        with tempfile.TemporaryDirectory() as t:
            ops=Fake();state=Path(t)/'state';rows=c.coordinate(manifest(),state,ops)
            self.assertEqual(len(rows),6);self.assertTrue(all(r['schema_failures']==1 and not r['final_oracle_completed'] for r in rows))
            with self.assertRaises(ValueError):c.coordinate(manifest(),state,ops)
            self.assertEqual(len(ops.runs),6)
    def test_uncertainty_halts(self):
        for failure in ('billing','containment','evidence','usage_missing','transport_or_internal','identity'):
            with self.subTest(failure=failure),tempfile.TemporaryDirectory() as t:
                ops=Fake(failure)
                with self.assertRaises(ValueError):c.coordinate(manifest(),Path(t)/'state',ops)
                self.assertEqual(len(ops.runs),1)
    def test_partial_and_budget_are_failures(self):
        d=dict(code='ok')
        x=dict(result=None,diagnostics=[d],failure='stale-digest-or-oversize')
        self.assertEqual(c.classify(x,{}),'stale-digest-or-oversize')
        x.update(result=dict(outcome='rejected',oracle=None),proposed_count=2)
        self.assertEqual(c.classify(x,{}),'rejected')
    def test_call_ceiling(self):
        baseline=c.budget.history.freeze(b'');raw=b''
        for i in range(120):
            raw+=(json.dumps(dict(kind='reserve',id=str(i),usd='2'))+'\n'+json.dumps(dict(kind='settled',id=str(i),usd='.001'))+'\n').encode()
        c.budget.check(raw,baseline,dispatch=False)
        with self.assertRaises(ValueError):c.budget.check(raw,baseline,dispatch=True)
    def test_duplicate_field_is_recorded_failure(self):
        self.assertEqual(c.classify(dict(result=dict(outcome='unavailable'),proposed_count=0,diagnostics=[dict(code='duplicate_field',stage='decode',status='failed')]),{}),'schema-or-response-failure')
    def test_unknown_decode_codes_halt(self):
        for code in (None,'unknown','usage_missing','transport_or_internal'):
            with self.assertRaises(ValueError):c.classify(dict(result=dict(outcome='unavailable'),proposed_count=0,diagnostics=[dict(code=code,stage='decode',status='failed')]),{})
if __name__=='__main__':unittest.main()
