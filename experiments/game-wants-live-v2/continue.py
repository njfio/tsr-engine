"""Prospective reviewed continuation binding after a complete frozen observer batch."""
import copy,json,importlib.util,sys,os,fcntl
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('pair_prepare',Path(__file__).with_name('prepare.py'));p=importlib.util.module_from_spec(s);s.loader.exec_module(p);a=p.a
load=lambda path:json.loads(path.read_text())
def save(path,value):
 with path.open('x') as f:json.dump(value,f,indent=2);f.write('\n')
def recorded(path,admission):
 rows=[a.e.strict(x) for x in a.e.bounded_file(path,a.base.capacity.JOURNAL_BYTES).splitlines()]
 assert rows and rows[0]['event']=={'kind':'admission','value':admission,'fixture_only':False}
 pending={};done={}
 for i,row in enumerate(rows):
  assert row==a.d.Driver.row(rows[:i],row['event'])
  if not i:continue
  event=row['event'];name=event['name']
  if event['kind']=='intent':
   assert name not in pending and name not in done;pending[name]=event
  else:
   assert event['kind']=='result' and name in pending and event['intent']==a.e.sha(pending.pop(name));done[name]=event['value']
 return pending,done

def prepare(arm,batch):
 assert arm in ('tsr','ordinary') and batch in ('crossing-first','growth-first','mixed-pursuit')
 base=ROOT/'target/game-wants-live-v3';home=base/arm;observations=home/('observations-'+batch);binding=load(observations/'input-binding.json');previous=load(ROOT/binding['admission_path']);old_cycle=load(ROOT/binding['cycle_path'])
 assert a.d.file_sha(ROOT/binding['admission_path'])==binding['admission_file_sha256'] and a.d.file_sha(ROOT/binding['cycle_path'])==binding['cycle_file_sha256']
 frozen=load(ROOT/'experiments/game-wants-v5/manifest.json');assert a.d.file_sha(ROOT/'experiments/game-wants-v5/manifest.json')==p.PACKET
 assert a.d.file_sha(ROOT/'experiments/game-wants-v5/protocol.json')==frozen['files']['experiments/game-wants-v5/protocol.json']
 common=load(base/'common-information.json');assert a.d.file_sha(base/'common-information.json')==load(home/'prepared-manifest.json')['common_information_sha256']
 assert load(home/'requirements.json')==common['wants']
 lock=os.fdopen(os.open(Path(previous['directory'])/'launch.lock',os.O_WRONLY|os.O_NOFOLLOW),'a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 for folder in (Path(previous['directory']),Path(old_cycle['journal'])):
  assert not (folder/'STOP').exists() and not any(folder.glob('checkpoint-*/STOP'))
 pending,done=recorded(Path(previous['directory'])/'adapter.jsonl',previous);cpending,cdone=recorded(Path(old_cycle['journal'])/'cycle.jsonl',old_cycle)
 assert not pending and not cpending and cdone.get('terminal',{}).get('status')=='proposal_retention'
 campaign=load(ROOT/'target/tsr-integrated-campaign-1/manifest.json');m=a.load('continuation_campaign','engine-b91-campaign.py');upgrade=load(base/'campaign-upgrade.json');ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),upgrade=upgrade,upgrade_hash=m.digest(upgrade));state=ledger.status();assert not state['stopped'] and not state['blocking_unknown_calls'] and not state['blocking_unknown_executions']
 c=a.d.Native(home/'native');budget=c.budget();assert not budget['stopped'] and not budget['exceeded'] and not any(budget[k] for k in ('unknown_usage','unknown_compute_requests','unknown_external_cost_requests','unsettled_requests','unresolved_tool_requests'))
 native_observed=c.cli('project','verifier','supplemental-inspect')['current_observations']
 for i in range(6):
  bundle=observations/('bundle-'+str(i));side=load(bundle/'observation.json');linked=load(bundle/'linked.json');registration=load(bundle/'registration.json');assert side in native_observed and side['admission']==linked['admission']==registration['id'] and side['receipt']==linked['receipt']
 folder=home/('continuation-'+batch)
 if folder.exists():
  assert not (folder/'adapter-admission.json').exists() and not (folder/'adapter').exists(), 'prepared admission exists; never overwrite'
  policy=load(folder/'criterion-policy.json');pid=load(folder/'criterion-policy-receipt.json')['id'];context=c.cli('project','criterion-context')
  assert context['policy']==[pid,policy]
  feedback_record=load(folder/'feedback-receipt.json');feedback_status=feedback_record.get('status',feedback_record.get('id'))
  assert feedback_status['requirements_revision']==context['frame']['requirements_revision'] and feedback_status['snapshot']==context['frame']['snapshot']
  assert context['published_context']['guidance']==load(folder/'feedback-input.json')['guidance']
  assert policy['requirements']==common['wants']
  bindings=[{'admission':x['admission'],'receipt':x['receipt']} for x in native_observed]
  assert all(x['observations']==bindings for x in policy['prerequisites'].values())
  files={k:(c.directory/'coding-work'/k).read_text() for k in previous['origin']['initial_sources']};request=c.request();coding=request['coding'];inputs=coding['relational']['inputs']
  assert [(x['id'],x['requirement']) for x in inputs['parents']]==[(x['id'],x['requirement']) for x in previous['origin']['public']['parents']]
  save(folder/'preparation-recovery.json',{'reason':'Operator assertion compared full meanings after explicit authored guidance; identities unchanged','policy':pid,'publication':context['frame']['publication'],'provider_calls':0,'repeated_native_mutations':False})
 else:
  folder.mkdir(exist_ok=False)
  protocol=load(ROOT/'experiments/game-wants-v5/protocol.json');wants=load(home/'requirements.json');before=c.request();status=c.cli('project','acceptance','status')
  guidance={k:v+'\nExperiment-authored observer context, not player feedback: '+protocol['feedback']['text']+' Current batch: '+batch+'. Six frozen six-input traces were each repeated twice. Preserve all recorded outcomes and finite-coverage limitations; no unobserved success or failure is implied.' for k,v in wants.items()}
  feedback={'requirements_revision':status['requirements_revision'],'decision_record':status['decision_record'],'guidance':guidance};save(folder/'feedback-input.json',feedback);fid=c.json_cli(('project','acceptance','feedback'),feedback);save(folder/'feedback-receipt.json',{'status':fid,'attribution':'synthetic_harness_verification'})
  context=c.cli('project','criterion-context');bindings=[{'admission':x['admission'],'receipt':x['receipt']} for x in native_observed];files={k:(c.directory/'coding-work'/k).read_text() for k in previous['origin']['initial_sources']};keys=sorted([*files,*['supplemental:'+x['admission']+':'+x['receipt'] for x in bindings]])
  policy={'state_root':c.request()['state_root'],'requirements_revision':status['requirements_revision'],'requirements':wants,'coverage':{k:keys for k in wants},'prerequisites':{k:{'kind':'observed_supplemental_set','observations':copy.deepcopy(bindings)} for k in wants},'assessment_scope':{k:[{'domain':'artifact','text':v}] for k,v in wants.items()}}
  save(folder/'criterion-policy.json',policy)
  pid=c.json_cli(('project','criterion-policy-refresh','--previous',context['policy'][0]),policy) if context['policy'] else c.json_cli(('project','criterion-policy'),policy)
  save(folder/'criterion-policy-receipt.json',{'id':pid});context=c.cli('project','criterion-context');request=c.request();coding=request['coding'];inputs=coding['relational']['inputs'];assert [(x['id'],x['requirement']) for x in inputs['parents']]==[(x['id'],x['requirement']) for x in before['coding']['relational']['inputs']['parents']]
 public=copy.deepcopy(previous['origin']['public']);public.update(parents=inputs.get('acceptance_parents',inputs['parents']),source_snapshots=[coding['snapshot']],public_returns=[a.e.sha(x) for x in inputs.get('returns',[])],public_relationships=[a.e.sha(x) for x in coding['relational'].get('history',[])],public_preferences=[a.e.sha(x) for x in inputs.get('generated_preferences',[])],public_meanings=[a.e.sha(x) for x in inputs.get('public_desires',{}).get('history',[])],public_pursuits=[a.e.sha(x) for x in coding.get('pursuit',{}).get('history',[])],public_supplemental=[a.e.sha(x) for x in inputs.get('supplemental',[])])
 origin={'directory':str(c.directory),'public':public,'initial_sources':files};campaign=load(ROOT/'target/tsr-integrated-campaign-1/manifest.json');m=a.load('continuation_campaign','engine-b91-campaign.py');upgrade=load(base/'campaign-upgrade.json');ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),upgrade=upgrade,upgrade_hash=m.digest(upgrade));state=ledger.status();assert not state['stopped'] and not state['blocking_unknown_calls'] and not state['blocking_unknown_executions']
 profile={'version':'public-criterion-return@v2','policy':pid,'policy_sha256':a.e.sha(policy),'imported_returns':[a.e.sha(x) for x in context['history']],'consistency_reassessment':'single-fresh@v1'}
 ad=a.admission(folder/'adapter',origin,c.cli('project','config'),campaign,None,max_decisions=None,max_executions=None,correction_allowance=20,feedback_cycle=previous['feedback_cycle'],transient_recovery=previous['transient_recovery'],process_profile=previous.get('process_profile'),unlimited_policy_sha256=previous['unlimited_policy_sha256'],criterion_return_profile=profile)
 ca={**copy.deepcopy(old_cycle),'journal':str(folder/'cycle'),'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))}
 typ=a.load('continue_plain','engine-browser-ordinary.py').Adapter if arm=='ordinary' else a.Adapter;adapter=typ(ad['directory'],ad,c,ledger)
 authority={'version':'paired-observation-continuation@v1','authorization':p.AUTH,'arm':arm,'batch':batch,'original_packet_sha256':p.PACKET,'previous_admission_sha256':a.e.sha(previous),'current_source':coding['snapshot'],'observation_bindings':bindings,'baseline':state,'scope':'Same one paired evaluation; no budget reset/new experiment; operator observation and admission assistance disclosed'}
 grant=adapter.scope_grant(a.e.sha(authority),'independent-agent:/root/binding_review; final continuation binding review before dispatch',None)
 for name,v in {'adapter-admission':ad,'cycle-admission':ca,'scope-authority':authority,'scope-grant':grant,'criterion-context':context}.items():save(folder/(name+'.json'),v)
 view=c.cli('project','cycle','request');req,ctx=adapter.project(view);body=adapter.proposal_body(view,adapter.wire_context(req,ctx));save(folder/'prospective-request-before-assessment.json',body)
 return {'prepared':str(folder),'admission_sha256':a.e.sha(ad),'grant_sha256':a.e.sha(grant),'observations':len(bindings),'provider_calls':0,'criteria_assessment_pending':True}
if __name__=='__main__':print(json.dumps(prepare(sys.argv[1],sys.argv[2]),indent=2))
