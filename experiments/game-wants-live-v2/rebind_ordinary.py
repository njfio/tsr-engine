"""Fresh binding for the never-dispatched ordinary initial arm after shared launcher repair."""
import copy,json,importlib.util,os,fcntl
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('pair_prepare',Path(__file__).with_name('prepare.py'));p=importlib.util.module_from_spec(s);s.loader.exec_module(p);a=p.a
load=lambda path:json.loads(path.read_text())
def save(path,value):
 with path.open('x') as f:json.dump(value,f,indent=2);f.write('\n')
def prepare():
 base=ROOT/'target/game-wants-live-v3';home=base/'ordinary';old=load(home/'adapter-admission.json');oldca=load(home/'cycle-admission.json');common=load(base/'common-information.json')
 assert a.e.sha(old)==load(home/'prepared-manifest.json')['admission_sha256']
 old_launch=load(home/'launch-manifest.json');assert a.d.file_sha(home/'cycle-admission.json')==old_launch['files'][str((home/'cycle-admission.json').relative_to(ROOT))]
 lock=os.fdopen(os.open(Path(old['directory'])/'launch.lock',os.O_WRONLY|os.O_NOFOLLOW),'a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 cycles=[a.e.strict(x) for x in (Path(oldca['journal'])/'cycle.jsonl').read_bytes().splitlines()];assert len(cycles)==1 and cycles[0]==a.d.Driver.row([],{'kind':'admission','value':oldca,'fixture_only':False})
 for folder in (Path(old['directory']),Path(oldca['journal'])):
  assert not (folder/'STOP').exists() and not any(folder.glob('checkpoint-*/STOP'))
 assert a.d.file_sha(base/'common-information.json')==load(home/'prepared-manifest.json')['common_information_sha256']
 rows=[a.e.strict(x) for x in (Path(old['directory'])/'adapter.jsonl').read_bytes().splitlines()];assert len(rows)==1 and rows[0]==a.d.Driver.row([],{'kind':'admission','value':old,'fixture_only':False})
 c=a.d.Native(home/'native');budget=c.budget();assert not budget['stopped'] and not budget['exceeded'] and not any(budget[k] for k in ('unknown_usage','unknown_compute_requests','unknown_external_cost_requests','unsettled_requests','unresolved_tool_requests'))
 request=c.request();assert c.cli('project','cycle','request')['branch']=='propose'
 coding=request['coding'];inputs=coding['relational']['inputs'];assert inputs['parents']==old['origin']['public']['parents'] and coding['snapshot']==old['origin']['public']['source_snapshots'][0]
 files={k:(c.directory/'coding-work'/k).read_text() for k in common['source']};assert files==common['source'] and load(home/'requirements.json')==common['wants']
 campaign=load(ROOT/'target/tsr-integrated-campaign-1/manifest.json');m=a.load('ordinary_rebind_campaign','engine-b91-campaign.py');up=load(base/'campaign-upgrade.json');ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),upgrade=up,upgrade_hash=m.digest(up));status=ledger.status();assert not status['stopped'] and not status['blocking_unknown_calls'] and not status['blocking_unknown_executions']
 old_trial=a.e.sha({'trial':old['directory'],'admission':a.e.sha(old)})
 with ledger.locked() as (_,history):assert not any(r['event'].get('key')==old_trial or r['event'].get('trial')==old_trial for r in history)
 folder=home/'rebound-initial';folder.mkdir(exist_ok=False)
 ad=a.admission(folder/'adapter',copy.deepcopy(old['origin']),c.cli('project','config'),campaign,None,max_decisions=None,max_executions=None,correction_allowance=20,feedback_cycle=old['feedback_cycle'],transient_recovery=old['transient_recovery'],process_profile='ordinary-memory@v1',unlimited_policy_sha256=old['unlimited_policy_sha256'])
 ca={**oldca,'journal':str(folder/'cycle'),'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))};adapter=a.load('ordinary_fresh','engine-browser-ordinary.py').Adapter(ad['directory'],ad,c,ledger)
 authority={'version':'paired-ordinary-initial-rebind@v1','authorization':p.AUTH,'original_packet_sha256':p.PACKET,'old_unadmitted_admission_sha256':a.e.sha(old),'reason':'Shared direct-criterion startup repair; initial arm has no criterion profile, original prompt/information/source/model unchanged','baseline':status,'no_old_effects':True}
 grant=adapter.scope_grant(a.e.sha(authority),'independent-agent:/root/binding_review; fresh ordinary binding review before dispatch',None)
 req,context=adapter.project(c.cli('project','cycle','request'));body=adapter.proposal_body(c.cli('project','cycle','request'),adapter.wire_context(req,context))
 for n,v in {'adapter-admission':ad,'cycle-admission':ca,'scope-authority':authority,'scope-grant':grant,'prospective-request':body}.items():save(folder/(n+'.json'),v)
 return {'path':str(folder),'admission_sha256':a.e.sha(ad),'grant_sha256':a.e.sha(grant),'provider_calls':0}
if __name__=='__main__':print(json.dumps(prepare(),indent=2))
