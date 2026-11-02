"""Authored offline fixtures only; no engine-created game or paid provider calls."""
import copy,hashlib,importlib.util,json,os,subprocess,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];PACKET=ROOT/'experiments/game-wants-v1'
s=importlib.util.spec_from_file_location('game_wants_packet',PACKET/'prepare.py');p=importlib.util.module_from_spec(s);s.loader.exec_module(p)
from scripts.test_engine_browser_adapter import BrowserAdapter,m
class Packet(unittest.TestCase):
 def test_semantic_parity_and_no_combination_requirement(self):
  history=[{'source':'authored-example','finding':'unknown','claim':'No reachable bank observed in this finite trace','parents':['crossing','growth']}]
  a=p.arm(False,history);b=p.arm(True,history)
  self.assertEqual(a['facts'],b['facts']);self.assertEqual(a['semantic_sha256'],b['semantic_sha256']);self.assertEqual(a['user_prompt_sha256'],b['user_prompt_sha256']);self.assertEqual(a['model'],b['model'])
  self.assertEqual(a['facts']['user_input']['text'],'ok, do a new game, frogger and snake please');self.assertNotEqual(a['formal_transitions'],b['formal_transitions'])
  self.assertEqual(a['facts']['unknown_rule'],p.protocol()['unknown_rule']);self.assertEqual(a['facts']['shared_consequences'],p.protocol()['shared_consequences']);self.assertEqual(a['facts']['observation_policy'],b['facts']['observation_policy']);self.assertIn('Independent games',a['facts']['shared_consequences']);self.assertIn('unknown',p.protocol()['unknown_rule']);self.assertEqual(history,a['facts']['history'])
 def test_stale_partial_and_unknown_evidence(self):
  source={'app.js':'a'*64};v={'version':'browser-observation@v1','files':source,'observations':[{'kind':'key','value':'ArrowUp'}]};before=copy.deepcopy(v)
  r=p.validate_observation(v,source);self.assertIsNone(r['criterion_verdict']);self.assertFalse(r['native_receipt']);self.assertIn('unknown',r['coverage']);self.assertEqual(v,before)
  with self.assertRaises(ValueError):p.validate_observation(v,{'app.js':'b'*64})
  with self.assertRaises(ValueError):p.validate_observation({**v,'observations':None},source)
 def test_search_policy_is_equal_and_keeps_unknown_mapping_explicit(self):
  mapping={'crossing':{'Up':'ArrowUp','Right':'ArrowRight','Down':'ArrowDown','Left':'ArrowLeft'},'growth':{'Up':'w','Right':'d','Down':'s','Left':'a'}}
  for batch in p.protocol()['observation_batches']:
   plan=p.search_plan(mapping,batch);self.assertEqual(plan,p.search_plan(copy.deepcopy(mapping),batch));self.assertTrue(all(1<=len(x)<=32 for x in plan));self.assertLessEqual(len(plan),384)
  self.assertIn(['ArrowUp']*32,p.search_plan(mapping,'crossing-first'))
  with self.assertRaisesRegex(ValueError,'unknown coverage'):p.search_plan({'crossing':{},'growth':{}},'crossing-first')
 @unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1','existing isolated browser opt-in')
 def test_actual_controls_independent_and_coupled_repeatability(self):
  results=[]
  with tempfile.TemporaryDirectory(dir=ROOT/'target',prefix='game-wants-fixture-') as d:
   work=Path(d)
   for kind in ('coupled','separate'):
    for name in ('app.js','style.css'):(work/name).write_bytes((PACKET/'fixtures'/name).read_bytes())
    (work/'index.html').write_bytes((PACKET/'fixtures'/f'{kind}.html').read_bytes())
    source={n:p.sha((work/n).read_bytes()) for n in ('index.html','app.js','style.css')}
    for batch,keys in p.sequences(kind).items():
     (work/'probe.json').write_text(json.dumps(p.probe(keys)))
     cmd=['/opt/homebrew/bin/docker','run','--rm','--init','--pull=never','--network=none','--read-only','--cap-drop=ALL','--security-opt=no-new-privileges','--pids-limit=256','--memory=2g','--cpus=2','--user','501:20','--tmpfs=/tmp:rw,nosuid,nodev,exec,size=256m','--mount',f'type=bind,src={work},dst=/project,readonly','--mount',f'type=bind,src={ROOT/"scripts/engine-browser-harness.cjs"},dst=/verify.cjs,readonly','--workdir=/project','--entrypoint=',m.IMAGE,'node','/verify.cjs','probe.json','--container-only']
     observed=[]
     for repeat in range(2):
      run=subprocess.run(cmd,capture_output=True,timeout=90);self.assertEqual(run.returncode,0,run.stderr.decode());value=json.loads(run.stdout);e=p.validate_observation(value,source);observed.append(value)
      results.append({'fixture':kind,'batch':batch,'repeat':repeat,'argv':cmd,'returncode':run.returncode,'stdout_sha256':p.sha(run.stdout),'stderr_sha256':p.sha(run.stderr),'evidence':e})
     self.assertEqual(observed[0],observed[1]);text=next(x['text'] for x in observed[0]['observations'] if x.get('selector')=='output')
     if batch=='crossing-first':self.assertIn('crossing reached',text);self.assertIn('growth 0',text)
     if batch=='growth-first':self.assertIn('growth 2',text);self.assertIn('crossing reached' if kind=='coupled' else 'crossing open',text)
     if batch=='partial':self.assertIn('crossing open',text);self.assertIsNone(e['criterion_verdict'])
     if batch=='mixed-pursuit':self.assertIn('body collision',text) if kind=='coupled' else self.assertIn('Crossing position 0,2',text)
    self.assertEqual(source,{n:p.sha((work/n).read_bytes()) for n in source})
  (ROOT/'target/game-wants-authored-browser-results.json').write_text(json.dumps(results,indent=2)+'\n')

@unittest.skipUnless(os.getenv('TSR_BROWSER_ADAPTER_AUTHORED')=='1' and os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1','native authored fixture and existing isolated browser opt-in')
class NativeFixtures(BrowserAdapter):
 def test_gameplay_receipt_and_ordinary_projection_parity(self):
  # Existing test setup supplies a fixture-only campaign; create a separate native
  # scope with these authored assets, never mutate a historical/live project.
  (self.root/'game-native').mkdir();self.c=m.d.Native(self.root/'game-native')
  assets={n:(PACKET/'fixtures'/n).read_text() for n in ('app.js','style.css')};assets['index.html']=(PACKET/'fixtures/coupled.html').read_text()
  config={**self.config,'editable':assets,'protected':{'browser-harness.cjs':(ROOT/'scripts/engine-browser-harness.cjs').read_text(),'probe.json':json.dumps(p.probe(p.sequences('coupled')['mixed-pursuit']))},'cases':[{'id':'browser_smoke','args':['probe.json'],'expected_stdout':'browser-observation@v1'}]}
  (self.root/'game-scope.json').write_text(json.dumps(config));wants={k:v for k,v in p.protocol()['operational_roots'].items() if k!='attribution'};(self.root/'game-wants.json').write_text(json.dumps(wants))
  self.c.cli('project','init','--spec',str(self.root/'game-scope.json'),'--requirements',str(self.root/'game-wants.json'))
  self.c.json_cli(('project-lifecycle-open',),{'state_root':self.c.request()['state_root'],'training':None,'tool_ms_ceiling':240000,'reason':'Authored evidence fixture only','non_tool':{'model_calls':10,'model_tokens':409600,'wall_ms':600000,'patch_bytes':49152,'external_cost_microusd':0}},'--admission')
  completion=self.c.cli('project','acceptance','status');self.c.json_cli(('project','acceptance','feedback'),{'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],'guidance':wants});self.c.cli('project','resource-unpriced-admit','--state-root',self.c.request()['state_root'])
  status=self.c.cli('project','acceptance','status')
  policy={'state_root':self.c.request()['state_root'],'requirements_revision':status['requirements_revision'],'requirements':wants,'coverage':{k:['app.js'] for k in wants},'prerequisites':{k:{'kind':'static_semantic'} for k in wants},'assessment_scope':{k:[{'domain':'artifact','text':v}] for k,v in wants.items()}}
  pid=self.c.json_cli(('project','criterion-policy'),policy)
  profile={'version':'public-criterion-return@v2','policy':pid,'policy_sha256':m.e.sha(policy),'imported_returns':[]}
  r=self.c.request();c=r['coding'];i=c['relational']['inputs'];parents=copy.deepcopy(i['parents'])
  public={'version':'public-browser-origin@v1','scope':r['next_attention']['scope'],'requirements_revision':i['requirements_revision'],'parents':parents,'protected':{f['path']:f['source_digest'] for f in c['files'] if not f['editable']},'editable':list(assets),'public_brief':p.protocol()['user_input']['text'],'source_snapshots':[c['snapshot']],'public_relationships':[],'public_returns':[]}
  self.a=m.admission(self.root/'game-adapter',{'directory':str(self.c.directory),'public':public,'initial_sources':assets},self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=2,max_executions=1,correction_allowance=1,criterion_return_profile=profile)
  a=m.Adapter(self.a['directory'],self.a,self.c,self.ledger,self.transport,worker=lambda view:self.adapter_instance.resume_process(view));self.adapter_instance=a;original=self.transport
  def transport(body,timeout):
   data=json.loads(body['messages'][1]['content'])
   if 'prerequisites' in data:
    detail=data['authenticated_control_observations'];self.assertEqual(detail['availability'],'available');self.assertIn('body 3',json.dumps(detail));self.assertIn('crossing open',json.dumps(detail));self.assertEqual(detail['snapshot'],self.c.request()['coding']['snapshot'])
    self.bodies.append(copy.deepcopy(body));self.calls.append(data)
    return {'value':{'verdict':'unknown','findings':{k:{'finding':'unknown','basis':'unknown','evidence':data['coverage'][k],'explanation':'Authored limited trace: growth and collision observed; full criterion coverage is not established','limitations':'No universal reachability or enjoyment proof'} for k in data['criteria']}},'receipt':{'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':.1}}
   if 'criteria' in data:return original(body,timeout)
   self.bodies.append(copy.deepcopy(body));self.calls.append(data)
   return {'value':{'alternatives':[{'edits':[{'path':'app.js','old_text':assets['app.js'],'new_text':assets['app.js']+'\n// Authored no-gameplay-change receipt test.\n'}],'rationale':'Authored fixture only; no observed gameplay claim'}]},'receipt':{'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':.1}}
  a.injected_transport=transport;view=a.native('request',None);req,context=a.project(view)
  ordinary=m.load('game_ordinary','engine-browser-ordinary.py');b=ordinary.Adapter(self.a['directory'],self.a,self.c,self.ledger,transport);_,plain=b.project(view)
  tw=a.wire_context(req,context);pw=b.wire_context(req,plain)
  self.assertEqual(tw['parents'],pw['task_memory']['goals']);self.assertEqual(tw['relational_history'],pw['task_memory']['historical_notes']);self.assertEqual(tw['own_decisions'],pw['task_memory']['own_decisions'])
  for key in ('public_brief','files','browser_returns','return_evidence','acceptance_parents'):self.assertEqual(tw[key],pw[key])
  self.assertEqual(a.proposal_body(view,tw)['messages'][0],b.proposal_body(view,pw)['messages'][0]);self.assertNotIn('relationship_initial',pw['choice_summary']['forms']);self.assertIn('relationship_initial',tw['choice_summary']['forms'])
  ca={'version':'local-project-cycle@v3','journal':str(self.root/'game-cycle'),'scope':view['scope'],'max_decisions':2,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3}
  cycle=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review);result=cycle.run();self.assertEqual(result['status'],'review_required');self.assertEqual(self.c.cli('project','status')['execution_status'],'passed')
  request=self.c.request();self.assertEqual(request['coding']['relational']['inputs']['parents'],parents);_,ctx=a.project(a.native('request',None));self.assertTrue(ctx['return_evidence']['cases'])
  receipts=request['coding']['relational']['inputs']['returns'];self.assertTrue(receipts);self.assertIn('body 3',json.dumps(receipts));self.assertIn('crossing open',json.dumps(receipts))
  criterion=m.load('fixture_control_criterion','engine-criterion-return.py');criterion.reflect(a)
  assessed=self.c.cli('project','criterion-context')['current'];self.assertEqual(assessed['assessment']['decision'],'unknown');self.assertTrue(all(f['finding']=='unknown' for f in assessed['assessment']['findings'].values()))
  before=(len(self.calls),self.ledger.status());self.assertIsNone(criterion.reflect(a));self.assertEqual(cycle.run(),result);self.assertEqual(before,(len(self.calls),self.ledger.status()))
  bad=copy.deepcopy(request);bad['coding']['relational']['inputs']['returns'][0]['receipt']='f'*64;self.c.request=lambda:bad
  with self.assertRaisesRegex(ValueError,'unowned'):a.project(a.native('request',None))
  (ROOT/'target/game-wants-control-criterion-native-result.json').write_text(json.dumps({'status':result['status'],'fixture_only':True,'model_calls_are_authored':True,'receipts':receipts,'criterion_detail_consumed':True,'criterion_decision':'unknown','parents_preserved':True,'replayed_without_effect':True,'plain_menu':pw['choice_summary'],'user_prompt_sha256':p.sha(tw['public_brief'].encode()),'system_prompt_sha256':p.sha(a.proposal_body(view,tw)['messages'][0]['content'].encode())},indent=2)+'\n')
