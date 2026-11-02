"""Explicit same-test final binding after local preparation timeout; no effect replay."""
import copy,importlib.util,json,os,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('resume_prepare',Path(__file__).with_name('prepare.py'));p=importlib.util.module_from_spec(s);s.loader.exec_module(p);a=p.a
load=lambda path:json.loads(path.read_text())
BASE=ROOT/'target/continuous-game-resume-v1'
OLD=BASE/'tsr/continuation-mixed-pursuit'
NEW=BASE/'tsr/continuation-mixed-pursuit-optimized'
def save(path,value):
    with path.open('x') as f:json.dump(value,f,indent=2);f.write('\n')
def prepare():
    prior=load(OLD/'adapter-admission.json');ca=load(OLD/'cycle-admission.json')
    rows=[a.e.strict(x) for x in (OLD/'adapter/adapter.jsonl').read_bytes().splitlines()]
    assert len(rows)==1 and rows[0]==a.d.Driver.row([],{'kind':'admission','value':prior,'fixture_only':False})
    assert not (OLD/'cycle').exists() and not (OLD/'prospective-request-before-assessment.json').exists()
    assert not (OLD/'adapter/STOP').exists()
    benchmark=load(BASE/'runtime-comparison-final/result.json')
    assert any(x.get('state')=='six' and x.get('exact_json_equivalent') is True for x in benchmark)
    assert any(x.get('state')=='eighteen' and x.get('profile')=='release' and x.get('returncode')==0 for x in benchmark)
    binary=a.d.file_sha(ROOT/'target/debug/tsr')
    assert binary==a.d.file_sha(ROOT/'target/release/tsr') and binary!=prior['binary_sha256']
    assert prior['binary_sha256']==a.d.file_sha(BASE/'pre-optimization-debug-tsr')
    for item in benchmark:
        if 'profile' in item:assert item['binary_sha256']==(prior['binary_sha256'] if item['profile']=='debug' else binary)
    c=a.d.Native(prior['origin']['directory']);budget=c.budget()
    assert not any(budget.get(k) for k in ('stopped','exceeded','unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests'))
    context=c.cli('project','criterion-context');assert context==load(OLD/'criterion-context.json')
    assert context['policy'][1]==load(OLD/'criterion-policy.json')
    view=c.cli('project','cycle','request')
    comparison=load(BASE/'runtime-comparison-final/eighteen-release.stdout');current=copy.deepcopy(view)
    # Native project_handle explicitly hashes canonical episode directory (project.rs:533).
    # The benchmark copy has a different directory. Preserve real handles in dispatch;
    # compare all other fields exactly and retain the handle differences for review.
    handles=[]
    for left,right in zip(comparison['request']['coding']['files'],current['request']['coding']['files'],strict=True):
        assert left['path']==right['path']
        handles.append({'path':left['path'],'benchmark':left.pop('inspection_handle'),'actual':right.pop('inspection_handle')})
    assert current==comparison
    proof=benchmark[-1]
    assert proof['stdout_sha256']==a.d.file_sha(BASE/'runtime-comparison-final/eighteen-release.stdout')
    for name,digest in proof['authoritative_files'].items():assert a.d.file_sha(c.directory/name)==digest,name
    campaign=load(ROOT/'target/continuous-game-campaign-v1/manifest.json');m=a.load('optimized_campaign','engine-b91-campaign.py');upgrade=load(BASE/'campaign-upgrade.json')
    ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),upgrade=upgrade,upgrade_hash=m.digest(upgrade));state=ledger.status()
    assert not state['stopped'] and not state['blocking_unknown_calls'] and not state['blocking_unknown_executions']
    NEW.mkdir(exist_ok=False)
    ad=copy.deepcopy(prior);ad.update(directory=str(NEW/'adapter'),binary_sha256=binary)
    ca=copy.deepcopy(ca);ca['journal']=str(NEW/'cycle')
    adapter=a.Adapter(ad['directory'],ad,c,ledger)
    authority={'authorization':p.AUTH,'scope':'Final frozen batch of same approved test; preserve all eighteen observations, original criteria and same campaign. No repeated feedback, policy, generation or execution.','prior_admission':a.e.sha(prior),'previous_binary':prior['binary_sha256'],'optimized_binary':binary,'profile':'release optimization with debug assertions and overflow checks enabled','benchmark_sha256':a.d.file_sha(BASE/'runtime-comparison-final/result.json'),'baseline':state,'operator_assistance':'Explicit optimized-runtime continuation after preserved local read timeout'}
    grant=adapter.scope_grant(a.e.sha(authority),'independent-agent:/root/binding_review; exact optimized final binding reviewed before dispatch',None)
    req,ctx=adapter.project(view);body=adapter.proposal_body(view,adapter.wire_context(req,ctx))
    for name,value in {'adapter-admission':ad,'cycle-admission':ca,'scope-authority':authority,'scope-grant':grant,'criterion-context':context,'prospective-request-before-assessment':body,'directory-handle-comparison':handles}.items():save(NEW/(name+'.json'),value)
    paths=[Path(__file__).resolve(),ROOT/'scripts/engine-native-cycle.py',ROOT/'experiments/continuous-game-resume-v1/manifest.json',BASE/'campaign-upgrade.json',BASE/'runtime-comparison-final/result.json',*sorted(NEW.glob('*.json')),*sorted(OLD.glob('*.json'))]
    save(NEW/'launch-manifest.json',{'version':'same-test-optimized-final@v1','files':{str(x.relative_to(ROOT)):a.d.file_sha(x) for x in paths}})
    print(json.dumps({'prepared':str(NEW),'baseline':state,'provider_calls_dispatched':0},indent=2))
def launch(mode):
    manifest=load(NEW/'launch-manifest.json')
    for name,digest in manifest['files'].items():assert a.d.file_sha(ROOT/name)==digest,name
    ad=load(NEW/'adapter-admission.json');grant=load(NEW/'scope-grant.json')
    args=[sys.executable,str(ROOT/'scripts/engine-native-cycle.py'),mode,'--browser-v1','--admission',str(NEW/'adapter-admission.json'),'--expected-sha256',a.e.sha(ad),'--cycle-admission',str(NEW/'cycle-admission.json'),'--campaign-manifest',str(ROOT/'target/continuous-game-campaign-v1/manifest.json'),'--campaign-upgrade',str(BASE/'campaign-upgrade.json'),'--scope-grant',str(NEW/'scope-grant.json'),'--scope-grant-sha256',a.e.sha(grant)]
    os.execv(sys.executable,args)
if __name__=='__main__':
    assert len(sys.argv)==2 and sys.argv[1] in ('prepare','check','run')
    prepare() if sys.argv[1]=='prepare' else launch(sys.argv[1])
