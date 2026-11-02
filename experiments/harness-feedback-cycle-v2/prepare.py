"""One prospective session preparation. No provider or worker dispatch."""
import copy,datetime,hashlib,importlib.util,json,shutil
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('a',ROOT/'scripts/engine-browser-adapter.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
def prepare(out):
 source=ROOT/'target/v2-tsr-return-preparation';rec=a.load('recovery','engine-browser-recovery.py')
 m=a.load('campaign','engine-b91-campaign.py');campaign=json.loads((ROOT/'target/tsr-integrated-campaign-1/manifest.json').read_text());upgrade=json.loads((ROOT/'target/v2-tsr-evidence-feedback/campaign-upgrade.json').read_text())
 rows=[json.loads(line) for line in Path(campaign['ledger_path']).read_text().splitlines() if line]
 upgrade={**upgrade,'new_implementation_sha256':hashlib.sha256(Path(m.__file__).read_bytes()).hexdigest(),'ledger_prefix_head':rows[-1]['hash'],'reviewer':'independent-agent:/root/handoff_audit','reason':'Reviewed full authorized feedback session capacity; preserve historical accounting and exact baseline, source, and trial bindings.'}
 ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),upgrade=upgrade,upgrade_hash=m.digest(upgrade));status=ledger.status()
 assert (status['calls'],status['execution_groups'],status['known_cost_microusd'])==(307,83,314198) and not status['stopped']
 assert status['effective_max_calls']>=status['calls'] and status['effective_max_execution_groups']>=status['execution_groups']
 old=json.loads((source/'adapter-admission.json').read_text());obs=ROOT/'target/browser-game-session-live10/independent-browser/observations.json';assessment=json.loads(obs.read_text())
 assert hashlib.sha256(obs.read_bytes()).hexdigest()==old['origin']['prior_review_feedback']['failed_response_sha256']
 assert len(assessment['checks'])==16 and all(v['pass'] for v in assessment['checks'])
 assert {p:hashlib.sha256(text.encode()).hexdigest() for p,text in old['origin']['initial_sources'].items()}==assessment['source']
 original=rec.tree(source/'native');out.mkdir();shutil.copytree(source/'native',out/'native');c=a.d.Native(out/'native')
 def save(n,v):(out/n).write_text(json.dumps(v,indent=2)+'\n')
 b=c.budget();assert not any(b[k] for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests'))
 life=c.cli('project-lifecycle-view');expires=(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(minutes=120)).isoformat()
 extension={'state_root':c.request()['state_root'],'previous':life['admission'],'limits':{'model_calls':b['model_calls']+200,'model_tokens':b['tokens_charged']+200*(a.base.capacity.INPUT_TOKENS+32768),'wall_ms':b['elapsed_wall_ms']+max(0,int((a.d.stamp(expires)-datetime.datetime.now(datetime.timezone.utc).timestamp())*1000)),'patch_bytes':b['patch_bytes_charged']+100*1048576,'external_cost_microusd':b['external_cost_charged_microusd']},'tool_ms_ceiling':b['tool_ms_charged']+6000000,'reason':'Fresh authorized finite harness feedback-cycle validation; original evaluation remains closed; no new wanting or acceptance'}
 save('native-resource-extension.json',extension);c.json_cli(('project-resource-extend',),extension,'--request')
 q=c.request();view=c.cli('project','cycle','request');assert view['branch']=='reconsider',view
 origin=copy.deepcopy(old['origin']);origin['directory']=str(c.directory);coding=q['coding'];origin['public'].update(source_snapshots=[coding['snapshot']],public_returns=[a.e.sha(v) for v in coding['relational']['inputs']['returns']],public_relationships=[a.e.sha(v) for v in coding['relational']['history']]);origin['initial_sources']={f['path']:f['source'] for f in coding['files'] if f['editable']}
 assert origin['initial_sources']==old['origin']['initial_sources'] and origin['public']['parents']==coding['relational']['inputs']['parents']
 policy={'version':'authorized-feedback-cycle@v1','authorization_id':'Sentinel_7b65a46a5854819197fcaa20febfe515','max_continuations':4}
 ad=a.admission(out/'adapter',origin,c.cli('project','config'),campaign,expires,max_decisions=90,max_executions=100,correction_allowance=20,feedback_cycle=policy,transient_recovery={'version':'exact-failed-zero-recovery@v1','max_retries':1,'backoff_seconds':120})
 ca={'version':'local-project-cycle@v3','journal':str(out/'cycle'),'scope':view['scope'],'max_decisions':90,'max_executions':100,'local_source_profile':'browser-scoped-source@v1','stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','completion_tokens':32768,'input_tokens':a.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'expires_at':expires,'implementation_sha256':a.d.file_sha(Path(a.loop.__file__)),'feedback_cycle':policy}
 save('adapter-admission.json',ad);save('cycle-admission.json',ca);save('campaign-upgrade.json',upgrade)
 authority={'version':'authorized-feedback-session@v1','user_message':'Sentinel_7b65a46a5854819197fcaa20febfe515','scope':'same fictional Lantern Maze v2 evidence-return workflow; no semantic outcome prescribed','max_calls':200,'max_operations':100,'max_microusd':10000000,'expires_at':expires,'provider_failure_retries':1,'provider_retry_backoff_seconds':120,'final_acceptance':False,'baseline':status}
 save('scope-authority.json',authority);adapter=a.Adapter(ad['directory'],ad,c,ledger);grant=adapter.scope_grant(a.e.sha(authority),'independent-agent:/root/handoff_audit',10000000,extend_authority=True);save('scope-grant.json',grant)
 _,context=adapter.project(view);save('public-preview.json',context);save('source-lineage.json',{'source':str(source),'source_native_files':original,'original_preserved':rec.tree(source/'native')==original,'observations_sha256':rec.sha(obs),'source_unchanged':True,'parents_unchanged':True})
 pins={'adapter':a.e.sha(ad),'grant':m.digest(grant),'manifest_files':{str(p.relative_to(ROOT)):rec.sha(p) for p in (Path(__file__),Path(__file__).with_name('README.md'))},'source_pins':ad['implementation'],'binary_sha256':ad['binary_sha256'],'baseline':status,'scope':authority}
 save('prepared-manifest.json',pins)
 print(json.dumps({'directory':str(out),'admission':a.e.sha(ad),'grant':m.digest(grant),'manifest':rec.sha(out/'prepared-manifest.json'),'branch':view['branch'],'provider_calls':0,'operations':0},indent=2))
if __name__=='__main__':
 import argparse
 p=argparse.ArgumentParser();p.add_argument('output',type=Path);args=p.parse_args();prepare(args.output.resolve())
