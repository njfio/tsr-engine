"""Authored native fixtures only; no model or target execution."""
import importlib.util
from pathlib import Path
import json
import unittest
from unittest.mock import patch
import test_engine_b91_patch_recovery as previous
spec=importlib.util.spec_from_file_location('final_recovery','scripts/engine-b91-final-recovery.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class FinalTests(previous.RecoveryTests):
    def test_final_two_and_one_call_recovery_same_campaign(self):
        parent=self.parent_failed();ledger=self.ledger();manifest_id=m.c.digest(ledger.manifest)
        with patch.object(previous.m,'proof_validate',return_value=manifest_id),patch.object(m.r,'proof_validate',return_value=manifest_id),patch.object(m,'proof_validate',return_value=manifest_id):
            root=self.root.parent/'patch';a=previous.m.admission(parent,root,{})
            self.root=root;self.a=a;original=self.provider
            def error_at(stage):
                def provider(body,timeout):
                    result=original(body,timeout)
                    if json.loads(body['messages'][1]['content'])['stage']==stage:
                        result['value']=None;result['receipt'].update(response_status='received_http_error',http_status=503,error_code='http_error',usage={},status='unknown',usd=None,response_request_ids=['aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee'])
                    return result
                return provider
            # Prefix generation is a fixture; campaign accounting starts with its fixed spent prefix.
            run=previous.m.Recovery(root,a,error_at('relation_after_verification'),self.worker,self.reviewer,fixture=True)
            (root/'fixture-only.marker').write_text('b91-native-fixture-only@v1\n')
            with self.assertRaises(ValueError):run.run()
            admission_file=root.parent/'patch-admission.json';m.e.private_save(admission_file,a)
            before=m.b.tree(root);self.calls=[];self.workers=[]
            finalroot=root.parent/'final-two';fa=m.admission(admission_file,finalroot,{})
            self.root=finalroot;self.a=fa
            def create():return m.campaign_runner(finalroot,fa,ledger,original,self.reviewer,fixture=True)
            result=create().run()
            self.assertEqual(self.calls,list(m.e.STAGES[-2:]));self.assertEqual(self.workers,[])
            self.assertEqual((result['calls'],result['execution_groups']),(2,0))
            self.assertEqual(create().run(),result);self.assertEqual(len(self.calls),2)
            self.assertEqual(m.b.tree(root),before)
            self.assertEqual((ledger.status()['calls'],ledger.status()['execution_groups']),(6,2))
            with self.assertRaises(m.d.Blocked):create().execute(None,'patch')
            with self.assertRaises(m.d.Blocked):create().commit_response(None,None,None)
            # A separately stopped final-only failure preserves its successful reflection.
            failroot=root.parent/'final-failed';faila=m.admission(admission_file,failroot,{})
            self.root=failroot;self.a=faila
            failrun=m.FinalRecovery(failroot,faila,error_at('final_proposal_only'),self.reviewer,fixture=True)
            with self.assertRaises(ValueError):failrun.run()
            failfile=root.parent/'final-failed-admission.json';m.e.private_save(failfile,faila)
            one=root.parent/'final-one';onea=m.admission(failfile,one,{})
            self.assertEqual(onea['calls'],1);self.assertEqual(onea['prefix']['first_stage'],5)
            self.root=one;self.a=onea;self.calls=[]
            onerun=m.campaign_runner(one,onea,ledger,original,self.reviewer,fixture=True)
            oneresult=onerun.run();self.assertEqual(self.calls,['final_proposal_only'])
            self.assertEqual(oneresult['calls'],1);self.assertEqual(oneresult['execution_groups'],0)
            self.assertEqual(onerun.run(),oneresult)
            (one/'prefix-projection.json').write_text('{}')
            with self.assertRaises(ValueError):onerun.run()
            # Known bounds/backoff and native unknown semantics do not dispatch more work.
            bad=dict(fa);bad['execution_groups']=1
            with self.assertRaises(ValueError):m.validate(bad,finalroot.resolve(),True)
            delayed=root.parent/'delayed';da=m.admission(admission_file,delayed,{},m.time.time()+300)
            dr=m.FinalRecovery(delayed,da,lambda *_:self.fail('early call'),self.reviewer,fixture=True)
            with self.assertRaisesRegex(m.d.Blocked,'backoff'):dr.run()
            unknown=root.parent/'unknown';ua=m.admission(admission_file,unknown,{})
            def timeout(*_):raise RuntimeError('uncertain provider')
            ur=m.campaign_runner(unknown,ua,ledger,timeout,self.reviewer,fixture=True)
            with self.assertRaises(RuntimeError):ur.run()
            self.assertTrue(ledger.status()['stopped'])
            with self.assertRaises(m.d.Blocked):m.campaign_runner(unknown,ua,ledger,original,self.reviewer,fixture=True).run()
for name in list(dir(FinalTests)):
    if name.startswith('test_') and name not in FinalTests.__dict__:setattr(FinalTests,name,None)
if __name__=='__main__':unittest.main()
