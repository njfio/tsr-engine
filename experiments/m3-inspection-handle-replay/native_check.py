"""Seven bounded native groups (eight arms), fake provider, no paid dispatch."""
from pathlib import Path
import hashlib,importlib.util,json,os,shutil,socket,sqlite3,subprocess,sys,time
ROOT=Path(__file__).resolve().parents[2]
HERE=ROOT/'experiments/m3-inspection-handle-replay'
BASE=ROOT/'target/m3-handle-replay-native'
TASK='b7b1fee58ff04e6456d81b023cc8754aac6602069743a7d6b5125d3fc1aeadf3'
CASES=['record','rules','multiple','truncation','stale','foreign','wrong_path','protected_test']
FIELDS={'protocol','id','task','state_root','state_version','remaining_tokens','remaining_action_calls','progress','inspection','deadline_ms','observation','allowed_test_ids','required_reads','failing_tests','failing_test_id'}
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def child(case):
    assert 'M3_TEST_PARENT_SENTINEL' not in os.environ
    def forbidden(*a,**kw):raise RuntimeError('offline network forbidden')
    sys.path.insert(0,str(ROOT/'scripts'))
    spec=importlib.util.spec_from_file_location('codec',HERE/'broker.py');codec=importlib.util.module_from_spec(spec);spec.loader.exec_module(codec)
    socket.socket=forbidden
    raw=sys.stdin.buffer.read(70001);assert len(raw)<=70000
    r=json.loads(raw);path='tests/answer.rs' if case=='protected_test' else 'src/lib.rs'
    def scope(request):
        assert set(request)==FIELDS and request['protocol']=='tsr-broker@inspection-handle-v1'
        assert request['task']==TASK and request['required_reads']==[path]
        assert request['allowed_test_ids']==['named-failing-test'] and request['failing_test_id']=='named-failing-test'
        assert request['failing_tests']==['answer_is_42']
        assert 1<=request['remaining_action_calls']<=20
        return path
    codec.allowed_path=scope;codec.transport.allowed_path=scope;codec.encode(r)
    n=20-r['remaining_action_calls'];directory=BASE/case
    value={'action':'stop'}
    if n==0:value=dict(action='inspect',path=path)
    elif n in (1,3):value=dict(action='test',test_id='named-failing-test',forecast=.9)
    elif n==2:
        handle=r['inspection']['handle'];assert r['inspection']['path']==path
        (directory/'issued-handle.txt').write_text(handle)
        if case=='foreign':handle=(BASE/'record/issued-handle.txt').read_text()
        value=dict(action='edit',path=path,inspection_handle=handle,old_text='{ 0 }',new_text='{ 42 }',forecast=.9)
        if case=='protected_test':value.update(old_text='(), 42)',new_text='(), 0)')
    if case=='stale' and n==3:
        assert r['inspection'] is None
        value=dict(action='edit',path=path,inspection_handle=(directory/'issued-handle.txt').read_text(),old_text='{ 42 }',new_text='{ 43 }',forecast=.9)
    response=dict(choices=[dict(finish_reason='tool_calls',message=dict(tool_calls=[dict(type='function',function=dict(name='propose',arguments=json.dumps(value)))]))],usage=dict(total_tokens=1))
    if case=='multiple':response['choices'][0]['message']['tool_calls']*=2
    if case=='truncation':response['choices'][0]['finish_reason']='length'
    try:
        result=codec.decode(response,r)
        if case=='wrong_path' and n==2:
            # Explicit native-boundary fixture: bypass codec path allowlist to challenge native binding.
            result['reply']['proposal']['path']='src/other.rs'
        with (directory/'boundary.jsonl').open('a') as f:f.write(json.dumps(dict(call=n,action=value['action'],inspection=r['inspection'],source_state=r['progress']['source_state'],proposal=result['reply']['proposal'],environment_cleared=True))+'\n')
        print(json.dumps(result))
    except codec.ValidationFailure as e:
        (directory/'codec-rejection.json').write_text(json.dumps(e.detail));raise

def run():
    assert shutil.disk_usage(ROOT).free>=200*2**30
    BASE.mkdir()
    ledger=BASE/'offline-ledger.jsonl';ledger.write_bytes(b'')
    original_ledgers={str(p):digest(p) for p in (ROOT/'target').glob('m3-provider-cost*.jsonl')}
    frozen=json.loads((ROOT/'docs/m3-fixed12-freeze.json').read_text())['sha256']
    assert all(digest(ROOT/p)==h for p,h in frozen.items())
    template=ROOT/'target/m3-two-model-v2-input-0'
    image=json.loads((ROOT/'docs/m3-two-model-v2-freeze.json').read_text())['sandbox_image']
    running_before=subprocess.check_output(['docker','ps','-q'],text=True).split()
    summary=[]
    for case in CASES:
        assert shutil.disk_usage(ROOT).free>=200*2**30
        directory=BASE/case;directory.mkdir();inputs=directory/'input';inputs.mkdir()
        for name in ('tasks.jsonl','profile.toml','broker-contexts.json','preparation.json'):shutil.copyfile(template/name,inputs/name)
        shutil.copytree(template/'cas',inputs/'cas')
        profile=(inputs/'profile.toml').read_text().replace('wall_clock_secs = 1800','wall_clock_secs = 120')
        if case=='protected_test':
            profile=profile.replace('paths = ["src/lib.rs"]','paths = ["tests/answer.rs"]')
            (inputs/'broker-contexts.json').write_text(json.dumps({TASK:dict(required_reads=['tests/answer.rs'],failing_test_id='named-failing-test')}))
        (inputs/'profile.toml').write_text(profile)
        wrapper=directory/'broker.py';wrapper.write_text('import runpy\nm=runpy.run_path('+repr(str(Path(__file__).resolve()))+')\nm["child"]('+repr(case)+')\n')
        mode='rules' if case=='rules' else 'record';out=directory/'out'
        env=os.environ|dict(M3_TEST_PARENT_SENTINEL='must-not-cross',TSR_M3_RUNTIME='docker',TSR_M3_USER='65534:65534',TSR_M3_IMAGE=image,TSR_M3_PREPARATION=str(inputs/'preparation.json'),TSR_M3_APPROVED_LIVE='100',TSR_M3_BROKER_SCRIPT=str(wrapper),TSR_M3_PYTHON=sys.executable,TSR_M3_COST_LEDGER=str(ledger),TSR_DB=str(directory/'startup.sqlite'))
        cmd=[str(ROOT/'target/m3-handle-replay-build/debug/tsr'),'task','run','--mode',mode,'--tasks',str(inputs/'tasks.jsonl'),'--driver','cheaper-inference','--profile',str(inputs/'profile.toml'),'--out',str(out)]
        with (directory/'native.log').open('w') as log:
            proc=subprocess.Popen(cmd,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True);start=time.monotonic()
            while proc.poll() is None:
                if time.monotonic()-start>180 or shutil.disk_usage(ROOT).free<200*2**30:
                    os.killpg(proc.pid,15);proc.wait(timeout=15);raise RuntimeError('native deadline/disk guard')
                time.sleep(1)
        result_path=out/f'results-{mode}.jsonl';result=json.loads(result_path.read_text()) if result_path.exists() else None
        source=out/'copies'/TASK/mode/'src/lib.rs'
        if case in ('record','rules'):
            assert proc.returncode==0 and result['outcome']=='accepted' and result['oracle']['evidence']['verified_success']
        elif case in ('multiple','truncation'):
            assert proc.returncode==0 and result['outcome']=='unavailable' and result['oracle'] is None
            assert (directory/'codec-rejection.json').exists()
        elif case in ('stale','foreign','wrong_path'):
            assert proc.returncode==2 and result is None
            assert 'inspection handle' in (directory/'native.log').read_text()
            assert ('{ 42 }' if case=='stale' else '{ 0 }') in source.read_text()
        else:
            assert proc.returncode==0 and result['outcome']=='rejected'
        stores=list((out/'state').glob('*/*.sqlite'));assert len(stores)==1
        with sqlite3.connect(stores[0].as_uri()+'?mode=ro',uri=True) as db:
            assert db.execute('pragma integrity_check').fetchone()[0]=='ok'
            events=[json.loads(row[0]) for row in db.execute('select payload from events order by seq')]
        (directory/'events.json').write_text(json.dumps(events))
        row=dict(case=case,mode=mode,exit=proc.returncode,outcome=None if result is None else result['outcome'],events=len(events),source_sha256=digest(source))
        summary.append(row);(BASE/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(row),flush=True)
        assert subprocess.check_output(['docker','ps','-q'],text=True).split()==running_before,'unexpected running containers'
    assert ledger.read_bytes()==b''
    assert all(digest(Path(p))==h for p,h in original_ledgers.items())
    assert all(digest(ROOT/p)==h for p,h in frozen.items())
    print('Eight arms across seven groups passed; zero provider dispatch or cost-ledger mutation.',flush=True)
if __name__=='__main__':run()
