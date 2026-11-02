"""Full driver mechanics with authored provider fixtures and cfg(test) native worker.
No model discovery, Cargo source execution, real inference, or live B9.1 evidence.
"""
import copy
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
spec=importlib.util.spec_from_file_location('b91_driver',Path(__file__).with_name('engine-b91-driver.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

# Evaluator-authored fixture response, deliberately confined to test source.
PATCH='''pub fn parse(input: &str) -> Result<Vec<i64>, String> {
    if input.is_empty() { return Ok(vec![]); }
    input.split(',').map(|field| field.parse::<i64>().map_err(|_| "invalid".to_owned())).collect()
}
'''
class NativeSequence(unittest.TestCase):
    def setUp(self):
        if not os.environ.get('TSR_B91_TEST_WORKER_BIN'):self.skipTest('requires explicit cfg(test) native worker binary')
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name)/'trial'
        self.calls=[];self.workers=[];self.reviews=[]
        self.a=m.admission(self.root,fixture=True)
    def provider(self,body,timeout):
        ctx=json.loads(body['messages'][1]['content']);stage=ctx['stage'];self.calls.append(stage)
        self.assertEqual(timeout,240);self.assertEqual(body['model'],self.a['model'])
        key='max_completion_tokens' if self.a['model']=='gpt-6-luna' else 'max_tokens'
        self.assertEqual(body[key],8192)
        self.assertNotIn('max_tokens' if key=='max_completion_tokens' else 'max_completion_tokens',body)
        self.assertEqual(ctx['cargo_interface']['library_import'],'parser')
        self.assertEqual(ctx['payload_contract'],m.e.payload_contract({},stage))
        self.assertLessEqual(len(json.dumps(body,ensure_ascii=(m.stage_payload_limit(self.a,stage)==16384)).encode()),m.stage_payload_limit(self.a,stage))
        if stage=='experiment':
            value={'experiment':{'harness':{'tests/tsr_supplemental_boundary.rs':'#[test] fn boundary() { assert!(parser::parse("1,,2").is_err()); }\n'},
                'cases':[{'id':'boundary','args':[],'expected_stdout':''}],
                'mapping':{'existing_behaviour':{'cases':['boundary'],'rationale':'Authored finite fixture check','limits':'Finite case only'}}}}
        elif stage=='boundary_patch':value={'edits':[{'path':'src/lib.rs','replacement':PATCH}],'rationale':'Authored fixture boundary repair, not model output'}
        else:
            history=ctx['relational_history'];prior=history[-1] if history else None
            value={'relational':{'parents':[x['id'] for x in ctx['parents']],
                'relationship':'Authored protocol fixture relationship','aspiration':'Investigate compatible boundary behavior',
                'discriminator':'Finite second-aim observation, not universal proof','revises':None,'catalysts':[],
                'development':None}}
            if prior:
                p=prior['proposal'];first=history[0]['admission']
                value['relational'].update(relationship=p['relationship'],aspiration=p['aspiration'],discriminator=p['discriminator'],
                    revises=prior['admission'],development={'original_revision':first,'prior_revision':prior['admission'],
                    'retained_meaning':p['aspiration'],'return_reveals':'Authored fixture interpretation of the available finite return',
                    'unresolved':'Universal equivalence and semantic fulfillment are not established','proposed_development':None})
        if stage=='final_proposal_only':
            # A novel proposal is preserved but not admitted/executed; unchanged evidence
            # does not justify a second relational revision.
            value={'edits':[{'path':'src/lib.rs','replacement':PATCH+'// fixture unexecuted possibility\n'}],
                   'rationale':'Fixture-only final alternative; no action authority'}
        if value.get('relational',{}).get('revises'):
            value['return_basis']={'binding':ctx['return_evidence']['binding'],'observed':[x['id'] for x in ctx['return_evidence']['cases']][:8],
                'inferred':[],'unknown':['Fixture assertion execution unknown']}
        return {'value':value,'receipt':{'response_status':'received','structural_status':'typed_json','status':'settled',
            'usd':'0.001','usage':{'prompt_tokens':100,'completion_tokens':200,'total_tokens':300},
            'elapsed_seconds':.1,'request_id':'fixture-'+str(len(self.calls))}}
    def worker(self,c,stage):
        self.workers.append(stage);out=self.root/('worker-'+stage+'.json')
        env={**os.environ,'TSR_B91_FIXTURE_DIRECTORY':str(c.directory),'TSR_B91_FIXTURE_OUTPUT':str(out),'TSR_B91_FIXTURE_STAGE':stage}
        result=subprocess.run([os.environ['TSR_B91_TEST_WORKER_BIN'],'--exact','engine::adaptive::coding::b91_fixture::fixture_worker','--nocapture'],
            env=env,capture_output=True,text=True,timeout=60)
        self.assertEqual(result.returncode,0,result.stdout+result.stderr)
        value=json.loads(out.read_text());self.assertEqual(value['target_commands_executed'],0)
        return value
    def reviewer(self,kind,context):
        self.reviews.append(kind)
        result={'approved':True,'state_root':context['state_root'],'subject_sha256':context['subject_sha256'],
            'review':{'author':'fixture author','reviewer':'fixture independent role','generator':'fixture generator',
                      'review':'Authored mechanics fixture, not real independent semantic acceptance'}}
        if kind=='semantic_coverage':
            status=context['subject']['status']['goal_completion']
            result['native_review']={'snapshot':status['snapshot'],'requirements_revision':status['requirements_revision'],
                'test_receipt':status['test_receipt'],'decision':'accept',
                'evidence':{key:{'met':True,'evidence':'Authored acceptance fixture only; no real semantic or live evidence'} for key in m.e.prepare.REQUIREMENTS}}
        return result
    def driver(self,**kwargs):
        d=m.Driver(self.root,self.a,self.provider,self.worker,self.reviewer,fixture=True,**kwargs)
        (self.root/'fixture-only.marker').write_text('b91-native-fixture-only@v1\n')
        return d
    def test_full_native_sequence(self):
        d=self.driver();result=d.run()
        self.assertEqual(result['calls'],5);self.assertEqual(result['execution_groups'],4)
        self.assertTrue(result['fixture_only']);self.assertFalse(result['live_b91_passed'])
        self.assertEqual(self.calls,list(m.e.STAGES[:-1]));self.assertEqual(self.workers,['seed','check_before','patch','check_after'])
        self.assertEqual(result['resources']['model_calls'],5)
        self.assertEqual(result['resources']['measured_prompt_tokens'],500)
        self.assertEqual(result['resources']['measured_output_tokens'],1000)
        self.assertEqual(result['resources']['external_cost_total_microusd'],5000)
        self.assertEqual(result['trial_resources']['model_calls'],5);self.assertFalse(result['trial_resources']['exceeded'])
        self.assertEqual(set(result['trial_resources']['episodes']),{'project','checks-before','checks-after'})
        self.assertEqual((self.root/'project/coding-work/src/lib.rs').read_text(),PATCH)
        # The real Rust serializer omits None while the durable provider journal
        # retains explicit null. Exercise this boundary through all six stages.
        first = d.journal().generated()[0]['proposals'][0]['proposal']
        self.assertIn('development', first);self.assertIsNone(first['development'])
        native = d.controller().request()['coding']['relational']['history'][0]['proposal']
        self.assertNotIn('development', native)
        self.assertEqual(m.e.relational_identity(first), m.e.relational_identity(native))
        before=(self.root/'driver.jsonl').read_bytes();self.assertEqual(self.driver().run(),result)
        self.assertEqual(before,(self.root/'driver.jsonl').read_bytes());self.assertEqual(len(self.calls),5);self.assertEqual(len(self.workers),4)
    def test_luna_expanded_final_stages_full_native_sequence(self):
        self.a=m.admission(self.root,fixture=True,model='gpt-6-luna',final_payload_32k=True)
        self.test_full_native_sequence()
        intents=[x for x in self.driver().journal().events() if x['kind']=='dispatch_intent']
        self.assertEqual([x['reservation']['input_tokens'] for x in intents],[16384]*4+[32768])
        self.assertNotIn('final_proposal_only',[x['stage'] for x in intents])
        self.assertEqual(self.a['input_tokens']+self.a['completion_tokens'],180224)

    def test_luna_full_native_sequence(self):
        self.a=m.admission(self.root,fixture=True,model='gpt-6-luna')
        self.test_full_native_sequence()

    def test_native_lineage_contract_across_generated_fixture_stages(self):
        original=self.provider;seen=[];contexts=[]
        def checked_provider(body,timeout):
            contexts.append(json.loads(body['messages'][1]['content']))
            return original(body,timeout)
        native_validate=m.e.Controller.validate
        def checked_validate(c,response):
            # Validation occurs after resource settlement, when guidance is available.
            request=c.request();ctx=contexts[-1]
            self.assertEqual(ctx['relational_contract'],m.e.relational_contract(request))
            suggestion=response['proposals'][0]
            if suggestion['kind']=='relational':
                p=suggestion['proposal'];m.e.validate_relational_lineage(request,p,ctx['stage'])
                seen.append(bool(p['revises']));bad=copy.deepcopy(p)
                if p['revises']:
                    bad['development']['original_revision']='f'*64
                    code='invalid_revision_ancestry'
                else:
                    bad['development']={'original_revision':'f'*64,'prior_revision':'e'*64,
                         'retained_meaning':'fixture','return_reveals':'fixture','unresolved':'fixture','proposed_development':None}
                    code='initial_development_forbidden'
                with self.assertRaisesRegex(m.e.StageValidationError,code):m.e.validate_relational_lineage(request,bad,ctx['stage'])
                try:verdict=native_validate(c,m.e.local.bind_proposal(request,json.dumps({'relational':bad})))
                except ValueError:pass  # Native CLI refuses invalid provenance before assessment.
                else:self.assertNotEqual(verdict['assessments'][0]['disposition'],'deferred')
                self.assertEqual(c.request()['state_root'],request['state_root'])
            return native_validate(c,response)
        d=self.driver();d.transport=checked_provider
        with patch.object(m.e.Controller,'validate',checked_validate):result=d.run()
        self.assertEqual(seen,[False,True,True]);self.assertEqual(result['calls'],5)
        before=(self.root/'driver.jsonl').read_bytes();self.assertEqual(self.driver().run(),result)
        self.assertEqual(before,(self.root/'driver.jsonl').read_bytes());self.assertEqual(len(self.calls),5)
    def test_interrupt_after_intent_and_after_effect_never_retries(self):
        # Each boundary is tested in a separate disposable trial, never a real side effect.
        for boundary in ('after_intent','after_effect'):
            for target in ('initialize_project','execute_seed','reserve_','transport_','settle_','admit_experiment','import_check_before','commit_patch','readmit_experiment','execute_check_after','close_check_after'):
                with self.subTest(boundary=boundary,target=target),tempfile.TemporaryDirectory() as tmp:
                    self.root=Path(tmp)/'trial';self.a=m.admission(self.root,fixture=True)
                    def interrupt(point,name):
                        if point==boundary and (name.startswith(target) if target.endswith('_') else name==target):raise RuntimeError('fixture interrupt')
                    d=self.driver(hook=interrupt)
                    with self.assertRaisesRegex(RuntimeError,'fixture interrupt'):d.run()
                    calls=len(self.calls);workers=len(self.workers)
                    with self.assertRaises(m.Blocked):self.driver().run()
                    self.assertEqual(len(self.calls),calls);self.assertEqual(len(self.workers),workers)
    def test_execution_overrun_preserves_known_receipt_and_stops(self):
        d=self.driver()
        with patch.dict(os.environ,{'TSR_B91_FIXTURE_OVERRUN':'1'}):
            with self.assertRaisesRegex(m.Blocked,'resource overrun'):d.run()
        self.assertEqual(self.workers,['seed']);self.assertEqual(self.calls,[])
        rows,pending,done=d.read()
        self.assertFalse(pending);self.assertIn('execute_seed',done)
        self.assertTrue(done['execute_seed']['resources']['exceeded'])
        self.assertGreater(done['execute_seed']['resources']['tool_ms_charged'],120000)
        self.assertEqual(done['execute_seed']['status']['execution_status'],'needs_repair')
        with self.assertRaisesRegex(m.Blocked,'resource overrun'):self.driver().run()
        self.assertEqual(self.workers,['seed']);self.assertEqual(self.calls,[])

    def test_known_invalid_provider_failure_is_metered_and_not_retried(self):
        original=self.provider
        def invalid(body,timeout):
            result=original(body,timeout);result['value']=None;result['receipt']['structural_status']='invalid';result['receipt']['finish_reason']='length';return result
        d=self.driver();d.transport=invalid
        with self.assertRaises(ValueError):d.run()
        budget=d.controller().budget();self.assertEqual(budget['model_calls'],1)
        self.assertEqual(budget['external_cost_total_microusd'],1000)
        outcome=next(x for x in d.journal().events() if x['kind']=='provider_outcome')
        self.assertEqual(outcome['validation']['code'],'truncated_output')
        self.assertEqual(outcome['validation']['stage'],'relation_after_seed')
        transport=next(v for k,v in d.read()[2].items() if k.startswith('transport_'))
        self.assertEqual(transport['quarantined_structure']['shape'],{'type':'null'})
        self.assertEqual(self.workers,['seed'])
        with self.assertRaises(m.Blocked):self.driver().run()
        self.assertEqual(len(self.calls),1)
    def test_review_pause_and_withdrawal_never_authorize_execution(self):
        d=self.driver();d.reviewer=lambda *_:None
        with self.assertRaisesRegex(m.Blocked,'review required'):d.run()
        self.assertEqual(self.workers,['seed']);self.assertEqual(len(self.calls),1)
        self.assertFalse(d.read()[1])
        (self.root/'STOP').write_text('operator withdrawal')
        with self.assertRaisesRegex(m.Blocked,'withdrawn'):self.driver().run()
        self.assertEqual(self.workers,['seed']);self.assertEqual(len(self.calls),1)
    def test_campaign_overrun_preserves_usage_without_proposal_application(self):
        original=self.provider
        def overrun(body,timeout):
            result=original(body,timeout);result['resource_stop']='resource_overrun';return result
        d=self.driver();d.transport=overrun
        with self.assertRaisesRegex(ValueError,'resources'):d.run()
        budget=d.controller().budget()
        self.assertEqual(budget['model_calls'],1)
        self.assertEqual(budget['measured_prompt_tokens'],100)
        self.assertEqual(budget['measured_output_tokens'],200)
        self.assertEqual(budget['external_cost_total_microusd'],1000)
        events=d.journal().events()
        self.assertEqual([x['status'] for x in events if x['kind']=='provider_outcome'],['resource_stopped'])
        self.assertFalse(any(x['kind'] in ('validated_proposal','review_only_store_intent') for x in events))
        self.assertFalse(d.read()[1]);self.assertEqual(self.reviews,[])
        with self.assertRaises(m.Blocked):self.driver().run()
        self.assertEqual(len(self.calls),1);self.assertEqual(self.workers,['seed'])

    def test_http_error_status_survives_both_receipts_without_raw_body(self):
        original=self.provider
        def failed(body,timeout):
            result=original(body,timeout);result['value']=None
            result['receipt'].update(response_status='received_http_error',http_status=429,
                error_code='http_error',status='unknown',usd=None,usage={},raw_body='private-error-sentinel')
            return result
        d=self.driver();d.transport=failed
        with self.assertRaises(ValueError):d.run()
        events=d.journal().events()
        self.assertEqual([x['http_status'] for x in events if x['kind']=='provider_outcome'],[429])
        transports=[x['event']['value'] for x in d.read()[0] if x['event'].get('name','').startswith('transport_') and x['event']['kind']=='result']
        self.assertEqual(transports[0]['receipt']['http_status'],429)
        self.assertNotIn('private-error-sentinel',(self.root/'driver.jsonl').read_text()+(self.root/'provider.jsonl').read_text())
        with self.assertRaises(m.Blocked):self.driver().run()
        self.assertEqual(len(self.calls),1)

    def test_unknown_charge_keeps_native_token_usage_and_blocks(self):
        original=self.provider
        def unknown(body,timeout):
            result=original(body,timeout);result['receipt']['status']='unknown';result['receipt']['usd']=None;return result
        d=self.driver();d.transport=unknown
        with self.assertRaises(ValueError):d.run()
        budget=d.controller().budget()
        self.assertEqual(budget['model_calls'],1);self.assertEqual(budget['measured_prompt_tokens'],100)
        self.assertEqual(budget['measured_output_tokens'],200);self.assertIsNone(budget['external_cost_total_microusd'])
        self.assertEqual(budget['unknown_external_cost_requests'],1)
        events=d.journal().events()
        outcomes=[x for x in events if x['kind']=='provider_outcome']
        self.assertEqual(outcomes[-1]['status'],'resource_stopped')
        self.assertFalse(any(x['kind']=='validated_proposal' for x in events))
        self.assertFalse(d.read()[1])
        with self.assertRaises(m.Blocked):self.driver().run()
        self.assertEqual(len(self.calls),1)

class Admission(unittest.TestCase):
    def test_expanded_profile_exact_stages_model_and_total(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=(Path(tmp)/'trial').resolve()
            a=m.admission(root,fixture=True,model='gpt-6-luna',final_payload_32k=True)
            m.validate_admission(a,root,True)
            self.assertEqual([m.stage_payload_limit(a,x) for x in m.e.STAGES],[16384]*4+[32768]*2)
            for mutation in ('early','model','total','missing'):
                bad=copy.deepcopy(a)
                if mutation=='early':bad['stage_payload_bytes'][m.e.STAGES[0]]=32768
                elif mutation=='model':bad['model']=m.e.MODEL
                elif mutation=='total':bad['input_tokens']=98304
                else:del bad['stage_payload_bytes'][m.e.STAGES[-1]]
                with self.assertRaises(ValueError):m.validate_admission(bad,root,True)
            legacy=m.admission(root,fixture=True,model='gpt-6-luna')
            m.validate_admission(legacy,root,True)
            self.assertEqual([m.stage_payload_limit(legacy,x) for x in m.e.STAGES],[16384]*6)

    def test_no_dispatch_without_explicit_optins(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'admission.json';p.write_text(json.dumps(m.admission(Path(tmp)/'trial',fixture=True)))
            result=subprocess.run(['python3',str(Path(m.__file__)),'run','--admission',str(p)],capture_output=True,text=True)
            self.assertNotEqual(result.returncode,0);self.assertFalse((Path(tmp)/'trial').exists())
    def test_changed_helper_and_expired_admission_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)/'trial';a=m.admission(root,fixture=True)
            for change in ('helper','expiry','limit'):
                bad=copy.deepcopy(a)
                if change=='helper':bad['implementation']['engine-local-model.py']='0'*64
                elif change=='expiry':bad['expires_at']='2000-01-01T00:00:00+00:00'
                else:bad['calls']=7
                with self.assertRaises(ValueError):m.Driver(root,bad,lambda *_:None,lambda *_:None,lambda *_:None,fixture=True)

if __name__=='__main__':unittest.main()
