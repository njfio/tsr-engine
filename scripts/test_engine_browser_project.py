"""Public projection tests; optional real-native authored request cross-language check."""
import copy
import importlib.util
import json
import os
from pathlib import Path
import unittest
s=importlib.util.spec_from_file_location('browser_project',Path(__file__).with_name('engine-browser-project.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class BrowserProjection(unittest.TestCase):
    def setUp(self):
        h='a'*64;p={'requirement':'playability','id':'b'*64,'meaning':'Authored control clarity aim'}
        self.r={'protocol':'fixture','state_root':h,'next_attention':{'scope':h,'state_root':h,'source_snapshot':h,'binding':h},'coding':{'kind':'browser_project@v1','browser_isolation':{'chromium_sandbox':True,'outer_container_required':True},'snapshot':h,'files':[{'path':'index.html','source':'<p>Start</p>','source_digest':h,'inspection_handle':h,'editable':True},{'path':'app.js','source':'// fixture','source_digest':h,'inspection_handle':h,'editable':True},{'path':'probe.json','source_digest':h,'editable':False}],'cases':[{'id':'smoke'}],'relational':{'narrative_capacity':{'profile':'browser-aggregate-text@v1','aggregate_response_bytes':131072,'per_field':'nonblank; no additional field-length cap'},'binding':h,'history':[],'inputs':{'snapshot':h,'requirements_revision':h,'parents':[p],'returns':[]}}}}
        self.o=self.origin(self.r)
    def origin(self,r):
        c=r['coding'];i=c['relational']['inputs']
        return {'version':'public-browser-origin@v1','scope':r['next_attention']['scope'],'requirements_revision':i['requirements_revision'],'parents':i['parents'],'protected':{f['path']:f['source_digest'] for f in c['files'] if not f['editable']},'editable':[f['path'] for f in c['files'] if f['editable']],'public_brief':'Authored fixture only, not an engine-produced game','source_snapshots':[c['snapshot']],'public_relationships':[],'public_returns':[m.e.sha(x) for x in i.get('returns',[])]}
    def test_projection_is_explicit_complete_and_stable(self):
        a=m.project(self.r,self.o,context_budget=16000);self.assertEqual(a,m.project(copy.deepcopy(self.r),self.o,context_budget=16000));self.assertEqual(len(a['files']),2)
        self.assertEqual(a['proposal_responsibilities'],m.RESPONSIBILITIES)
        self.assertIn('controller performs native preview after receiving',a['constraints']['edits'])
        self.assertNotIn('protected',a);self.assertNotIn('future_conditions',a)
    def test_report_sha256_is_distinct_full_file_identity_across_windows(self):
        import hashlib
        self.r['coding']['files'][1]['source']='const marker=1;\n'*2000
        source=self.r['coding']['files'][1]['source']
        first=m.project(self.r,self.o,context_budget=16000)
        f=next(f for f in first['files'] if f['path']=='app.js')
        self.assertFalse(f['window']['complete'])
        self.assertEqual(f['source_sha256'],hashlib.sha256(source.encode()).hexdigest())
        self.assertNotEqual(f['source_digest'],f['source_sha256'])
        later=m.project(self.r,self.o,{'path':'app.js','offset':f['window']['next_offset']},16000)
        self.assertEqual(next(x for x in later['files'] if x['path']=='app.js')['source_sha256'],f['source_sha256'])
        self.r['coding']['files'][1]['source']+='// changed'
        changed=m.project(self.r,self.o,context_budget=16000)
        self.assertNotEqual(next(x for x in changed['files'] if x['path']=='app.js')['source_sha256'],f['source_sha256'])
    def test_protected_stale_cross_scope_history_and_size_fail(self):
        for change in [lambda r:r['coding']['files'][2].update(source='protected secret'),lambda r:r['coding']['browser_isolation'].update(chromium_sandbox='false'),lambda r:r['coding'].update(snapshot='c'*64),lambda r:r['next_attention'].update(scope='c'*64),lambda r:r['coding']['relational']['history'].append({'private':'not admitted'}),lambda r:r['coding']['relational']['inputs'].update(public_desires={'private':'not admitted'}),lambda r:r['coding']['relational']['inputs'].update(returns=[{'private':'unadmitted diagnostic'}]),lambda r:r['coding']['files'][0].update(source='x'*1048577)]:
            r=copy.deepcopy(self.r);change(r)
            with self.assertRaises(ValueError):m.project(r,self.o)
    def test_history_envelope_legacy_current_and_malformed(self):
        from types import SimpleNamespace
        helper=m.load('history_envelope_test','engine-browser-desire.py')
        inputs=self.r['coding']['relational']['inputs']
        adapter=SimpleNamespace(admission={'origin':{'public':{}}},read=lambda:({},[],{}))
        before=copy.deepcopy(self.r)
        self.assertEqual(helper.public_history(inputs),[])
        helper.verify_history(adapter,self.r)
        self.assertEqual(self.r,before)
        for bad in (None,{},[],False,{'private':'not admitted'}, {'history':None,'active':[]}, {'history':{},'active':[]}, {'history':[]}, {'history':[],'active':None}, {'history':[None],'active':[]}, {'history':[],'active':[0]}):
            request=copy.deepcopy(self.r);request['coding']['relational']['inputs']['public_desires']=bad
            original=copy.deepcopy(request)
            for operation in (lambda:helper.public_history(request['coding']['relational']['inputs']),lambda:helper.verify_history(adapter,request),lambda:m.project(request,self.o)):
                with self.subTest(bad=bad),self.assertRaisesRegex(ValueError,'public meaning history:'):operation()
            self.assertEqual(request,original)
        inputs['acceptance_parents']=copy.deepcopy(inputs['parents'])
        inputs['public_desires']={'active':copy.deepcopy(inputs['parents']),'history':[],'current':copy.deepcopy(inputs['parents']),'authority':'authored native-shaped envelope'}
        helper.verify_history(adapter,self.r)
        self.assertEqual(m.project(self.r,self.o)['public_desire_history'],[])
        row={'revision':'a'*64,'previous':{'meaning':'Before'},'next':{'meaning':'After'},'superseded':True,'withdrawn':False,'review_note_digest':'b'*64}
        inputs['public_desires']['history']=[row];before=copy.deepcopy(self.r)
        with self.assertRaisesRegex(ValueError,'unadmitted public meaning export'):m.project(self.r,self.o)
        with self.assertRaisesRegex(ValueError,'unowned public meaning history'):helper.verify_history(adapter,self.r)
        origin=copy.deepcopy(self.o);origin['public_meanings']=[m.e.sha(row)]
        projected=m.project(self.r,origin)['public_desire_history']
        self.assertEqual(projected,[{k:row[k] for k in ('revision','previous','next','superseded','withdrawn')}])
        self.assertEqual(self.r,before)
    def test_exact_multifile_decode_and_native_preview_required(self):
        seen=[];v={'edits':[{'path':'index.html','old_text':'Start','new_text':'Play'},{'path':'app.js','old_text':'fixture','new_text':'revision'}],'rationale':'Authored change'}
        response=m.bind_edit(self.r,v,seen.append);self.assertEqual(seen,[response]);self.assertEqual(len(response['proposals'][0]['patches']),2)
        for edits in [[v['edits'][0]]*2,[{'path':'probe.json','old_text':'x','new_text':'y'}],[{'path':'app.js','old_text':'absent','new_text':'y'}]]:
            with self.assertRaises(ValueError):m.bind_edit(self.r,{'edits':edits,'rationale':'Authored'},seen.append)
        deleted=m.bind_edit(self.r,{'edits':[{'path':'app.js','old_text':'fixture','new_text':''}],'rationale':'Authored deletion'},seen.append)
        self.assertEqual(deleted['proposals'][0]['patches'][0]['new_text'],'')
        def denied(_):raise ValueError('native stale rejection')
        with self.assertRaisesRegex(ValueError,'native stale'):m.bind_edit(self.r,v,denied)
    def test_utf8_span_aggregate_and_exact_failure_priority(self):
        def value(n):return {'edits':[{'path':'app.js','old_text':'// fixture','new_text':'é'*n}],'rationale':'Authored byte test'}
        response=m.bind_edit(self.r,value(4096),lambda _:None)
        self.assertEqual(len(response['proposals'][0]['patches'][0]['new_text'].encode()),8192)
        with self.assertRaises(ValueError) as c:m.bind_edit(self.r,value(524289),lambda _:None)
        self.assertIn('1048578',c.exception.diagnostic)
        v=value(524289);v['edits'][0]['old_text']='not in source'
        with self.assertRaisesRegex(ValueError,'stale_edit_span'):m.bind_edit(self.r,v,lambda _:None)
    def test_large_source_windows_are_explicit_and_navigate(self):
        r=copy.deepcopy(self.r);r['coding']['files'][1]['source']='const marker=1;\n'*2000
        c=m.project(r,self.o,context_budget=16000);f=next(f for f in c['files'] if f['path']=='app.js')
        self.assertFalse(f['window']['complete']);offset=f['window']['next_offset']
        later=m.project(r,self.o,{'path':'app.js','offset':offset},16000)
        g=next(f for f in later['files'] if f['path']=='app.js')
        self.assertEqual(g['window']['start'],offset);self.assertLessEqual(len(m.e.canonical(later)),16000)
        self.assertEqual(g['source'].encode(),r['coding']['files'][1]['source'].encode()[offset:g['window']['end']])
        with self.assertRaises(ValueError):m.project(r,self.o,{'path':'probe.json','offset':0})
    def test_checkpoint_feedback_is_attributed_advisory_not_authority(self):
        self.r['coding']['relational']['inputs']['parents'].append({'requirement':'depth','id':'d'*64,'meaning':'Keep meaningful choices available'})
        c={'requirements_revision':'a'*64,'decision_record':None};b={'model_calls':0};cp=m.checkpoint(self.r,c,b)
        draft=m.feedback_draft(cp,self.r,c,'authored-test-message-not-live-feedback',{'playability':'Authored preference, not measured satisfaction'})
        self.assertEqual(draft['native_feedback']['requirements_revision'],c['requirements_revision']);self.assertNotIn('authority',draft['native_feedback']);self.assertEqual(draft['native_feedback']['guidance']['depth'],'Keep meaningful choices available');self.assertEqual(draft['retained_contributors'],['depth'])
        r=copy.deepcopy(self.r);r['state_root']='f'*64
        with self.assertRaises(ValueError):m.feedback_draft(cp,r,c,'id',{'playability':'changed'})
        with self.assertRaises(ValueError):m.checkpoint(self.r,c,{'unresolved_tool_requests':1})
        with self.assertRaises(ValueError):m.feedback_draft(cp,self.r,c,'id',{'invented_new_permission':'allowed'})
    @unittest.skipUnless(os.getenv('TSR_BROWSER_AUTHORED_REQUEST'),'run native authored test first for cross-language integration')
    def test_actual_native_serialization(self):
        r=json.loads(Path(os.environ['TSR_BROWSER_AUTHORED_REQUEST']).read_text());o=self.origin(r);context=m.project(r,o)
        self.assertEqual(len(context['files']),2);self.assertTrue(context['browser_returns']);self.assertIn('browser-observation@v1',json.dumps(context['browser_returns']))
        self.assertNotIn('chromium.launch',json.dumps(context));self.assertNotIn('checks',json.dumps(context['files']))
        returns=context['evidence_applicability']['native_returns'];self.assertTrue(returns)
        self.assertTrue(all(v['source_snapshot']==r['coding']['snapshot'] and v['applicability']=='current_source' for v in returns))
if __name__=='__main__':unittest.main()
