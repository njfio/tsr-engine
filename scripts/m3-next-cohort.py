"""Finite local-only seven-function AgentID cohort after the preceding queue."""
from concurrent.futures import ThreadPoolExecutor
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import time

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('queue',ROOT/'scripts/m3-detached-qualification.py')
queue=importlib.util.module_from_spec(spec);spec.loader.exec_module(queue)
queue.STATE=ROOT/'target/m3-next-cohort-state';queue.STATE.mkdir()
status=queue.STATE/'waiting.json';deadline=time.monotonic()+7200
while not (ROOT/'target/m3-detached-state/complete.json').exists():
    queue.save(status,dict(status='waiting-for-native-queue',heartbeat=queue.stamp()))
    if time.monotonic()>deadline:
        queue.save(status,dict(status='timeout-waiting',finished=queue.stamp()));raise SystemExit(1)
    time.sleep(10)
names=subprocess.run(['docker','ps','--format','{{.Names}}'],capture_output=True,text=True,timeout=15,check=True).stdout.splitlines()
if any(n.startswith('tsr-m3-') for n in names):
    queue.save(status,dict(status='blocked',reason='M3 container remains after native completion'));raise SystemExit(1)
plan=json.loads((ROOT/'docs/m3-next-cohort-plan.json').read_text())
screen=ROOT/'target/m3-agentid-next-screen'
argv=[queue.ENGINE,'corpus','mutants',str(ROOT/'target/m3-second-source/repo'),'--package','agentid-core',
      '--screen-target','lib','--mutant','('+'|'.join(plan['mutation_filters'])+')','--out',str(screen)]
queue.save(status,dict(status='screening',started=queue.stamp()))
result=queue.guarded('screen',queue.run,'screen',argv,queue.env_for(True))
manifest=screen/'tasks.jsonl'
if result.get('exit')!=0 or not manifest.exists():
    queue.save(queue.STATE/'complete.json',dict(status='screen-failed',result=result));raise SystemExit(1)
tasks=list(map(json.loads,manifest.read_text().splitlines()))
if not 0<len(tasks)<=plan['screen_max_candidates']:
    queue.save(queue.STATE/'complete.json',dict(status='screen-count-refused',count=len(tasks)));raise SystemExit(1)
inputs=ROOT/'target/m3-next-cohort-input';inputs.mkdir()
for task in tasks:
    directory=inputs/task['id'];directory.mkdir()
    (directory/'tasks.jsonl').write_text(json.dumps(task)+'\n')
    shutil.copytree(screen/'cas',directory/'cas')

def qualify(task):
    return queue.run(task['id'],[queue.ENGINE,'corpus','qualify','--tasks',str(inputs/task['id']/'tasks.jsonl'),
        '--out',str(ROOT/'target'/('m3-qualified-agentid-next-'+task['id'][:12]))],queue.env_for(True))

queue.save(status,dict(status='qualifying',candidates=len(tasks),started=queue.stamp()))
with ThreadPoolExecutor(max_workers=2) as pool:
    futures=[pool.submit(queue.guarded,task['id'],qualify,task) for task in tasks]
    results=[future.result() for future in futures]
queue.save(queue.STATE/'complete.json',dict(status='completed',finished=queue.stamp(),results=results))
