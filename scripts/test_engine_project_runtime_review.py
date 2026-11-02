"""Authored reviewer fixtures plus native controller/fake worker; no target execution."""
import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path
import test_engine_b91_driver as h
import test_engine_b91_campaign as campaign
spec=importlib.util.spec_from_file_location('runtime_project_review',Path(__file__).with_name('engine-project-runtime-review.py'))
r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)

class ProjectReview(unittest.TestCase):
    def setUp(self):
        self.base=h.NativeSequence(methodName='test_full_native_sequence');self.base.setUp();self.addCleanup(self.base.doCleanups)
        self.base.a=h.m.admission(self.base.root,fixture=True,model='gpt-6-luna',final_payload_32k=True)
        path=self.base.root.parent/'campaign.jsonl';manifest=campaign.m.manifest('c'*64,path,fixture=True)
        self.ledger=campaign.m.Ledger(path,manifest,campaign.m.digest(manifest),create=True)
        self.invoked=[];self.mode='accept';self.final_calls=[]
    def transport(self,body,timeout):
        self.assertEqual(timeout,240);payload=json.loads(body['messages'][1]['content']);binding=payload['binding']
        self.invoked.append(binding['kind']);self.assertNotIn(h.m.e.prepare.TESTS,json.dumps(payload))
        self.assertNotIn('Authored mechanics fixture',json.dumps(payload))
        findings={k:{'finding':'met','basis':'static_source','evidence':[binding['evidence_ids'][0]],
                     'explanation':'OFFLINE authored gate fixture','limitations':'No live semantic correctness claim'} for k in payload['criteria']}
        if self.mode=='reject':findings['epistemic_grounding']['finding']='not_met'
        if self.mode=='unknown':findings['epistemic_grounding'].update(finding='unknown',basis='unknown')
        if self.mode=='timeout':raise TimeoutError('fixture interrupted review')
        return {'value':{'review_id':binding['review_id'],'verdict':self.mode,'findings':findings},
                'receipt':{'status':'settled','usd':'0.001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'response_status':'received','finish_reason':'stop','elapsed_seconds':0.01}}
    def make(self):
        driver=self.base.driver()
        def final(kind,context):self.final_calls.append(kind);return self.base.reviewer(kind,context)
        driver.reviewer=r.RuntimeReviewer(driver,r.admission(self.base.a,self.ledger.manifest),self.ledger,self.transport,final)
        return driver
    def test_full_callback_path_and_restart(self):
        driver=self.make();result=driver.run()
        self.assertEqual(result['calls'],5);self.assertEqual(result['execution_groups'],4)
        self.assertEqual(len(self.invoked),6);self.assertEqual(self.final_calls,['semantic_coverage'])
        self.assertEqual(self.ledger.status()['calls'],6);self.assertEqual(self.ledger.status()['known_cost_microusd'],6000)
        before=self.ledger.path.read_bytes();again=self.make().run();self.assertEqual(again,result)
        self.assertEqual(self.ledger.path.read_bytes(),before);self.assertEqual(len(self.invoked),6)
    def negative(self,mode):
        self.mode=mode
        with self.assertRaisesRegex(r.ReviewStopped,'project runtime review '+self.mode):self.make().run()
        self.assertEqual(len(self.invoked),1);self.assertEqual(self.base.workers,['seed'])
        self.assertEqual(self.make().controller().request()['coding']['relational']['history'],[])
        with self.assertRaisesRegex(r.ReviewStopped,'project runtime review '+self.mode):self.make().run()
        self.assertEqual(len(self.invoked),1)
    def test_negative_does_not_admit_relation(self):self.negative('reject')
    def test_unknown_does_not_admit_relation(self):self.negative('unknown')
    def test_uncertain_review_never_retries(self):
        self.mode='timeout'
        with self.assertRaises(TimeoutError):self.make().run()
        self.mode='accept'
        with self.assertRaisesRegex(r.ReviewStopped,'dispatch unknown'):self.make().run()
        self.assertEqual(len(self.invoked),1);self.assertTrue(self.ledger.status()['unknown_calls'])

if __name__=='__main__':unittest.main()

class ProjectReviewBoundaries(unittest.TestCase):
    """Pure callback boundary fixtures; actual native path is tested above."""
    def setUp(self):
        from types import SimpleNamespace
        from unittest.mock import patch
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);root=Path(self.tmp.name)
        self.candidate={'proposals':[{'kind':'edit_set','patches':[]}]}
        self.context={'kind':'patch_scope','state_root':'a'*64,'source_snapshot':'b'*64,'subject':self.candidate,'subject_sha256':r.digest(self.candidate)}
        self.projection={'source':{'path':'src/lib.rs','text':h.m.e.prepare.SOURCE},'parents':[], 'relational_history':[], 'cargo_interface':{'existing_case_ids':['empty_input_handled','existing_behaviour_unchanged']},'constraints':{'patch_path':'src/lib.rs'},'observed_returns':[], 'supplemental_returns':[], 'return_evidence':{}}
        self.request={'state_root':'a'*64,'coding':{'snapshot':'b'*64}}
        journal=SimpleNamespace(events=lambda:[{'kind':'dispatch_intent','stage':'boundary_patch'}],generated=lambda:[self.candidate])
        controller=SimpleNamespace(request=lambda:self.request)
        self.driver=SimpleNamespace(root=root,identity='c'*64,fixture=True,admission={'model':'gpt-6-luna','expires_at':'2099-01-01T00:00:00+00:00'},journal=lambda:journal,controller=lambda:controller)
        self.patch=patch.object(r.e,'project',return_value=(self.request,{'messages':[{}, {'content':json.dumps(self.projection)}]}));self.patch.start();self.addCleanup(self.patch.stop)
        self.mode='accept';self.calls=0
        path=root/'ledger';manifest=campaign.m.manifest('d'*64,path,fixture=True);self.ledger=campaign.m.Ledger(path,manifest,campaign.m.digest(manifest),create=True)
        self.final=[]
        self.reviewer=r.RuntimeReviewer(self.driver,r.admission(self.driver.admission,self.ledger.manifest),self.ledger,self.provider,lambda k,c:self.final.append(k))
    def provider(self,body,timeout):
        self.calls+=1;data=json.loads(body['messages'][1]['content']);b=data['binding']
        findings={k:{'finding':'met','basis':'static_source','evidence':[b['evidence_ids'][0]],'explanation':'fixture','limitations':'fixture'} for k in r.CRITERIA}
        if self.mode=='wrong_receipt':findings['epistemic_grounding'].update(basis='observed_test_receipt',evidence=['test_receipt:'+'f'*64])
        if self.mode=='malformed':findings.pop('parent_fidelity')
        usage={'prompt_tokens':100,'completion_tokens':10,'total_tokens':110}
        if self.mode=='missing_usage':usage={}
        if self.mode=='invalid_usage':usage['prompt_tokens_details']={'cached_tokens':101}
        return {'value':{'review_id':b['review_id'],'verdict':'accept','findings':findings},'receipt':{'status':'settled','usd':'0.001','usage':usage,'elapsed_seconds':0.01}}
    def test_stale_and_ungenerated_source_before_dispatch(self):
        import copy
        for key in ('state_root','source_snapshot','subject_sha256'):
            value=copy.deepcopy(self.context);value[key]='f'*64
            with self.assertRaises(ValueError):self.reviewer('patch_scope',value)
        changed=copy.deepcopy(self.context);changed['subject']={'proposals':[]};changed['subject_sha256']=r.digest(changed['subject'])
        with self.assertRaises(ValueError):self.reviewer('patch_scope',changed)
        self.assertEqual(self.calls,0)
    def test_malformed(self):
        self.mode='malformed'
        with self.assertRaisesRegex(r.ReviewStopped,'invalid report'):self.reviewer('patch_scope',self.context)
    def test_wrong_receipt(self):
        self.mode='wrong_receipt'
        with self.assertRaisesRegex(r.ReviewStopped,'invalid report'):self.reviewer('patch_scope',self.context)
    def test_missing_usage(self):
        self.mode='missing_usage'
        with self.assertRaisesRegex(r.ReviewStopped,'usage/cost'):self.reviewer('patch_scope',self.context)
    def test_invalid_usage(self):
        self.mode='invalid_usage'
        with self.assertRaisesRegex(r.ReviewStopped,'usage/cost'):self.reviewer('patch_scope',self.context)
    def test_wrong_campaign_rejected(self):
        wrong=r.admission(self.driver.admission,self.ledger.manifest);wrong['campaign_manifest_sha256']='f'*64
        with self.assertRaises(ValueError):r.RuntimeReviewer(self.driver,wrong,self.ledger,self.provider,lambda *_:None)
        self.assertEqual(self.calls,0)

    def test_budget_and_final_delegation(self):
        for n in range(6):(self.reviewer.root/f'{n}.intent.json').write_text('{}')
        with self.assertRaisesRegex(r.ReviewStopped,'call budget'):self.reviewer('patch_scope',self.context)
        self.assertEqual(self.calls,0)
        self.reviewer('semantic_coverage',{'no_model_candidate_required':True});self.assertEqual(self.final,['semantic_coverage'])
