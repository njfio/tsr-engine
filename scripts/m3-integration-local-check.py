"""Inert scripted proposal driver, solely for local sealed-pipeline preflight."""
import importlib.util
import json
import re
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
if '--approved-live' in sys.argv:
    request=json.load(sys.stdin)
    spec=importlib.util.spec_from_file_location('adapter',ROOT/'scripts/m3-integration-broker.py');adapter=importlib.util.module_from_spec(spec);spec.loader.exec_module(adapter)
    number=20-request['remaining_action_calls']
    value=dict(action='stop',path='',id='',expected_digest='',old_text='',new_text='',forecast=None)
    if number==0:value.update(action='inspect',path='src/lib.rs')
    elif number in (1,3):value.update(action='test',id='named-failing-test',forecast=0.9)
    elif number==2:
        source=b'/// Return the integer 42. This is an isolated synthetic integration task.\npub fn answer() -> u32 { 0 }\n'
        digest=re.search(r'path=src/lib.rs digest=([0-9a-f]{64})',request['observation']).group(1)
        value.update(action='edit',path='src/lib.rs',expected_digest=digest,old_text='{ 0 }',new_text='{ 42 }',forecast=0.9)
    response=dict(choices=[dict(finish_reason='tool_calls',message=dict(tool_calls=[dict(type='function',function=dict(name='propose',arguments=json.dumps(value)))]))],usage=dict(total_tokens=1))
    print(json.dumps(adapter.edit.decode(response,request)))
else:
    import os
    base=ROOT/'target/m3-integration-input'
    out=ROOT/'target/m3-integration-local-check-v2'
    if out.exists():raise ValueError('local preflight state exists')
    env=os.environ|dict(TSR_M3_RUNTIME='docker',TSR_M3_USER='65534:65534',TSR_M3_IMAGE='sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6',TSR_M3_PREPARATION=str(base/'preparation.json'),TSR_M3_APPROVED_LIVE='100',TSR_M3_BROKER_SCRIPT=str(Path(__file__).resolve()),TSR_M3_PYTHON=sys.executable,TSR_M3_COST_LEDGER=str(out/'unused-ledger.jsonl'),TSR_DB=str(out/'startup.sqlite'))
    subprocess.run([str(ROOT/'target/m3-exact-build/debug/tsr'),'task','run','--mode','rules','--tasks',str(base/'tasks.jsonl'),'--driver','cheaper-inference','--profile',str(base/'profile.toml'),'--out',str(out)],env=env,check=True,timeout=1800)
    print((out/'results-rules.jsonl').read_text())
