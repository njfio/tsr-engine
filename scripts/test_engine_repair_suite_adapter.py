"""Pinned suite wiring only: native initialization, zero model or target effects."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from types import SimpleNamespace

spec=importlib.util.spec_from_file_location('suite_driver',Path(__file__).with_name('engine-b91-driver.py'))
d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d)
suite=d.e.module('suite_test_registry','engine-repair-suite.py')

class SuiteAdapter(unittest.TestCase):
    def draft(self,root,scenario_id):
        return d.admission(root,fixture=True,model='gpt-6-luna',final_payload_32k=True,schema_corrections=True,review_schema_corrections=True,scenario_id=scenario_id)
    def test_all_registered_admissions_and_tamper(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=(Path(tmp)/'run').resolve()
            for key in suite.TASKS:
                a=self.draft(root,key);d.validate_admission(a,root,True)
                self.assertEqual(a['version'],'b91-live-admission@v5')
                self.assertIn('engine-repair-suite.py',a['implementation'])
                self.assertEqual(a['evaluation_suite_sha256'],suite.suite_sha256())
                for field,value in [('scenario_id','unknown'),('scenario','0'*64),('evaluation_suite_sha256','0'*64)]:
                    wrong=copy.deepcopy(a);wrong[field]=value
                    with self.assertRaises(ValueError):d.validate_admission(wrong,root,True)
    def test_native_initialize_origin_restart_and_no_effects(self):
        with tempfile.TemporaryDirectory() as tmp:
            for key in suite.TASKS:
                root=Path(tmp)/key;a=self.draft(root,key)
                def forbidden(*_):self.fail('provider/worker/reviewer must not execute')
                driver=d.Driver(root,a,forbidden,forbidden,forbidden,fixture=True)
                controller=driver.initialize();origin=driver.journal().origin
                self.assertEqual(origin['scenario_id'],key)
                self.assertEqual(controller.cli('project','config'),d.e.expected_configuration(suite.select(key)))
                self.assertEqual({p['requirement']:p['meaning'] for p in origin['parents']},suite.select(key).REQUIREMENTS)
                d.e.validate_origin(origin)
                before=(root/'driver.jsonl').read_bytes()
                d.Driver(root,a,forbidden,forbidden,forbidden,fixture=True).initialize()
                self.assertEqual(before,(root/'driver.jsonl').read_bytes())
                wrong=copy.deepcopy(origin);wrong['scenario']='f'*64
                with self.assertRaises(ValueError):d.e.validate_origin(wrong)
                wrong=copy.deepcopy(origin);wrong['scenario_id']=next(k for k in suite.TASKS if k!=key)
                with self.assertRaises(ValueError):d.e.validate_origin(wrong)
    def test_seed_observation_names_follow_registered_scenario(self):
        for key in suite.TASKS:
            selected=suite.select(key)
            request={'coding':{'files':[{'editable':True,'source':selected.FAILED}], 'relational':{'history':[], 'inputs':{'returns':[{'observations':[{'case_id':selected.FIRST_CASE,'passed':True},{'case_id':selected.SECOND_CASE,'passed':False}]}]}}}}
            class ReachedForecast(Exception):pass
            def cli(*_):raise ReachedForecast()
            journal=SimpleNamespace(origin={'scenario_id':key})
            with self.assertRaises(ReachedForecast):d.e.require_stage(journal,SimpleNamespace(cli=cli),request,'relation_after_seed')
            request['coding']['relational']['inputs']['returns'][0]['observations'][0]['case_id']='empty_input_handled'
            with self.assertRaisesRegex(ValueError,'seed return'):d.e.require_stage(journal,SimpleNamespace(cli=cli),request,'relation_after_seed')
    def test_malformed_origin_rejected(self):
        for origin in ({'version':'b91-project-origin@v2'}, {'version':'b91-project-origin@v2','scenario_id':None}, {'scenario_id':[]}):
            with self.assertRaises(ValueError):d.e.validate_origin(origin)
    def test_legacy_default_is_unchanged_and_no_cross_task_global(self):
        expected=d.e.expected_configuration()
        for key in suite.TASKS:
            d.e.expected_configuration(suite.select(key))
            self.assertEqual(d.e.expected_configuration(),expected)
        with tempfile.TemporaryDirectory() as tmp:
            a=d.admission(Path(tmp)/'legacy',fixture=True)
            self.assertNotIn('scenario_id',a);self.assertNotIn('engine-repair-suite.py',a['implementation'])
            d.validate_admission(a,(Path(tmp)/'legacy').resolve(),True)


class SuiteNativeSequence(unittest.TestCase):
    def test_two_complete_authored_sequences_real_native_no_target_commands(self):
        import os
        if not os.environ.get('TSR_B91_TEST_WORKER_BIN'):self.skipTest('explicit cfg(test) worker required')
        from test_engine_b91_driver import NativeSequence
        runtime=d.e.module('suite_runtime_projection','engine-project-runtime-review.py')
        for key in suite.TASKS:
            helper=NativeSequence();helper.setUp()
            try:
                helper.a=d.admission(helper.root,fixture=True,model='gpt-6-luna',final_payload_32k=True,schema_corrections=True,review_schema_corrections=True,scenario_id=key)
                selected=suite.select(key);seen=[];reviewed=[]
                def provider(body,timeout):
                    ctx=json.loads(body['messages'][1]['content']);seen.append(ctx)
                    self.assertEqual({p['requirement']:p['meaning'] for p in ctx['parents']},selected.REQUIREMENTS)
                    self.assertNotIn(selected.TESTS,body['messages'][1]['content'])
                    self.assertNotIn(suite.TASKS[key]['oracle_limits'],body['messages'][1]['content'])
                    self.assertIn('synthetic Rust program',body['messages'][0]['content'])
                    result=helper.provider(body,timeout)
                    if ctx['stage']=='experiment':
                        assertion='assert!(parser::clamp(0,1,0).is_err());' if key=='singleton-clamp' else 'assert!(parser::prefix("é",2).is_err());'
                        result['value']['experiment']['harness']={'tests/tsr_supplemental_boundary.rs':'#[test] fn boundary() { '+assertion+' }\n'}
                    if ctx['stage']=='boundary_patch':
                        # Deliberately NOT a reference repair: fake worker supplies authored
                        # outcomes. This verifies wiring, not task correctness.
                        result['value']['edits'][0]['replacement']=selected.SOURCE
                    return result
                def reviewer(kind,context):
                    if kind=='relational':
                        body,_,_=runtime.build(driver,kind,context,{'fixture_review':True})
                        evidence=json.loads(body['messages'][1]['content'])['evidence']
                        self.assertEqual(evidence['requirements:'+runtime.digest(selected.REQUIREMENTS)],selected.REQUIREMENTS)
                        reviewed.append(kind)
                    result=helper.reviewer(kind,context)
                    if kind=='semantic_coverage':
                        result['native_review']['evidence']={k:{'met':True,'evidence':'Authored fixture only, not a real semantic judgment'} for k in selected.REQUIREMENTS}
                    return result
                driver=d.Driver(helper.root,helper.a,provider,helper.worker,reviewer,fixture=True)
                (helper.root/'fixture-only.marker').write_text('b91-native-fixture-only@v1\n')
                result=driver.run()
                self.assertEqual(result['status'],'completed_operator_reviewed')
                self.assertEqual([x['stage'] for x in seen],list(d.e.STAGES[:-1]))
                self.assertEqual(len(reviewed),3)
                self.assertEqual(seen[0]['source']['text'],selected.FAILED)
                self.assertEqual(seen[0]['cargo_interface']['existing_case_ids'],[selected.FIRST_CASE,selected.SECOND_CASE])
                before=(helper.root/'driver.jsonl').read_bytes()
                self.assertEqual(driver.run(),result)
                self.assertEqual(before,(helper.root/'driver.jsonl').read_bytes())
                self.assertEqual(len(seen),5)
            finally:helper.doCleanups()

if __name__=='__main__':unittest.main()
