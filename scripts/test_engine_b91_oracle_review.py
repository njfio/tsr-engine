"""Evaluator-authored oracle contrasts. No model inference or Rust target execution.

Labels are independent review inputs, NOT an automated semantic classifier.
Structurally valid arbitrary Rust may still invert or ignore a requirement.
"""
import copy
import json
import unittest
from types import SimpleNamespace
import test_engine_b91_project_envelope as base
import test_engine_b91_driver as driver

# These examples are test-only and never sent to a provider or used as live evidence.
CASES = {
    'valid': ('assert!(parser::parse("2,,5").is_err());', True,
              'A finite empty-field rejection check supports the second aim.'),
    'inverted': ('assert_eq!(parser::parse("2,,5"), Ok(vec![2,5]));', False,
                 'Accepting dropped fields contradicts the second aim.'),
    'vacuous': ('assert!(true);', False,
                'No observation of the parser distinguishes any implementation.'),
    'wrong_aim': ('assert!(parser::parse("").is_ok());', False,
                  'Exactly-empty input alone cannot test the nonempty second aim.'),
}

class OracleReview(unittest.TestCase):
    def setUp(self):
        self.f=base.ProjectEnvelope();self.f.setUp();self.addCleanup(self.f.doCleanups)
        self.request=self.f.controller.request()
    def proposal(self,statement):
        return {'experiment':{'harness':{'tests/tsr_supplemental_oracle.rs':'#[test] fn oracle_probe(){'+statement+'}\n'},
            'cases':[{'id':'oracle_probe','args':[],'expected_stdout':''}],
            'mapping':{'existing_behaviour':{'cases':['oracle_probe'],
                'rationale':'Model claim: checks the requirement. Not proof.',
                'limits':'Authored finite fixture, not a live outcome.'}}}}
    def test_native_deferred_does_not_establish_oracle_correctness(self):
        root=self.request['state_root']
        for name,(statement,approved,reason) in CASES.items():
            with self.subTest(name=name):
                value=self.proposal(statement)
                base.m.validate_trial_payload(value,self.request,'experiment')
                response=base.m.local.bind_proposal(self.request,json.dumps(value))
                verdict=self.f.controller.validate(response)
                self.assertEqual(verdict['assessments'][0]['disposition'],'deferred')
                # Validate is read-only and does not compile, admit or execute any fixture.
                self.assertEqual(self.f.controller.request()['state_root'],root)
                subject={'proposal_record':'f'*64,'proposal':response['proposals'][0]}
                calls=[];seen=[]
                def reviewer(kind,ctx):
                    seen.append(copy.deepcopy(ctx))
                    return {'approved':approved,'state_root':ctx['state_root'],
                        'subject_sha256':ctx['subject_sha256'],
                        'review':{'author':'fixture evaluator','reviewer':'independent fixture reviewer',
                                  'generator':'authored test fixture','review':reason}}
                owner=SimpleNamespace(read=lambda:([],{},{}),fixture=True,reviewer=reviewer,
                    op=lambda key,fn:calls.append(key) or fn())
                if approved:
                    driver.m.Driver.review(owner,self.f.controller,'experiment',subject,'experiment')
                    self.assertEqual(calls,['review_experiment'])
                else:
                    for _ in range(2):
                        with self.assertRaises(driver.m.Blocked):
                            driver.m.Driver.review(owner,self.f.controller,'experiment',subject,'experiment')
                    self.assertEqual(calls,[])
                self.assertEqual(seen[0]['subject'],subject)
                self.assertEqual(seen[0]['subject_sha256'],base.m.sha(subject))
                self.assertEqual(seen[0]['source_snapshot'],response['proposals'][0]['experiment']['snapshot'])
    def test_explanation_cannot_reuse_approval_after_assertion_or_mapping_change(self):
        good=self.proposal(CASES['valid'][0]);bad=self.proposal(CASES['inverted'][0])
        # Identical explanation does not mask changed executable assertions.
        self.assertEqual(good['experiment']['mapping'],bad['experiment']['mapping'])
        for changed in (bad,copy.deepcopy(good)):
            if changed==good:changed['experiment']['mapping']['existing_behaviour']['rationale']='Different unsupported claim'
            original={'proposal':base.m.local.bind_proposal(self.request,json.dumps(good))['proposals'][0]}
            subject={'proposal':base.m.local.bind_proposal(self.request,json.dumps(changed))['proposals'][0]}
            self.assertNotEqual(base.m.sha(original),base.m.sha(subject))
            calls=[]
            def reviewer(kind,ctx):
                return {'approved':True,'state_root':ctx['state_root'],'subject_sha256':base.m.sha(original),
                    'review':{'author':'fixture evaluator','reviewer':'independent reviewer',
                              'generator':'authored fixture','review':'Prior subject only'}}
            owner=SimpleNamespace(read=lambda:([],{},{}),fixture=True,reviewer=reviewer,
                op=lambda key,fn:calls.append(key) or fn())
            with self.assertRaises(driver.m.Blocked):
                driver.m.Driver.review(owner,self.f.controller,'experiment',subject,'experiment')
            self.assertEqual(calls,[])
    def test_case_identity_review_context_and_independent_refusals(self):
        # Static fixtures only. No Rust compiler, target execution or inferred inventory.
        cases=[
            ('root', '#[test] fn probe(){assert!(true);}', 'probe', True),
            ('nested', 'mod inner { #[test] fn probe(){assert!(true);} }', 'inner::probe', True),
            ('file_prefix', '#[test] fn probe(){assert!(true);}', 'tsr_supplemental_oracle::probe', False),
            ('wrong_module', 'mod inner { #[test] fn probe(){assert!(true);} }', 'outer::probe', False),
            ('missing', 'fn helper(){}', 'probe', False),
            ('extra_unlisted', '#[test] fn probe(){assert!(true);} #[test] fn extra(){assert!(true);}', 'probe', False),
            ('duplicate', '#[test] fn probe(){assert!(true);} #[test] fn probe(){assert!(true);}', 'probe', False),
            ('unverifiable_macro', 'make_tests!();', 'probe', False),
        ]
        for label,source,ident,approved in cases:
            proposal=self.proposal('assert!(true);')['experiment']
            proposal['harness']={'tests/tsr_supplemental_oracle.rs':source}
            proposal['cases'][0]['id']=ident
            proposal['mapping']['existing_behaviour']['cases']=[ident]
            subject={'proposal':{'experiment':proposal}}
            frozen=copy.deepcopy(subject);seen=[];calls=[]
            def reviewer(kind,ctx):
                seen.append(ctx)
                return {'approved':approved,'state_root':ctx['state_root'],'subject_sha256':ctx['subject_sha256'],
                        'review':{'author':'fixture author','reviewer':'fixture identity reviewer','generator':'fixture generator',
                                  'review':'Authored identity verdict only; no semantic acceptance or execution.'}}
            owner=SimpleNamespace(read=lambda:([],{},{}),fixture=True,reviewer=reviewer,op=lambda key,fn:calls.append(key) or fn())
            if approved:driver.m.Driver.review(owner,self.f.controller,'experiment',subject,label)
            else:
                with self.assertRaises(driver.m.Blocked):driver.m.Driver.review(owner,self.f.controller,'experiment',subject,label)
                self.assertEqual(calls,[])
            ctx=seen[0];contract=ctx['experiment_review']
            self.assertEqual(contract['existing_case_ids'],[x['id'] for x in self.f.controller.request()['coding']['cases']])
            self.assertEqual(contract['requirements'],base.m.prepare.REQUIREMENTS)
            self.assertIn('abstain',' '.join(contract['checks']))
            self.assertEqual(ctx['subject'],frozen)
            self.assertEqual(ctx['subject_sha256'],base.m.sha(frozen))
            self.assertEqual(subject,frozen)
        # Review criteria describe language identity, not any expected parser result.
        payload=base.m.payload_contract(self.request,'experiment')['experiment']
        self.assertIn('file names are binary target names',' '.join(payload['rules']))

    def test_missing_second_aim_mapping_is_structurally_rejected(self):
        value=self.proposal(CASES['wrong_aim'][0])
        value['experiment']['mapping']={'empty_input':value['experiment']['mapping']['existing_behaviour']}
        with self.assertRaisesRegex(base.m.StageValidationError,'second_aim_mapping_required'):
            base.m.validate_trial_payload(value,self.request,'experiment')

    def test_assembled_context_preserves_normative_requirements_and_separate_source(self):
        fake=self.f.fake()
        _,body=base.m.project(self.f.journal,fake,'experiment')
        ctx=json.loads(body['messages'][1]['content'])
        parents=json.dumps(ctx['parents'])
        self.assertIn('no trimming or dropped fields',parents)
        self.assertIn('empty fields and overflow remain errors',parents)
        self.assertEqual(ctx['source']['path'],'src/lib.rs')
        self.assertEqual(ctx['source']['text'],base.m.prepare.FAILED)
        def strings(value):
            if isinstance(value,str):yield value
            elif isinstance(value,dict):
                for child in value.values():yield from strings(child)
            elif isinstance(value,list):
                for child in value:yield from strings(child)
        texts=[body['messages'][0]['content'],*strings(ctx)]
        self.assertFalse(any(base.m.prepare.TESTS in text for text in texts))
        self.assertEqual(list(ctx),sorted(ctx))  # Actual canonical message ordering, not a recency claim.
        self.assertNotIn('example',ctx['payload_contract']['experiment'])
        for statement,_,_ in CASES.values():self.assertFalse(any(statement in text for text in texts))

if __name__=='__main__':unittest.main()
