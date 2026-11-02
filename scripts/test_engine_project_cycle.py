"""Injected local callbacks; no provider/target effects or live inference evidence."""
import copy
import datetime
import importlib.util
import tempfile
import unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('project_cycle',Path(__file__).with_name('engine-project-cycle.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class Cycle(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name).resolve()/'journal'
        self.state='a';self.branch='propose';self.guidance='first';self.plan=None;self.actions=[];self.calls=[];self.reviews=[];self.counter=0
        self.a={'version':'local-project-cycle@v1','journal':str(self.root),'scope':{'id':'fixture'},'max_decisions':2,'max_executions':2,
            'expires_at':(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(hours=1)).isoformat(),'implementation_sha256':m.d.file_sha(Path(m.__file__))}
    def request(self):return {'scope':self.a['scope'],'state_root':self.state,'source_snapshot':self.state,'resource_root':str(self.counter),'binding':self.state+str(self.counter),'branch':self.branch,'reason':'fixture','request':{'guidance':self.guidance},'plan':self.plan,'dispatch_authorized':False}
    def native(self,command,value):
        if command=='request':return self.request()
        if command=='preview':return {'cycle':self.request(),'response_digest':m.e.sha(value['response'])}
        if command=='advance':
            self.assertEqual(value['review']['binding'],self.request()['binding']);self.assertEqual(value['review']['response_digest'],m.e.sha(value['response']))
            self.plan={'commitment':value['review']['response_digest']};self.branch='resume'
            return {'next':self.request(),'executed':False}
        if command=='resume':
            self.assertEqual(value['binding'],self.request()['binding']);self.actions.append(self.guidance)
            self.state+='b';self.guidance='second';self.plan=None;self.branch='idle' if len(self.actions)==2 else 'propose'
            return {'verdict':'failed_tests' if len(self.actions)==1 else 'passed'}
        self.fail(command)
    def meter(self,key,value):return {key:value,'receipt':{'mode':'fixture','status':'settled','usage':{'prompt_tokens':10,'completion_tokens':10,'total_tokens':20},'usd':'0.001','elapsed_seconds':0.1}}
    def proposal(self,state):
        self.calls.append(copy.deepcopy(state));self.counter+=1
        return self.meter('response',{'state_root':state['state_root'],'proposals':[{'fixture_choice':state['request']['guidance']}]})
    def review(self,state,response):
        self.reviews.append(copy.deepcopy(response));self.counter+=1
        return self.meter('judgment',{'decision':'approve','reviewer':'separate-fixture','generator':'fixture','response_digest':state['response_digest']})
    def runner(self,**kwargs):return m.Cycle(self.root,self.a,self.native,self.proposal,self.review,**kwargs)
    def test_two_choices_mixed_returns_restart_idle(self):
        def pause(event,name):
            if event=='after_result' and name=='resume_0':raise RuntimeError('checkpoint')
        with self.assertRaisesRegex(RuntimeError,'checkpoint'):self.runner(hook=pause).run()
        self.assertEqual(self.actions,['first']);result=self.runner().run();self.assertEqual(result['status'],'idle')
        self.assertEqual(self.actions,['first','second']);self.assertEqual(len(self.calls),2);self.assertEqual(len(self.reviews),2)
        before=(self.root/'cycle.jsonl').read_bytes();self.assertEqual(self.runner().run(),result);self.assertEqual(before,(self.root/'cycle.jsonl').read_bytes())
    def refresh(self):
        import json
        if not (self.root/'cycle.jsonl').exists():self.runner()
        request={'version':'context-refresh@v1','cycle_admission_sha256':m.e.sha(self.a),'request_id':'authored-run','input_ids':['authored-evidence'],'inputs_sha256':'a'*64}
        (self.root/'CONTEXT_REFRESH').write_text(json.dumps(request));return request
    def test_refresh_waits_for_action_and_assessment_then_restart_is_inert(self):
        request=self.refresh();assessed=[];original=self.native
        def native(command,value):
            result=original(command,value)
            if command=='resume':assessed.append(self.state)
            return result
        self.native=native
        result=self.runner().run()
        self.assertEqual(result['status'],'context_refresh_required');self.assertEqual(result['refresh'],request)
        self.assertEqual(assessed,['ab']);self.assertEqual(len(self.calls),1);self.assertEqual(len(self.actions),1)
        before=(self.root/'cycle.jsonl').read_bytes();self.assertEqual(self.runner().run(),result);self.assertEqual(before,(self.root/'cycle.jsonl').read_bytes())
    def test_refresh_never_clears_unknown_action(self):
        self.refresh()
        def pause(event,name):
            if event=='after_effect' and name=='resume_0':raise RuntimeError('authored interrupted outcome')
        with self.assertRaises(RuntimeError):self.runner(hook=pause).run()
        with self.assertRaisesRegex(m.d.Blocked,'uncertain'):self.runner().run()
        self.assertEqual(len(self.actions),1)
    def test_refresh_stop_wins_and_stale_binding_fails(self):
        import json
        request=self.refresh();request['cycle_admission_sha256']='b'*64
        (self.root/'CONTEXT_REFRESH').write_text(json.dumps(request))
        with self.assertRaisesRegex(ValueError,'binding'):self.runner().run()
        self.assertFalse(self.calls)
        self.refresh();(self.root/'STOP').write_text('AUTHORED stop')
        with self.assertRaises(m.d.Blocked):self.runner().run()
        self.assertFalse(self.calls)
    def test_refresh_arriving_during_review_does_not_skip_reviewed_action(self):
        original=self.review
        def review(state,response):
            result=original(state,response);self.refresh();return result
        self.review=review;result=self.runner().run()
        self.assertEqual(result['status'],'context_refresh_required');self.assertEqual(len(self.reviews),1);self.assertEqual(self.actions,['first'])
    def test_refresh_after_valid_abstention_preserves_the_outcome(self):
        self.refresh()
        self.proposal=lambda state:self.meter('response',{'abstain':'AUTHORED justified retention'})
        result=self.runner().run()
        self.assertEqual(result['status'],'context_refresh_required')
        self.assertEqual(result['prior_outcome'],'proposal_abstention');self.assertFalse(self.actions)
    def test_equal_guidance_no_manufactured_choice_change(self):
        state=self.request();self.assertEqual(self.proposal(state)['response'],self.proposal(state)['response'])
    def test_pending_after_effect_never_repeated(self):
        def interrupt(event,name):
            if event=='after_effect' and name=='resume_0':raise RuntimeError('interrupted')
        with self.assertRaises(RuntimeError):self.runner(hook=interrupt).run()
        with self.assertRaisesRegex(m.d.Blocked,'uncertain'):self.runner().run()
        self.assertEqual(len(self.actions),1)
    def test_unknown_provider_stops(self):
        def unknown(_):raise TimeoutError('fixture')
        self.proposal=unknown
        with self.assertRaises(TimeoutError):self.runner().run()
        with self.assertRaisesRegex(m.d.Blocked,'uncertain'):self.runner().run()
        self.assertEqual(self.actions,[])
    def test_rejected_review_no_apply(self):
        original=self.review
        def reject(s,r):
            v=original(s,r);v['judgment']['decision']='reject';return v
        self.review=reject;self.assertEqual(self.runner().run()['status'],'review_reject');self.assertEqual(self.actions,[])
    def test_native_review_required_no_calls(self):
        self.branch='review_required';self.assertEqual(self.runner().run()['status'],'review_required');self.assertEqual(self.calls,[])
    def test_idle_budget_stop_and_unreviewed_pending(self):
        self.a['max_decisions']=0;self.assertEqual(self.runner().run()['status'],'decision_budget');self.assertEqual(self.calls,[])
    def test_advance_review_required_replay_stops(self):
        original=self.native
        def native(c,v):
            if c=='advance':return {'review_required':True,'next':self.request(),'executed':False}
            return original(c,v)
        self.native=native
        def pause(event,name):
            if event=='after_result' and name=='advance_0':raise RuntimeError('pause')
        with self.assertRaises(RuntimeError):self.runner(hook=pause).run()
        self.assertEqual(self.runner().run()['status'],'review_required');self.assertEqual(len(self.calls),1);self.assertEqual(self.actions,[])
    def test_invalid_cost_and_oversized_context_stop(self):
        original=self.proposal
        def bad(state):
            value=original(state);value['receipt']['usd']='-1';return value
        self.proposal=bad
        with self.assertRaises(m.d.Blocked):self.runner().run()
        self.assertEqual(self.actions,[])
    def test_input_bound_before_callback(self):
        self.guidance='x'*33000
        with self.assertRaisesRegex(m.d.Blocked,'context bound'):self.runner().run()
        self.assertEqual(self.calls,[])
    def test_no_progress_advance_stops(self):
        original=self.native
        def native(c,v):
            if c=='advance':return {'review_required':False,'next':self.request(),'executed':False}
            return original(c,v)
        self.native=native;self.assertEqual(self.runner().run()['status'],'native_no_progress');self.assertEqual(len(self.calls),1)
    def test_scope_drift_no_call(self):
        original=self.native
        def drift(c,v):
            result=original(c,v);result['scope']={'different':True};return result
        self.native=drift
        with self.assertRaisesRegex(ValueError,'scope'):self.runner().run()
        self.assertEqual(self.calls,[])
    def test_review_cannot_substitute_response(self):
        original=self.review
        def wrong(s,r):
            v=original(s,r);v['judgment']['response_digest']='wrong';return v
        self.review=wrong
        with self.assertRaisesRegex(m.d.Blocked,'digest drift'):self.runner().run()
        self.assertEqual(self.actions,[])
    def test_reviewer_mutation_cannot_replace_generated_response(self):
        original=self.review
        def mutate(state,response):
            response['proposals']=[{'fixture_choice':'injected'}]
            value=original(state,response);value['judgment']['response_digest']=m.e.sha(response);return value
        self.review=mutate
        with self.assertRaisesRegex(m.d.Blocked,'digest drift'):self.runner().run()
        self.assertEqual(self.actions,[])
    def test_state_drift_during_generation(self):
        original=self.proposal
        def changed(s):
            v=original(s);self.state='other';return v
        self.proposal=changed
        with self.assertRaisesRegex(m.d.Blocked,'state changed'):self.runner().run()
        self.assertEqual(self.actions,[])

class NativeBoundary(unittest.TestCase):
    def test_actual_native_choices_returns_and_restart(self):
        import json
        import test_engine_b91_driver as fixture
        base=fixture.NativeSequence(methodName='test_full_native_sequence');base.setUp();self.addCleanup(base.doCleanups)
        driver=base.driver();controller=driver.initialize();root=(base.root/'cycle-log').resolve()
        def command(*args):return controller.cli('project','cycle',*args)
        initial=command('request');calls=[];executions=[];offered=[]
        admission={'version':'local-project-cycle@v1','journal':str(root),'scope':initial['scope'],'max_decisions':2,'max_executions':2,
            'expires_at':driver.admission['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.__file__))}
        def native(kind,value):
            if kind=='request':return command('request')
            if kind=='preview':return controller.json_cli(('project','cycle','preview'),value['response'],'--response')
            if kind=='advance':
                result=controller.json_cli(('project','cycle','advance'),value)
                if not executions:
                    status=controller.cli('project','status');p=status['pending']
                    controller.json_cli(('project','evaluator-forecast'),{'state_root':controller.request()['state_root'],
                        'snapshot':controller.cli('project','acceptance','status')['snapshot'],'commitment':p['commitment'],
                        'action':p['action'],'probability':.1,'evaluator':'B9.1 frozen evaluator-authored seed'})
                    result['next']=command('request')
                return result
            current=command('request');self.assertEqual(value['binding'],current['binding']);self.assertEqual(current['branch'],'resume')
            # Exact native cycle gate/plan, then existing cfg(test) Runner; no actual
            # target execution and no claim of exercising production CLI resume.
            stage='seed' if not executions else 'patch';result=base.worker(controller,stage);executions.append(result)
            return result
        def measured(key,value):return {key:value,'receipt':{'mode':'fixture','status':'settled','usage':{'prompt_tokens':10,'completion_tokens':10,'total_tokens':20},'usd':'0.001','elapsed_seconds':0.01}}
        def propose(view):
            request=view['request'];calls.append(request)
            source=fixture.m.e.prepare.FAILED if not request['coding']['recent_observations'] else fixture.PATCH
            response=fixture.m.e.local.bind_proposal(request,json.dumps({'edits':[{'path':'src/lib.rs','replacement':source}],
                'rationale':'Authored offline alternative conditioned on observed return, not a model result'}))
            offered.append(response);return measured('response',response)
        def review(view,response):return measured('judgment',{'decision':'approve','reviewer':'fixture independent role','generator':'fixture generator','response_digest':view['response_digest']})
        def pause(event,name):
            if event=='after_result' and name=='resume_0':raise RuntimeError('acknowledged checkpoint')
        with self.assertRaisesRegex(RuntimeError,'acknowledged checkpoint'):m.Cycle(root,admission,native,propose,review,hook=pause).run()
        result=m.Cycle(root,admission,native,propose,review).run()
        self.assertEqual(len(calls),2);self.assertEqual(len(executions),2);self.assertNotEqual(offered[0],offered[1])
        self.assertEqual(executions[0]['execution']['verdict'],'failed_tests');self.assertEqual(executions[1]['execution']['verdict'],'passed')
        self.assertEqual(result['status'],'review_required')
        self.assertEqual(result['native_reason'],'independent_acceptance_or_shareable_feedback_required')
        before=(root/'cycle.jsonl').read_bytes();self.assertEqual(m.Cycle(root,admission,native,propose,review).run(),result)
        self.assertEqual(before,(root/'cycle.jsonl').read_bytes());self.assertEqual(len(executions),2)
