#!/usr/bin/env python3
"""Separate two-request, zero-execution B9.1 continuation. Prepare/inspect never dispatch."""
import argparse
import datetime
import fcntl
import importlib.util
import json
import os
from pathlib import Path
import shutil
import time
import tempfile

spec=importlib.util.spec_from_file_location('continuation_driver',Path(__file__).with_name('engine-b91-driver.py'))
d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d)
e=d.e

def implementation():
    return {**d.implementation(),Path(__file__).name:d.file_sha(Path(__file__))}

def tree(root):
    root=Path(root); result={}
    for p in sorted(root.rglob('*')):
        if p.is_symlink():raise ValueError('symlink source')
        if p.is_file():result[p.relative_to(root).as_posix()]=d.file_sha(p)
    return result

def prefix(parent_admission):
    a=e.strict(e.bounded_file(parent_admission,16384));root=Path(a['trial']).resolve()
    v=object.__new__(d.Driver);v.root=root;v.file=root/'driver.jsonl';v.admission=a;v.fixture=a['mode']=='fixture'
    _,pending,done=v.read();j=v.journal();status=j.status()
    # Native request construction may materialize derived CAS caches even for reads.
    # Perform every native inspection on a disposable byte-copy, never the parent.
    inventory=tree(root/'project')
    with tempfile.TemporaryDirectory(prefix='b91-prefix-read-') as tmp:
        copied=Path(tmp)/'project';shutil.copytree(root/'project',copied)
        if tree(copied)!=inventory:raise ValueError('parent copy drift')
        c=d.Native(copied);r=c.request();current=c.cli('project','status');b=c.budget();config=c.cli('project','config')
    if tree(root/'project')!=inventory:raise ValueError('parent changed during inspection')
    if a['model']!='gpt-6-luna' or pending or not (root/'STOP').is_file():raise ValueError('terminal Luna prefix required')
    if status['calls_consumed_or_reserved']!=4 or any(status[k] for k in ('unknown_or_failed','application_unknown','unsettled_metering','unapplied_response')):raise ValueError('settled four-call prefix required')
    if any(done.get(k,{}).get('status',{}).get('execution_status')!='passed' for k in ('execute_patch','execute_check_after')):raise ValueError('passed patch and unchanged experiment required')
    expected=done['execute_patch']['status']
    if current['goal_completion']['snapshot']!=expected['goal_completion']['snapshot'] or current['latest_execution']['commitment']!=expected['latest_execution']['commitment'] or r.get('pending'):raise ValueError('verified source/action drift')
    if any(b[k] for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage')):raise ValueError('unknown parent resources')
    if config!=e.expected_configuration() or len(r['coding']['relational']['history'])!=2:raise ValueError('prefix configuration/history')
    return {'parent':str(root),'parent_admission':str(Path(parent_admission).resolve()),'admission_hash':e.sha(a),
        'files':{name:d.file_sha(root/name) for name in ('driver.jsonl','provider.jsonl','origin.json','STOP')},
        'project_files':tree(root/'project'),'state_root':r['state_root'],
        'snapshot':current['goal_completion']['snapshot'],'requirements_revision':current['goal_completion']['requirements_revision'],
        'commitment':current['latest_execution']['commitment'],'fixture_only':v.fixture}

def admission(parent,root):
    p=prefix(parent);now=datetime.datetime.now(datetime.timezone.utc)
    return {'version':'b91-continuation@v1','mode':'fixture' if p['fixture_only'] else 'live','trial':str(Path(root).resolve()),
        'created_at':now.isoformat(),'expires_at':(now+datetime.timedelta(hours=1)).isoformat(),'prefix':p,
        'model':'gpt-6-luna','calls':2,'execution_groups':0,'patch_bytes':0,'payload_bytes':32768,
        'input_tokens':65536,'completion_tokens':16384,'call_seconds':240,'implementation':implementation(),
        'binary_sha256':d.file_sha(d.ROOT/'target/debug/tsr')}

def validate(a,root,fixture,*,expired=False,inspect_prefix=True):
    fields={'version','mode','trial','created_at','expires_at','prefix','model','calls','execution_groups','patch_bytes','payload_bytes','input_tokens','completion_tokens','call_seconds','implementation','binary_sha256'}
    if set(a)!=fields or a['version']!='b91-continuation@v1' or a['mode']!=('fixture' if fixture else 'live'):raise ValueError('continuation admission shape/mode')
    fixed={'model':'gpt-6-luna','calls':2,'execution_groups':0,'patch_bytes':0,'payload_bytes':32768,'input_tokens':65536,'completion_tokens':16384,'call_seconds':240}
    if any(type(a[k])!=type(v) or a[k]!=v for k,v in fixed.items()):raise ValueError('continuation limits')
    if a['implementation']!=implementation() or a['binary_sha256']!=d.file_sha(d.ROOT/'target/debug/tsr'):raise ValueError('implementation drift')
    if Path(a['trial']).resolve()!=root or any(p.is_symlink() for p in (root,*root.parents)):raise ValueError('continuation path')
    if not fixture and (not root.is_relative_to(d.ROOT/'target') or root==d.ROOT/'target'):raise ValueError('disposable target required')
    start,end=d.stamp(a['created_at']),d.stamp(a['expires_at'])
    if not start<end<=start+3600 or (not expired and not start<=time.time()<end):raise ValueError('expired continuation')
    p=a['prefix'];parent=Path(p['parent'])
    if root==parent or root.is_relative_to(parent) or parent.is_relative_to(root):raise ValueError('separate continuation required')
    if inspect_prefix:
        if prefix(p['parent_admission'])!=p:raise ValueError('parent prefix drift')
    elif (e.sha(e.strict(e.bounded_file(p['parent_admission'],16384)))!=p['admission_hash']
          or any(d.file_sha(parent/name)!=digest for name,digest in p['files'].items())
          or tree(parent/'project')!=p['project_files']):
        raise ValueError('parent prefix drift')

class Continuation(d.Driver):
    def __init__(self,root,a,transport,reviewer,*,fixture=False,hook=None):
        self.root=Path(root).resolve();self.admission=a;self.fixture=fixture
        self.file=self.root/'driver.jsonl'
        terminal=False
        if self.file.exists():
            _,pending,done=self.read()
            terminal='terminal' in done and not pending
        validate(a,self.root,fixture,expired=terminal)
        self.transport=transport;self.reviewer=reviewer;self.hook=hook or (lambda *_:None);self.active=set()
        self.root.mkdir(parents=True,exist_ok=True);self.file=self.root/'driver.jsonl';self.identity=e.sha(a)
        if not self.file.exists():
            if any(self.root.iterdir()):raise ValueError('fresh continuation directory required')
            e.private_save(self.file,self.row([],{'kind':'admission','value':a,'fixture_only':fixture}))
        self.read()
    def validate_current_admission(self):validate(self.admission,self.root,self.fixture,inspect_prefix=False)
    def execute(self,*_):raise d.Blocked('zero execution authority')
    def commit_response(self,*_):raise d.Blocked('zero commitment authority')
    def journal(self):
        p=self.admission['prefix'];j=e.Journal(self.root/'provider.jsonl',self.root/'origin.json',e.sha(e.strict(e.bounded_file(self.root/'origin.json'))))
        # Explicit clone relocation, not a rewritten origin or historical journal.
        if j.origin['directory']!=str(Path(p['parent'])/'project'):raise ValueError('origin relocation mismatch')
        j.origin={**j.origin,'directory':str(self.root/'project')}
        historical=e.bounded_file(Path(p['parent'])/'provider.jsonl',8*1024*1024)
        if not e.bounded_file(j.path,8*1024*1024).startswith(historical):raise ValueError('provider prefix drift')
        return j
    def verify_clone(self):
        c=self.controller();r=c.request();s=c.cli('project','status');p=self.admission['prefix']
        if s['goal_completion']['snapshot']!=p['snapshot'] or s['goal_completion']['requirements_revision']!=p['requirements_revision'] or s['latest_execution']['commitment']!=p['commitment'] or r.get('pending'):raise ValueError('continuation source/commitment drift')
        if tree(self.root/'parent-shadow')!={k.removeprefix('shadow/'):v for k,v in p['project_files'].items() if k.startswith('shadow/')}:raise ValueError('archived resource provenance drift')
        if c.cli('project','config')!=e.expected_configuration():raise ValueError('configuration drift')
        return c
    def initialize(self):
        parent=Path(self.admission['prefix']['parent'])
        def clone():
            shutil.copytree(parent/'project',self.root/'project',symlinks=False)
            if tree(self.root/'project')!=self.admission['prefix']['project_files']:raise ValueError('copy drift')
            # Old accounting is retained byte-for-byte, outside the new episode's
            # native shadow namespace. Main history and CAS are never rewritten.
            (self.root/'project/shadow').rename(self.root/'parent-shadow')
            for name in ('origin.json','provider.jsonl'):shutil.copyfile(parent/name,self.root/name)
            c=self.controller()
            if c.request()['state_root']!=self.admission['prefix']['state_root']:raise ValueError('cloned native prefix drift')
            return {'parent':self.admission['prefix'],'relocation':str(c.directory),'old_resources_archived':True}
        self.op('clone_prefix',clone);c=self.verify_clone()
        self.op('open_continuation',lambda:c.json_cli(('project-lifecycle-open',),{
            'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':1,
            'reason':'Separately admitted proposal-only continuation; historical resources archived, no execution authority',
            'non_tool':{'model_calls':2,'model_tokens':81920,'wall_ms':max(1,int((d.stamp(self.admission['expires_at'])-time.time())*1000)),
                'patch_bytes':0,'external_cost_microusd':0}},'--admission'))
        self.op('unpriced_accounting',lambda:c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root']))
        # Rebinding is native and source/intent checked. Never republish authored goals.
        if 'generate_final_proposal_only' not in self.read()[2]:
            e.project(self.journal(),c,self.journal().status()['next_stage'],model='gpt-6-luna',payload_limit=32768)
        return c
    def resource_summary(self):
        b=self.controller().budget()
        return {'episodes':{'continuation':b},'exceeded':b['exceeded'] or b['model_calls']>2 or b['tokens_charged']>81920 or b['tool_ms_charged']!=0 or b['patch_bytes_charged']!=0,
            'historical_resources':'Unchanged parent trial and parent-shadow archive; not charged a second time'}
    def generate(self,c,stage):
        if stage not in e.STAGES[-2:]:raise ValueError('continuation stage')
        self.verify_clone()
        return super().generate(c,stage)
    def run(self):
        with os.fdopen(os.open(self.root/'run.lock',os.O_WRONLY|os.O_CREAT|os.O_NOFOLLOW,0o600),'a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
            _,pending,done=self.read()
            if 'terminal' in done:
                result=done['terminal']
                if pending:raise ValueError('terminal uncertainty')
                if (self.root/'terminal.json').exists():
                    if e.strict(e.bounded_file(self.root/'terminal.json'))!=result:raise ValueError('terminal drift')
                else:e.private_save(self.root/'terminal.json',result)
                self.verify_clone();return result
            c=self.initialize();self.relation(c,'relation_after_verification');final=self.generate(c,'final_proposal_only')
            self.review(c,'semantic_coverage',{'final_proposal':final,'status':c.cli('project','status'),
                'requirements':e.prepare.REQUIREMENTS,'limits':'Independent advisory review only; no acceptance adoption or new execution.'},'final_semantic')
            self.op('close_continuation',lambda:c.cli('project-lifecycle-close-advisory','--reason','Two-request continuation concluded; no new execution or training label'))
            self.verify_clone()
            result=self.op('terminal',lambda:{'status':'completed_reviewed_continuation','calls':self.journal().status()['calls_consumed_or_reserved']-4,
                'execution_groups':0,'fixture_only':self.fixture,'live_b91_passed':False,'resources':self.resource_summary(),
                'parent':self.admission['prefix'],'acceptance_adopted':False,'final_proposal_only':True})
            e.private_save(self.root/'terminal.json',result);return result

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('mode',choices=('prepare','inspect','run'));p.add_argument('--parent-admission',type=Path);p.add_argument('--trial',type=Path);p.add_argument('--admission',type=Path,required=True);p.add_argument('--admission-sha256');p.add_argument('--reviews',type=Path);p.add_argument('--allow-paid-inference',action='store_true');a=p.parse_args()
    if a.mode=='prepare':
        if not a.parent_admission or not a.trial:p.error('parent admission and separate trial required')
        value=admission(a.parent_admission,a.trial);e.private_save(a.admission,value);print(json.dumps({'admission_sha256':e.sha(value),'dispatch_enabled':False}));return
    value=e.strict(e.bounded_file(a.admission,1024*1024));root=Path(value['trial']).resolve()
    if a.mode=='inspect':
        v=object.__new__(Continuation);v.root=root;v.file=root/'driver.jsonl';v.admission=value;v.fixture=value['mode']=='fixture'
        rows,pending,done=v.read();print(json.dumps({'dispatch_enabled':False,'events':len(rows),'pending':list(pending),'done':list(done)}));return
    if not a.allow_paid_inference or a.admission_sha256!=e.sha(value) or not a.reviews:p.error('exact admission, paid opt-in and independent reviews required')
    transport=e.module('continuation_transport','engine-synthetic-transport.py')
    run=Continuation(root,value,lambda body,timeout:transport.invoke(body,timeout,expected_model='gpt-6-luna',payload_limit=32768),d.FileReviewer(a.reviews))
    try:print(json.dumps(run.run(),indent=2))
    except (ValueError,d.Blocked) as error:
        if not str(error).startswith('independent review required:') and not (root/'STOP').exists():
            e.private_save(root/'STOP',{'reason':str(error),'unused_allowance_expired':True})
        print(json.dumps({'status':'blocked','reason':str(error),'no_retry':True}))
if __name__ == '__main__':
    import runpy, sys
    runpy.run_path(str(Path(__file__).with_name('engine-output-release.py')))['check_cli'](__file__,sys.argv[1:])

if __name__=='__main__':main()
