"""Actual NativeProcessBroker + production entrypoints; local fake transport only."""
from pathlib import Path
import hashlib,importlib.util,json,os,runpy,shutil,socket,subprocess,sys,time
ROOT=Path(__file__).resolve().parents[2];HERE=Path(__file__).resolve().parent
BASE=ROOT/'target/m3-handle-live-offline-native-v3'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def child(index):
    assert 'M3_TEST_PARENT_SENTINEL' not in os.environ
    sys.path.insert(0,str(HERE));import live_core as c
    def forbidden(*a,**kw):raise AssertionError('network forbidden in offline test')
    socket.socket=forbidden
    c.MANIFEST=BASE/f'freeze-{index}.json'
    c.T.credential=lambda:'offline-placeholder'
    original_fail=c.fail
    def fail(code):
        (BASE/str(index)/'preflight-error.json').write_text(json.dumps(dict(code=code,environment_keys=sorted(os.environ))))
        original_fail(code)
    c.fail=fail
    def request(route,key,data=None,timeout=30):
        assert key=='offline-placeholder'
        if route=='models':return {'data':[{'id':'deepseek-v4.1-flash','pricing':{'list_input_per_million':'0.30','list_output_per_million':'1.20'}}]}
        assert route=='chat/completions'
        r=json.loads(data['messages'][1]['content']);n=20-r['remaining_action_calls']
        # Exact serialized bodies are local evidence, never provider-dispatched.
        (BASE/str(index)/f'body-{n}.json').write_text(json.dumps(data,indent=2)+'\n')
        if n==0:a=dict(action='inspect',path=c.FILE)
        elif n==1:a=dict(action='test',test_id='named-failing-test',forecast=.5)
        else:a=dict(action='stop')
        return dict(choices=[dict(finish_reason='tool_calls',message=dict(tool_calls=[dict(type='function',function=dict(name='propose',arguments=json.dumps(a)))]))],usage=dict(prompt_tokens=5,completion_tokens=5,total_tokens=10),billing=dict(status='settled',currency='USD',billed_cost_usd='0.000001'),request_id='offline-'+str(index)+'-'+str(n))
    c.T.request_json=request
    runpy.run_path(str(HERE/f'arm-{index}.py'),run_name='__main__')
def main():
    assert not BASE.exists(),'one-shot offline fixture'
    assert shutil.disk_usage(ROOT).free>=240*2**30
    BASE.mkdir();freeze=json.loads((ROOT/'docs/m3-handle-live-execution-freeze.json').read_text())
    ledger=ROOT/freeze['ledger'];before=sha(ledger)
    running=subprocess.check_output(['docker','ps','-q'],text=True).split()
    rows=[]
    for index in (0,1):
        directory=BASE/str(index);directory.mkdir();base=directory/'input'
        shutil.copytree(ROOT/freeze['arms'][index]['input'],base)
        m=json.loads(json.dumps(freeze));m['ledger']=str((directory/'fake-ledger.jsonl').relative_to(ROOT));(ROOT/m['ledger']).write_bytes(ledger.read_bytes())
        m['state']=str((directory/'state').relative_to(ROOT));state=ROOT/m['state'];state.mkdir()
        m['arms'][index]['input']=str(base.relative_to(ROOT));manifest=BASE/f'freeze-{index}.json';manifest.write_text(json.dumps(m))
        (state/'current.json').write_text(json.dumps(dict(index=index,deadline=time.time()+1800,global_deadline=time.time()+3600,manifest_sha256=sha(manifest))))
        wrapper=directory/'offline-entry.py';wrapper.write_text('import sys\nsys.stderr=open('+repr(str(directory/'broker-stderr.log'))+',"a")\nimport runpy\nm=runpy.run_path('+repr(str(Path(__file__).resolve()))+')\nm["child"]('+str(index)+')\n')
        arm=m['arms'][index]
        env=os.environ|dict(M3_TEST_PARENT_SENTINEL='must-not-cross',TSR_M3_RUNTIME='docker',TSR_M3_USER='65534:65534',TSR_M3_IMAGE=arm['sandbox_image'],TSR_M3_PREPARATION=str(base/'preparation.json'),TSR_M3_APPROVED_LIVE='100',TSR_M3_BROKER_SCRIPT=str(wrapper),TSR_M3_PYTHON=sys.executable,TSR_M3_COST_LEDGER=str(ROOT/m['ledger']),TSR_DB=str(directory/'startup.sqlite'))
        cmd=[str(ROOT/arm['engine']),'task','run','--mode','record','--tasks',str(base/'tasks.jsonl'),'--driver','cheaper-inference','--profile',str(base/'profile.toml'),'--out',str(base/'record')]
        with (directory/'native.log').open('w') as log:
            p=subprocess.Popen(cmd,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True);start=time.monotonic()
            while p.poll() is None:
                if time.monotonic()-start>1800 or shutil.disk_usage(ROOT).free<200*2**30:
                    os.killpg(p.pid,15);p.wait(timeout=15);raise RuntimeError('offline native deadline/disk guard')
                time.sleep(1)
        result=json.loads((base/'record/results-record.jsonl').read_text())
        assert p.returncode==0 and result['outcome']=='rejected' and result['oracle'] is not None,(p.returncode,result)
        boundary=[json.loads(l) for l in (base/'boundary.jsonl').read_text().splitlines()]
        assert [r['action'] for r in boundary]==['inspect','test','stop'] and all(r['environment_cleared'] for r in boundary)
        diagnostics=[json.loads(l) for l in (directory/'fake-ledger-diagnostics.jsonl').read_text().splitlines()]
        assert len(diagnostics)==3 and all(r['code']=='ok' and r['settled'] for r in diagnostics)
        assert sha(ledger)==before
        assert subprocess.check_output(['docker','ps','-q'],text=True).split()==running
        row=dict(interface=arm['interface'],exit=p.returncode,outcome=result['outcome'],oracle_completed=True,actions=[r['action'] for r in boundary],fake_settlements=3,real_paid_calls=0)
        if index==1:
            sys.path.insert(0,str(HERE));import controller
            row['freshness']=controller.verify_handle_evidence(arm,{},directory/'freshness')
        rows.append(row);print(json.dumps(row),flush=True)
    (BASE/'summary.json').write_text(json.dumps(dict(checks=rows,real_ledger_unchanged=before,real_paid_calls=0),indent=2)+'\n')
if __name__=='__main__':main()
