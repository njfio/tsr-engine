"""Real native validation and assembled fixture prompts; zero inference/target execution."""
import copy,importlib.util,json,tempfile,unittest,subprocess,sys
from pathlib import Path
from unittest.mock import patch
import test_engine_b91_project_envelope as base
import test_engine_b91_oracle_review as oracle
import test_engine_b91_driver as driver
from types import SimpleNamespace
s=importlib.util.spec_from_file_location('shadow',Path(__file__).with_name('engine-b91-shadow.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Shadow(unittest.TestCase):
    def setUp(self):
        self.f=base.ProjectEnvelope();self.f.setUp();self.addCleanup(self.f.doCleanups)
        fake=self.f.fake();self.original,self.body=base.m.project(self.f.journal,fake,'experiment')
        self.tmp=tempfile.TemporaryDirectory(dir=m.ROOT/'target');self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name)/'run'
        mocked=patch.object(m,'frozen_input',side_effect=lambda:(copy.deepcopy(self.original),copy.deepcopy(self.body)));mocked.start();self.addCleanup(mocked.stop)
        self.identity=m.prepare(self.root)
        self.v=m.ValidationOnly(self.root)
    def value(self,statement='assert!(parser::parse("2,,5").is_err());'):
        return {'experiment':{'harness':{'tests/tsr_supplemental_probe.rs':'#[test] fn probe(){'+statement+'}'},'cases':[{'id':'probe','args':[],'expected_stdout':''}],'mapping':{'existing_behaviour':{'cases':['probe'],'rationale':'Authored finite fixture only.','limits':'Not a universal proof or model outcome.'}}}}
    def reply(self,value):return {'value':value,'receipt':{'status':'settled','usd':'0.001','response_status':'received','finish_reason':'stop','usage':{'prompt_tokens':100,'completion_tokens':200,'total_tokens':300}}}
    def test_full_six_request_fake_transport_real_validator_and_restart_no_dispatch(self):
        sent=[]
        def transport(body,deadline,expected_model):
            self.assertEqual(body['model'],expected_model);self.assertEqual(deadline,240)
            ctx=json.loads(body['messages'][1]['content']);self.assertEqual(ctx['source']['text'],m.e.prepare.FAILED)
            self.assertNotIn('shadow_control',body['messages'][1]['content'])
            sent.append(expected_model);return self.reply(self.value())
        before=self.v.c.request()['state_root'];m.run(self.root,self.identity,fixture=True,transport=transport)
        self.assertEqual(sent,list(m.MODELS)+list(reversed(m.MODELS)))
        self.assertEqual(len(list(self.root.glob('candidate-*.json'))),6)
        self.assertEqual(self.v.c.request()['state_root'],before)
        self.assertFalse(self.v.c.cli('project-budget')['active'])
        with self.assertRaisesRegex(ValueError,'existing dispatch'):m.run(self.root,self.identity,fixture=True,transport=transport)
        self.assertEqual(len(sent),6)
        out=subprocess.run([sys.executable,str(Path(m.__file__)),'inspect','--directory',str(self.root)],capture_output=True,text=True,check=True)
        self.assertFalse(json.loads(out.stdout)['executed'])
    def test_requested_models_keep_messages_and_use_explicit_limits(self):
        root=Path(self.tmp.name)/'requested'
        identity=m.prepare(root,('deepseek-v4.1-flash-1017','gpt-6-luna'))
        sent=[]
        def transport(body,deadline,expected_model):
            self.assertEqual(body['model'],expected_model)
            self.assertEqual(body['messages'],self.body['messages'])
            key='max_completion_tokens' if expected_model=='gpt-6-luna' else 'max_tokens'
            self.assertEqual(body[key],8192)
            self.assertNotIn('max_tokens' if key=='max_completion_tokens' else 'max_completion_tokens',body)
            sent.append(expected_model);return self.reply(self.value())
        m.run(root,identity,fixture=True,transport=transport)
        self.assertEqual(sent,['deepseek-v4.1-flash-1017','gpt-6-luna','gpt-6-luna','deepseek-v4.1-flash-1017'])
        self.assertEqual(len(list(root.glob('candidate-*.json'))),4)

    def test_real_control_and_negative_schema_semantic_boundaries(self):
        self.assertEqual(self.v.control()['structural_control'],'deferred')
        root=self.v.c.request()['state_root']
        for name,(statement,approved,reason) in oracle.CASES.items():
            result=self.v.validate(self.value(statement))
            self.assertEqual(result['semantic_status'],'unreviewed')
            # Independent authored verdict, not model or native semantic classification.
            calls=[]
            def reviewer(kind,ctx):
                return {'approved':approved,'state_root':ctx['state_root'],'subject_sha256':ctx['subject_sha256'],
                        'review':{'author':'fixture evaluator','reviewer':'independent fixture reviewer','generator':'authored test fixture','review':reason}}
            owner=SimpleNamespace(read=lambda:([],{},{}),fixture=True,reviewer=reviewer,op=lambda key,fn:calls.append(key) or fn())
            subject={'proposal':result['experiment']}
            if approved:driver.m.Driver.review(owner,self.v.c,'experiment',subject,'oracle')
            else:
                with self.assertRaises(driver.m.Blocked):driver.m.Driver.review(owner,self.v.c,'experiment',subject,'oracle')
                self.assertEqual(calls,[])
            self.assertEqual(self.v.c.request()['state_root'],root)
        bad=self.value();bad['experiment']['mapping']={}
        with self.assertRaisesRegex(m.e.StageValidationError,'second_aim_mapping_required'):self.v.validate(bad)
        bad=self.value();bad['experiment']['harness']={'tests/contract.rs':'invalid protected replacement'}
        with self.assertRaises(ValueError):self.v.validate(bad)
        self.assertEqual(self.v.control()['structural_control'],'deferred')
    def test_expiry_and_source_drift_stop_before_transport(self):
        calls=[]
        with patch.object(m.time,'time',return_value=self.v.binding['expires_at']-269):
            with self.assertRaises(ValueError):m.run(self.root,self.identity,fixture=True,transport=lambda *a,**k:calls.append(1))
        self.assertEqual(calls,[]);self.assertFalse((self.root/'events.jsonl').exists())
        p=self.root/'validator/coding-work/src/lib.rs'
        # Native source paths are discovered from the request rather than guessed.
        work=list((self.root/'validator').rglob('src/lib.rs'))
        self.assertTrue(work);work[0].write_text('changed')
        with self.assertRaises(ValueError):m.run(self.root,self.identity,fixture=True,transport=lambda *a,**k:calls.append(1))
        self.assertEqual(calls,[])
    def test_preflight_consuming_runway_stops_before_intent(self):
        manifest=json.loads((self.root/'manifest.json').read_text());sent=[]
        initial=m.time.time()
        clock=[initial]
        original=m.ValidationOnly.control
        def slow_control(validator):
            result=original(validator)
            clock[0]=manifest['expires_at']-269
            return result
        with patch.object(m.time,'time',side_effect=lambda:clock[0]), patch.object(m.ValidationOnly,'control',slow_control):
            with self.assertRaisesRegex(ValueError,'run wall runway exhausted after preflight'):
                m.run(self.root,self.identity,fixture=True,transport=lambda *a,**k:sent.append(1))
        self.assertEqual(sent,[]);self.assertFalse((self.root/'events.jsonl').exists())
        self.assertIn(str(m.ROOT/'scripts/engine-b91-evidence.py'),manifest['implementation'])

    def test_even_rehashed_private_payload_stops_before_transport(self):
        manifest=json.loads((self.root/'manifest.json').read_text());model=m.MODELS[0];p=self.root/(model+'.json')
        body=json.loads(p.read_text());body['messages'][1]['content']='PRIVATE injected source';p.write_text(json.dumps(body))
        manifest['bodies'][model]=m.file_sha(p);(self.root/'manifest.json').write_text(json.dumps(manifest));sent=[]
        with self.assertRaisesRegex(ValueError,'non-allowlisted payload'):
            m.run(self.root,m.e.sha(manifest),fixture=True,transport=lambda *a,**k:sent.append(1))
        self.assertEqual(sent,[]);self.assertFalse((self.root/'events.jsonl').exists())

    def test_unknown_dispatch_is_not_retried_and_qwen_live_is_blocked(self):
        manifest=json.loads((self.root/'manifest.json').read_text());manifest['live_authorized']=True
        (self.root/'manifest.json').write_text(json.dumps(manifest));identity=m.e.sha(manifest)
        calls=[]
        with self.assertRaisesRegex(ValueError,'Qwen total-token'):m.run(self.root,identity,allow_paid=True,transport=lambda *a,**k:calls.append(1))
        self.assertEqual(calls,[])
        def fail(*a,**k):calls.append(1);raise TimeoutError('authored unknown')
        with self.assertRaises(TimeoutError):m.run(self.root,identity,fixture=True,transport=fail)
        with self.assertRaisesRegex(ValueError,'existing dispatch'):m.run(self.root,identity,fixture=True,transport=fail)
        self.assertEqual(len(calls),1)
if __name__=='__main__':unittest.main()
