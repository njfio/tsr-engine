#!/usr/bin/env python3
"""Narrow separately admitted recovery after proven uncharged boundary-patch HTTP503.
No original journal, reservation or failed attempt is rewritten or refunded.
"""
import argparse
import json
import datetime
import fcntl
import importlib.util
import os
from pathlib import Path
import shutil
import tempfile
import time

spec=importlib.util.spec_from_file_location('b91_recovery_base',Path(__file__).with_name('engine-b91-continuation.py'))
b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
d=b.d;e=d.e
campaign=e.module('b91_recovery_campaign','engine-b91-campaign.py')

def implementation():
    return {**b.implementation(),Path(__file__).name:d.file_sha(Path(__file__)),
            'engine-b91-campaign.py':d.file_sha(Path(__file__).with_name('engine-b91-campaign.py'))}

def proof_validate(proof,parent,provider_id):
    if set(proof)!={'manifest','upgrade','lookup','review','hashes'}:raise ValueError('proof shape')
    for name in ('manifest','upgrade','lookup','review'):
        if d.file_sha(proof[name])!=proof['hashes'][name]:raise ValueError('proof file drift')
    m=e.strict(e.bounded_file(proof['manifest']));u=e.strict(e.bounded_file(proof['upgrade']));r=e.strict(e.bounded_file(proof['review']))
    ledger=campaign.Ledger(m['ledger_path'],m,campaign.digest(m),upgrade=u,upgrade_hash=campaign.digest(u))
    ledger.reconcile_failed_zero(parent,proof['lookup'],r,campaign.digest(r),dry_run=True)
    with ledger.locked() as (_,rows):
        matches=[x['event'] for x in rows if x['event'].get('kind')=='bill_reconciled' and x['event'].get('provider_request_id')==provider_id and x['event'].get('review_sha256')==campaign.digest(r)]
        if len(matches)!=1 or matches[0]['microusd']!=0:raise ValueError('recorded exact failed-zero reconciliation required')
    return campaign.digest(m)

def prefix(parent_admission,proof):
    a=e.strict(e.bounded_file(parent_admission,16384));root=Path(a['trial']).resolve()
    v=object.__new__(d.Driver);v.root=root;v.file=root/'driver.jsonl';v.admission=a;v.fixture=a['mode']=='fixture'
    _,pending,done=v.read();j=v.journal();events=j.events();intents=[x for x in events if x['kind']=='dispatch_intent']
    if pending or a['model']!='gpt-6-luna' or len(intents)!=4 or intents[-1]['stage']!='boundary_patch':raise ValueError('exact terminal patch prefix required')
    failed=intents[-1];outcomes=[x for x in events if x['kind']=='provider_outcome' and x['id']==failed['id']]
    if len(outcomes)!=1 or outcomes[0].get('response_status')!='received_http_error' or outcomes[0].get('http_status')!=503 or outcomes[0].get('error_code')!='http_error':raise ValueError('exact HTTP503 required')
    ids=outcomes[0].get('response_request_ids',[])
    if len(ids)!=1:raise ValueError('one exact provider identity required')
    if j.status()['unknown_or_failed']!=[failed['id']] or j.status()['application_unknown'] or j.status()['unapplied_response']:raise ValueError('other uncertain provider effects')
    if any(k in done for k in ('commit_patch','execute_patch','generate_boundary_patch')):raise ValueError('patch already applied/generated')
    if any(done.get(k,{}).get('status',{}).get('execution_status')!='needs_repair' for k in ('execute_seed','execute_check_before')):raise ValueError('expected prior returns required')
    for k in ('admit_relation_after_seed','admit_experiment','admit_relation_after_experiment','import_check_before','close_check_before'):
        if k not in done:raise ValueError('missing reviewed prefix')
    historical_tool=done['execute_seed']['resources']['tool_ms_charged']+done['execute_check_before']['resources']['tool_ms_charged']
    if historical_tool+120000>240000:raise ValueError('cumulative tool ceiling exhausted')
    campaign_id=proof_validate(proof,root,ids[0])
    inventory=b.tree(root/'project')
    with tempfile.TemporaryDirectory(prefix='b91-patch-prefix-') as tmp:
        copied=Path(tmp)/'project';shutil.copytree(root/'project',copied)
        c=d.Native(copied);request=c.request();status=c.cli('project','status');budget=c.budget()
        if c.cli('project','config')!=e.expected_configuration():raise ValueError('scope drift')
    if b.tree(root/'project')!=inventory:raise ValueError('source changed during inspection')
    expected=done['execute_seed']['status']
    if request.get('pending') or status['latest_execution']!=expected['latest_execution'] or status['goal_completion']['snapshot']!=expected['goal_completion']['snapshot'] or len(request['coding']['relational']['history'])!=2:raise ValueError('native prefix drift')
    if budget['unresolved_tool_requests'] or budget['unsettled_requests'] or budget['unknown_external_cost_requests']!=1:raise ValueError('unexpected native resource uncertainty')
    names=('driver.jsonl','provider.jsonl','origin.json')
    return {'parent':str(root),'parent_admission':str(Path(parent_admission).resolve()),'admission_hash':e.sha(a),
        'files':{n:d.file_sha(root/n) for n in names},'stop':d.file_sha(root/'STOP') if (root/'STOP').exists() else None,
        'project_files':inventory,'before_files':b.tree(root/'checks-before'),
        'state_root':request['state_root'],'snapshot':status['goal_completion']['snapshot'],
        'requirements_revision':status['goal_completion']['requirements_revision'],'commitment':status['latest_execution']['commitment'],
        'historical_tool_ms':historical_tool,'failed_request':failed['id'],'provider_id':ids[0],'campaign':campaign_id,'proof':proof,
        'experiment_admission':done['admit_experiment'],'experiment':done['generate_experiment']['response'],'fixture_only':v.fixture}

def admission(parent,root,proof):
    p=prefix(parent,proof);now=datetime.datetime.now(datetime.timezone.utc)
    return {'version':'b91-patch-recovery@v1','mode':'fixture' if p['fixture_only'] else 'live','trial':str(Path(root).resolve()),
        'created_at':now.isoformat(),'expires_at':(now+datetime.timedelta(hours=1)).isoformat(),'prefix':p,'model':'gpt-6-luna',
        'calls':3,'execution_groups':2,'patch_bytes':4096,'tool_ms':120000,'input_tokens':81920,'completion_tokens':24576,'call_seconds':240,
        'stage_payload_bytes':{stage:32768 if stage in e.STAGES[-2:] else 16384 for stage in e.STAGES},
        'implementation':implementation(),'binary_sha256':d.file_sha(d.ROOT/'target/debug/tsr')}

def validate(a,root,fixture,*,expired=False,full=False):
    if set(a)!={'version','mode','trial','created_at','expires_at','prefix','model','calls','execution_groups','patch_bytes','tool_ms','input_tokens','completion_tokens','call_seconds','stage_payload_bytes','implementation','binary_sha256'}:raise ValueError('recovery admission shape')
    fixed={'version':'b91-patch-recovery@v1','model':'gpt-6-luna','calls':3,'execution_groups':2,'patch_bytes':4096,'tool_ms':120000,'input_tokens':81920,'completion_tokens':24576,'call_seconds':240}
    if any(type(a.get(k))!=type(v) or a[k]!=v for k,v in fixed.items()) or a['mode']!=('fixture' if fixture else 'live'):raise ValueError('recovery bounds/mode')
    if a['implementation']!=implementation() or a['binary_sha256']!=d.file_sha(d.ROOT/'target/debug/tsr'):raise ValueError('implementation drift')
    if a['stage_payload_bytes']!={stage:32768 if stage in e.STAGES[-2:] else 16384 for stage in e.STAGES}:raise ValueError('stage bounds')
    if Path(a['trial']).resolve()!=root or any(p.is_symlink() for p in (root,*root.parents)):raise ValueError('recovery path')
    if not fixture and (not root.is_relative_to(d.ROOT/'target') or root==d.ROOT/'target'):raise ValueError('disposable target required')
    start,end=d.stamp(a['created_at']),d.stamp(a['expires_at'])
    if not start<end<=start+3600 or (not expired and not start<=time.time()<end):raise ValueError('expired recovery')
    p=a['prefix'];parent=Path(p['parent'])
    if root==parent or root.is_relative_to(parent) or parent.is_relative_to(root):raise ValueError('separate recovery required')
    if e.sha(e.strict(e.bounded_file(p['parent_admission'])))!=p['admission_hash'] or any(d.file_sha(parent/n)!=h for n,h in p['files'].items()) or b.tree(parent/'project')!=p['project_files'] or b.tree(parent/'checks-before')!=p['before_files']:raise ValueError('parent history drift')
    if (d.file_sha(parent/'STOP') if (parent/'STOP').exists() else None)!=p['stop']:raise ValueError('parent stop drift')
    if proof_validate(p['proof'],parent,p['provider_id'])!=p['campaign']:raise ValueError('campaign drift')
    if full and prefix(p['parent_admission'],p['proof'])!=p:raise ValueError('prefix mismatch')

class Recovery(d.Driver):
    def __init__(self,root,a,transport,worker,reviewer,*,fixture=False,hook=None):
        self.root=Path(root).resolve();self.admission=a;self.fixture=fixture;self.file=self.root/'driver.jsonl'
        terminal=self.file.exists() and 'terminal' in self.read()[2]
        validate(a,self.root,fixture,expired=terminal,full=True)
        self.transport=transport;self.worker=worker;self.reviewer=reviewer;self.hook=hook or (lambda *_:None);self.active=set();self.identity=e.sha(a)
        self.root.mkdir(parents=True,exist_ok=True)
        if not self.file.exists():
            if any(self.root.iterdir()):raise ValueError('fresh recovery required')
            e.private_save(self.file,self.row([],{'kind':'admission','value':a,'fixture_only':fixture}))
    def validate_current_admission(self):
        validate(self.admission,self.root,self.fixture)
        if 'clone_prefix' in self.read()[2]:self.verify_archive()
    def projection(self):
        p=self.admission['prefix']
        return {'parent_files':p['files'],'failed_request_retained':p['failed_request'],'imported_successful_stages':list(e.STAGES[:3]),'original_calls_consumed':4,'new_calls_admitted':3,'namespace':e.sha(self.admission)}
    def verify_archive(self):
        p=self.admission['prefix'];archive=self.root/'parent-evidence'
        if e.strict(e.bounded_file(self.root/'prefix-projection.json'))!=self.projection():raise ValueError('projection declaration drift')
        if (d.file_sha(archive/'STOP') if (archive/'STOP').exists() else None)!=p['stop']:raise ValueError('archived STOP drift')
        if any(d.file_sha(archive/n)!=h for n,h in p['files'].items()):raise ValueError('archived history drift')
        if b.tree(archive/'checks-before')!=p['before_files']:raise ValueError('archived experiment drift')
        if b.tree(self.root/'parent-shadow')!={k.removeprefix('shadow/'):v for k,v in p['project_files'].items() if k.startswith('shadow/')}:raise ValueError('archived accounting drift')
        rows=e.bounded_file(archive/'provider.jsonl',8*1024*1024).splitlines()
        stop=next(i for i,x in enumerate(rows) if e.strict(x)['event'].get('id')==p['failed_request'])
        projected=e.bounded_file(self.root/'provider.jsonl',8*1024*1024).splitlines()
        if projected[:stop]!=rows[:stop]:raise ValueError('imported provider projection drift')
    def journal(self):
        j=e.Journal(self.root/'provider.jsonl',self.root/'origin.json',e.sha(e.strict(e.bounded_file(self.root/'origin.json'))))
        if j.origin['directory']!=str(Path(self.admission['prefix']['parent'])/'project'):raise ValueError('origin drift')
        j.origin={**j.origin,'directory':str(self.root/'project')};j.recovery_namespace=e.sha(self.admission)
        return j
    def initialize(self):
        p=self.admission['prefix'];parent=Path(p['parent'])
        def clone():
            shutil.copytree(parent/'project',self.root/'project')
            if b.tree(self.root/'project')!=p['project_files']:raise ValueError('native copy drift')
            (self.root/'project/shadow').rename(self.root/'parent-shadow')
            archive=self.root/'parent-evidence';archive.mkdir()
            for name in ('driver.jsonl','provider.jsonl','origin.json'):shutil.copyfile(parent/name,archive/name)
            if p['stop'] is not None:shutil.copyfile(parent/'STOP',archive/'STOP')
            shutil.copytree(parent/'checks-before',archive/'checks-before')
            shutil.copyfile(parent/'origin.json',self.root/'origin.json')
            rows=[e.strict(x) for x in e.bounded_file(parent/'provider.jsonl',8*1024*1024).splitlines()]
            stop=next(i for i,x in enumerate(rows) if x['event'].get('id')==p['failed_request'])
            # Explicit derived prefix projection, not the complete historical audit.
            with (self.root/'provider.jsonl').open('xb') as f:
                for row in rows[:stop]:f.write(e.canonical(row)+b'\n')
            e.private_save(self.root/'prefix-projection.json',self.projection())
            if self.controller().request()['state_root']!=p['state_root']:raise ValueError('clone state drift')
            return {'parent':p,'archive':'parent-evidence','old_resources_archived':True}
        self.op('clone_prefix',clone);c=self.controller()
        self.op('open_recovery',lambda:c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,
            'tool_ms_ceiling':60000,'reason':'Separately admitted patch recovery; original failed request retained and counted',
            'non_tool':{'model_calls':3,'model_tokens':106496,'wall_ms':max(1,int((d.stamp(self.admission['expires_at'])-time.time())*1000)),
                'patch_bytes':4096,'external_cost_microusd':0}},'--admission'))
        self.op('unpriced_accounting',lambda:c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root']))
        return c
    def generate(self,c,stage):
        if stage not in e.STAGES[3:]:raise ValueError('recovery generation scope')
        return super().generate(c,stage)
    def execute(self,c,stage):
        if stage not in ('patch','check_after'):raise ValueError('recovery execution scope')
        return super().execute(c,stage)
    def resource_summary(self):
        result=super().resource_summary();result['exceeded']=result['exceeded'] or result['tool_ms_charged']>120000 or result['model_calls']>3
        result['historical_resources']='Parent and parent-shadow remain counted in original campaign; no refunds or double charge'
        return result
    def run(self):
        with os.fdopen(os.open(self.root/'run.lock',os.O_WRONLY|os.O_CREAT|os.O_NOFOLLOW,0o600),'a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
            _,pending,done=self.read()
            if pending:raise d.Blocked('uncertain recovery; no retry')
            if 'terminal' in done:
                self.verify_archive();return done['terminal']
            c=self.initialize();p=self.admission['prefix']
            result=self.patch_tail(c,p['experiment_admission'],p['experiment'])
            result.update(calls=self.journal().status()['calls_consumed_or_reserved']-3,execution_groups=2,parent=p['parent'],historical_calls=4,failed_request=p['failed_request'])
            return self.op('terminal',lambda:result)


def campaign_runner(root,a,ledger,transport,worker,reviewer,*,fixture=False,hook=None):
    """Same callable used by CLI and tests; campaign owns aggregate reservations."""
    if campaign.digest(ledger.manifest)!=a['prefix']['campaign'] or ledger.manifest['fixture_only']!=fixture:raise ValueError('same campaign required')
    trial=campaign.digest({'trial':str(Path(root).resolve()),'admission':e.sha(a)})
    run=Recovery(root,a,None,None,reviewer,fixture=fixture,hook=hook)
    ledger.enroll(trial,e.sha(a))
    def invoke(body,timeout):
        intent=[x for x in run.journal().events() if x['kind']=='dispatch_intent'][-1]
        stage=json.loads(body['messages'][1]['content'])['stage']
        return ledger.run_call(trial,intent['id'],body,timeout,transport,payload_limit=d.stage_payload_limit(a,stage))
    def execute(c,stage):
        return ledger.run_execution(trial,stage,e.sha(a),lambda:worker(c,stage))
    run.transport=invoke;run.worker=execute
    return run

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('mode',choices=('prepare','inspect','run'))
    p.add_argument('--parent-admission',type=Path);p.add_argument('--proof',type=Path);p.add_argument('--trial',type=Path)
    p.add_argument('--admission',type=Path,required=True);p.add_argument('--admission-sha256');p.add_argument('--reviews',type=Path)
    p.add_argument('--allow-paid-inference',action='store_true');p.add_argument('--allow-target-execution',action='store_true');args=p.parse_args()
    if args.mode=='prepare':
        if not args.parent_admission or not args.proof or not args.trial:p.error('parent admission, proof manifest and fresh trial required')
        a=admission(args.parent_admission,args.trial,e.strict(e.bounded_file(args.proof)))
        e.private_save(args.admission,a);print(json.dumps({'admission_sha256':e.sha(a),'dispatch_enabled':False}));return
    a=e.strict(e.bounded_file(args.admission,1024*1024));root=Path(a['trial']).resolve()
    if args.mode=='inspect':
        v=object.__new__(Recovery);v.root=root;v.file=root/'driver.jsonl';v.admission=a;v.fixture=a['mode']=='fixture'
        rows,pending,done=v.read();print(json.dumps({'dispatch_enabled':False,'events':len(rows),'pending':list(pending),'done':list(done)}));return
    if not args.allow_paid_inference or not args.allow_target_execution or args.admission_sha256!=e.sha(a) or not args.reviews:p.error('exact admission and both execution opt-ins plus independent reviews required')
    proof=a['prefix']['proof'];cm=e.strict(e.bounded_file(proof['manifest']));u=e.strict(e.bounded_file(proof['upgrade']))
    ledger=campaign.Ledger(cm['ledger_path'],cm,campaign.digest(cm),upgrade=u,upgrade_hash=campaign.digest(u))
    if ledger.status()['stopped']:raise d.Blocked('campaign reconciliation required')
    ledger.record_price(campaign.official_preflight())
    transport=e.module('recovery_transport','engine-synthetic-transport.py')
    def invoke(body,timeout):
        stage=json.loads(body['messages'][1]['content'])['stage']
        return transport.invoke(body,timeout,expected_model='gpt-6-luna',payload_limit=d.stage_payload_limit(a,stage))
    run=campaign_runner(root,a,ledger,invoke,d.live_worker,d.FileReviewer(args.reviews))
    try:result=run.run()
    except (ValueError,d.Blocked) as error:result={'status':'blocked','reason':str(error),'no_retry':True,'live_b91_passed':False}
    print(json.dumps({'driver':result,'campaign':ledger.status()},indent=2))
if __name__ == '__main__':
    import runpy, sys
    runpy.run_path(str(Path(__file__).with_name('engine-output-release.py')))['check_cli'](__file__,sys.argv[1:])

if __name__=='__main__':main()
