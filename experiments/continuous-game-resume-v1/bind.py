"""Fresh choice admission after exact local response migration; no provider dispatch."""
import copy,importlib.util,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('resume_prepare',Path(__file__).with_name('prepare.py'));p=importlib.util.module_from_spec(s);s.loader.exec_module(p);a=p.a
load=lambda path:json.loads(path.read_text())
def save(path,value):
 with path.open('x') as f:json.dump(value,f,indent=2);f.write('\n')
def prepare():
 base=ROOT/'target/continuous-game-resume-v1';home=base/'tsr';prior=ROOT/'target/continuous-game-live-v1/tsr/continuation-crossing-first'
 manifest=load(p.FROZEN/'manifest.json')
 for path,digest in manifest['files'].items():assert a.d.file_sha(ROOT/path)==digest,path
 migration=a.load('resume_exact_migration','engine-criterion-migration.py');plan=load(base/'migration-plan.json')
 journal=object.__new__(a.d.Driver);journal.admission=plan;journal.fixture=False;journal.root=Path(plan['directory']);journal.file=journal.root/'migration.jsonl'
 _,pending,done=journal.read();assert not pending and set(done)=={'apply'}
 previous=load(prior/'adapter-admission.json');old_cycle=load(prior/'cycle-admission.json');value,request=migration.retained(migration.FrozenReader(previous))
 c=a.d.Native(home/'native');context=c.cli('project','criterion-context');assert context['current']['assessment']==value and context['current']['id']==done['apply']
 for folder in (Path(previous['directory']),Path(old_cycle['journal']),journal.root):assert not (folder/'STOP').exists() and not any(folder.glob('checkpoint-*/STOP'))
 budget=c.budget();assert not any(budget.get(k) for k in ('stopped','exceeded','unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests'))
 campaign=load(ROOT/'target/continuous-game-campaign-v1/manifest.json');m=a.load('resume_ledger','engine-b91-campaign.py');upgrade=load(base/'campaign-upgrade.json');ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),upgrade=upgrade,upgrade_hash=m.digest(upgrade));state=ledger.status();assert not state['stopped'] and not state['blocking_unknown_calls'] and not state['blocking_unknown_executions']
 view=c.request();coding=view['coding'];inputs=coding['relational']['inputs'];files={k:(c.directory/'coding-work'/k).read_text() for k in previous['origin']['initial_sources']}
 assert files==previous['origin']['initial_sources'] and coding['snapshot']==value['frame']['snapshot']
 assert context['policy'][1]==load(prior/'criterion-policy.json')
 public=copy.deepcopy(previous['origin']['public']);public.update(parents=inputs.get('acceptance_parents',inputs['parents']),source_snapshots=[coding['snapshot']],public_returns=[a.e.sha(x) for x in inputs.get('returns',[])],public_relationships=[a.e.sha(x) for x in coding['relational'].get('history',[])],public_preferences=[a.e.sha(x) for x in inputs.get('generated_preferences',[])],public_meanings=[a.e.sha(x) for x in inputs.get('public_desires',{}).get('history',[])],public_pursuits=[a.e.sha(x) for x in coding.get('pursuit',{}).get('history',[])],public_supplemental=[a.e.sha(x) for x in inputs.get('supplemental',[])])
 assert public['parents']==previous['origin']['public']['parents']
 profile={**previous['criterion_return_profile'],'imported_returns':[a.e.sha(x) for x in context['history']]}
 folder=home/'continuation-crossing-first';folder.mkdir(exist_ok=False)
 origin={'directory':str(c.directory),'public':public,'initial_sources':files}
 ad=a.admission(folder/'adapter',origin,c.cli('project','config'),campaign,None,max_decisions=None,max_executions=None,correction_allowance=previous['correction_allowance'],feedback_cycle=previous['feedback_cycle'],transient_recovery=previous['transient_recovery'],process_profile=previous.get('process_profile'),unlimited_policy_sha256=previous['unlimited_policy_sha256'],criterion_return_profile=profile)
 ca={**copy.deepcopy(old_cycle),'journal':str(folder/'cycle'),'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))};adapter=a.Adapter(ad['directory'],ad,c,ledger)
 authority={'authorization':p.AUTH,'scope':'Same ONE test, same campaign and frozen observations; no model assessment or executed observation repeated','parent_admission':a.e.sha(previous),'parent_failed_apply':request,'migration_plan':a.e.sha(plan),'migration_receipt':done['apply'],'original_packet':'2b4b1e02fd1fcac3f7066d6c7ac43f877c26d450fd4bd3d382ebe9d6ac560804','corrected_packet':p.PACKET,'baseline':state,'operator_assistance':'Explicit code/representation repair and derivative admission; original failure retained'}
 grant=adapter.scope_grant(a.e.sha(authority),'independent-agent:/root/binding_review; exact resumed choice binding review before dispatch',None)
 for name,data in {'adapter-admission':ad,'cycle-admission':ca,'scope-authority':authority,'scope-grant':grant,'criterion-context':context}.items():save(folder/(name+'.json'),data)
 request_view=c.cli('project','cycle','request');req,ctx=adapter.project(request_view);save(folder/'prospective-request.json',adapter.proposal_body(request_view,adapter.wire_context(req,ctx)))
 return {'directory':str(folder),'admission':a.e.sha(ad),'provider_calls':0}
if __name__=='__main__':print(json.dumps(prepare(),indent=2))
