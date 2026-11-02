"""Offline report regressions; no process, runtime, credential or provider access."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

spec=importlib.util.spec_from_file_location('report',Path(__file__).with_name('m3-continuation-report.py'))
r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)

class Report(unittest.TestCase):
    def test_process_closure_denial_and_boolean_consistency(self):
        process=dict(status='completed',exit=0,errors=[])
        result=dict(outcome='rejected',oracle=dict(accepted=False),denials=[])
        stored=[('id',dict(kind=dict(kind='task_closed',reason='oracle:false')))]
        receipt=dict(status='checked',verified_success=False)
        r.verify_terminal_record(process,process,result,stored,receipt)
        for change in ('process','receipt','oracle','closure','denial','missing','duplicate','late'):
            with self.subTest(change=change):
                p,q,e,v=map(copy.deepcopy,(process,result,stored,receipt))
                if change=='process':p['exit']=1
                elif change=='receipt':v['verified_success']=True
                elif change=='oracle':q['oracle']['accepted']=True
                elif change=='closure':e[0][1]['kind']['reason']='oracle:true'
                elif change=='denial':q['denials']=['inert']
                elif change=='missing':e=[]
                elif change=='duplicate':e*=2
                else:e.append(('other',dict(kind=dict(kind='observed'))))
                with self.assertRaises(ValueError):r.verify_terminal_record(p,process,q,e,v)

    def test_root_observation_does_not_imply_completed_pair(self):
        task='0c5a91-inert';manifest=dict(arms=[dict(task=task)])
        historical=dict(arms=[dict(task=task,mode='record',result=dict(state_root_at_start='root-a',outcome='unavailable'))])
        single=r.compare_roots(manifest,[],historical)[0]
        self.assertFalse(single['both_observed']);self.assertIsNone(single['equal_when_both_observed'])
        for root,expected in [('root-a',True),('root-b',False)]:
            current=[dict(task=task,mode='rules',evidence_status='verified',result=dict(state_root_at_start=root,outcome='unavailable'))]
            compared=r.compare_roots(manifest,current,historical)[0]
            self.assertTrue(compared['both_observed'])
            self.assertIs(compared['equal_when_both_observed'],expected)
            self.assertNotIn('completed_pairs',compared)

    def test_partial_export_and_intent_without_process_remain_unverified(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td);docs=root/'docs';target=root/'target';docs.mkdir();target.mkdir()
            state=target/'state';state.mkdir()
            manifest=dict(state='target/state',baseline='docs/m3-continuation-baseline.json',sha256={},arms=[dict(task='a',mode='rules',input='target/a'),dict(task='b',mode='record',input='target/b'),dict(task='b',mode='rules',input='target/b')])
            def write(path,value):path.write_text(json.dumps(value)+'\n')
            write(state/'terminal.json',dict(status='halted-review-required'))
            write(state/'0-intent.json',dict(index=0))
            (state/'evidence-0').mkdir();write(state/'evidence-0'/'partial.json',{})
            write(state/'1-intent.json',dict(index=1))
            write(docs/'m3-continuation-readiness.json',manifest)
            write(docs/'m3-continuation-baseline.json',r.budget.freeze(b''))
            write(docs/'m3-continuation-launch.json',dict(controller_pid=99999))
            write(docs/'m3-progress-validation-export-sha256.json',{})
            write(docs/'m3-progress-validation-result.json',dict(arms=[]))
            (docs/'m3-progress-validation-execution').mkdir()
            for name in ('m3-continuation-authorization.json','m3-continuation-authorization-provenance.json','m3-continuation-live-preflight.json','m3-continuation-price-preflight.json'):write(docs/name,{})
            for name in ('m3-provider-cost.jsonl','m3-provider-cost-diagnostics.jsonl','m3-continuation-launch.log'):(target/name).write_text('')
            fake=SimpleNamespace(verify_freeze=lambda m:None,require_idle=lambda:None)
            with patch.object(r,'ROOT',root),patch.object(r,'load',return_value=fake),patch.object(r.subprocess,'check_output',return_value=''):
                r.main()
            report=json.loads((docs/'m3-continuation-result.json').read_text())
            self.assertEqual([a['evidence_status'] for a in report['arms']],['partial_export','unverified_attempt','not_attempted'])
            self.assertFalse(report['audit']['all_attempted_evidence_verified'])
            self.assertEqual(report['completed_pairs'],[])

if __name__=='__main__':unittest.main()
