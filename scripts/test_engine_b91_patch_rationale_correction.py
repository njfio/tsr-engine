"""Normal driver correction at native patch boundary; authored fixture, no live effects."""
import json
import unittest
import test_engine_b91_driver as base
import test_engine_b91_schema_correction as correction
import test_engine_project_runtime_review as rr
m=base.m

class PatchRationale(correction.Correction):
    def provider(self,body,timeout):
        result=base.NativeSequence.provider(self,body,timeout)
        stage=json.loads(body['messages'][1]['content'])['stage']
        if len(body['messages'])>2:self.feedback.append(body['messages'][-1])
        if stage=='boundary_patch':
            request=m.Native(self.root/'project').request()
            binding=(request['state_root'],request['coding']['snapshot'],(self.root/'project/coding-work/src/lib.rs').read_bytes())
            self.assertEqual(self.workers,['seed','check_before'])
            self.assertNotIn('patch_scope',self.runtime_harness.invoked)
            if self.invalids:
                self.invalids-=1;self.invalid_binding=binding;result['value']['rationale']='x'*592
            else:self.assertEqual(binding,self.invalid_binding)
        return result
    def test_native_patch_rationale_correction_then_reviewed_clean_stop(self):
        d=self.driver()
        harness=rr.ProjectReview();harness.base=self;harness.invoked=[];harness.mode='accept';self.runtime_harness=harness
        path=self.root.parent/'campaign.jsonl';cm=rr.campaign.m
        manifest=cm.manifest('a'*64,path,fixture=True);ledger=cm.Ledger(path,manifest,cm.digest(manifest),create=True)
        trial='b'*64;ledger.enroll(trial,m.e.sha(self.a))
        original=d.transport;worker=d.worker
        d.transport=lambda body,timeout:ledger.run_call(trial,[x for x in d.journal().events() if x['kind']=='dispatch_intent'][-1]['id'],body,timeout,original,payload_limit=m.stage_payload_limit(self.a,json.loads(body['messages'][1]['content'])['stage']))
        d.worker=lambda c,stage:ledger.run_execution(trial,stage,m.e.sha(self.a),lambda:worker(c,stage))
        d.reviewer=rr.r.RuntimeReviewer(d,rr.r.admission(self.a,manifest),ledger,harness.transport,self.reviewer)
        r=d.run()
        self.assertEqual(r['calls'],6);self.assertEqual(r['resources']['model_calls'],6)
        self.assertEqual(ledger.status()['calls'],12);self.assertEqual(ledger.status()['execution_groups'],4)
        self.assertEqual(len(harness.invoked),6)
        self.assertEqual(len(self.workers),4);self.assertEqual(len(self.feedback),1)
        self.assertIn('invalid_rationale_length',self.feedback[0]['content'])
        self.assertEqual(sum(x['kind']=='schema_correction' for x in d.journal().events()),1)
        self.assertEqual(harness.invoked.count('patch_scope'),1)
        self.assertEqual(r['final_proposal_status'],'skipped_after_independent_acceptance')
        self.assertNotIn('final_proposal_only',self.calls)
        self.assertEqual(self.calls.count('boundary_patch'),2)
        ledger_before=ledger.path.read_bytes()
        before=(len(self.calls),len(self.workers));self.driver().run();self.assertEqual(before,(len(self.calls),len(self.workers)))
        self.assertEqual(ledger.path.read_bytes(),ledger_before)

for name in dir(correction.Correction):
    if name.startswith('test_') and name not in PatchRationale.__dict__:setattr(PatchRationale,name,None)
