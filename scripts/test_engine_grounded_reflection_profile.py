"""Prospective profile mechanics with authored outputs/fake worker, not live semantic evidence."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('grounded_driver',Path(__file__).with_name('engine-b91-driver.py'))
d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d)
suite=d.e.module('grounded_suite','engine-repair-suite.py')

def draft(root,grounded=True,key='unicode-prefix'):
    return d.admission(root,fixture=True,model='gpt-6-luna',final_payload_32k=True,
        schema_corrections=True,review_schema_corrections=True,scenario_id=key,grounded_reflection=grounded)

class Profile(unittest.TestCase):
    def test_admission_origin_restart_and_tamper(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)/'run';a=draft(root);legacy=draft(root,False)
            self.assertNotIn('evidence_profile',legacy)
            self.assertNotIn('engine-repair-response-format.py',legacy['implementation'])
            self.assertEqual(a['version'],'b91-live-admission@v6')
            d.validate_admission(a,root.resolve(),True)
            for field,value in [('version','b91-live-admission@v5'),('evidence_profile','other'),('scenario_id',None)]:
                bad=copy.deepcopy(a);bad[field]=value
                with self.assertRaises(ValueError):d.validate_admission(bad,root.resolve(),True)
            bad=copy.deepcopy(a);bad['implementation'].pop('engine-repair-response-format.py')
            with self.assertRaises(ValueError):d.validate_admission(bad,root.resolve(),True)
            def forbidden(*_):self.fail('no provider/worker/reviewer effects')
            driver=d.Driver(root,a,forbidden,forbidden,forbidden,fixture=True);driver.initialize()
            origin=driver.journal().origin
            self.assertEqual(origin['version'],'b91-project-origin@v3')
            self.assertEqual(origin['evidence_profile'],a['evidence_profile'])
            for field,value in [('version','b91-project-origin@v2'),('evidence_profile','other'),('scenario_id',None)]:
                bad=copy.deepcopy(origin);bad[field]=value
                with self.assertRaises(ValueError):d.e.validate_origin(bad)
            before=(root/'driver.jsonl').read_bytes()
            d.Driver(root,a,forbidden,forbidden,forbidden,fixture=True).initialize()
            self.assertEqual(before,(root/'driver.jsonl').read_bytes())
    def test_response_format_counts_toward_unchanged_payload_bound(self):
        protocol=d.e.module('grounded_format_boundary','engine-repair-response-format.py')
        for stage,limit in [('experiment',16384),('boundary_patch',16384),('relation_after_verification',32768)]:
            body={'model':'gpt-6-luna','stream':False,'max_completion_tokens':8192,'messages':[{'role':'user','content':''}]}
            if stage in ('experiment','boundary_patch'):body['response_format']=protocol.response_format(stage,['requested_behavior','existing_behaviour'])
            size=lambda:len(json.dumps(body,allow_nan=False,ensure_ascii=(limit==16384)).encode())
            body['messages'][0]['content']='x'*(limit-size())
            self.assertEqual(size(),limit);d.e.enforce_payload_bound(body,stage,limit)
            body['messages'][0]['content']+='x'
            with self.assertRaises(d.e.PayloadBoundError):d.e.enforce_payload_bound(body,stage,limit)
    def test_assembled_six_stage_contexts_both_scenarios_and_legacy(self):
        with tempfile.TemporaryDirectory() as tmp:
            for key in suite.TASKS:
                for grounded in (False,True):
                    root=Path(tmp)/(key+str(grounded));a=draft(root,grounded,key)
                    def forbidden(*_):self.fail('no effects')
                    driver=d.Driver(root,a,forbidden,forbidden,forbidden,fixture=True)
                    controller=driver.initialize();journal=driver.journal()
                    for stage in d.e.STAGES:
                        request,body=d.e.project(journal,controller,stage,model='gpt-6-luna',payload_limit=d.stage_payload_limit(a,stage))
                        context=json.loads(body['messages'][1]['content'])
                        self.assertEqual('case_fact_summary' in context['return_evidence'],grounded)
                        self.assertEqual('case_fact_reflection' in context['relational_contract'],grounded)
                        self.assertEqual('response_format' in body,grounded and stage in ('experiment','boundary_patch'))
                        self.assertLessEqual(len(json.dumps(body,ensure_ascii=(d.stage_payload_limit(a,stage)==16384)).encode()),d.stage_payload_limit(a,stage))
                        if grounded:self.assertEqual(context['return_evidence']['case_fact_summary'],d.e.return_evidence.case_fact_summary(request))
                    if grounded:
                        with self.assertRaisesRegex(ValueError,'Luna'):d.e.project(journal,controller,'experiment',model=d.e.MODEL)

class NativeFlow(unittest.TestCase):
    def setup_flow(self,reject=False):
        from test_engine_b91_driver import NativeSequence
        helper=NativeSequence();helper.setUp();self.addCleanup(helper.doCleanups)
        helper.a=draft(helper.root);selected=suite.select('unicode-prefix');seen=[];reviewed=[]
        runtime=d.e.module('grounded_runtime','engine-project-runtime-review.py')
        def provider(body,timeout):
            context=json.loads(body['messages'][1]['content']);seen.append(context)
            self.assertEqual('response_format' in body,context['stage'] in ('experiment','boundary_patch'))
            value=helper.provider(body,timeout)
            if context['stage']=='experiment':
                value['value']['experiment']['harness']={'tests/tsr_supplemental_boundary.rs':'#[test] fn boundary() { assert!(parser::prefix("é",2).is_err()); }\n'}
            if context['stage']=='boundary_patch':
                value['value']['edits'][0]['replacement']=selected.SOURCE # Fake receipt only, not authored repair evidence.
            development=value['value'].get('relational',{}).get('development')
            if development is not None:
                development['return_reveals']=('Every test passed' if reject else context['return_evidence']['case_fact_summary'])
                development['unresolved']='Fixture-only hypothesis remains unverified; no universal semantic claim.'
                # Valid support subset does not determine full-summary content.
                value['value']['return_basis']['observed']=value['value']['return_basis']['observed'][:1]
            return value
        def reviewer(kind,context):
            if kind=='relational' and context['subject'].get('return_basis'):
                facts=context['subject']['return_basis']['case_fact_summary']
                self.assertEqual(facts['summary'],context['subject']['proposal']['proposal']['development']['return_reveals'])
                reviewed.append(facts)
                body,_,_=runtime.build(driver,kind,context,{'fixture_review':True})
                self.assertIn(facts['summary'],body['messages'][1]['content'])
            result=helper.reviewer(kind,context)
            if kind=='semantic_coverage':result['native_review']['evidence']={k:{'met':True,'evidence':'Authored mechanics fixture, not true semantic assessment'} for k in selected.REQUIREMENTS}
            return result
        driver=d.Driver(helper.root,helper.a,provider,helper.worker,reviewer,fixture=True)
        (helper.root/'fixture-only.marker').write_text('b91-native-fixture-only@v1\n')
        return helper,driver,seen,reviewed
    def test_full_five_stage_flow_and_restart(self):
        helper,driver,seen,reviewed=self.setup_flow()
        result=driver.run()
        self.assertEqual(result['status'],'completed_operator_reviewed')
        self.assertEqual([x['stage'] for x in seen],list(d.e.STAGES[:-1]))
        self.assertEqual(len(reviewed),2)
        self.assertEqual(helper.workers,['seed','check_before','patch','check_after'])
        before=(helper.root/'driver.jsonl').read_bytes()
        self.assertEqual(driver.run(),result);self.assertEqual(before,(helper.root/'driver.jsonl').read_bytes())
        self.assertEqual(len(helper.calls),5)
    def test_unsupported_passing_claim_rejected_before_storage(self):
        helper,driver,seen,reviewed=self.setup_flow(reject=True)
        with self.assertRaisesRegex(d.e.StageValidationError,'return_basis_rejected'):driver.run()
        events=driver.journal().events()
        self.assertEqual(len(driver.journal().generated()),2)
        self.assertEqual(events[-1]['kind'],'provider_outcome');self.assertEqual(events[-1]['status'],'invalid')
        self.assertEqual(helper.workers,['seed','check_before'])
        self.assertEqual(len(helper.calls),3);self.assertEqual(reviewed,[])

if __name__=='__main__':unittest.main()
