#!/usr/bin/env python3
"""Prepare isolated local data only; never apply a campaign grant or dispatch effects.

Requires a separately reviewed release and explicit future authority record.
Neither file is manufactured here. Resulting grant remains prospective.
"""
import argparse, datetime, importlib.util, json
from pathlib import Path

def prospective_upgrade_row(rows,upgrade,module):
    return module.Ledger.row(rows,{'kind':'upgrade','key':module.digest(upgrade),'value':upgrade})

def main():
    p=argparse.ArgumentParser(description=__doc__)
    for name in ('directory','release','authority','campaign-manifest'):
        p.add_argument('--'+name,type=Path,required=True)
    args=p.parse_args()
    spec=importlib.util.spec_from_file_location('information_prepare_adapter',Path(__file__).with_name('engine-project-cycle-adapter.py'))
    a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)
    e=a.e;m=a.load('information_prepare_campaign','engine-b91-campaign.py')
    read=lambda path:e.strict(e.bounded_file(path))
    release=read(args.release);authority=read(args.authority);cm=read(args.campaign_manifest)
    if release.get('preparer_sha256')!=a.d.file_sha(Path(__file__)):raise ValueError('reviewed preparation implementation drift')
    scenario=e.scenario_prepare('sequential-pursuit-v1')
    if release['implementation']!=a.pins() or release['binary_sha256']!=a.d.file_sha(e.ROOT/'target/debug/tsr') or release['packet_sha256']!=scenario.pin:
        raise ValueError('reviewed release source/binary/packet drift')
    profile=authority.get('version')
    if profile not in ('native-information-authority@v1','native-information-authority@v2'):raise ValueError('explicit future scope authority record required')
    renewed=profile.endswith('@v2')
    expected={'version':profile,'release_sha256':e.sha(release),'max_calls':16,'max_information_reads':1,'max_program_executions':2,'correction_allowance':2,'campaign_max_calls':239 if renewed else 238,'campaign_max_execution_groups':64,'campaign_cap_microusd':5000000,'reservation_microusd':800000,'wall_seconds':5400}
    if renewed:expected['representation_regeneration']=True
    if any(authority.get(k)!=v or type(authority.get(k))!=type(v) for k,v in expected.items()) or set(authority)!=set(expected)|{'user_message_id'} or not isinstance(authority['user_message_id'],str) or not authority['user_message_id'].strip():
        raise ValueError('explicit future scope authority record required')
    if cm.get('fixture_only'):raise ValueError('live-shaped preparation requires the existing campaign, never a new fixture budget')
    ledger_path=Path(cm['ledger_path']);before=ledger_path.read_bytes()
    rows=[json.loads(line) for line in before.splitlines()]
    reviewer=release['reviewer']
    if not isinstance(reviewer,str) or not reviewer.strip():raise ValueError('independent release review identity required')
    upgrade={'version':'b91-campaign-upgrade@v1','old_manifest_sha256':m.digest(cm),'old_implementation_sha256':cm['implementation_sha256'],'new_implementation_sha256':a.d.file_sha(Path(m.__file__)),'ledger_prefix_head':rows[-1]['hash'],'reviewer':reviewer,'reason':'Reviewed native information detour release; prospective exact cap extension, no reset'}
    ledger=m.Ledger(ledger_path,cm,m.digest(cm),upgrade=upgrade,upgrade_hash=m.digest(upgrade));status=ledger.status()
    if (status['calls'],status['execution_groups'],status['known_cost_microusd'],status['effective_max_calls'],status['effective_max_execution_groups'])!=((223,61,181596,238,64) if renewed else (222,61,180395,230,62)) or status['stopped']:
        raise ValueError('campaign baseline drift/unknown effects; review before preparing')
    root=args.directory.resolve()
    if root.exists():raise ValueError('fresh separate preparation directory required')
    root.mkdir(parents=True)
    for name,value in [('campaign-before',status),('upgrade',upgrade),('authority',authority),('release',release)]:m.fresh(root/(name+'.json'),value)
    predicted=prospective_upgrade_row(rows,upgrade,m)
    m.fresh(root/'prospective-upgrade-row.json',predicted)
    grant={'version':'native-information-extension@v2' if renewed else 'native-information-extension@v1','manifest_sha256':m.digest(cm),'ledger_prefix_head':predicted['hash'],'previous_max_calls':238 if renewed else 230,'max_calls':239 if renewed else 238,'previous_max_execution_groups':64 if renewed else 62,'max_execution_groups':64,'cap_microusd':m.CAP,'reservation_microusd':m.RESERVE,'authority_sha256':m.digest(authority),'reviewer':reviewer,'suite_manifest_sha256':e.sha(scenario.manifest()),'source_class':'newly-authored-nonprivate-native-information-pursuit'}
    m.validate_evaluation_grant(grant,m.digest(cm),predicted['hash'],238 if renewed else 230,64 if renewed else 62)
    m.fresh(root/'prospective-grant.json',grant)
    scenario.prepare(root/'frozen');(root/'state').mkdir();c=a.d.Native(root/'state')
    c.cli('project','init','--spec',str(root/'frozen/scope.json'),'--source',str(root/'frozen/source'),'--requirements',str(root/'frozen/requirements.json'))
    c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':240000,'reason':'Explicit bounded public information pursuit preparation','non_tool':{'model_calls':16,'model_tokens':655360,'wall_ms':5400000,'patch_bytes':4096,'external_cost_microusd':800000}},'--admission')
    c.json_cli(('project','acceptance','feedback'),{'requirements_revision':c.cli('project','acceptance','status')['requirements_revision'],'decision_record':None,'guidance':scenario.REQUIREMENTS})
    e.freeze_origin(root/'frozen',c,root/'origin.json',scenario_id='sequential-pursuit-v1');origin=read(root/'origin.json')
    c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root'])
    info=c.json_cli(('project','information-configure'),{'state_root':c.request()['state_root'],'manifest':scenario.information_manifest(root/'frozen/information'),'local_context_bytes':131072,'max_reads':1,'max_read_ms':5000,'exploration_allowed':True})
    expiry=(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(seconds=5400)).isoformat()
    ad=a.admission(root/'adapter',origin,cm,expiry,release=True,correction_allowance=2,max_executions=3,max_decisions=7,information_admission=info['context']['admission'],representation_regeneration=renewed)
    ca={'version':'local-project-cycle@v4','journal':str(root/'cycle'),'scope':origin['scope'],'max_decisions':7,'max_executions':3,'max_information_reads':1,'expires_at':expiry,'implementation_sha256':a.d.file_sha(Path(a.loop.__file__))}
    m.fresh(root/'adapter-admission.json',ad);m.fresh(root/'cycle-admission.json',ca)
    if ledger_path.read_bytes()!=before:raise ValueError('campaign changed concurrently; prepared grant is stale')
    print(json.dumps({'prepared':str(root),'adapter_sha256':e.sha(ad),'prospective_grant_sha256':m.digest(grant),'expiry':expiry,'campaign_unchanged':True,'grant_applied':False,'provider_calls':0,'operations':0}))

if __name__=='__main__':main()
