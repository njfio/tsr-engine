"""Offline temporal evidence fixtures; no live semantic competence claim."""
import copy
import json
import unittest
import test_engine_project_runtime_review as old
r=old.r

class Temporal(old.ProjectReviewBoundaries):
    claim="At proposal creation the protected case failed."
    def setUp(self):
        super().setUp()
        self.proposal={'kind':'experiment','experiment':{'snapshot':'b'*64,'harness':{'tests/check.rs':'fixture assertion'},'cases':[{'id':'case'}],
            'mapping':{'existing_behaviour':{'limits':self.claim}}}}
        self.candidate={'state_root':'a'*64,'proposals':[self.proposal]}
        self.driver.journal().generated=lambda:[self.candidate]
        self.driver.stored=lambda _: 'e'*64
        self.projection['return_evidence']={'cases':[{'case_id':'existing_behaviour_unchanged','outcome':'failed'}]}
        self.projection['observed_returns']=[{'receipt':'1'*64,'verdict':'failed_tests'}]
        r.e.project.return_value=(self.request,{'messages':[{}, {'content':json.dumps(self.projection)}]})
        original={'kind':'experiment','state_root':'a'*64,'source_snapshot':'b'*64,
            'subject':{'proposal_record':'e'*64,'proposal':self.proposal}}
        original['subject_sha256']=r.digest(original['subject'])
        self.original=self.reviewer('experiment',original)
        self.done={'generate_experiment':{'response':copy.deepcopy(self.candidate)},'review_experiment':self.original,'admit_experiment':'d'*64}
        self.driver.read=lambda:([],{},self.done)
        self.request['state_root']='f'*64;self.request['coding']['snapshot']='c'*64
        self.projection['return_evidence']={'cases':[{'case_id':'existing_behaviour_unchanged','outcome':'passed'}]}
        self.projection['observed_returns']=[{'receipt':'2'*64,'verdict':'passed'}]
        r.e.project.return_value=(self.request,{'messages':[{}, {'content':json.dumps(self.projection)}]})
        self.context={'kind':'experiment_readmission','state_root':'f'*64,'source_snapshot':'c'*64,
            'subject':{'previous_admission':'d'*64,'generated_experiment':self.candidate}}
        self.context['subject_sha256']=r.digest(self.context['subject'])
    def build(self):return r.build(self.driver,'experiment_readmission',self.context,self.reviewer.a)
    def test_old_failure_current_pass_separate_no_conclusion(self):
        body,_,binding=self.build();data=json.loads(body['messages'][1]['content'])
        history=next(v for k,v in data['evidence'].items() if k.startswith('experiment_temporal_context:'))
        original=next(v for k,v in history['evidence_at_original_review'].items() if k.startswith('case_catalog:'))
        current=next(v for k,v in data['evidence'].items() if k.startswith('case_catalog:'))
        self.assertEqual(original['cases'][0]['outcome'],'failed');self.assertEqual(current['cases'][0]['outcome'],'passed')
        self.assertEqual(history['original_source_snapshot'],'b'*64);self.assertEqual(history['current_source_snapshot'],'c'*64)
        self.assertEqual(len(binding['observed_receipts']),2)
        self.assertNotIn('Attributed model gate',json.dumps(data));self.assertNotIn('OFFLINE authored gate',json.dumps(data))
        self.assertEqual(data['evidence']['candidate:'+r.digest(self.candidate)],self.candidate)
    def test_false_historical_claim_is_not_rewritten(self):
        # An authored historical lie remains visible beside contrary original evidence;
        # this test does not pretend structural validation judges natural-language truth.
        self.proposal['experiment']['mapping']['existing_behaviour']['limits']='At proposal creation the protected case passed.'
        self.context['subject_sha256']=r.digest(self.context['subject'])
        with self.assertRaisesRegex(ValueError,'lineage drift'):self.build()
    def test_changed_harness_and_prior_admission_rejected(self):
        for key in ('harness','previous_admission'):
            with self.subTest(key=key):
                saved=copy.deepcopy(self.context)
                if key=='harness':self.context['subject']['generated_experiment']['proposals'][0]['experiment']['harness']['tests/check.rs']='changed'
                else:self.context['subject']['previous_admission']='0'*64
                self.context['subject_sha256']=r.digest(self.context['subject'])
                with self.assertRaises(ValueError):self.build()
                self.context=saved;self.candidate=self.context['subject']['generated_experiment']
    def test_original_role_and_outcome_tamper_rejected(self):
        saved=self.original['review']['reviewer'];self.original['review']['reviewer']='unrelated-reviewer'
        with self.assertRaisesRegex(ValueError,'unavailable'):self.build()
        self.original['review']['reviewer']=saved
        path=next(self.reviewer.root.glob('*.outcome.json'));outcome=json.loads(path.read_text());outcome['intent_sha256']='0'*64;path.write_text(json.dumps(outcome))
        with self.assertRaisesRegex(ValueError,'provenance drift'):self.build()
    def test_unicode_review_uses_recorded_validation_digest(self):
        path=next(self.reviewer.root.glob('*.outcome.json'));outcome=json.loads(path.read_text())
        outcome['value']['findings']['epistemic_grounding']['explanation']='Finite evidence — rôle remains bounded.'
        outcome['validation']['candidate_sha256']=r.cycle.digest(outcome['value'])
        self.assertNotEqual(outcome['validation']['candidate_sha256'],r.digest(outcome['value']))
        path.write_text(json.dumps(outcome));self.build()
        outcome['validation']['candidate_sha256']='0'*64;path.write_text(json.dumps(outcome))
        with self.assertRaisesRegex(ValueError,'original experiment review not admitted'):self.build()
    def test_stale_current_source_rejected(self):
        self.context['source_snapshot']='0'*64
        with self.assertRaisesRegex(ValueError,'stale source'):self.build()
    def test_original_receipt_request_tamper_rejected(self):
        path=next(self.reviewer.root.glob('*.request.json'));body=json.loads(path.read_text());data=json.loads(body['messages'][1]['content'])
        key=next(k for k in data['evidence'] if k.startswith('case_catalog:'));data['evidence'][key]['cases'][0]['outcome']='passed'
        body['messages'][1]['content']=json.dumps(data);path.write_text(json.dumps(body))
        with self.assertRaisesRegex(ValueError,'provenance drift'):self.build()
for name in dir(old.ProjectReviewBoundaries):
    if name.startswith('test_') and name not in Temporal.__dict__:setattr(Temporal,name,None)

class FalseHistorical(Temporal):
    claim='At proposal creation the protected case passed.'
    def test_false_original_claim_remains_visible_against_old_failure(self):
        body,_,binding=self.build();data=json.loads(body['messages'][1]['content'])
        history=next(v for k,v in data['evidence'].items() if k.startswith('experiment_temporal_context:'))
        catalog=next(v for k,v in history['evidence_at_original_review'].items() if k.startswith('case_catalog:'))
        self.assertEqual(catalog['cases'][0]['outcome'],'failed')
        self.assertEqual(data['evidence']['candidate:'+r.digest(self.candidate)]['proposals'][0]['experiment']['mapping']['existing_behaviour']['limits'],self.claim)
        # A separately authored negative reviewer finding remains a terminal rejection;
        # no natural-language truth detector or winning verdict is claimed here.
        findings={k:{'finding':'met','basis':'static_source','evidence':[binding['evidence_ids'][0]],'explanation':'fixture','limitations':'fixture'} for k in r.CRITERIA}
        findings['epistemic_grounding']['finding']='not_met'
        self.assertEqual(r.validate({'review_id':binding['review_id'],'verdict':'reject','findings':findings},binding),'reject')
for name in dir(Temporal):
    if name.startswith('test_') and name not in FalseHistorical.__dict__:setattr(FalseHistorical,name,None)
