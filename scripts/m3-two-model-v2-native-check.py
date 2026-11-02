"""Offline native ProcessBroker test: real entrypoints, fake provider, no credentials/network."""
from pathlib import Path
import json
import os
import re
import runpy
import socket
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'target/m3-two-model-v2-native-check'

def child(index):
    # Native ProcessBroker clears env. Assert it, including an intentional parent sentinel.
    if any(k in os.environ for k in ('TSR_M3_EXPERIMENT_ATTEMPT','M3_TEST_PARENT_SENTINEL')):raise ValueError('environment leaked')
    sys.path.insert(0,str(ROOT/'scripts'))
    import m3_two_model_v2_core as core
    def forbidden(*a,**kw):raise AssertionError('offline: credentials/network forbidden')
    socket.socket=forbidden
    core.edit.transport.credential=forbidden
    core.edit.transport.request_json=forbidden
    def complete(request,ledger):
        wire=core.edit.encode(request)
        expected=['deepseek-v4.1-flash','glm-5.3-flash'][index%2]
        if wire['model']!=expected:raise ValueError('wrong model binding')
        number=20-request['remaining_action_calls']
        value=dict(action='stop')
        if number==0:value=dict(action='inspect',path='src/lib.rs')
        elif number in (1,3):value=dict(action='test',test_id='named-failing-test',forecast=.9)
        elif number==2:
            digest=re.search(r'path=src/lib.rs digest=([0-9a-f]{64})',request['observation']).group(1)
            value=dict(action='edit',path='src/lib.rs',expected_digest=digest,old_text='{ 0 }',new_text='{ 42 }',forecast=.9)
        response=dict(choices=[dict(finish_reason='tool_calls',message=dict(tool_calls=[dict(type='function',function=dict(name='propose',arguments=json.dumps(value)))]))],usage=dict(total_tokens=1))
        result=core.edit.decode(response,request)
        with (BASE/f'arm-{index}-boundary.jsonl').open('a') as f:f.write(json.dumps(dict(index=index,model=wire['model'],environment_cleared=True,action=value['action']))+'\n')
        return result
    core.edit.transport.complete_with_diagnostics=complete
    # Execute the exact production entrypoint, reusing only the offline-patched core module.
    runpy.run_path(str(ROOT/f'scripts/m3-two-model-v2-arm-{index}.py'),run_name='__main__')

def main():
    if '--approved-live' in sys.argv:
        # Dedicated wrapper filenames are trusted, controller-owned test configuration.
        index=int(Path(sys.argv[0]).stem.rsplit('-',1)[1]);child(index);return
    BASE.mkdir()
    before=(ROOT/'target/m3-provider-cost.jsonl').read_bytes()
    manifest=json.loads((ROOT/'docs/m3-two-model-v2-freeze.json').read_text())
    for index,arm in enumerate(manifest['arms']):
        wrapper=BASE/f'wrapper-{index}.py'
        wrapper.write_text("import runpy\nrunpy.run_path("+repr(str(Path(__file__).resolve()))+",run_name='__main__')\n")
        # runpy changes argv[0]; embed index in a copied test module pathname instead.
        wrapper.write_text(Path(__file__).read_text().replace("index=int(Path(sys.argv[0]).stem.rsplit('-',1)[1]);child(index)",f"index={index};child(index)").replace("ROOT=Path(__file__).resolve().parents[1]",'ROOT=Path('+repr(str(ROOT))+')'))
        base=ROOT/arm['input'];out=BASE/f'out-{index}'
        env=os.environ|dict(M3_TEST_PARENT_SENTINEL='must-not-cross',TSR_M3_EXPERIMENT_ATTEMPT='wrong',TSR_M3_RUNTIME='docker',TSR_M3_USER='65534:65534',TSR_M3_IMAGE=manifest['sandbox_image'],TSR_M3_PREPARATION=str(base/'preparation.json'),TSR_M3_APPROVED_LIVE='100',TSR_M3_BROKER_SCRIPT=str(wrapper),TSR_M3_PYTHON=sys.executable,TSR_M3_COST_LEDGER=str(ROOT/manifest['ledger']),TSR_DB=str(BASE/f'startup-{index}.sqlite'))
        subprocess.run([str(ROOT/manifest['engine']),'task','run','--mode','record','--tasks',str(base/'tasks.jsonl'),'--driver','cheaper-inference','--profile',str(base/'profile.toml'),'--out',str(out)],env=env,check=True,timeout=1800)
        result=json.loads((out/'results-record.jsonl').read_text());boundary=[json.loads(l) for l in (BASE/f'arm-{index}-boundary.jsonl').read_text().splitlines()]
        assert result['outcome']=='accepted' and result['oracle']['evidence']['verified_success']
        assert [r['action'] for r in boundary]==['inspect','test','edit','test','stop']
        assert all(r['model']==arm['model'] and r['environment_cleared'] for r in boundary)
        print(json.dumps(dict(index=index,model=arm['model'],native_result=result['outcome'],boundary_calls=len(boundary))),flush=True)
    assert (ROOT/'target/m3-provider-cost.jsonl').read_bytes()==before
    print('All six exact entrypoints passed the native cleared-environment boundary; zero provider calls.')
if __name__=='__main__':main()
