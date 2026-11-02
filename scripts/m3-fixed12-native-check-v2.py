"""Offline actual native boundary: all real inputs, fake HTTP/billing, no credentials/network."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import runpy
import socket
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'target/m3-fixed12-native-check-v2'
def child(index,blocked=False):
    if 'M3_PARENT_SENTINEL' in os.environ:raise ValueError('native environment leaked')
    sys.path.insert(0,str(ROOT/'scripts'))
    import m3_fixed12_core as core
    core.MANIFEST=BASE/'offline-manifest.json';core.budget.BASELINE=BASE/'baseline.json'
    m=json.loads(core.MANIFEST.read_text());arm=m['arms'][index];current={}
    def forbidden(*a,**kw):raise AssertionError('real network forbidden')
    socket.socket=forbidden;core.edit.transport.credential=lambda:'OFFLINE-NOT-A-CREDENTIAL'
    encode=core.edit.encode
    def capture(request):
        current.clear();current.update(request);body=encode(request)
        assert body['model']=='deepseek-v4.1-flash'
        assert request['failing_tests']==arm['failing_tests']
        if request['observation'].startswith('context='):
            assert arm['file'] in request['observation']
        # Validate the real task's test/edit/stop schema examples without executing them.
        for value in core.edit.examples(request):core.edit.decode(response(value),request)
        return body
    core.edit.encode=capture;core.edit.transport.encode=capture
    def response(value):return dict(choices=[dict(finish_reason='tool_calls',message=dict(tool_calls=[dict(type='function',function=dict(name='propose',arguments=json.dumps(value)))]))],usage=dict(prompt_tokens=1,completion_tokens=1,total_tokens=2),billing=dict(status='settled',currency='USD',billed_cost_usd='0'),request_id='offline')
    def http(route,key,body=None,**kw):
        if route=='models':return dict(data=[dict(id='deepseek-v4.1-flash',pricing=dict(list_input_per_million='.15',list_output_per_million='.60'))])
        assert route=='chat/completions' and body['model']=='deepseek-v4.1-flash'
        if blocked:value=dict(action='test',test_id='named-failing-test',forecast=.5)
        elif current['remaining_action_calls']==20:value=dict(action='inspect',path=arm['file'])
        else:value=dict(action='test',test_id='named-failing-test',forecast='deliberately-invalid-offline')
        with (BASE/'boundary.jsonl').open('a') as f:f.write(json.dumps(dict(index=index,blocked=blocked,model=body['model'],failing_tests=len(current['failing_tests']),wire_bytes=len(json.dumps(body).encode()),action=value['action'],environment_cleared=True))+'\n')
        return response(value)
    core.edit.transport.request_json=http
    runpy.run_path(str(ROOT/'scripts/m3-fixed12-broker.py'),run_name='__main__')
def main():
    BASE.mkdir();before=(ROOT/'target/m3-provider-cost.jsonl').read_bytes()
    m=json.loads((ROOT/'docs/m3-fixed12-freeze.json').read_text());offline=m|dict(ledger=str((BASE/'ledger.jsonl').relative_to(ROOT)))
    (BASE/'ledger.jsonl').write_bytes(b'');(BASE/'baseline.json').write_text(json.dumps(dict(bytes=0,sha256=hashlib.sha256(b'').hexdigest(),settled_usd='0')));(BASE/'offline-manifest.json').write_text(json.dumps(offline))
    spec=importlib.util.spec_from_file_location('c',ROOT/'scripts/m3-fixed12-v4-controller.py');c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c);ops=c.Operations(offline);rows=[]
    jobs=[(a,True) for a in m['arms'] if a['mode']=='rules' and a['index']!=1]
    for arm,blocked in jobs:
        index=arm['index'];label=str(index)+('-blocked' if blocked else '');state=BASE/label;state.mkdir();wrapper=state/'offline.py'
        wrapper.write_text('import importlib.util\ns=importlib.util.spec_from_file_location("offline",'+repr(str(Path(__file__).resolve()))+')\nm=importlib.util.module_from_spec(s);s.loader.exec_module(m)\nm.child('+str(index)+','+str(blocked)+')\n')
        inputs=ROOT/arm['input'];base=state/'input';base.mkdir()
        for name in ('tasks.jsonl','preparation.json','profile.toml','broker-contexts.json'):(base/name).write_bytes((inputs/name).read_bytes())
        import shutil
        shutil.copytree(inputs/'cas',base/'cas');testarm=arm|dict(input=str(base.relative_to(ROOT)))
        env=os.environ|dict(M3_PARENT_SENTINEL='must-not-cross',TSR_M3_RUNTIME='docker',TSR_M3_USER='65534:65534',TSR_M3_IMAGE=arm['sandbox_image'],TSR_M3_PREPARATION=str(base/'preparation.json'),TSR_M3_APPROVED_LIVE='100',TSR_M3_BROKER_SCRIPT=str(wrapper),TSR_M3_PYTHON=sys.executable,TSR_M3_COST_LEDGER=str(BASE/'ledger.jsonl'),TSR_DB=str(state/'startup.sqlite'))
        ops.attempt_state=state;ops.diagnostic_before=ops.diagnostic_path.read_bytes() if ops.diagnostic_path.exists() else b'';ledger_before=ops.ledger()
        labelnative=arm['task'][:12]+'-'+arm['mode']
        with (state/(labelnative+'.log')).open('x') as log:
            process=subprocess.run([str(ROOT/m['engine']),'task','run','--mode',arm['mode'],'--tasks',str(base/'tasks.jsonl'),'--driver','cheaper-inference','--profile',str(base/'profile.toml'),'--out',str(base/arm['mode'])],env=env,stdout=log,stderr=subprocess.STDOUT,timeout=180)
        assert process.returncode==0
        checked=ops.export_and_verify(testarm,dict(status='completed',exit=0,offline=True),state/'evidence');ids=c.bind_accounting(ledger_before,ops.ledger(),checked['diagnostics']);outcome=c.classify(checked,{})
        assert outcome==('rejected' if blocked else 'schema-or-response-failure')
        if not blocked:
            source=base/arm['mode']/'copies'/arm['task']/arm['mode']/arm['file'];assert hashlib.sha256(source.read_bytes()).hexdigest()==arm['mutated_source_sha256']
            assert checked['proposal_kinds']==['inspect'] and len(ids)==2
        else:assert checked['proposal_kinds']==[] and checked['stop_reasons']==['blocked'] and len(ids)==1
        rows.append(dict(index=index,mode=arm['mode'],blocked=blocked,outcome=outcome,calls=len(ids),evidence_sha256=checked['manifest_sha256']))
        print(json.dumps(rows[-1]),flush=True)
    assert (ROOT/'target/m3-provider-cost.jsonl').read_bytes()==before
    (BASE/'summary.json').write_text(json.dumps(dict(checks=rows,provider_calls=0,shared_ledger_unchanged=True),indent=2)+'\n')
if __name__=='__main__':main()
