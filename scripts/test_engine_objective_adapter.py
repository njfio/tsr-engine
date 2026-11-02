"""Real native CLI/common Cycle with authored transport and test-binary worker only.

No provider, container, generated program or campaign target execution. The outputs
are explicitly authored wiring fixtures, not evidence that the seed solves anything.
"""
import copy,datetime,time,importlib.util,json,os,subprocess,tempfile,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('objective_adapter',Path(__file__).with_name('engine-objective-adapter.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class FullNative(unittest.TestCase):
 def test_three_native_rounds_shared_transport_review_restart(self):
  if not os.environ.get('TSR_OBJECTIVE_TEST_WORKER_BIN'):self.skipTest('explicit cfg(test) worker required')
  import test_engine_b91_campaign_extension as ext
  accounting=ext.EvaluationExtension(methodName='test_prospective_objective_extension_full_dispatch_chain');accounting.setUp();self.addCleanup(accounting.doCleanups);accounting.test_prospective_objective_extension_full_dispatch_chain()
  ledger=accounting.l;baseline=ledger.status()
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp).resolve();(root/'fixture-only.marker').write_text('objective-native-fixture-only@v1\n');packet=m.e.ROOT/os.environ.get('TSR_OBJECTIVE_TEST_PACKET','experiments/objective-service-v1')
   spec=json.loads((packet/'project-spec.json').read_bytes());public=json.loads((packet/'public.json').read_bytes());conditions=json.loads((packet/'conditions.json').read_bytes())['rounds']
   source=root/'source';source.mkdir()
   for key in ('editable','protected'):
    for path,body in spec[key].items():(source/path).write_text(body)
   (root/'spec.json').write_text(json.dumps(spec));requirements={p['id']:p['meaning'] for p in public['parents']};(root/'requirements.json').write_text(json.dumps(requirements))
   class Native(m.d.Native):
    def cli(self,*args):
     r=subprocess.run([str(m.e.ROOT/'target/debug/tsr'),'engine','--directory',str(self.directory),*args],capture_output=True,text=True,timeout=60)
     if r.returncode:raise ValueError('authored native fixture refused: '+r.stderr)
     return m.e.strict(r.stdout)
   (root/'state').mkdir();c=Native(root/'state');c.cli('project','init','--spec',str(root/'spec.json'),'--source',str(source),'--requirements',str(root/'requirements.json'))
   expiry=(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(hours=2)).isoformat()
   c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':300000,'reason':'Authored finite objective wiring only','non_tool':{'model_calls':24,'model_tokens':983040,'wall_ms':7200000,'patch_bytes':4096,'external_cost_microusd':0}},'--admission')
   c.json_cli(('project','acceptance','feedback'),{'requirements_revision':c.cli('project','acceptance','status')['requirements_revision'],'decision_record':None,'guidance':requirements})
   c.json_cli(('project','objective-configure'),{'state_root':c.request()['state_root'],'conditions':[{'id':v['id'],'input':v['body']} for v in conditions],'service_entry':'service.rs','max_output_bytes':2048,'timeout_ms':1000})
   c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root'])
   origin=m.freeze_origin(c,public,'2'*64,checker_profile=os.environ.get('TSR_OBJECTIVE_TEST_CHECKER'));a=m.admission(root/'adapter',origin,ledger.manifest,expiry)
   fact_history={};bodies=[];workers=[];generated=[];corrupted_review=False;started=time.monotonic()
   def transport(body,timeout):
    nonlocal corrupted_review
    self.assertEqual(adapter.native('request',None)['branch'],'reconcile')
    print('stage',len(bodies)+1,'elapsed_seconds',round(time.monotonic()-started,2),flush=True);bodies.append(copy.deepcopy(body));self.assertEqual(body['response_format'],{'type':'json_object'});data=json.loads(body['messages'][1]['content']);context=data.get('context',data)
    self.assertLessEqual(len(json.dumps(body).encode()),32768)
    receipts=context['objective']['context']['receipts']
    self.assertEqual(context['independent_receipt_checks'],[adapter.receipt_assessment(r) for r in receipts])
    for fact in context['independent_receipt_checks']:
     if fact['receipt'] in fact_history:self.assertEqual(fact_history[fact['receipt']],fact)
     fact_history[fact['receipt']]=copy.deepcopy(fact)
    round_index=context['objective']['context']['round']
    for later in conditions[round_index+1:]:self.assertNotIn(later['body'],json.dumps(context))
    self.assertNotIn(spec['protected']['service.rs'],json.dumps(context));self.assertNotIn(str(c.directory),json.dumps(context))
    if len(bodies)==1:
     value={'alternatives':[{'kind':'edit_set','patches':None,'rationale':'Malformed authored format control'}]}
    elif 'response_schema' in data:
     if not corrupted_review:
      corrupted_review=True;value={'decision':'approve','findings':None}
     else:
      props=data['response_schema']['properties'];value={k:v['const'] for k,v in props.items() if 'const' in v};value['decision']='met' if data['candidate'].get('objective_stage')=='round_review' else 'accept' if data['candidate'].get('objective_stage')=='objective_review' else 'approve';value['evidence']='Evaluator-authored assessment only; not semantic or model evidence.';value['findings']={k:{'finding':'met','basis':'static_source','receipts':[],'explanation':'Authored control, not evidence of correct source','limitations':'The worker and judgment are evaluator-authored'} for k in data['criteria']}
    elif not generated:
     sv=context['source'];patch={k:sv[k] for k in ('path',)};patch.update(old_text=sv['source'],new_text=sv['source']+'\n// Authored enacted patch; stub smoke only.\n')
     value={'alternatives':[{'kind':'edit_set','patches':[patch],'rationale':'Authored patch-to-smoke transition, no target correctness claim'}]};generated.append(value)
    elif len(generated)==1:
     p={'parents':[p['id'] for p in context['parents']],'relationship':'Authored joint means preserve original contributor meanings','aspiration':'Retain the existing continuity and discovery wanting','discriminator':'Observed service outcomes can inform means without forcing new goals','revises':None,'catalysts':[]}
     value={'alternatives':[{'kind':'relational','proposal':p}]};generated.append(value)
    elif 'reconsideration' in context:
     prior=context['relational']['history'][-1];p=copy.deepcopy(prior['proposal']);line=context['relational_contract']['allowed_lineage'][0]
     p.update(revises=line['revises'],development={'original_revision':line['original_revision'],'prior_revision':line['prior_revision'],'retained_meaning':p['aspiration'],'return_reveals':'Authored worker receipt reports a finite output; no live adaptation established','unresolved':'Useful semantic development and actual program behavior remain unproven','proposed_development':None})
     value={'alternatives':[{'kind':'relational','proposal':p,'return_basis':{'binding':context['return_evidence']['binding'],'observed':[r['id'] for r in context['return_evidence']['cases']],'inferred':[],'unknown':['Authored outputs are not actual target program behavior.']}}]};generated.append(value)
    else:
     value={'alternatives':[{'kind':'service','choice':{'rationale':'Authored same strategy to exercise routing, no behavioral claim','success_probability':0.5}}]};generated.append(value)
    return {'value':value,'receipt':{'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':0.01}}
   def worker(view):
    self.assertEqual(view['branch'],'resume' if not workers else 'service_resume');out=root/('output-'+str(len(workers))+'.json');text=(['','11>12,12>21,21>22,22>31,31>32,32>11\n','11>12,12>21,21>22,22>31,31>32,32>11\n','11>21,21>22,22>31,31>32,32>41,41>11\n'] if os.environ.get('TSR_OBJECTIVE_TEST_CHECKER') else ['','11,21,31\n','12,21,31\n','12,31\n'])[len(workers)]
    env={**os.environ,'TSR_OBJECTIVE_FIXTURE_DIRECTORY':str(c.directory),'TSR_OBJECTIVE_FIXTURE_OUTPUT':str(out),'TSR_OBJECTIVE_FIXTURE_STDOUT':text}
    r=subprocess.run([os.environ['TSR_OBJECTIVE_TEST_WORKER_BIN'],'--exact','engine::adaptive::coding::project::objective::tests::fixture_worker','--nocapture'],env=env,capture_output=True,text=True,timeout=90)
    self.assertEqual(r.returncode,0,r.stdout+r.stderr);result=json.loads(out.read_bytes());self.assertEqual(result['target_commands_executed'],0);
    if not workers:
     self.assertEqual(result['result']['verdict'],'passed')
     next_view=c.cli('project','cycle','request');self.assertEqual(next_view['branch'],'propose',json.dumps(next_view))
    workers.append(result);return result
   adapter=m.Adapter(root/'adapter',a,c,ledger,transport,worker=worker)
   request,context=adapter.project(adapter.native('request',None))
   for bad in (None,1,'text',{},[None],[1]):
    with self.assertRaises(m.shape.FormatFailure):adapter.decode_proposal({'alternatives':[{'kind':'edit_set','patches':bad,'rationale':'bad shape'}]},request,context,adapter.native('request',None))
   source_view=context['source'];patch={k:source_view[k] for k in ('path',)};patch.update(old_text=source_view['source'],new_text=source_view['source']+'\n// authored preview only\n')
   preview,_=adapter.decode_proposal({'alternatives':[{'kind':'edit_set','patches':[patch],'rationale':'Authored schema preview only'}]},request,context,adapter.native('request',None));self.assertEqual(preview['proposals'][0]['kind'],'edit_set')
   ca={'version':'local-project-cycle@v5','journal':str(root/'cycle'),'scope':origin['scope'],'max_decisions':9,'max_executions':5,'max_service_rounds':3,'expires_at':expiry,'implementation_sha256':m.d.file_sha(Path(m.loop.__file__))}
   def hook(event,name):
    if event=='after_result' and name=='service_resume_0':raise RuntimeError('fixture checkpoint')
   with self.assertRaisesRegex(RuntimeError,'fixture checkpoint'):m.loop.Cycle(root/'cycle',ca,adapter.native,adapter.proposal,adapter.review,hook=hook).run()
   adapter=m.Adapter(root/'adapter',a,c,ledger,transport,worker=worker)
   result=m.loop.Cycle(root/'cycle',ca,adapter.native,adapter.proposal,adapter.review).run()
   self.assertEqual(result['status'],'objective_final_assessment');self.assertFalse(result['acceptance_recorded']);self.assertEqual(len(workers),4);self.assertEqual(len(bodies),20)
   obj=c.cli('project','objective-status');self.assertEqual(obj['context']['round'],3);self.assertIsNone(obj['context']['final_review']);self.assertEqual(obj['context']['receipts'][1]['previous_output'],obj['context']['receipts'][0]['output'])
   # Unattributed native review prose cannot enter the provider projection.
   from unittest.mock import patch as mock_patch
   original_cli=c.cli
   def foreign(*args):
    v=original_cli(*args)
    if args==('project','objective-status'):
     v=copy.deepcopy(v);v['context']['round_reviews'][0]['evidence']='PRIVATE FOREIGN OPERATOR TEXT'
    return v
   with mock_patch.object(c,'cli',side_effect=foreign):
    with self.assertRaisesRegex(ValueError,'foreign/private'):adapter.project(adapter.native('request',None))
   view=adapter.native('request',None)
   for field in ('state_root','source_snapshot'):
    stale=copy.deepcopy(view);stale[field]='0'*64
    with self.assertRaisesRegex(ValueError,'drift'):adapter.project(stale)
   def substituted(*args):
    v=original_cli(*args)
    if args==('project','objective-status'):
     v=copy.deepcopy(v);v['context']['receipts'][0]['output']='forged output'
    return v
   with mock_patch.object(c,'cli',side_effect=substituted),mock_patch.object(adapter,'receipt_assessment',side_effect=AssertionError('must reject before deriving facts')):
    with self.assertRaisesRegex(ValueError,'foreign native service receipt'):adapter.project(view)
   with mock_patch.object(adapter,'receipt_assessment',return_value={'oversized':'x'*28001}):
    with self.assertRaisesRegex(m.d.Blocked,'complete public objective context'):adapter.project(view)
   self.assertEqual(len(fact_history),3)
   check=adapter.receipt_assessment(obj['context']['receipts'][-1]);bad=copy.deepcopy(obj['context']['receipts'][-1]);bad['output']='999999>11\n' if os.environ.get('TSR_OBJECTIVE_TEST_CHECKER') else '999999\n';other=adapter.receipt_assessment(bad)
   self.assertNotEqual(check['input_sha256'],other['input_sha256']);self.assertFalse(other['assessment'][m.receipt_checker(origin)[1]])
   before=(root/'cycle/cycle.jsonl').read_bytes();self.assertEqual(m.loop.Cycle(root/'cycle',ca,adapter.native,adapter.proposal,adapter.review).run(),result);self.assertEqual(before,(root/'cycle/cycle.jsonl').read_bytes());self.assertEqual(len(bodies),20)
   print('maximum_serialized_request_bytes',max(len(json.dumps(b,ensure_ascii=False,allow_nan=False).encode()) for b in bodies),'total_elapsed_seconds',round(time.monotonic()-started,2),flush=True)
   self.assertEqual(ledger.status()['calls']-baseline['calls'],20);self.assertEqual(ledger.status()['execution_groups']-baseline['execution_groups'],4)
if __name__=='__main__':unittest.main()
