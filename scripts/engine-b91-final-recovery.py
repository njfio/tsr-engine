#!/usr/bin/env python3
"""Separately counted final-stage HTTP503 recovery; no execution authority."""
import argparse
import datetime
import fcntl
import json
import os
from pathlib import Path
import shutil
import tempfile
import time
import importlib.util
spec=importlib.util.spec_from_file_location('final_recovery_base',Path(__file__).with_name('engine-b91-patch-recovery.py'))
r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)
b=r.b;d=r.d;e=r.e;c=r.campaign

def implementation():
    return {**r.implementation(),'engine-b91-patch-recovery.py':d.file_sha(Path(r.__file__)),
            Path(__file__).name:d.file_sha(Path(__file__)),'engine-b91-recovery-billing.py':d.file_sha(Path(__file__).with_name('engine-b91-recovery-billing.py'))}

_reading=set()
def read_history(root):
    key=str(Path(root).resolve())
    if key in _reading or len(_reading)>=100:raise ValueError('cyclic or unbounded recovery lineage')
    _reading.add(key)
    try:return _read_history(root)
    finally:_reading.remove(key)

def _read_history(root):
    root=Path(root).resolve();raw=e.bounded_file(root/'driver.jsonl',8*1024*1024)
    native=[e.strict(x) for x in raw.splitlines()];a=native[0]['event']['value'];fixture=a['mode']=='fixture'
    if a['version']=='b91-patch-recovery@v1':
        r.validate(a,root,fixture,expired=True,full=True);cls=r.Recovery
    elif a['version']=='b91-final-recovery@v1':
        validate(a,root,fixture,expired=True);cls=FinalRecovery
    else:raise ValueError('verified patch recovery lineage required')
    view=object.__new__(cls);view.root=root;view.file=root/'driver.jsonl';view.admission=a;view.fixture=fixture
    _,pending,done=view.read();view.verify_archive()
    if pending:raise ValueError('uncertain native application')
    return d,raw,native,a,view,done,view.journal()

def proof_validate(proof,parent,provider_id):
    if set(proof)!={'manifest','upgrade','lookup','review','hashes'}:raise ValueError('proof shape')
    for k in ('manifest','upgrade','lookup','review'):
        if d.file_sha(proof[k])!=proof['hashes'][k]:raise ValueError('proof drift')
    module=e.module('final_billing','engine-b91-recovery-billing.py')
    class FinalLedger(module.RecoveryLedger):
        def read_history(self,root):return read_history(root)
    manifest=e.strict(e.bounded_file(proof['manifest']));upgrade=e.strict(e.bounded_file(proof['upgrade']));review=e.strict(e.bounded_file(proof['review']))
    ledger=FinalLedger(manifest['ledger_path'],manifest,c.digest(manifest),upgrade=upgrade,upgrade_hash=c.digest(upgrade))
    ledger.reconcile_failed_zero(parent,proof['lookup'],review,c.digest(review),dry_run=True)
    with ledger.locked() as (_,rows):
        matched=[x['event'] for x in rows if x['event'].get('kind')=='bill_reconciled' and x['event'].get('provider_request_id')==provider_id and x['event'].get('review_sha256')==c.digest(review)]
        if len(matched)!=1 or matched[0]['microusd']!=0:raise ValueError('recorded exact failed-zero proof required')
    return c.digest(manifest)

def prefix(parent_admission,proof):
    a=e.strict(e.bounded_file(parent_admission,2*1024*1024));root=Path(a['trial']).resolve()
    _,_,_,historical,view,done,j=read_history(root)
    if a!=historical:raise ValueError('parent admission differs from journal')
    events=j.events();intents=[x for x in events if x['kind']=='dispatch_intent']
    if len(intents) not in (5,6):raise ValueError('only final two stages supported')
    failed=intents[-1];index=len(intents)-1
    if failed['stage']!=e.STAGES[index]:raise ValueError('failed stage mismatch')
    outcomes=[x for x in events if x['kind']=='provider_outcome' and x['id']==failed['id']]
    if len(outcomes)!=1 or outcomes[0].get('http_status')!=503 or outcomes[0].get('response_status')!='received_http_error':raise ValueError('exact HTTP503 required')
    ids=outcomes[0].get('response_request_ids',[])
    if len(ids)!=1 or j.status()['unknown_or_failed']!=[failed['id']] or j.status()['application_unknown'] or j.status()['unapplied_response']:raise ValueError('unknown prefix')
    if a['version']=='b91-patch-recovery@v1':
        if any(done.get(k,{}).get('status',{}).get('execution_status')!='passed' for k in ('execute_patch','execute_check_after')):raise ValueError('verified patch and experiment required')
        if any(k not in done for k in ('import_check_after','close_check_after')):raise ValueError('missing experiment provenance')
        anchor={'patch':done['execute_patch']['status'],'check':done['execute_check_after']['status'],
                'root':str(root),'files':{n:d.file_sha(root/n) for n in ('driver.jsonl','provider.jsonl')},'checks':b.tree(root/'checks-after')}
    else:anchor=a['prefix']['anchor']
    inventory=b.tree(root/'project')
    with tempfile.TemporaryDirectory(prefix='b91-final-read-') as tmp:
        copy=Path(tmp)/'project';shutil.copytree(root/'project',copy);native=d.Native(copy)
        request=native.request();status=native.cli('project','status');budget=native.budget()
        if native.cli('project','config')!=e.expected_configuration():raise ValueError('scope drift')
    if b.tree(root/'project')!=inventory or request.get('pending') or status['latest_execution']!=anchor['patch']['latest_execution'] or status['goal_completion']['snapshot']!=anchor['patch']['goal_completion']['snapshot']:raise ValueError('verified source drift')
    if budget['unresolved_tool_requests'] or budget['unsettled_requests'] or budget['unknown_external_cost_requests']!=1:raise ValueError('unreconciled native effects')
    campaign=proof_validate(proof,root,ids[0])
    return {'parent':str(root),'parent_admission':str(Path(parent_admission).resolve()),'admission_hash':e.sha(a),
        'files':{n:d.file_sha(root/n) for n in ('driver.jsonl','provider.jsonl','origin.json')},'stop':d.file_sha(root/'STOP') if (root/'STOP').exists() else None,
        'project_files':inventory,'state_root':request['state_root'],'snapshot':status['goal_completion']['snapshot'],
        'requirements_revision':status['goal_completion']['requirements_revision'],'anchor':anchor,'failed_request':failed['id'],
        'provider_id':ids[0],'first_stage':index,'proof':proof,'campaign':campaign,'fixture_only':view.fixture}

def admission(parent,root,proof,not_before=None):
    p=prefix(parent,proof);now=datetime.datetime.now(datetime.timezone.utc);count=6-p['first_stage']
    start=max(now.timestamp(),not_before or now.timestamp())
    return {'version':'b91-final-recovery@v1','mode':'fixture' if p['fixture_only'] else 'live','trial':str(Path(root).resolve()),
        'created_at':now.isoformat(),'not_before':start,'expires_at':datetime.datetime.fromtimestamp(start+3600,datetime.timezone.utc).isoformat(),
        'prefix':p,'model':'gpt-6-luna','calls':count,'execution_groups':0,'patch_bytes':0,'input_tokens':32768*count,
        'completion_tokens':8192*count,'payload_bytes':32768,'call_seconds':240,'implementation':implementation(),'binary_sha256':d.file_sha(d.ROOT/'target/debug/tsr')}

def validate(a,root,fixture,*,expired=False):
    expected={'version','mode','trial','created_at','not_before','expires_at','prefix','model','calls','execution_groups','patch_bytes','input_tokens','completion_tokens','payload_bytes','call_seconds','implementation','binary_sha256'}
    count=6-a['prefix']['first_stage']
    fixed={'version':'b91-final-recovery@v1','model':'gpt-6-luna','calls':count,'execution_groups':0,'patch_bytes':0,'input_tokens':32768*count,'completion_tokens':8192*count,'payload_bytes':32768,'call_seconds':240}
    if set(a)!=expected or count not in (1,2) or any(type(a.get(k))!=type(v) or a[k]!=v for k,v in fixed.items()) or a['mode']!=('fixture' if fixture else 'live'):raise ValueError('zero execution admission shape')
    if a['implementation']!=implementation() or a['binary_sha256']!=d.file_sha(d.ROOT/'target/debug/tsr'):raise ValueError('implementation drift')
    if Path(a['trial']).resolve()!=root or any(p.is_symlink() for p in (root,*root.parents)):raise ValueError('path drift')
    if not fixture and (not root.is_relative_to(d.ROOT/'target') or root==d.ROOT/'target'):raise ValueError('disposable root')
    start,end=d.stamp(a['created_at']),d.stamp(a['expires_at'])
    if type(a['not_before']) not in (int,float) or not start<=a['not_before']<end<=a['not_before']+3600 or (not expired and time.time()>=end):raise ValueError('invalid admission chronology')
    p=a['prefix'];parent=Path(p['parent'])
    if root==parent or root.is_relative_to(parent) or parent.is_relative_to(root):raise ValueError('separate clone required')
    if e.sha(e.strict(e.bounded_file(p['parent_admission'],2*1024*1024)))!=p['admission_hash'] or any(d.file_sha(parent/n)!=h for n,h in p['files'].items()) or b.tree(parent/'project')!=p['project_files']:raise ValueError('parent drift')
    if (d.file_sha(parent/'STOP') if (parent/'STOP').exists() else None)!=p['stop']:raise ValueError('stop drift')
    anchor=p['anchor'];aroot=Path(anchor['root'])
    if any(d.file_sha(aroot/n)!=h for n,h in anchor['files'].items()) or b.tree(aroot/'checks-after')!=anchor['checks']:raise ValueError('verified evidence anchor drift')
    if proof_validate(p['proof'],parent,p['provider_id'])!=p['campaign']:raise ValueError('campaign proof drift')

class FinalRecovery(b.Continuation):
    def __init__(self,root,a,transport,reviewer,*,fixture=False,hook=None):
        self.root=Path(root).resolve();self.admission=a;self.fixture=fixture;self.file=self.root/'driver.jsonl'
        terminal=self.file.exists() and 'terminal' in self.read()[2]
        validate(a,self.root,fixture,expired=terminal)
        self.transport=transport;self.reviewer=reviewer;self.hook=hook or (lambda *_:None);self.active=set();self.identity=e.sha(a)
        self.root.mkdir(parents=True,exist_ok=True)
        if not self.file.exists():
            if any(self.root.iterdir()):raise ValueError('fresh clone required')
            e.private_save(self.file,self.row([],{'kind':'admission','value':a,'fixture_only':fixture}))
    def validate_current_admission(self):
        validate(self.admission,self.root,self.fixture)
        if time.time()<self.admission['not_before']:raise d.Blocked('backoff not elapsed')
        if 'clone_prefix' in self.read()[2]:self.verify_archive()
    def projection(self):
        p=self.admission['prefix'];return {'parent_files':p['files'],'failed_request_retained':p['failed_request'],'imported_successful_stages':list(e.STAGES[:p['first_stage']]),'new_calls_admitted':self.admission['calls'],'namespace':e.sha(self.admission)}
    def verify_archive(self):
        p=self.admission['prefix'];archive=self.root/'parent-evidence'
        if e.strict(e.bounded_file(self.root/'prefix-projection.json'))!=self.projection() or any(d.file_sha(archive/n)!=h for n,h in p['files'].items()):raise ValueError('archive drift')
        if (d.file_sha(archive/'STOP') if (archive/'STOP').exists() else None)!=p['stop']:raise ValueError('archived STOP drift')
        if b.tree(self.root/'parent-shadow')!={k.removeprefix('shadow/'):v for k,v in p['project_files'].items() if k.startswith('shadow/')}:raise ValueError('accounting archive drift')
        rows=e.bounded_file(archive/'provider.jsonl',8*1024*1024).splitlines();stop=next(i for i,x in enumerate(rows) if e.strict(x)['event'].get('id')==p['failed_request'])
        if e.bounded_file(self.root/'provider.jsonl',8*1024*1024).splitlines()[:stop]!=rows[:stop]:raise ValueError('projection prefix drift')
    def journal(self):
        j=e.Journal(self.root/'provider.jsonl',self.root/'origin.json',e.sha(e.strict(e.bounded_file(self.root/'origin.json'))))
        j.origin={**j.origin,'directory':str(self.root/'project')};j.recovery_namespace=e.sha(self.admission);return j
    def verify_clone(self):
        self.verify_archive();native=self.controller();s=native.cli('project','status');p=self.admission['prefix']
        if s['goal_completion']['snapshot']!=p['snapshot'] or s['goal_completion']['requirements_revision']!=p['requirements_revision'] or s['latest_execution']!=p['anchor']['patch']['latest_execution'] or native.request().get('pending'):raise ValueError('verified source drift')
        return native
    def initialize(self):
        p=self.admission['prefix'];parent=Path(p['parent'])
        def clone():
            shutil.copytree(parent/'project',self.root/'project')
            if b.tree(self.root/'project')!=p['project_files']:raise ValueError('copy drift')
            (self.root/'project/shadow').rename(self.root/'parent-shadow');archive=self.root/'parent-evidence';archive.mkdir()
            for name in p['files']:shutil.copyfile(parent/name,archive/name)
            if p['stop'] is not None:shutil.copyfile(parent/'STOP',archive/'STOP')
            shutil.copyfile(parent/'origin.json',self.root/'origin.json')
            rows=[e.strict(x) for x in e.bounded_file(parent/'provider.jsonl',8*1024*1024).splitlines()];stop=next(i for i,x in enumerate(rows) if x['event'].get('id')==p['failed_request'])
            with (self.root/'provider.jsonl').open('xb') as f:
                for row in rows[:stop]:f.write(e.canonical(row)+b'\n')
            e.private_save(self.root/'prefix-projection.json',self.projection())
            if self.controller().request()['state_root']!=p['state_root']:raise ValueError('state clone drift')
            return {'parent':p,'zero_execution':True}
        self.op('clone_prefix',clone);native=self.verify_clone()
        self.op('open_final',lambda:native.json_cli(('project-lifecycle-open',),{'state_root':native.request()['state_root'],'training':None,'tool_ms_ceiling':1,'reason':'Separately admitted zero-execution final continuation',
            'non_tool':{'model_calls':self.admission['calls'],'model_tokens':self.admission['input_tokens']+self.admission['completion_tokens'],'wall_ms':max(1,int((d.stamp(self.admission['expires_at'])-time.time())*1000)),'patch_bytes':0,'external_cost_microusd':0}},'--admission'))
        self.op('unpriced_accounting',lambda:native.cli('project','resource-unpriced-admit','--state-root',native.request()['state_root']))
        return native
    def resource_summary(self):
        budget=self.controller().budget();return {'episodes':{'final':budget},'exceeded':budget['exceeded'] or budget['model_calls']>self.admission['calls'] or budget['tool_ms_charged']!=0 or budget['patch_bytes_charged']!=0,'historical_resources':'All prior attempts remain charged in campaign and archived'}
    def run(self):
        with os.fdopen(os.open(self.root/'run.lock',os.O_WRONLY|os.O_CREAT|os.O_NOFOLLOW,0o600),'a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);_,pending,done=self.read()
            if pending:raise d.Blocked('uncertain effect; no retry')
            if 'terminal' in done:self.verify_clone();return done['terminal']
            native=self.initialize()
            if self.admission['prefix']['first_stage']==4:self.relation(native,'relation_after_verification')
            final=self.generate(native,'final_proposal_only')
            review=self.review(native,'semantic_coverage',{'final_proposal':final,'status':native.cli('project','status'),'requirements':e.prepare.REQUIREMENTS,'limits':'Independently review exact unchanged verified source; no execution authority.'},'final_semantic')
            if review['native_review'].get('decision') not in ('accept','reject'):raise d.Blocked('complete independent review required')
            self.op('adopt_final_semantic',lambda:native.json_cli(('project','acceptance','review'),review['native_review']))
            # This episode performed no action; close advisory without a new training outcome.
            self.op('close_final',lambda:native.cli('project-lifecycle-close-advisory','--reason','Zero-execution continuation; source review separately recorded, no new action outcome'))
            self.verify_clone()
            return self.op('terminal',lambda:{'status':'completed_reviewed_final_continuation','calls':self.admission['calls'],'execution_groups':0,'fixture_only':self.fixture,'live_b91_passed':False,'resources':self.resource_summary(),'parent':self.admission['prefix']['parent'],'final_proposal_only':True,'acceptance_review':review['native_review']['decision']})

def campaign_runner(root,a,ledger,transport,reviewer,*,fixture=False,hook=None):
    if c.digest(ledger.manifest)!=a['prefix']['campaign'] or ledger.manifest['fixture_only']!=fixture:raise ValueError('same campaign required')
    trial=c.digest({'trial':str(Path(root).resolve()),'admission':e.sha(a)})
    run=FinalRecovery(root,a,None,reviewer,fixture=fixture,hook=hook);ledger.enroll(trial,e.sha(a))
    def invoke(body,timeout):
        intent=[x for x in run.journal().events() if x['kind']=='dispatch_intent'][-1]
        return ledger.run_call(trial,intent['id'],body,timeout,transport,payload_limit=32768)
    run.transport=invoke;return run

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('mode',choices=('prepare','inspect','run'));parser.add_argument('--parent-admission',type=Path);parser.add_argument('--proof',type=Path);parser.add_argument('--trial',type=Path);parser.add_argument('--admission',type=Path,required=True);parser.add_argument('--admission-sha256');parser.add_argument('--reviews',type=Path);parser.add_argument('--not-before',type=float);parser.add_argument('--allow-paid-inference',action='store_true');args=parser.parse_args()
    if args.mode=='prepare':
        if not args.parent_admission or not args.proof or not args.trial:parser.error('parent, proof and fresh root required')
        a=admission(args.parent_admission,args.trial,e.strict(e.bounded_file(args.proof)),args.not_before);e.private_save(args.admission,a);print(json.dumps({'admission_sha256':e.sha(a),'dispatch_enabled':False}));return
    a=e.strict(e.bounded_file(args.admission,4*1024*1024));root=Path(a['trial']).resolve()
    if args.mode=='inspect':
        view=object.__new__(FinalRecovery);view.root=root;view.file=root/'driver.jsonl';view.admission=a;view.fixture=a['mode']=='fixture';rows,pending,done=view.read();print(json.dumps({'dispatch_enabled':False,'events':len(rows),'pending':list(pending),'done':list(done)}));return
    if not args.allow_paid_inference or args.admission_sha256!=e.sha(a) or not args.reviews:parser.error('exact admission, paid opt-in and review directory required')
    p=a['prefix']['proof'];manifest=e.strict(e.bounded_file(p['manifest']));upgrade=e.strict(e.bounded_file(p['upgrade']));ledger=c.Ledger(manifest['ledger_path'],manifest,c.digest(manifest),upgrade=upgrade,upgrade_hash=c.digest(upgrade))
    if ledger.status()['stopped']:raise d.Blocked('campaign reconciliation required')
    if time.time()<a['not_before']:raise d.Blocked('backoff not elapsed')
    ledger.record_price(c.official_preflight());transport=e.module('final_transport','engine-synthetic-transport.py')
    run=campaign_runner(root,a,ledger,lambda body,timeout:transport.invoke(body,timeout,expected_model='gpt-6-luna',payload_limit=32768),d.FileReviewer(args.reviews))
    try:result=run.run()
    except (ValueError,d.Blocked) as error:result={'status':'blocked','reason':str(error),'no_retry':True,'live_b91_passed':False}
    print(json.dumps({'driver':result,'campaign':ledger.status()},indent=2))
if __name__ == '__main__':
    import runpy, sys
    runpy.run_path(str(Path(__file__).with_name('engine-output-release.py')))['check_cli'](__file__,sys.argv[1:])

if __name__=='__main__':main()
