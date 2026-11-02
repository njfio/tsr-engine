"""Authored native registration + real browser + counted authored assessment. No provider."""
import copy,json,os,subprocess,unittest,shutil
from unittest.mock import patch
from pathlib import Path
from scripts.test_engine_browser_adapter import BrowserAdapter,m as adapter
from scripts.test_game_wants_mapping import fixture,m as mapping
ROOT=Path(__file__).resolve().parents[1]
@unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1' and os.getenv('TSR_BROWSER_ADAPTER_AUTHORED')=='1','existing isolated browser and native authored opt-in')
class RegisteredControls(BrowserAdapter):
 def test_registered_controls_preserve_state_and_feed_scoped_assessment(self):
  self.run_registered(False)
 def test_six_cumulative_bundles_unknown_capture_restart_and_complete_request(self):
  self.run_registered(True)
 def test_policy_refresh_preserves_contributors_and_history(self):
  self.run_registered(True,refresh=True)
 def run_registered(self,cumulative,refresh=False):
  web=adapter;files,map_record,map_review=fixture('separate')
  parent=self.root/'mapped-parent';parent.mkdir();self.c=web.d.Native(parent)
  original_probe=mapping.plan(map_record,files,map_review,'crossing-first',['ArrowUp'])['probe']
  config={**self.config,'timeout_ms':5000,'editable':files,'protected':{'browser-harness.cjs':(ROOT/'scripts/engine-browser-harness.cjs').read_text(),'probe.json':json.dumps(original_probe)}}
  wants={'crossing':'An observed control sequence reaches the advertised crossing bank.','growth':'An observed collection sequence increases body length.'}
  self.c.json_cli(('project','init'),config,'--spec');original_config=self.c.cli('project','config');self.c.json_cli(('project','acceptance','requirements'),wants)
  def open_episode(c):
   c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':240000,'reason':'Authored source-bound browser mapping fixture only','non_tool':{'model_calls':10,'model_tokens':1000000,'wall_ms':1800000,'patch_bytes':1000000,'external_cost_microusd':0}},'--admission')
  def verify(c):
   response=c.cli('project','verify-admit','--state-root',c.request()['state_root'],'--snapshot',c.cli('project','acceptance','status')['snapshot'])
   c.json_cli(('project','run','--commit-only'),response,'--response');c.cli('project','resume');return c.cli('project','acceptance','status')['test_receipt']
  open_episode(self.c);status=self.c.cli('project','acceptance','status');self.c.json_cli(('project','acceptance','feedback'),{'requirements_revision':status['requirements_revision'],'decision_record':status['decision_record'],'guidance':wants})
  verify(self.c);before=self.c.request();parents=copy.deepcopy(before['coding']['relational']['inputs']['parents']);snapshot=before['coding']['snapshot']
  plans={'crossing':mapping.plan(map_record,files,map_review,'crossing-first',['ArrowUp']*4),'growth':mapping.plan(map_record,files,map_review,'growth-first',['d','w','d'])}
  bundle={'version':'supplemental-browser@v3','snapshot':snapshot,'requirements_revision':status['requirements_revision'],'author':'authored mapper','reviewer':'authored independent mapping reviewer','generator':'authored fixture developer','review':'Authored mapping review only; no paid model or spontaneous design. Separate games valid.','harness':{f'supplemental/{k}.json':json.dumps(v['probe']) for k,v in plans.items()},'probe_entry':'browser-harness.cjs','cases':[{'id':'mapped_'+k,'args':[f'supplemental/{k}.json'],'expected_stdout':'browser-observation@v1'} for k in plans],'mapping':{k:{'cases':['mapped_'+k],'rationale':'Read documented controls and actual final DOM state','limits':'Exact finite sequence only, not human fulfillment or universal success'} for k in plans}}
  bindings=[];observations=[]
  for number in range(6 if cumulative else 1):
   current_bundle=copy.deepcopy(bundle)
   if cumulative and number:
    # Intentionally partial real control traces. One unavailable readout is an
    # authored mapping-error injection; it must remain visible and unknown.
    for root in wants:
     plan=mapping.plan(map_record,files,map_review,root+'-first',['ArrowLeft'] if root=='crossing' else ['a'])
     if number==3:plan['probe']['checks'][0]['selector']='#unavailable-authored-fixture'
     current_bundle['harness']['supplemental/'+root+'.json']=json.dumps(plan['probe'])
    current_bundle['review']+=' Authored partial/error fixture '+str(number)
   ident=self.c.json_cli(('project','verifier','supplemental-admit'),current_bundle)
   spec=self.c.cli('project','verifier','supplemental-spec','--admission',ident)
   self.assertEqual(spec['editable'],config['editable']);self.assertTrue(all(spec['protected'][k]==v for k,v in config['protected'].items()));self.assertEqual(spec['cases'][0],config['cases'][0])
   folder=self.root/('mapped-check-'+str(number));folder.mkdir();other=web.d.Native(folder);other.json_cli(('project','init'),spec,'--spec');open_episode(other);receipt=verify(other)
   linked=self.c.cli('project','verifier','supplemental-link','--admission',ident,'--from',str(folder),'--receipt',receipt)
   self.assertEqual(self.c.cli('project','verifier','supplemental-link','--admission',ident,'--from',str(folder),'--receipt',receipt),linked)
   observed=self.c.cli('project','verifier','supplemental-inspect')['current_observations'][-1]
   if not observed['browser_cases']:(ROOT/'target/game-wants-empty-browser-diagnostic.json').write_text(json.dumps(observed,indent=2)+'\n')
   self.assertTrue(all(x=='unknown' for x in observed['findings'].values()));self.assertEqual(len(observed['browser_cases']),1 if cumulative and number==3 else 2)
   if not cumulative:
    for case in observed['browser_cases']:
     root=case['case_id'].removeprefix('mapped_');mapping.bind_observation(plans[root],json.loads(case['stdout']),files)
   bindings.append({'admission':ident,'receipt':receipt});observations.append(observed)
  self.assertEqual(self.c.cli('project','config'),original_config);self.assertEqual(self.c.request()['coding']['relational']['inputs']['parents'],parents);self.assertEqual(self.c.request()['coding']['snapshot'],snapshot)
  keys=['supplemental:'+b['admission']+':'+b['receipt'] for b in bindings]
  prerequisite={'kind':'observed_supplemental_set','observations':bindings} if cumulative else {'kind':'observed_supplemental',**bindings[0]}
  policy={'state_root':self.c.request()['state_root'],'requirements_revision':status['requirements_revision'],'requirements':wants,'coverage':{k:sorted(keys) for k in wants},'prerequisites':{k:copy.deepcopy(prerequisite) for k in wants},'assessment_scope':{k:[{'domain':'artifact','text':v}] for k,v in wants.items()}}
  if cumulative:
   for mutation in ('missing','reordered','duplicate','foreign'):
    bad=copy.deepcopy(policy);items=bad['prerequisites']['crossing']['observations']
    if mutation=='missing':items.pop(0)
    elif mutation=='reordered':items.reverse()
    elif mutation=='duplicate':items[-1]=copy.deepcopy(items[0])
    else:items[0]['receipt']='0'*64
    with self.assertRaises(Exception):self.c.json_cli(('project','criterion-policy'),bad)
   self.c=web.d.Native(parent) # reload persisted parent, no resets or re-execution
  pid=self.c.json_cli(('project','criterion-policy'),policy)
  self.c.cli('project','resource-unpriced-admit','--state-root',self.c.request()['state_root'])
  request=self.c.request();coding=request['coding'];inputs=coding['relational']['inputs']
  public={'version':'public-browser-origin@v1','scope':request['next_attention']['scope'],'requirements_revision':inputs['requirements_revision'],'parents':inputs['parents'],'protected':{f['path']:f['source_digest'] for f in coding['files'] if not f['editable']},'editable':list(files),'public_brief':'Authored offline mapping verification; no new player request','source_snapshots':[snapshot],'public_relationships':[],'public_returns':[web.e.sha(x) for x in inputs['returns']],'public_supplemental':[web.e.sha(x) for x in inputs['supplemental']]}
  profile={'version':'public-criterion-return@v2','policy':pid,'policy_sha256':web.e.sha(policy),'imported_returns':[]}
  admission=web.admission(self.root/'mapped-adapter',{'directory':str(parent),'public':public,'initial_sources':files},self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=2,max_executions=0,criterion_return_profile=profile)
  sent=[]
  def transport(body,timeout):
   payload=json.loads(body['messages'][1]['content']);sent.append(copy.deepcopy(payload));text=json.dumps(payload['evidence']);self.assertIn('crossing reached',text);self.assertIn('body 3',text);self.assertIn('mapped_crossing',text);self.assertIn('mapped_growth',text)
   if cumulative:
    self.assertEqual(payload['prerequisites']['crossing']['observations'],bindings)
    exported=[json.loads(value) for value in payload['evidence'].values() if value.startswith('{') and 'browser_cases' in value]
    self.assertEqual(len(exported),len(observations))
    for expected in observations:
     actual=[v for v in exported if v.get('admission')==expected['admission'] and v.get('receipt')==expected['receipt']]
     self.assertEqual(len(actual),1);self.assertEqual(actual[0]['browser_cases'],expected['browser_cases'])

    if len(observations)>=6:
     self.assertIn('crossing open',text);self.assertTrue(any(c['exit']!=0 for v in exported for c in v['browser_cases']))
   finding='unknown' if cumulative else 'met';basis='unknown' if cumulative else 'observed_supplemental'
   return {'value':{'verdict':'unknown' if cumulative else 'accept','findings':{k:{'finding':finding,'basis':basis,'evidence':payload['coverage'][k],'explanation':'Authored assessment: cumulative unavailable and partial observations remain unknown' if cumulative else 'Authored semantic assessment of witnessed finite criterion only','limitations':'No universal reachability, human fulfillment, coupling or user acceptance claim'} for k in wants}},'receipt':{'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':.1}}
  a=web.Adapter(admission['directory'],admission,self.c,self.ledger,transport);criterion=web.load('mapped_criterion','engine-criterion-return.py')
  if cumulative:
   original_op=a.op;crashed=[]
   def interrupted(name,fn):
    result=original_op(name,fn)
    if name.endswith('_input') and isinstance(result,dict) and result.get('contract_version')=='criterion-evidence@v7' and not crashed:
     crashed.append(name);raise RuntimeError('authored retained cumulative capture interruption')
    return result
   a.op=interrupted
   with self.assertRaisesRegex(RuntimeError,'cumulative capture'):criterion.reflect(a)
   self.assertEqual(sent,[])
   a=web.Adapter(admission['directory'],admission,web.d.Native(parent),self.ledger,transport)
  real_run=subprocess.run
  def diagnose(*args,**kwargs):
   result=real_run(*args,**kwargs)
   if result.returncode:print('AUTHORED native diagnostic:',result.stderr.decode(errors='replace'),flush=True)
   return result
  try:
   with patch.object(subprocess,'run',diagnose):
    if cumulative:criterion.reflect(a)
    else:
     launcher=web.load('mapped_startup_launcher','engine-native-cycle.py')
     fresh_cycle=type('AuthoredFreshCycle',(),{'read':lambda self:([{'authored':'admission'}],{},{})})()
     launcher.complete_fresh_direct_return(a,fresh_cycle,{'journal':str(self.root/'fresh-startup')})
  except Exception:
   shutil.copytree(self.root,ROOT/('target/cumulative-failed-'+self.root.name),dirs_exist_ok=False)
   raise
  current=self.c.cli('project','criterion-context')['current'];self.assertEqual(current['assessment']['decision'],'unknown' if cumulative else 'accept');self.assertFalse(self.c.cli('project','acceptance','status')['accepted'])
  self.assertEqual(self.c.request()['coding']['relational']['inputs']['parents'],parents);before_count=self.ledger.status();self.assertIsNone(criterion.reflect(a));self.assertEqual(self.ledger.status(),before_count);self.assertEqual(len(sent),1)
  if refresh:
   initial_revision=status['requirements_revision'];initial_assessment=current
   for phase in ('same-source','changed-source'):
    if phase=='changed-source':
     # Authored source-version fixture, not an engine-authored game change.
     files={**files,'app.js':files['app.js']+'\n// Authored version-bound observation fixture.\n'}
     (parent/'coding-work/app.js').write_text(files['app.js']);verify(self.c)
     bindings=[];observations=[]
    fresh_snapshot=self.c.request()['coding']['snapshot']
    added=copy.deepcopy(bundle);added['snapshot']=fresh_snapshot;added['review']+=' Authored refreshed evidence '+phase
    ident=self.c.json_cli(('project','verifier','supplemental-admit'),added)
    self.assertIsNone(self.c.cli('project','criterion-context')['frame'])
    spec=self.c.cli('project','verifier','supplemental-spec','--admission',ident)
    folder=self.root/('refreshed-check-'+phase);folder.mkdir();other=web.d.Native(folder);other.json_cli(('project','init'),spec,'--spec');open_episode(other);receipt=verify(other)
    self.c.cli('project','verifier','supplemental-link','--admission',ident,'--from',str(folder),'--receipt',receipt)
    observed=self.c.cli('project','verifier','supplemental-inspect')['current_observations'][-1];bindings.append({'admission':ident,'receipt':receipt});observations.append(observed)
    keys=['supplemental:'+b['admission']+':'+b['receipt'] for b in bindings]
    next_policy={**copy.deepcopy(policy),'state_root':self.c.request()['state_root'],'coverage':{k:sorted(keys) for k in wants},'prerequisites':{k:{'kind':'observed_supplemental_set','observations':copy.deepcopy(bindings)} for k in wants}}
    before=self.c.request()['state_root']
    for mutation in ('scope','criteria','predecessor','missing'):
     bad=copy.deepcopy(next_policy);previous=pid
     if mutation=='scope':bad['assessment_scope']['crossing'][0]['text']='Changed claim'
     elif mutation=='criteria':bad['requirements']['crossing']='Changed goal'
     elif mutation=='predecessor':previous='0'*64
     else:bad['prerequisites']['crossing']['observations']=[]
     with self.assertRaises(Exception):self.c.json_cli(('project','criterion-policy-refresh','--previous',previous),bad)
     self.assertEqual(self.c.request()['state_root'],before)
    prior=pid;pid=self.c.json_cli(('project','criterion-policy-refresh','--previous',prior),next_policy)
    with self.assertRaises(Exception):self.c.json_cli(('project','criterion-policy-refresh','--previous',prior),next_policy)
    context=self.c.cli('project','criterion-context');self.assertIsNone(context['current']);self.assertIn(initial_assessment,context['history'])
    request=self.c.request();coding=request['coding'];inputs=coding['relational']['inputs'];self.assertEqual(inputs['parents'],parents);self.assertEqual(inputs['requirements_revision'],initial_revision)
    public.update(source_snapshots=[coding['snapshot']],public_returns=[web.e.sha(x) for x in inputs['returns']],public_supplemental=[web.e.sha(x) for x in inputs['supplemental']])
    profile={'version':'public-criterion-return@v2','policy':pid,'policy_sha256':web.e.sha(next_policy),'imported_returns':[web.e.sha(x) for x in context['history']]}
    fresh=web.admission(self.root/('refresh-adapter-'+phase),{'directory':str(parent),'public':copy.deepcopy(public),'initial_sources':files},self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=2,max_executions=0,criterion_return_profile=profile)
    newer=web.Adapter(fresh['directory'],fresh,self.c,self.ledger,transport)
    criterion.reflect(newer);saved=self.c.cli('project','criterion-context');self.assertEqual(saved['current']['assessment']['decision'],'unknown');self.assertIn(initial_assessment,saved['history'])
    before=self.ledger.status();self.assertIsNone(criterion.reflect(newer));self.assertEqual(self.ledger.status(),before)
    self.assertEqual(self.c.request()['coding']['relational']['inputs']['parents'],parents)
    policy=next_policy
   (ROOT/'target/game-wants-policy-refresh-result.json').write_text(json.dumps({'authored_only':True,'phases':['same-source','changed-source'],'parents_preserved':True,'requirements_revision_preserved':True,'history':self.c.cli('project','criterion-context')['history'],'sent_payloads':sent,'inert_replay':True,'provider_calls':0},indent=2)+'\n')
   return
  (ROOT/('target/game-wants-cumulative-controls-result.json' if cumulative else 'target/game-wants-registered-controls-result.json')).write_text(json.dumps({'authored_only':True,'mapping':map_record,'mapping_review':map_review,'plans':plans,'observation':observed,'all_observations':observations,'sent_payloads':sent,'criterion':current,'parents_preserved':True,'inert_replay':True,'provider_calls':0},indent=2)+'\n')
