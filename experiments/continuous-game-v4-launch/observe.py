"""Reviewed operator observation execution. One bundle per invocation, never retries."""
import json,importlib.util,sys,fcntl,os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];LABEL=os.environ.get('TSR_LIVE_LABEL','v4b')
s=importlib.util.spec_from_file_location('pair_prepare',Path(__file__).with_name('prepare.py'));p=importlib.util.module_from_spec(s);s.loader.exec_module(p);a=p.a
s=importlib.util.spec_from_file_location('pair_mapping',ROOT/'experiments/continuous-game-v4/mapping.py');mapping=importlib.util.module_from_spec(s);s.loader.exec_module(mapping)
def load(path):return json.loads(path.read_text())
def save(path,value):
 with path.open('x') as f:json.dump(value,f,indent=2);f.write('\n')
BATCHES=('crossing-first','growth-first','mixed-pursuit')
def ensure_ready(native,ad,cycle,ledger):
 for folder in (Path(ad['directory']),Path(cycle['journal'])):
  if (folder/'STOP').exists() or any(folder.glob('checkpoint-*/STOP')):raise ValueError('operator observer stopped')
 b=native.budget();state=ledger.status()
 if b['stopped'] or b['exceeded'] or b['unsettled_requests'] or b['unresolved_tool_requests'] or b['unknown_usage'] or b['unknown_compute_requests'] or b['unknown_external_cost_requests']:raise ValueError('unsettled native effects')
 if state['stopped'] or state['blocking_unknown_calls'] or state['blocking_unknown_executions']:raise ValueError('unsettled campaign effects')
def run(arm,batch,index):
 if arm not in ('tsr',) or batch not in BATCHES or type(index) is not int or not 0<=index<7:raise ValueError('exact paired observer coordinates required')
 p.check_pins();base=ROOT/('target/continuous-game-live-'+LABEL);home=base/arm;stage=home/('observations-'+batch);binding=load(stage/'input-binding.json')
 ad_path=ROOT/binding['admission_path'];cycle_path=ROOT/binding['cycle_path']
 assert a.d.file_sha(ad_path)==binding['admission_file_sha256'] and a.d.file_sha(cycle_path)==binding['cycle_file_sha256']
 assert a.d.file_sha(stage/'mapping.json')==binding['mapping_file_sha256'] and a.d.file_sha(stage/'mapping-review.json')==binding['mapping_review_file_sha256']
 ad=load(ad_path);cycle=load(cycle_path);native=a.d.Native(home/'native')
 assert Path(ad['origin']['directory'])==native.directory
 lock=os.fdopen(os.open(Path(ad['directory'])/'launch.lock',os.O_WRONLY|os.O_NOFOLLOW),'a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 campaign=load(ROOT/('target/continuous-game-campaign-'+LABEL+'/manifest.json'));m=a.load('observer_campaign','engine-b91-campaign.py');up=load(base/'campaign-upgrade.json');ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),upgrade=up,upgrade_hash=m.digest(up))
 adapter=a.Adapter(ad['directory'],ad,native,ledger)
 cycle_type=a.load('observer_ordinary','engine-browser-ordinary.py').Cycle if ad.get('process_profile')=='ordinary-memory@v1' else a.loop.Cycle
 controller=cycle_type(cycle['journal'],cycle,adapter.native,adapter.proposal,adapter.review)
 if adapter.read()[1] or controller.read()[1]:raise ValueError('settled input session required')
 terminal=controller.read()[2].get('terminal',{})
 if terminal.get('status')!='proposal_retention':raise ValueError('eligible retained checkpoint required; abstention/rejection stops observation protocol')
 ensure_ready(native,ad,cycle,ledger)
 native_observations=[x['observation'] for x in native.cli('project','verifier','supplemental-inspect')['records'] if 'observation' in x]
 previous=[]
 for earlier in BATCHES[:BATCHES.index(batch)]:previous += [home/('observations-'+earlier)/('bundle-'+str(i)) for i in range(len(mapping.search(load(home/('observations-'+earlier)/'mapping.json'),earlier)))]
 previous += [stage/('bundle-'+str(i)) for i in range(index)]
 for folder in previous:
  linked=load(folder/'linked.json');obs=load(folder/'observation.json')
  if obs not in native_observations or obs['admission']!=linked['admission'] or obs['receipt']!=linked['receipt']:raise ValueError('prior observer import incomplete')
 if any((stage/('bundle-'+str(i))).exists() for i in range(index,7)):raise ValueError('duplicate, reordered or interrupted observer bundle')
 files={k:(native.directory/'coding-work'/k).read_text() for k in ad['origin']['initial_sources']};record=load(stage/'mapping.json');review=load(stage/'mapping-review.json')
 assert mapping.validate(record,files,review)['status']=='mapped'
 request=native.request();snapshot=request['coding']['snapshot'];status=native.cli('project','acceptance','status')
 mapping.check_discovery(record,load(home/('discovery-'+snapshot+'.json')))
 frozen=home/('mapping-freeze-'+snapshot+'.json');identity={'mapping_sha256':mapping.digest(record),'review_sha256':mapping.digest(review)}
 if not frozen.exists():save(frozen,identity)
 if load(frozen)!=identity:raise ValueError('mapping changed on an unchanged source')
 words=mapping.search(record,batch)
 if index>=len(words):raise ValueError('no such word: route unavailable for this batch')
 keys=words[index];plan=mapping.plan(record,files,review,batch,keys)
 operation=stage/('bundle-'+str(index));operation.mkdir(exist_ok=False)
 bundle={'version':'supplemental-browser@v3','snapshot':snapshot,'requirements_revision':status['requirements_revision'],'author':'operator mapping /root','reviewer':review['reviewer'],'generator':'CheaperInference/gpt-6-luna game author','review':review['attribution']+' '+review['limitations'],'harness':{'supplemental/repeat-'+str(i)+'.json':json.dumps(plan['probe']) for i in range(2)},'probe_entry':'browser-harness.cjs','cases':[{'id':'repeat_'+str(i),'args':['supplemental/repeat-'+str(i)+'.json'],'expected_stdout':'browser-observation@v1'} for i in range(2)],'mapping':{k:{'cases':['repeat_0','repeat_1'],'rationale':'Exact frozen ordered controls and actual final DOM readouts; no inferred gameplay verdict','limits':review['limitations']} for k in ('crossing','growth')}}
 save(operation/'plan.json',plan);save(operation/'bundle.json',bundle)
 ident=native.json_cli(('project','verifier','supplemental-admit'),bundle);save(operation/'registration.json',{'id':ident})
 spec=native.cli('project','verifier','supplemental-spec','--admission',ident);folder=operation/'native-check';folder.mkdir();other=a.d.Native(folder);other.json_cli(('project','init'),spec,'--spec')
 other.json_cli(('project-lifecycle-open',),{'state_root':other.request()['state_root'],'training':None,'tool_ms_ceiling':240000,'reason':'Approved paired source-bound finite observer bundle; two repeated control traces','non_tool':{'model_calls':0,'model_tokens':0,'wall_ms':1800000,'patch_bytes':0,'external_cost_microusd':0}},'--admission')
 response=other.cli('project','verify-admit','--state-root',other.request()['state_root'],'--snapshot',other.cli('project','acceptance','status')['snapshot']);other.json_cli(('project','run','--commit-only'),response,'--response')
 group='operator_observation_'+batch+'_'+str(index);save(operation/'execution-intent.json',{'trial':adapter.trial,'group':group,'admission':a.e.sha(ad),'registration':ident,'plan_sha256':plan['plan_sha256']})
 def worker():
  value=other.cli('project','resume');receipt=other.cli('project','acceptance','status')['test_receipt'];save(operation/'execution-return.json',{'result':value,'receipt':receipt});return {'receipt':receipt,'registration':ident,'source':snapshot,'plan_sha256':plan['plan_sha256']}
 ensure_ready(native,ad,cycle,ledger)
 returned=ledger.run_execution(adapter.trial,group,a.e.sha(ad),worker)
 linked=native.cli('project','verifier','supplemental-link','--admission',ident,'--from',str(folder),'--receipt',returned['receipt']);save(operation/'linked.json',{'link':linked,'admission':ident,'receipt':returned['receipt']})
 matches=[x for x in native.cli('project','verifier','supplemental-inspect')['current_observations'] if x['admission']==ident and x['receipt']==returned['receipt']]
 if len(matches)!=1:raise ValueError('exact imported observation missing')
 observed=matches[0]
 if observed['snapshot']!=snapshot or observed['requirements_revision']!=status['requirements_revision']:raise ValueError('imported source/criteria changed')
 save(operation/'observation.json',observed)
 return {'registered':ident,'receipt':returned['receipt'],'raw_cases':len(observed['browser_cases']),'criteria':observed['findings'],'no_semantic_verdict':True}
if __name__=='__main__':print(json.dumps(run(sys.argv[1],sys.argv[2],int(sys.argv[3])),indent=2))
