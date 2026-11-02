"""Authored transport, real native controller, optional sealed browser worker.
No provider calls; fixture ledger is separate from the historical campaign.
"""
import copy
import datetime
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import subprocess
import sys
import unittest
s=importlib.util.spec_from_file_location('browser_adapter',Path(__file__).with_name('engine-browser-adapter.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
campaign=m.base.load('browser_test_campaign','engine-b91-campaign.py')
class ProspectiveGrant(unittest.TestCase):
    def test_scope_only_grant_cannot_reset_or_expand_caps(self):
        g={'version':'native-browser-initial@v1','manifest_sha256':'a'*64,'ledger_prefix_head':'b'*64,'previous_max_calls':272,'max_calls':272,'previous_max_execution_groups':74,'max_execution_groups':74,'cap_microusd':campaign.CAP,'reservation_microusd':campaign.RESERVE,'authority_sha256':'c'*64,'reviewer':'Authored local review','suite_manifest_sha256':'d'*64,'source_class':'newly-authored-nonprivate-browser-project'}
        campaign.validate_evaluation_grant(g,'a'*64,'b'*64,272,74)
        for field,value in [('max_calls',283),('max_execution_groups',77),('previous_max_calls',0),('source_class','old-graph-authority'),('ledger_prefix_head','e'*64)]:
            bad={**g,field:value}
            with self.assertRaises(ValueError):campaign.validate_evaluation_grant(bad,'a'*64,'b'*64,272,74)
@unittest.skipUnless(os.getenv('TSR_BROWSER_ADAPTER_AUTHORED')=='1','explicit authored native integration opt-in')
class BrowserAdapter(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory(dir=m.e.ROOT/'target',prefix='browser-adapter-authored-');self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name).resolve()
        (self.root/'native').mkdir();self.c=m.d.Native(self.root/'native')
        self.config={'goal':'Authored infrastructure fixture only','image':m.IMAGE,'user':'501:20','browser':True,'browser_container_only':True,'editable':{'index.html':'<button id="start">Start</button><p id="readout"></p><canvas width="288" height="192"></canvas><p id="history"></p><script src="app.js"></script>','app.js':'// fixture\n'},'protected':{'browser-harness.cjs':Path(__file__).with_name('engine-browser-harness.cjs').read_text(),'probe.json':json.dumps({'version':'browser-probe@v1','assets':['index.html','app.js'],'entry':'index.html','actions':[{'kind':'click','value':'#start'},{'kind':'key','value':'ArrowRight'},{'kind':'key','value':'ArrowDown'},{'kind':'key','value':'Space'}],'checks':[{'selector':'#start','text':'Play'},{'selector':'#readout','text':'Paused; moves 2; score 2; input pause'}]})},'probe_entry':'browser-harness.cjs','cases':[{'id':'browser_smoke','args':['probe.json'],'expected_stdout':'browser-observation@v1'}],'timeout_ms':30000}
        scope=self.root/'scope.json';scope.write_text(json.dumps(self.config));req=self.root/'requirements.json';req.write_text(json.dumps({'controls':'Start control is usable; this authored check does not measure player satisfaction'}))
        self.c.cli('project','init','--spec',str(scope),'--requirements',str(req))
        self.c.json_cli(('project-lifecycle-open',),{'state_root':self.c.request()['state_root'],'training':None,'tool_ms_ceiling':120000,'reason':'Authored browser adapter verification','non_tool':{'model_calls':10,'model_tokens':409600,'wall_ms':600000,'patch_bytes':49152,'external_cost_microusd':0}},'--admission')
        completion=self.c.cli('project','acceptance','status')
        self.c.json_cli(('project','acceptance','feedback'),{'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],'guidance':json.loads(req.read_text())})
        self.c.cli('project','resource-unpriced-admit','--state-root',self.c.request()['state_root'])
        r=self.c.request();c=r['coding'];i=c['relational']['inputs']
        public={'version':'public-browser-origin@v1','scope':r['next_attention']['scope'],'requirements_revision':i['requirements_revision'],'parents':i['parents'],'protected':{f['path']:f['source_digest'] for f in c['files'] if not f['editable']},'editable':list(self.config['editable']),'public_brief':'Authored infrastructure check; not a game or real player feedback','source_snapshots':[c['snapshot']],'public_relationships':[],'public_returns':[]}
        origin={'directory':str(self.c.directory),'public':public,'initial_sources':self.config['editable']}
        self.manifest=campaign.manifest('a'*64,self.root/'campaign.jsonl',fixture=True);self.ledger=campaign.Ledger(self.root/'campaign.jsonl',self.manifest,campaign.digest(self.manifest),create=True)
        self.a=m.admission(self.root/'adapter',origin,self.c.cli('project','config'),self.manifest,(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(minutes=30)).isoformat(),max_decisions=2,max_executions=1,correction_allowance=1)
        self.calls=[];self.bodies=[];self.bad_once=False;self.playable=False;self.oversize_once=False
    def transport(self,body,timeout):
        self.assertEqual(body['response_format'],{'type':'json_object'});self.assertEqual(timeout,240)
        self.bodies.append(json.loads(json.dumps(body)))
        data=json.loads(body['messages'][1]['content']);self.calls.append(data)
        if 'criteria' in data:
            value={'verdict':'accept','findings':{k:{'finding':'met','basis':'static_source','evidence':[data['binding']['evidence_ids'][0]],'explanation':'Authored semantic judgment only','limitations':'Not model competence or player satisfaction'} for k in data['criteria']}}
        elif self.bad_once:
            self.bad_once=False;value={'alternatives':[{'edits':'wrong scalar','rationale':'Authored invalid shape'}]}
        elif self.playable:
            js=Path(__file__).with_name('testdata').joinpath('browser-project/control-fixture.js').read_text()
            value={'alternatives':[{'edits':[{'path':'index.html','old_text':self.config['editable']['index.html'],'new_text':'<button id="start">Start</button><canvas width="288" height="192"></canvas><p id="readout"></p><p id="history"></p><script src="app.js"></script>'},{'path':'app.js','old_text':self.config['editable']['app.js'],'new_text':js}],'rationale':'Authored playable-scale harness fixture, not engine output'}]}
        elif self.oversize_once:
            self.oversize_once=False;value={'alternatives':[{'edits':[{'path':'app.js','old_text':'fixture','new_text':'x'*1048577}],'rationale':'Authored over-bound fixture'}]}
        else:value={'alternatives':[{'edits':[{'path':'index.html','old_text':'Start','new_text':'Play'},{'path':'app.js','old_text':'fixture','new_text':'revision'}],'rationale':'Authored two-file change'}]}
        return {'value':value,'receipt':{'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':.1}}
    def adapter(self):return m.Adapter(self.root/'adapter',self.a,self.c,self.ledger,self.transport,worker=lambda view:self.adapter_instance.resume_process(view))
    def test_native_predispatch_disposition_restart_same_body(self):
        a=self.adapter();self.adapter_instance=a
        view=a.native('request',None);request,context=a.project(view)
        body=a.proposal_body(view,a.wire_context(request,context))
        wire=json.loads(body['messages'][1]['content'])
        self.assertEqual(wire['choice_summary']['source_request_sha256'],wire['proposal_references']['request_sha256'])
        self.assertIn('retain',wire['choice_summary']['forms'])
        self.assertIn('edit',wire['choice_summary']['forms'])
        self.assertNotIn('success_preference',wire['choice_summary']['forms'])
        self.assertEqual(wire['files'],context['files'])
        self.assertIn('choice_summary',body['messages'][0]['content'])
        body['messages'][0]['content']+=' Authored Unicode identity check: Ω.'
        original=a.op
        def interrupted(name,fn,**kw):
            if name.startswith('dispatch_'):raise RuntimeError('authored before-dispatch interruption')
            return original(name,fn,**kw)
        a.op=interrupted
        with self.assertRaisesRegex(RuntimeError,'before-dispatch'):
            a.checked('proposal',body,lambda value:(value,{}))
        self.assertEqual(self.calls,[])
        self.assertEqual(self.c.budget()['unsettled_requests'],1)
        a=self.adapter();self.adapter_instance=a;a.launch_lock_owned=True
        self.assertEqual(a.recover_predispatch(),1)
        self.assertEqual(self.c.budget()['known_not_dispatched_requests'],1)
        self.assertEqual(self.c.budget()['unsettled_requests'],0)
        a=self.adapter();self.adapter_instance=a;a.launch_lock_owned=True
        self.assertEqual(a.recover_predispatch(),0)
        self.assertTrue(a.recovery_ready())
        result=a.checked('proposal',body,lambda value:(value,{}))
        self.assertIn('response',result)
        self.assertEqual(len(self.calls),1);self.assertEqual(self.bodies,[body])
        budget=self.c.budget();self.assertEqual(budget['model_calls'],2)
        self.assertEqual(budget['known_not_dispatched_requests'],1)
        self.assertEqual(budget['unsettled_requests'],0)
        self.assertEqual(self.ledger.status()['calls'],1)

    def test_review_predispatch_and_interrupted_proof_restart(self):self._review_recovery('proof')
    def test_review_settled_response_restart_without_new_call(self):self._review_recovery('settled')
    def test_review_pending_native_settlement_restart(self):self._review_recovery('settle_pending')
    def test_review_bound_content_restart(self):self._review_recovery('bound')
    def test_settled_rejection_is_replayed_not_regenerated(self):self._review_recovery('settled','reject')
    def test_settled_uncertainty_is_replayed_not_regenerated(self):self._review_recovery('settled','unknown')
    def _review_recovery(self,mode,verdict='accept'):
        if verdict!='accept':
            original_transport=self.transport
            def judged(body,timeout):
                result=original_transport(body,timeout)
                if 'criteria' in json.loads(body['messages'][1]['content']):
                    result['value']['verdict']=verdict
                    for finding in result['value']['findings'].values():finding['finding']='unknown' if verdict=='unknown' else 'not_met'
                return result
            self.transport=judged
        a=self.adapter();self.adapter_instance=a
        proposal=a.proposal(a.native('request',None))['response'];preview=a.native('preview',{'response':proposal})
        view={**a.native('request',None),'response_digest':preview['response_digest']}
        if mode=='proof':
            original=a.op
            def fail(name,fn,**kw):
                if name.startswith('dispatch_'):raise RuntimeError('authored review predispatch')
                return original(name,fn,**kw)
            a.op=fail
        else:
            point,prefix=('after_intent','settle_') if mode=='settle_pending' else ('after_result','bound_output_') if mode=='bound' else ('after_result','settle_')
            def hook(phase,name):
                if phase==point and name.startswith(prefix):raise RuntimeError('authored review interruption')
            a.hook=hook
        with self.assertRaisesRegex(RuntimeError,'authored review'):a.review(view,proposal)
        count=len(self.calls);self.assertEqual(count,1 if mode=='proof' else 2)
        a=self.adapter();self.adapter_instance=a;a.launch_lock_owned=True
        if mode=='proof':
            def proof_stop(phase,name):
                if phase=='after_intent' and name.startswith('predispatch_input_'):raise RuntimeError('authored proof interruption')
            a.hook=proof_stop
            with self.assertRaisesRegex(RuntimeError,'proof interruption'):a.recover_predispatch()
            a=self.adapter();self.adapter_instance=a;a.launch_lock_owned=True
        stop=a.root/'STOP';stop.write_text('Authored explicit Stop')
        with self.assertRaisesRegex(ValueError,'Stop'):a.recover_settled_response()
        stop.unlink() # Fixture-only explicit resume.
        a.recover_settled_response();a.recover_predispatch()
        self.assertTrue(a.recovery_ready())
        result=a.review(view,proposal)
        self.assertEqual(result['judgment']['decision'],'approve' if verdict=='accept' else verdict)
        self.assertEqual(len(self.calls),2);self.assertEqual(self.ledger.status()['calls'],2)
        self.assertEqual(self.c.budget()['unsettled_requests'],0)
        self.assertEqual(len(a.corrections.events()),0)
        self.assertEqual(self.c.budget()['model_calls'],3 if mode=='proof' else 2)

    def test_settled_replay_rechecks_per_request_token_limits(self):
        for key in ('prompt_tokens','completion_tokens'):
            with self.subTest(key=key):
                a=self.adapter();self.adapter_instance=a
                # No provider action: independently authored oversized replay tuple
                # exercises the exact measured gate shared with normal dispatch.
                usage={'prompt_tokens':1,'completion_tokens':1}
                usage[key]=self.a['input_tokens' if key=='prompt_tokens' else 'completion_tokens']+1
                usage['total_tokens']=usage['prompt_tokens']+usage['completion_tokens']
                import unittest.mock
                helper=m.load('oversized_replay_fixture','engine-browser-response-recovery.py')
                helper.replay=lambda *_:('authored',{}, {'mode':'fixture','status':'settled','usd':'0.0001','elapsed_seconds':.1,'usage':usage},0)
                with unittest.mock.patch.object(m,'load',return_value=helper):
                    with self.assertRaises(Exception):a.replay_checked('review',{})
                self.assertEqual(self.calls,[])

    def test_corrected_review_predispatch_reuses_exact_feedback_wire(self):self._corrected_review_recovery(False)
    def test_corrected_review_settled_reuses_exact_feedback_wire(self):self._corrected_review_recovery(True)
    def _corrected_review_recovery(self,settled):
        a=self.adapter();self.adapter_instance=a
        proposal=a.proposal(a.native('request',None))['response'];preview=a.native('preview',{'response':proposal})
        view={**a.native('request',None),'response_digest':preview['response_digest']}
        transport=self.transport;first=[True];captured=[]
        def malformed_once(body,timeout):
            result=transport(body,timeout)
            if first[0]:first[0]=False;result['value']['review_id']='authored invalid model-owned binding'
            return result
        a.injected_transport=malformed_once
        checked=a.checked
        def capture(kind,body,*args,**kwargs):captured.append(copy.deepcopy(body));return checked(kind,body,*args,**kwargs)
        a.checked=capture;op=a.op
        def fail(name,fn,**kw):
            if not settled and name.startswith('dispatch_') and len(a.corrections.events())==1:raise RuntimeError('authored corrected predispatch')
            value=op(name,fn,**kw)
            if settled and name.startswith('settle_') and len(a.corrections.events())==1:raise RuntimeError('authored corrected predispatch')
            return value
        a.op=fail
        with self.assertRaisesRegex(RuntimeError,'corrected predispatch'):a.review(view,proposal)
        a=self.adapter();self.adapter_instance=a;a.launch_lock_owned=True;a.recover_settled_response();a.recover_predispatch()
        self.assertTrue(a.can_resume_checked('review',captured[0]))
        result=a.review(view,proposal)
        self.assertEqual(result['judgment']['decision'],'approve')
        self.assertEqual(len(a.corrections.events()),1);self.assertEqual(self.ledger.status()['calls'],3)
        self.assertEqual(self.c.budget()['model_calls'],3 if settled else 4)

    def test_expectations_reindex_and_reject_foreign_contributor(self):
        a=self.adapter();view=a.native('request',None);request,context=a.project(view)
        parent=request['coding']['relational']['inputs']['parents'][0]['id']
        expectation={'parents':[parent],'anticipated':'Authored visible label change','grounds':'Exact source change','uncertainty':'No player satisfaction evidence'}
        first={'edits':[{'path':'index.html','old_text':'Start','new_text':'Play'}],'rationale':'First authored alternative'}
        second={'edits':[{'path':'index.html','old_text':'Start','new_text':'Begin'}],'rationale':'Second authored alternative','expectation':expectation}
        r,_=a.decode_proposal({'alternatives':[first,second]},request,context,view)
        self.assertEqual(r['pursuit_expectations'],{'1':expectation})
        first['expectation']={**expectation,'anticipated':'Different first hypothesis'}
        r,_=a.decode_proposal({'alternatives':[first,second]},request,context,view)
        self.assertEqual(r['pursuit_expectations'],{'0':first['expectation'],'1':expectation})
        second['expectation']={**expectation,'parents':['f'*64]}
        with self.assertRaises(ValueError):a.decode_proposal({'alternatives':[first,second]},request,context,view)

    @unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1','explicit existing sealed browser fixture opt-in')
    def test_qualitative_expectation_return_review_and_next_serialized_choice(self):
        self.playable=True
        script=self.root/'authored-observer.cjs';script.write_text('// Non-executed authored report fixture')
        policy={'version':'source-bound-browser-assessment@v1','script':str(script),'sha256':m.assessment.sha(script),'limitations':'Authored observer reports text only; no player fulfillment or link activation'}
        self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=6,max_executions=2,correction_allowance=1,independent_assessment=policy,feedback_cycle={'version':'authorized-feedback-cycle@v2','authorization_id':'authored-pursuit-cycle','max_continuations':3})
        a=self.adapter();self.adapter_instance=a;original=self.transport;seen_reports=[]
        def observer():
            source=m.assessment.sources(self.c.directory/'coding-work',self.a['origin']['public']['editable'])
            return __import__('types').SimpleNamespace(returncode=0,stdout=json.dumps({'source':source,'checks':[{'name':'Authored text observation','pass':True}],'desktop_text':'Authored report: Play label visible','coverage':'Authored text only, no enjoyment'}).encode(),stderr=b'')
        a.assessment_runner=observer
        def transport(body,timeout):
            data=json.loads(body['messages'][1]['content']);result=original(body,timeout)
            if 'criteria' in data:
                if 'evidence_coverage' in data['criteria']:
                    reports=[(k,v) for k,v in data['evidence'].items() if isinstance(v,dict) and v.get('version')=='attributed-reflection-assessment@v1']
                    self.assertEqual(len(reports),1);key,report=reports[0]
                    self.assertNotIn(key,data['binding']['observed_receipts']);self.assertFalse(report['native_observation'])
                    self.assertEqual(report['report']['observations']['desktop_text'],'Authored report: Play label visible');seen_reports.append(report)
                if 'interpretation' in data.get('response_schema',{}).get('properties',{}):
                    result['value']={}
                    result['value']['interpretation']={'conclusion':'supported','observed':'Authored fixture receipt reports controls responding','revised_expectation':None,'uncertainty':'Authored checks do not establish enjoyment','contrast':'Functional controls are observed; player experience remains unobserved','possibilities':['Retain pending genuine player experience']}
            elif 'pursuit_history' in data:
                self.assertEqual(data['pursuit_history']['history'][0]['reviewed_reflections'][0]['reflection']['conclusion'],'supported')
                result['value']={'abstain':'Authored choice retains the fixture after the reviewed return; no live learning claim'}
            else:
                result['value']['alternatives'][0]['expectation']={'parents':[data['parents'][0]['id']],'anticipated':'The public controls will respond to clicks and keys','grounds':'Authored fixture implementation supplies those handlers','uncertainty':'This predicts control behavior, not enjoyment'}
            return result
        a.injected_transport=transport;view=a.native('request',None);root=self.root/'pursuit-cycle'
        ca={'version':'local-project-cycle@v3','journal':str(root),'scope':view['scope'],'max_decisions':6,'max_executions':2,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'feedback_cycle':self.a['feedback_cycle']}
        result=m.loop.Cycle(root,ca,a.native,a.proposal,a.review).run()
        self.assertEqual(self.ledger.status()['calls'],5)
        self.assertEqual(self.ledger.status()['execution_groups'],2)
        self.assertEqual(len(seen_reports),2);self.assertEqual(seen_reports[0],seen_reports[1])
        self.assertEqual(self.c.cli('project','status')['execution_status'],'passed')
        history=self.c.cli('project','pursuit-context')['history'];self.assertEqual(len(history),1);self.assertEqual(len(history[0]['reviewed_reflections']),1)
        # Ask for a normal next proposal after explicit local fixture continuation,
        # never user feedback or live authority. The returned evidence is serialized.
        request=self.c.request();_,context=a.project(a.native('request',None));self.assertIn('pursuit_history',context)
        before=(self.root/'campaign.jsonl').read_bytes()
        helper=m.load('fixture_pursuit','engine-pursuit-reflection.py');self.assertIsNone(helper.reflect(a))
        self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
        self.assertEqual(result['status'],'proposal_abstention')
        self.assertTrue(any('pursuit_history' in json.loads(b['messages'][1]['content']) for b in self.bodies))

    def test_complete_semantic_review_aggregate_is_not_per_field(self):
        a=self.adapter();view=a.native('request',None)
        p=a.proposal(view);preview=a.native('preview',{'response':p['response']})
        original=self.transport
        def large_review(body,timeout):
            result=original(body,timeout)
            for finding in result['value']['findings'].values():
                finding['explanation']='x'*40000
                finding['limitations']='y'*40000
            return result
        a.injected_transport=large_review
        with self.assertRaisesRegex(Exception,'no correction') as rejected:
            a.review({**a.native('request',None),'response_digest':preview['response_digest']},p['response'])
        self.assertIn('aggregate browser memory envelope',str(rejected.exception.__cause__))
        self.assertEqual(self.ledger.status()['calls'],2)
        self.assertEqual(len(a.corrections.events()),0)
        self.assertFalse(any(k.startswith('bound_output_') and v.get('model_content',{}).get('verdict') for k,v in a.read()[2].items()))

    def test_complete_three_file_source_in_actual_generator_and_review_wire(self):
        # Authored source volume exceeds the former 12 KiB review and 32 KiB wire gates.
        for name in ('index.html','app.js'):
            self.config['editable'][name] += '\n' + ('<!-- authored context -->' if name.endswith('html') else '// authored context') * 1500
        # Reinitialize a separate genuine native fixture with all three files.
        self.config['editable']['style.css']='/* authored styles */' * 1500
        probe=json.loads(self.config['protected']['probe.json']);probe['assets'].append('style.css');self.config['protected']['probe.json']=json.dumps(probe)
        other=self.root/'large-native';other.mkdir();self.c=m.d.Native(other)
        scope=self.root/'large-spec.json';scope.write_text(json.dumps(self.config))
        req=self.root/'large-req.json';req.write_text(json.dumps({'controls':'Authored controls'}))
        self.c.cli('project','init','--spec',str(scope),'--requirements',str(req))
        self.c.json_cli(('project-lifecycle-open',),{'state_root':self.c.request()['state_root'],'training':None,'tool_ms_ceiling':120000,'reason':'Authored capacity check','non_tool':{'model_calls':10,'model_tokens':3000000,'wall_ms':600000,'patch_bytes':1000000,'external_cost_microusd':0}},'--admission')
        self.c.cli('project','resource-unpriced-admit','--state-root',self.c.request()['state_root'])
        r=self.c.request();origin=self.a['origin'];origin['directory']=str(other)
        origin['initial_sources']=dict(self.config['editable']);public=origin['public']
        public.update(scope=r['next_attention']['scope'],parents=r['coding']['relational']['inputs']['parents'],requirements_revision=r['coding']['relational']['inputs']['requirements_revision'],source_snapshots=[r['coding']['snapshot']],editable=list(self.config['editable']),protected={f['path']:f['source_digest'] for f in r['coding']['files'] if not f['editable']})
        self.a=m.admission(self.root/'adapter',origin,self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=2,max_executions=1,correction_allowance=1)
        a=self.adapter();view=a.native('request',None);p=a.proposal(view);preview=a.native('preview',{'response':p['response']})
        a.review({**a.native('request',None),'response_digest':preview['response_digest']},p['response'])
        self.assertEqual(len(self.bodies),2)
        for body in self.bodies:
            self.assertGreater(len(m.base.capacity.serialized(body)),32768)
            self.assertLessEqual(m.base.capacity.prompt_upper(body)+32768,m.base.capacity.CONTEXT_TOKENS)
            context=json.loads(body['messages'][1]['content'])
            encoded=json.dumps(context)
            for source in self.config['editable'].values():self.assertIn(json.dumps(source)[1:-1],encoded)
            self.assertNotIn(self.config['protected']['browser-harness.cjs'],encoded)

    def test_serialized_review_representation_after_correction_and_restart(self):
        self.bad_once=True;original=self.transport
        def transport(body,timeout):
            result=original(body,timeout);data=json.loads(body['messages'][1]['content'])
            if 'criteria' not in data and isinstance(result['value'].get('alternatives',[{}])[0].get('edits'),list):
                result['value']['alternatives'][0]['expectation']={'parents':list(data['proposal_references']['parents']),'anticipated':'Authored label update','grounds':'Source edit','uncertainty':'Untested'}
            return result
        a=self.adapter();a.injected_transport=transport;view=a.native('request',None);p=a.proposal(view)
        preview=a.native('preview',{'response':p['response']});review_view={**a.native('request',None),'response_digest':preview['response_digest']}
        a.review(review_view,p['response']);payload=json.loads(self.bodies[-1]['messages'][1]['content'])
        representation=next(v for v in payload['evidence'].values() if isinstance(v,dict) and v.get('version')=='authenticated-proposal-representation@v1')
        self.assertEqual(representation['model_submission']['alternatives'][0]['expectation']['parents'],['p0'])
        self.assertEqual(p['response']['pursuit_expectations']['0']['parents'],list(representation['reference_catalog']['parents'].values()))
        self.assertEqual(payload['criteria'],m.base.review_contract.CRITERIA)
        self.assertGreater(len(self.bodies[1]['messages']),2)
        done=a.read()[2];key=next(k for k in done if k.startswith('generated_'));ident=key.removeprefix('generated_');fresh=self.adapter()
        actual=fresh.proposal_representation_evidence(self.c.request(),fresh.verify_owned_output(ident),ident,p['response'],done[key]['return_basis'])
        self.assertEqual(actual,representation);self.assertEqual(self.ledger.status()['calls'],3)

    def test_prior_criticism_bound_to_origin_and_current_source(self):
        prior={'source_snapshot':self.a['origin']['public']['source_snapshots'][0],'scope':self.a['origin']['public']['scope'],'parents_sha256':m.e.sha(self.a['origin']['public']['parents']),'failed_response_sha256':'f'*64,'assessment':'Authored independently checked criticism, not observed gameplay','attribution':'Authored reviewer','status':'unadmitted_response_independently_assessed'}
        self.a['origin']['prior_review_feedback']=prior
        a=self.adapter();view=a.native('request',None);_,context=a.project(view);self.assertEqual(context['review_criticism']['items'],[prior])
        prior['source_snapshot']='e'*64
        with self.assertRaises(ValueError):a.project(view)
        prior['source_snapshot']=self.a['origin']['public']['source_snapshots'][0];prior['attribution']=''
        with self.assertRaises(ValueError):a.project(view)

    def test_host_metadata_regenerates_entire_review_not_identifier_repair(self):
        a=self.adapter();view=a.native('request',None);p=a.proposal(view)
        preview=a.native('preview',{'response':p['response']});original=self.transport;first=True
        def malformed(body,timeout):
            nonlocal first
            self.assertNotIn('Return {review_id',body['messages'][0]['content'])
            result=original(body,timeout)
            if first:
                first=False;result['value']['review_id']='wrong-host-field'
            return result
        a.injected_transport=malformed
        review=a.review({**a.native('request',None),'response_digest':preview['response_digest']},p['response'])
        self.assertEqual(review['judgment']['decision'],'approve')
        self.assertEqual(self.ledger.status()['calls'],3)
        self.assertEqual(len(a.corrections.events()),1)
        self.assertEqual(self.bodies[1]['messages'],self.bodies[2]['messages'][:-1])
        self.assertIn('unsupported_field',self.bodies[2]['messages'][-1]['content'])
        self.assertEqual(len([k for k in a.read()[2] if k.startswith('rejected_')]),1)

    def test_semantic_rejection_feedback_changes_fresh_proposal_and_restart(self):
        # Authored judgments/code, real native state; no paid calls or execution.
        self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=3,max_executions=0,correction_allowance=1)
        a=self.adapter();original=self.transport;proposal_number=0
        def transport(body,timeout):
            nonlocal proposal_number
            data=json.loads(body['messages'][1]['content']);result=original(body,timeout)
            if 'criteria' in data:
                result['value']['verdict']='reject'
                finding=next(iter(result['value']['findings'].values()));finding.update(finding='not_met',explanation='Authored rejection: proposed claim lacks observed movement evidence')
            else:
                proposal_number+=1
                if proposal_number==2:
                    self.assertIn('review_criticism',data)
                    self.assertFalse(data['review_criticism']['items'][0]['executed'])
                    result['value']={'abstain':'Authored stop after receiving rejection; not forced agreement'}
            return result
        a.injected_transport=transport;view=a.native('request',None);root=self.root/'reject-cycle'
        ca={'version':'local-project-cycle@v3','journal':str(root),'scope':view['scope'],'max_decisions':3,'max_executions':0,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3}
        cycle=m.loop.Cycle(root,ca,a.native,a.proposal,a.review);result=cycle.run()
        self.assertEqual(result['status'],'proposal_abstention');self.assertEqual(self.ledger.status()['calls'],3)
        self.assertEqual(len(a.corrections.events()),0)
        self.assertEqual(len([k for k in cycle.read()[2] if k.startswith('rejected_proposal_')]),1)
        before=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter();self.assertEqual(m.loop.Cycle(root,ca,fresh.native,fresh.proposal,fresh.review).run(),result);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())

    def test_unknown_bill_stops_without_redispatch(self):
        original=self.transport
        def unknown(body,timeout):
            result=original(body,timeout);result['receipt']['usd']=None;return result
        a=m.Adapter(self.root/'adapter',self.a,self.c,self.ledger,unknown)
        with self.assertRaises(m.d.Blocked):a.proposal(a.native('request',None))
        self.assertTrue(self.ledger.status()['stopped']);self.assertEqual(self.ledger.status()['calls'],1)
        with self.assertRaises(Exception):a.proposal(a.native('request',None))
        self.assertEqual(self.ledger.status()['calls'],1)
    def test_length_then_complete_multifile_native_proposal_review_and_replay(self):
        a=self.adapter();view=a.native('request',None);first=True
        def incomplete(body,timeout):
            nonlocal first
            result=self.transport(body,timeout)
            self.assertEqual(body['max_completion_tokens'],32768)
            if first:
                first=False;result['value']=None
                result['receipt'].update(http_status=200,response_status='received',finish_reason='length',error_code='incomplete_output',parse_location='wire',usage_status='reported')
            return result
        a.injected_transport=incomplete
        p=a.proposal(view);preview=a.native('preview',{'response':p['response']})
        review=a.review({**a.native('request',None),'response_digest':preview['response_digest']},p['response'])
        self.assertEqual(review['judgment']['decision'],'approve')
        self.assertEqual(self.ledger.status()['calls'],3)
        self.assertEqual(len(a.corrections.events()),1)
        self.assertEqual(self.bodies[0]['messages'],self.bodies[1]['messages'][:-1])
        self.assertEqual(len(p['response']['proposals'][0]['patches']),2)
        fresh=self.adapter();self.assertEqual(len(fresh.corrections.events()),1)
        self.assertEqual(self.c.budget()['unsettled_requests'],0)
    def test_relationship_text_uses_aggregate_without_rewriting(self):
        a=self.adapter();view=a.native('request',None);request=self.c.request()
        relation={'parents':[p['id'] for p in request['coding']['relational']['inputs']['parents']], 'relationship':'x'*519,'aspiration':'Authored fixture aspiration','discriminator':'Authored check','revises':None,'catalysts':[]}
        # Structural-only control isolates text from the separately tested lineage gate.
        from unittest.mock import patch
        with patch.object(m.web.e,'validate_relational_lineage'):
            for size in (519,6000):
                relation['relationship']='x'*size
                response=m.web.bind_edit(request,{'relational':relation},lambda _:None)
                self.assertEqual(response['proposals'][0]['proposal']['relationship'],relation['relationship'])
            relation['relationship']='x'*(m.web.NATIVE_RESPONSE_LIMIT+1)
            with self.assertRaises(ValueError) as raised:m.web.bind_edit(request,{'relational':relation},lambda _:None)
            self.assertIn('"max_utf8_bytes": 131072',raised.exception.diagnostic)
        # Catalyst authority is now validated by validate_relational_lineage, not
        # a blanket browser prohibition. Do not test it with that validator mocked.
        bad={**relation,'relationship':'Authored valid-size meaning','parents':['a'*64,'b'*64],'catalysts':['forbidden']}
        with patch.object(m.web.e,'relational_contract',return_value={'parent_ids':bad['parents'],'catalyst_references':[],'allowed_lineage':[]}):
            with self.assertRaises(m.e.StageValidationError) as rejected:m.web.bind_edit(request,{'relational':bad},lambda _:None)
            self.assertEqual(rejected.exception.code,'unavailable_catalyst_or_parent_loss')
        with self.assertRaises(ValueError):m.web.bind_edit(request,{'relational':{**relation,'parents':['f'*64,'e'*64]}},lambda _:None)
    def test_browser_size_regeneration_and_native_boundary(self):
        self.oversize_once=True;a=self.adapter();view=a.native('request',None)
        p=a.proposal(view)
        self.assertEqual(self.ledger.status()['calls'],2)
        self.assertEqual(len(a.corrections.events()),1)
        feedback=json.loads(self.bodies[1]['messages'][-1]['content'])['protocol_correction']
        self.assertEqual(feedback['native_structure']['actual_utf8_bytes'],1048577)
        self.assertEqual(feedback['native_structure']['max_utf8_bytes'],1048576)
        self.assertEqual(self.bodies[0]['messages'],self.bodies[1]['messages'][:-1])
        request=self.c.request()
        candidate={'edits':[{'path':'app.js','old_text':self.config['editable']['app.js'],'new_text':'x'*65536}],'rationale':'Authored exact byte boundary'}
        response=m.web.bind_edit(request,candidate,lambda _:None)
        a.native('preview',{'response':response})
        bad=copy.deepcopy(response);bad['proposals'][0]['patches'][0]['new_text']='x'*131073
        with self.assertRaises(Exception):a.native('preview',{'response':bad})
        candidate['edits'][0]['old_text']='not present'
        candidate['edits'][0]['new_text']='x'*1048577
        with self.assertRaises(m.e.StageValidationError) as rejected:m.web.bind_edit(request,candidate,lambda _:None)
        self.assertEqual(rejected.exception.code,'stale_edit_span')
    def test_scoped_inspection_is_counted_and_terminal_replay_inert(self):
        a=self.adapter();view=a.native('request',None);responses=[{'inspect':{'path':'app.js','offset':0}},{'abstain':'Authored stop after source inspection'}]
        def inspect(body,timeout):
            result=self.transport(body,timeout);result['value']=responses.pop(0);return result
        a.injected_transport=inspect
        root=self.root/'inspect-cycle';ca={'version':'local-project-cycle@v3','journal':str(root),'scope':view['scope'],'max_decisions':2,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3}
        result=m.loop.Cycle(root,ca,a.native,a.proposal,a.review).run()
        self.assertEqual(result['status'],'proposal_abstention');self.assertEqual(self.ledger.status()['calls'],2)
        self.assertEqual(self.calls[1]['source_access']['current_view'],{'path':'app.js','offset':0})
        before=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter()
        self.assertEqual(m.loop.Cycle(root,ca,fresh.native,fresh.proposal,fresh.review).run(),result)
        self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
        request,context=fresh.project(fresh.native('request',None))
        for item in [{'path':'probe.json','offset':0},{'path':'app.js','offset':-1}]:
            with self.assertRaises(ValueError):fresh.decode_proposal({'inspect':item},request,context,view)
    def test_actual_cli_check_no_dispatch(self):
        ca={'version':'local-project-cycle@v3','journal':str(self.root/'cycle'),'scope':self.a['origin']['public']['scope'],'max_decisions':2,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3}
        for name,value in [('adapter.json',self.a),('cycle.json',ca),('campaign.json',self.manifest),('upgrade.json',None)]:
            (self.root/name).write_text(json.dumps(value))
        result=subprocess.run([sys.executable,str(Path(m.base.__file__).with_name('engine-native-cycle.py')),'--browser-v1','check','--admission',str(self.root/'adapter.json'),'--expected-sha256',m.e.sha(self.a),'--cycle-admission',str(self.root/'cycle.json'),'--campaign-manifest',str(self.root/'campaign.json'),'--campaign-upgrade',str(self.root/'upgrade.json')],capture_output=True,text=True,timeout=90)
        self.assertEqual(result.returncode,0,result.stderr)
        value=json.loads(result.stdout);self.assertEqual(value['provider_calls'],0);self.assertEqual(value['target_executions'],0)
        self.assertEqual(self.ledger.status()['calls'],0)
    def test_initial_no_returns_contract_and_honest_abstention(self):
        a=self.adapter();view=a.native('request',None);request,context=a.project(view)
        self.assertEqual(context['browser_returns'],[])
        self.assertEqual(context['return_evidence']['cases'],[])
        wire=json.loads(json.dumps(a.proposal_body(view,context)))
        exported=json.loads(wire['messages'][1]['content'])
        self.assertEqual(exported['proposal_responsibilities'],m.web.RESPONSIBILITIES)
        self.assertIn('not prerequisites',exported['proposal_responsibilities']['generator'])
        self.assertIn('controller previews proposals, obtains independent review, and executes only with authorization',wire['messages'][0]['content'])
        before_sources={p:(self.c.directory/'coding-work'/p).read_bytes() for p in self.config['editable']}
        def abstain(body,timeout):
            result=self.transport(body,timeout);result['value']={'abstain':'Authored no useful proposal judgment; not overridden'};return result
        a.injected_transport=abstain
        root=self.root/'abstain-cycle';ca={'version':'local-project-cycle@v3','journal':str(root),'scope':view['scope'],'max_decisions':2,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3}
        result=m.loop.Cycle(root,ca,a.native,a.proposal,a.review).run()
        self.assertEqual(result['status'],'proposal_abstention');self.assertEqual(self.ledger.status()['calls'],1);self.assertEqual(self.ledger.status()['execution_groups'],0)
        self.assertEqual(before_sources,{p:(self.c.directory/'coding-work'/p).read_bytes() for p in self.config['editable']})
    def test_generation_review_correction_and_stale_source(self):
        self.bad_once=True;a=self.adapter();self.adapter_instance=a
        view=a.native('request',None);p=a.proposal(view);v=a.native('preview',{'response':p['response']});review=a.review({**a.native('request',None),'response_digest':v['response_digest']},p['response'])
        generator=json.loads(self.bodies[0]['messages'][1]['content'])
        reviewer=json.loads(self.bodies[-1]['messages'][1]['content'])
        review_context=next(v for k,v in reviewer['evidence'].items() if reviewer['reference_catalog']['aliases'][k].startswith('context:'))
        self.assertTrue(all(k.startswith('e') for k in reviewer['evidence']))
        self.assertEqual(generator['proposal_responsibilities'],review_context['proposal_responsibilities'])
        self.assertEqual(generator['browser_returns'],[])
        self.assertEqual(review['judgment']['decision'],'approve');self.assertEqual(self.ledger.status()['calls'],3)
        self.assertEqual(len(a.corrections.events()),1);self.assertEqual(self.c.budget()['unsettled_requests'],0)
        self.assertNotIn('chromium.launch',json.dumps(self.calls));self.assertNotIn(self.config['protected']['probe.json'],json.dumps(self.calls))
        forged=copy.deepcopy(review['judgment']);forged['reviewer']='separate-runtime-invocation:'+'f'*64
        with self.assertRaises(ValueError):a.verify_owned_review(forged,p['response'])
        original=self.c.request
        foreign=copy.deepcopy(original());foreign['coding']['relational']['inputs']['returns']=[{'receipt':'f'*64}]
        self.c.request=lambda:foreign
        with self.assertRaisesRegex(ValueError,'unowned browser receipt'):a.project(a.native('request',None))
        self.c.request=original
        foreign=copy.deepcopy(original());foreign['coding']['relational']['history']=[{'proposal':{'parents':[p['id'] for p in self.a['origin']['public']['parents']],'relationship':'foreign','aspiration':'foreign','discriminator':'foreign','revises':None,'catalysts':[]}}]
        self.c.request=lambda:foreign
        with self.assertRaisesRegex(ValueError,'unowned relationship'):a.project(a.native('request',None))
        self.c.request=original
        before=len(self.calls);(self.c.directory/'coding-work/app.js').write_text('// foreign source')
        with self.assertRaises(ValueError):a.proposal(a.native('request',None))
        self.assertEqual(before,len(self.calls))
    @unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1','sealed authored worker opt-in')
    def test_rejection_then_fresh_review_and_actual_browser_execution(self):
        self.playable=True
        self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=3,max_executions=1,correction_allowance=1)
        a=self.adapter();self.adapter_instance=a;original=self.transport;reviews=0;proposals=0
        def transport(body,timeout):
            nonlocal reviews,proposals
            data=json.loads(body['messages'][1]['content']);result=original(body,timeout)
            if 'criteria' in data:
                reviews+=1
                if reviews==1:
                    result['value']['verdict']='reject';next(iter(result['value']['findings'].values())).update(finding='not_met',explanation='Authored claim has not been observed; propose honestly')
            else:
                proposals+=1
                if proposals==2:
                    self.assertIn('review_criticism',data)
                    result['value']['alternatives'][0]['rationale']='Authored replacement: functionality is untested until the protected worker returns evidence'
            return result
        a.injected_transport=transport;root=self.root/'reject-execute-cycle';view=a.native('request',None)
        ca={'version':'local-project-cycle@v3','journal':str(root),'scope':view['scope'],'max_decisions':3,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3}
        result=m.loop.Cycle(root,ca,a.native,a.proposal,a.review).run()
        self.assertEqual(result['status'],'review_required');self.assertEqual(self.c.cli('project','status')['execution_status'],'passed')
        self.assertEqual(self.ledger.status()['calls'],4);self.assertEqual(self.ledger.status()['execution_groups'],1);self.assertEqual(len(a.corrections.events()),0)
        before=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter();self.adapter_instance=fresh
        self.assertEqual(m.loop.Cycle(root,ca,fresh.native,fresh.proposal,fresh.review).run(),result);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())

    @unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1','sealed authored worker opt-in')
    def test_complete_shared_cycle_real_browser_and_inert_restart(self):
        self.playable=True
        a=self.adapter();self.adapter_instance=a;root=self.root/'cycle';view=a.native('request',None)
        ca={'version':'local-project-cycle@v3','journal':str(root),'scope':view['scope'],'max_decisions':2,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3}
        result=m.loop.Cycle(root,ca,a.native,a.proposal,a.review).run()
        self.assertEqual(result['status'],'review_required')
        self.assertTrue(all(len(json.dumps(body,ensure_ascii=False).encode())<=m.base.capacity.PAYLOAD_BYTES for body in self.bodies))
        self.assertGreater(len((self.c.directory/'coding-work/app.js').read_bytes()),2048)
        self.assertEqual(self.ledger.status()['calls'],2);self.assertEqual(self.ledger.status()['execution_groups'],1)
        self.assertEqual(self.c.cli('project','status')['execution_status'],'passed')
        # Actual receipt facts survive projection and cannot be replaced by fabricated history.
        request,context=a.project(a.native('request',None));self.assertTrue(context['browser_returns'])
        before=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter();self.adapter_instance=fresh
        self.assertEqual(m.loop.Cycle(root,ca,fresh.native,fresh.proposal,fresh.review).run(),result)
        self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
        self.assertFalse(self.c.cli('project','acceptance','status')['accepted'])
        # A new admission, not a reopened terminal journal, carries an explicitly
        # reviewed public checkpoint and genuinely attributed (here authored) feedback.
        completion=self.c.cli('project','acceptance','status');budget=self.c.budget()
        cp=m.web.checkpoint(request,completion,budget)
        feedback=m.web.feedback_draft(cp,request,completion,'authored-feedback-not-real-player',{'controls':'Authored next-session guidance: retain usability and improve contrast'})
        self.c.json_cli(('project','acceptance','feedback'),feedback['native_feedback'])
        after=self.c.request();public=copy.deepcopy(self.a['origin']['public'])
        public.update(parents=after['coding']['relational']['inputs']['parents'],source_snapshots=[after['coding']['snapshot']],public_returns=[m.e.sha(x) for x in after['coding']['relational']['inputs']['returns']],public_relationships=[m.e.sha(x) for x in after['coding']['relational']['history']])
        self.assertEqual([p['id'] for p in public['parents']],[p['id'] for p in self.a['origin']['public']['parents']])
        origin={'directory':str(self.c.directory),'public':public,'initial_sources':{f['path']:f['source'] for f in after['coding']['files'] if f['editable']}}
        new=m.admission(self.root/'next-adapter',origin,self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=2,max_executions=1)
        next_adapter=m.Adapter(self.root/'next-adapter',new,self.c,self.ledger,self.transport)
        _,next_context=next_adapter.project(next_adapter.native('request',None));self.assertTrue(next_context['browser_returns'])
        self.assertEqual(self.c.budget()['model_calls'],budget['model_calls'])
        self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
        self.assertEqual(next_context['parents'],public['parents'])
        later=json.loads(json.dumps(next_adapter.proposal_body(next_adapter.native('request',None),next_context)))
        later_context=json.loads(later['messages'][1]['content'])
        self.assertEqual(later_context['proposal_responsibilities'],m.web.RESPONSIBILITIES)
        self.assertIn('retain usability and improve contrast',json.dumps(later_context['parents']))
        self.assertTrue(later_context['browser_returns'])
        self.assertIn('Do not invent',later_context['proposal_responsibilities']['evidence'])
        tampered=copy.deepcopy(new);tampered['origin']['public']['public_returns']=[];tampered['directory']=str(self.root/'missing-proof')
        missing=m.Adapter(self.root/'missing-proof',tampered,self.c,self.ledger,self.transport)
        with self.assertRaisesRegex(ValueError,'unowned browser receipt'):missing.project(missing.native('request',None))
        if os.getenv('TSR_BROWSER_ADAPTER_REPORT'):
            Path(os.environ['TSR_BROWSER_ADAPTER_REPORT']).write_text(json.dumps({'authored_transport_only':True,'actual_sealed_browser':True,'result':result,'ledger':self.ledger.status(),'context':context,'accepted':False},indent=2)+'\n')
if __name__=='__main__':unittest.main()
