"""Prospective bounded structural reviewer correction, fixture-only/no live effects."""
import json
import unittest
import test_engine_project_runtime_review as old
import test_engine_b91_patch_rationale_correction as patch
r=old.r

class Boundaries(old.ProjectReviewBoundaries):
    def setUp(self):
        super().setUp()
        self.driver.admission.update(version='b91-live-admission@v4',schema_corrections=2,review_schema_corrections=1,shared_schema_corrections=2)
        self.invalid_remaining=1;self.feedback=[];self.behavior='invalid_then_valid'
        self.reviewer=r.RuntimeReviewer(self.driver,r.admission(self.driver.admission,self.ledger.manifest),self.ledger,self.provider,lambda *_:None)
    def provider(self,body,timeout):
        result=super().provider(body,timeout)
        payload=json.loads(body['messages'][1]['content'])
        if 'structural_correction' in payload:self.feedback.append(payload['structural_correction'])
        result['receipt'].update(response_status='received',finish_reason='stop')
        if self.behavior=='timeout':raise TimeoutError('fixture')
        if self.behavior=='unknown_cost':result['receipt']['usd']=None
        if self.behavior in ('reject','malformed_reject'):
            result['value']['verdict']='reject';result['value']['findings']['scope_integrity']['finding']='not_met'
            if self.behavior=='malformed_reject':result['value']['findings']['epistemic_grounding']['evidence']=['unlisted:fixture']
        elif self.invalid_remaining:
            self.invalid_remaining-=1;result['value']['findings']['epistemic_grounding']['evidence']=['unlisted:fixture']
        if self.behavior=='malformed_unknown_finding':result['value']['findings']['scope_integrity']['finding']='unknown'
        return result
    def recreate(self):return r.RuntimeReviewer(self.driver,r.admission(self.driver.admission,self.ledger.manifest),self.ledger,self.provider,lambda *_:None)
    def test_invalid_valid_replay(self):
        self.assertTrue(self.reviewer('patch_scope',self.context)['approved'])
        self.assertEqual(self.calls,2);self.assertEqual(r.correction_count(self.driver.root),1)
        self.assertEqual(self.feedback[0]['validation_code'],'schema_enum')
        before=self.ledger.path.read_bytes();self.recreate()('patch_scope',self.context)
        self.assertEqual(self.calls,2);self.assertEqual(before,self.ledger.path.read_bytes())
    def test_repeated_invalid_exhausted(self):
        self.invalid_remaining=3
        for _ in range(2):
            with self.assertRaisesRegex(r.ReviewStopped,'invalid report'):self.recreate()('patch_scope',self.context)
        self.assertEqual(self.calls,2)
    def test_shared_generator_exhaustion(self):
        self.driver.journal().events=lambda:[{'kind':'dispatch_intent','stage':'boundary_patch'},{'kind':'schema_correction'},{'kind':'schema_correction'}]
        with self.assertRaisesRegex(r.ReviewStopped,'shared correction'):self.reviewer('patch_scope',self.context)
        self.assertEqual(self.calls,1);self.assertEqual(r.correction_count(self.driver.root),0)
    def test_review_correction_blocks_third_shared_generator_correction(self):
        from types import SimpleNamespace
        self.reviewer('patch_scope',self.context)
        events=[{'kind':'schema_correction','id':'previous'},
                {'kind':'dispatch_intent','stage':'boundary_patch','id':'failed','state_root':'a'*64},
                {'kind':'provider_outcome','id':'failed','status':'invalid','validation':{'code':'invalid_rationale_length'}}]
        driver=object.__new__(r.d.Driver);driver.root=self.driver.root;driver.admission=self.driver.admission
        driver.read=lambda:([],{},{});driver.check_resources=lambda:None
        driver.journal=lambda:SimpleNamespace(events=lambda:events,append=lambda _:self.fail('third shared correction reserved'))
        with self.assertRaisesRegex(r.d.Blocked,'shared correction allowance exhausted'):
            driver._generate(None,'boundary_patch')
        self.assertEqual(self.calls,2)
    def test_semantic_reject_no_correction(self):
        self.behavior='reject'
        with self.assertRaisesRegex(r.ReviewStopped,'review reject'):self.reviewer('patch_scope',self.context)
        self.assertEqual(self.calls,1);self.assertEqual(r.correction_count(self.driver.root),0)
    def test_malformed_negative_not_corrected(self):
        self.behavior='malformed_reject'
        with self.assertRaisesRegex(r.ReviewStopped,'invalid report'):self.reviewer('patch_scope',self.context)
        self.assertEqual(self.calls,1);self.assertEqual(r.correction_count(self.driver.root),0)
    def test_malformed_unknown_finding_not_corrected(self):
        self.behavior='malformed_unknown_finding'
        with self.assertRaisesRegex(r.ReviewStopped,'invalid report'):self.reviewer('patch_scope',self.context)
        self.assertEqual(self.calls,1);self.assertEqual(r.correction_count(self.driver.root),0)
    def test_unknown_cost_no_correction(self):
        self.behavior='unknown_cost'
        with self.assertRaisesRegex(r.ReviewStopped,'usage/cost'):self.reviewer('patch_scope',self.context)
        self.assertEqual(self.calls,1);self.assertEqual(r.correction_count(self.driver.root),0)
    def test_correction_timeout_restart_no_repeat(self):
        original=self.provider
        def provider(body,timeout):
            if 'structural_correction' in json.loads(body['messages'][1]['content']):self.behavior='timeout'
            return original(body,timeout)
        self.reviewer.transport=provider
        with self.assertRaises(TimeoutError):self.reviewer('patch_scope',self.context)
        with self.assertRaisesRegex(r.ReviewStopped,'dispatch unknown'):self.recreate()
        self.assertEqual(self.calls,2)
    def test_correction_marker_tamper(self):
        self.reviewer('patch_scope',self.context)
        path=next(self.reviewer.root.glob('*.correction.json'));value=json.loads(path.read_text());value['driver_admission_sha256']='f'*64;path.write_text(json.dumps(value))
        with self.assertRaisesRegex(ValueError,'correct'):self.recreate()
    def test_deleted_marker_cannot_refund_shared_slot(self):
        self.reviewer('patch_scope',self.context)
        next(self.reviewer.root.glob('*.correction.json')).unlink()
        with self.assertRaises(ValueError):r.correction_count(self.driver.root)
        self.assertEqual(self.calls,2)
    def test_stale_after_invalid_no_correction(self):
        original=self.provider
        def provider(body,timeout):
            result=original(body,timeout);self.request['state_root']='f'*64;return result
        self.reviewer.transport=provider
        with self.assertRaisesRegex(r.ReviewStopped,'source drift'):self.reviewer('patch_scope',self.context)
        self.assertEqual(self.calls,1);self.assertEqual(r.correction_count(self.driver.root),0)
for name in dir(old.ProjectReviewBoundaries):
    if name.startswith('test_') and name not in Boundaries.__dict__:setattr(Boundaries,name,None)

class Native(patch.PatchRationale):
    def setUp(self):
        super().setUp()
        self.a=patch.m.admission(self.root,fixture=True,model='gpt-6-luna',final_payload_32k=True,schema_corrections=True,review_schema_corrections=True)
    def test_native_shared_corrections_and_clean_stop(self):
        d=self.driver();harness=old.ProjectReview();harness.base=self;harness.invoked=[];harness.mode='accept';self.runtime_harness=harness
        cm=old.campaign.m;path=self.root.parent/'campaign.jsonl';manifest=cm.manifest('a'*64,path,fixture=True);ledger=cm.Ledger(path,manifest,cm.digest(manifest),create=True)
        trial='b'*64;ledger.enroll(trial,patch.m.e.sha(self.a));original=d.transport;worker=d.worker
        d.transport=lambda body,timeout:ledger.run_call(trial,[x for x in d.journal().events() if x['kind']=='dispatch_intent'][-1]['id'],body,timeout,original,payload_limit=patch.m.stage_payload_limit(self.a,json.loads(body['messages'][1]['content'])['stage']))
        d.worker=lambda c,stage:ledger.run_execution(trial,stage,patch.m.e.sha(self.a),lambda:worker(c,stage))
        def review(body,timeout):
            result=harness.transport(body,timeout);payload=json.loads(body['messages'][1]['content'])
            if payload['binding']['kind']=='experiment_readmission' and 'structural_correction' not in payload:
                result['value']['findings']['epistemic_grounding']['evidence']=['not-a-recorded-reference']
            return result
        d.reviewer=r.RuntimeReviewer(d,r.admission(self.a,manifest),ledger,review,self.reviewer)
        result=d.run()
        self.assertEqual(result['calls'],6);self.assertEqual(len(harness.invoked),7);self.assertEqual(ledger.status()['calls'],13)
        self.assertEqual(ledger.status()['execution_groups'],4);self.assertEqual(r.correction_count(self.root),1)
        self.assertEqual(sum(x['kind']=='schema_correction' for x in d.journal().events()),1)
        self.assertEqual(result['final_proposal_status'],'skipped_after_independent_acceptance')
        before=ledger.path.read_bytes();self.assertEqual(self.driver().run(),result);self.assertEqual(ledger.path.read_bytes(),before)
for name in dir(patch.PatchRationale):
    if name.startswith('test_') and name not in Native.__dict__:setattr(Native,name,None)
