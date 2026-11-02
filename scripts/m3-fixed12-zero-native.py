"""Bounded native zero-response budget regression; broker invocation is forbidden."""
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[1]
def main():
    state=ROOT/'target/m3-fixed12-zero-native';state.mkdir();base=state/'input';base.mkdir()
    m=json.loads((ROOT/'docs/m3-fixed12-freeze.json').read_text());before=(ROOT/m['ledger']).read_bytes();arm=m['arms'][0];inputs=ROOT/arm['input']
    for name in ('tasks.jsonl','preparation.json','profile.toml','broker-contexts.json'):shutil.copyfile(inputs/name,base/name)
    shutil.copytree(inputs/'cas',base/'cas');p=base/'profile.toml';s=p.read_text();assert s.count('generator_calls = 20')==1;p.write_text(s.replace('generator_calls = 20','generator_calls = 0'))
    wrapper=state/'forbidden.py';wrapper.write_text("raise AssertionError('zero-call budget invoked broker')\n")
    ledger=state/'ledger.jsonl';ledger.write_bytes(b'')
    env=os.environ|dict(TSR_M3_RUNTIME='docker',TSR_M3_USER='65534:65534',TSR_M3_IMAGE=arm['sandbox_image'],TSR_M3_PREPARATION=str(base/'preparation.json'),TSR_M3_APPROVED_LIVE='100',TSR_M3_BROKER_SCRIPT=str(wrapper),TSR_M3_PYTHON=sys.executable,TSR_M3_COST_LEDGER=str(ledger),TSR_DB=str(state/'startup.sqlite'))
    label=arm['task'][:12]+'-'+arm['mode']
    with (state/(label+'.log')).open('x') as log:r=subprocess.run([str(ROOT/m['engine']),'task','run','--mode',arm['mode'],'--tasks',str(base/'tasks.jsonl'),'--driver','cheaper-inference','--profile',str(base/'profile.toml'),'--out',str(base/arm['mode'])],env=env,stdout=log,stderr=subprocess.STDOUT,timeout=180)
    assert r.returncode==0
    spec=importlib.util.spec_from_file_location('c',ROOT/'scripts/m3-fixed12-v6-controller.py');c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)
    ops=c.Operations(m|dict(ledger=str(ledger.relative_to(ROOT))));ops.attempt_state=state;ops.diagnostic_before=b''
    checked=ops.export_and_verify(arm|dict(input=str(base.relative_to(ROOT))),dict(status='completed',exit=0,offline=True),state/'evidence')
    assert checked['diagnostics']==[] and checked['proposed_count']==0 and checked['stop_reasons']==['budget:generator_calls']
    assert c.bind_accounting(b'',ledger.read_bytes(),[])==[] and c.classify(checked,{})=='rejected'
    assert (ROOT/m['ledger']).read_bytes()==before
    (state/'summary.json').write_text(json.dumps(dict(status='passed',provider_calls=0,diagnostics=0,proposed=0,closure=checked['closure_reasons'],evidence_sha256=checked['manifest_sha256']),indent=2)+'\n');print((state/'summary.json').read_text())
if __name__=='__main__':main()
