"""Native authored fixture preparation only: no real provider or target executions."""
import copy, importlib.util, json, os, shutil, subprocess, sys, tempfile, unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('semantic_shadow',Path(__file__).with_name('engine-semantic-shadow.py'))
m=importlib.util.module_from_spec(s);s.loader.exec_module(m)

class SemanticShadow(unittest.TestCase):
    def setUp(self):
        if not os.environ.get('TSR_SEMANTIC_SHADOW_WORKER'):self.skipTest('explicit cfg(test) native fixture worker required')
        self.tmp=tempfile.TemporaryDirectory(dir=m.ROOT/'target');self.addCleanup(self.tmp.cleanup)
        self.root=Path(self.tmp.name)/'pair';self.worker=Path(os.environ['TSR_SEMANTIC_SHADOW_WORKER'])
        self.identity=m.prepare(self.root,self.worker)
    def response(self,b):
        c=json.loads(b['messages'][1]['content']);p=copy.deepcopy(c['prior']['proposal']);prior=c['prior']['admission']
        p.update(revises=prior,development={'original_revision':prior,'prior_revision':prior,
            'retained_meaning':p['aspiration'],'return_reveals':'Authored test control, not a live response.',
            'unresolved':'No real target execution or model result.','proposed_development':None})
        return {'value':{'relational':p},'receipt':{'response_status':'received','structural_status':'typed_json','status':'settled',
            'usd':'0.001','usage':{'prompt_tokens':100,'completion_tokens':200,'total_tokens':300},'request_id':'fixture'}}
    def test_pair_native_controls_privacy_and_format_invariance(self):
        manifest=m.ready(self.root,self.identity);self.assertFalse(manifest['live_authorized'])
        contexts={a:json.loads(json.loads((self.root/(a+'.body.json')).read_text())['messages'][1]['content']) for a in m.ARMS}
        self.assertEqual(m.normalize(contexts['reject']),m.normalize(contexts['unknown']))
        q=m.e.Controller(self.root/'native/reject').request();baseline=m.project(q,'reject')
        q['private_review']='PRIVATE canary'
        q['coding']['relational']['inputs']['returns'][0]['observations'][0]['stdout']='PRIVATE canary'
        self.assertEqual(m.project(q,'reject'),baseline)
        extra=copy.deepcopy(q);extra['coding']['relational']['history'][0]['proposal']['unadmitted']='PRIVATE canary'
        with self.assertRaisesRegex(ValueError,'hypothesis fields'):m.project(extra,'reject')
        self.assertNotIn('PRIVATE',json.dumps(m.body(baseline)))
        q=json.loads(json.dumps(q,indent=4));self.assertEqual(m.project(q,'reject'),baseline)
        for c in contexts.values():
            self.assertNotIn('source',c);self.assertNotIn('evidence',c['semantic_return'])
        # Both controls use actual native validator, not merely equivalent Python shape.
        for arm in m.ARMS:
            q=m.e.Controller(self.root/'native'/arm).request()
            self.assertEqual(m.native_validate(self.root,arm,m.control(q),self.worker,m.native_binding(q))['native'],'deferred')
            wrong=m.native_binding(q);wrong['relational']='0'*64
            with self.assertRaisesRegex(ValueError,'response binding'):m.native_validate(self.root,arm,m.control(q),self.worker,wrong)
    def test_restart_cannot_dispatch_twice_or_adopt(self):
        calls=[]
        def fake(b,deadline,expected_model,payload_limit):
            self.assertEqual(payload_limit,32768);self.assertEqual(expected_model,'gpt-6-luna');self.assertEqual(deadline,240)
            self.assertEqual(b['max_completion_tokens'],8192);self.assertNotIn('max_tokens',b)
            calls.append(b);return self.response(b)
        m.run(self.root,self.identity,transport=fake,fixture=True)
        self.assertEqual(len(calls),2);m.ready(self.root,self.identity)
        rows=[json.loads(x) for x in (self.root/'events.jsonl').read_text().splitlines()]
        self.assertEqual(rows[-1]['status'],'complete_advisory_only')
        with self.assertRaises(FileExistsError):m.run(self.root,self.identity,transport=fake,fixture=True)
        self.assertEqual(len(calls),2)
    def test_unknown_preserves_intent_and_blocks_retry(self):
        calls=[]
        def unknown(*a,**kw):calls.append(1);raise TimeoutError('fixture unknown')
        with self.assertRaises(TimeoutError):m.run(self.root,self.identity,transport=unknown,fixture=True)
        rows=[json.loads(x) for x in (self.root/'events.jsonl').read_text().splitlines()]
        self.assertEqual(rows[0]['kind'],'intent');self.assertEqual(rows[-1]['status'],'unknown_completion')
        self.assertEqual(rows[-1]['consumed'],1)
        self.assertEqual(m.inspect(self.root,self.identity)['unknown_reservations'],[0])
        with self.assertRaises(FileExistsError):m.run(self.root,self.identity,transport=unknown,fixture=True)
        self.assertEqual(len(calls),1)
    def test_expired_drift_and_scope_fail_before_dispatch(self):
        path=self.root/'manifest.json';manifest=json.loads(path.read_text());manifest['expires_at']=0
        path.write_text(json.dumps(manifest))
        with self.assertRaisesRegex(ValueError,'chronology'):m.ready(self.root,m.e.sha(manifest))
        created=manifest['created_at'];manifest['created_at']=m.time.time()+3600;manifest['expires_at']=manifest['created_at']+3300;path.write_text(json.dumps(manifest))
        with self.assertRaisesRegex(ValueError,'chronology'):m.ready(self.root,m.e.sha(manifest))
        manifest['created_at']=created;manifest['expires_at']=m.time.time()+3300;path.write_text(json.dumps(manifest))
        (self.root/'native/reject/coding-work/src/lib.rs').write_text('pub fn private_data() {}')
        with self.assertRaisesRegex(ValueError,'drift'):m.ready(self.root,m.e.sha(manifest))
    def test_dependency_drift_and_post_response_source_change(self):
        from unittest.mock import patch
        with patch.object(m,'implementation',return_value={'changed':'dependency'}):
            with self.assertRaisesRegex(ValueError,'implementation drift'):m.ready(self.root,self.identity)
        def stale(b,*a,**kw):
            r=self.response(b)
            (self.root/'native/unknown/coding-work/src/lib.rs').write_text('pub fn changed() {}')
            return r
        m.run(self.root,self.identity,transport=stale,fixture=True)
        rows=[json.loads(x) for x in (self.root/'events.jsonl').read_text().splitlines()]
        self.assertEqual(rows[-1]['status'],'context_drift_after_response')
        self.assertEqual(rows[1]['kind'],'metering');self.assertFalse((self.root/'candidate-0.json').exists())

    def test_malformed_usage_keeps_reported_cost_unknown_tokens(self):
        def bad_usage(b,*a,**kw):
            r=self.response(b);r['receipt']['usage']=None;return r
        m.run(self.root,self.identity,transport=bad_usage,fixture=True)
        result=m.inspect(self.root,self.identity)
        self.assertEqual(result['known_usd_subtotal'],'0.001')
        self.assertEqual(result['unknown_reservations'],[0]);self.assertEqual(result['calls_consumed_or_reserved'],1)

    def test_cli_preflight_fresh_process_unknown_restart_and_expired_inspect(self):
        script=m.ROOT/'scripts/engine-semantic-shadow.py'
        def cli(mode,identity):
            r=subprocess.run([sys.executable,str(script),mode,'--directory',str(self.root),'--manifest-sha256',identity],capture_output=True,text=True,timeout=60)
            self.assertEqual(r.returncode,0,r.stdout+r.stderr);return json.loads(r.stdout)
        self.assertFalse(cli('preflight',self.identity)['live_authorized'])
        def unknown(*args,**kwargs):raise TimeoutError('authored interruption')
        with self.assertRaises(TimeoutError):m.run(self.root,self.identity,transport=unknown,fixture=True)
        before=(self.root/'events.jsonl').read_bytes()
        code="""import importlib.util,sys
from pathlib import Path
s=importlib.util.spec_from_file_location('m',sys.argv[1]);m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
def forbidden(*args,**kwargs):raise AssertionError('redispatch')
try:m.run(Path(sys.argv[2]),sys.argv[3],transport=forbidden,fixture=True)
except FileExistsError:print('blocked_existing_journal')
else:raise AssertionError('unexpected resume')
"""
        r=subprocess.run([sys.executable,'-c',code,str(script),str(self.root),self.identity],capture_output=True,text=True,timeout=60)
        self.assertEqual(r.returncode,0,r.stdout+r.stderr);self.assertIn('blocked_existing_journal',r.stdout)
        path=self.root/'manifest.json';manifest=json.loads(path.read_text());manifest['expires_at']=0;path.write_text(json.dumps(manifest))
        report=cli('inspect',m.e.sha(manifest));self.assertTrue(report['expired']);self.assertFalse(report['dispatch_authorized'])
        self.assertEqual(report['unknown_reservations'],[0]);self.assertEqual((self.root/'events.jsonl').read_bytes(),before)

    def test_interleaved_positive_control_cannot_replace_invalid_verdict(self):
        from unittest.mock import patch
        q=m.e.Controller(self.root/'native/unknown').request();valid=m.control(q);invalid=copy.deepcopy(valid)
        invalid['relational']['parents'][0]='0'*64
        original=m.call_fixture;interleaved=[];paths=[]
        def interleave(root,worker,mode,**values):
            try:original(root,worker,mode,**values)
            finally:
                if mode=='validate':
                    paths.append(values['TSR_SEMANTIC_SHADOW_OUTPUT'])
                    supplied=json.loads(Path(values['TSR_SEMANTIC_SHADOW_RESPONSE']).read_text())
                    if '0'*64 in supplied['proposals'][0]['proposal']['parents'] and not interleaved:
                        interleaved.append(True)
                        self.assertEqual(m.native_validate(self.root,'unknown',valid,self.worker,m.native_binding(q))['native'],'deferred')
        with patch.object(m,'call_fixture',side_effect=interleave):
            with self.assertRaisesRegex(ValueError,'distinct published active parent identities required'):
                m.native_validate(self.root,'unknown',invalid,self.worker,m.native_binding(q))
        self.assertTrue(interleaved);self.assertEqual(len(paths),2);self.assertEqual(len(set(paths)),2)

    def test_copied_root_cannot_reuse_admission(self):
        other=Path(self.tmp.name)/'copied';shutil.copytree(self.root,other)
        with self.assertRaisesRegex(ValueError,'directory binding'):
            m.run(other,self.identity,transport=lambda *_a,**_k:self.fail('copied admission dispatched'),fixture=True)
        self.assertFalse((other/'events.jsonl').exists())

    def test_controls_cannot_consume_dispatch_runway(self):
        from unittest.mock import patch
        actual=m.native_validate;clock=[m.time.time()];count=[0]
        manifest=json.loads((self.root/'manifest.json').read_text())
        def slow_control(*args,**kwargs):
            result=actual(*args,**kwargs);count[0]+=1
            if count[0]==4:clock[0]=manifest['expires_at']-100
            return result
        with patch.object(m.time,'time',side_effect=lambda:clock[0]),patch.object(m,'native_validate',side_effect=slow_control):
            with self.assertRaisesRegex(ValueError,'runway exhausted after controls'):
                m.run(self.root,self.identity,transport=lambda *_a,**_k:self.fail('dispatch after runway loss'),fixture=True)
        self.assertEqual(m.inspect(self.root,self.identity)['calls_consumed_or_reserved'],0)
        cli=m.e.Controller.cli;seen=[0]
        def expired_first_arm(controller,*args):
            result=cli(controller,*args)
            if args==('project-budget',):
                seen[0]+=1
                if seen[0]==3:result['elapsed_wall_ms']=result['limits']['wall_ms']-1000
            return result
        with patch.object(m.e.Controller,'cli',new=expired_first_arm):
            with self.assertRaisesRegex(ValueError,'native validation runway exhausted after controls'):
                m.ready(self.root,self.identity)
        self.assertEqual(seen[0],3)

    def test_unapproved_run_and_output_overrun_stop(self):
        with self.assertRaisesRegex(ValueError,'admission'):m.run(self.root,self.identity,allow_paid=True)
        self.assertFalse((self.root/'events.jsonl').exists())
        def over(b,*a,**kw):
            r=self.response(b);r['receipt']['usage']['completion_tokens']=8193;r['receipt']['usage']['total_tokens']=8293;return r
        m.run(self.root,self.identity,transport=over,fixture=True)
        rows=[json.loads(x) for x in (self.root/'events.jsonl').read_text().splitlines()]
        self.assertEqual(rows[-1]['status'],'unknown_or_overrun');self.assertEqual(rows[-1]['consumed'],1)

if __name__=='__main__':unittest.main()
