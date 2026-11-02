"""Finite approved paired pilot after the native queue releases its two workers."""
from concurrent.futures import ThreadPoolExecutor
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('queue', ROOT / 'scripts/m3-detached-qualification.py')
queue = importlib.util.module_from_spec(spec)
spec.loader.exec_module(queue)
queue.STATE = ROOT / 'target/m3-live-pilot-state'
queue.STATE.mkdir()
status = queue.STATE / 'waiting.json'
deadline = time.monotonic() + 7200
while not (ROOT / 'target/m3-detached-state/complete.json').exists():
    queue.save(status, dict(status='waiting-for-native-queue', heartbeat=queue.stamp()))
    if time.monotonic() > deadline:
        queue.save(status, dict(status='timeout-waiting', finished=queue.stamp()))
        raise SystemExit(1)
    time.sleep(10)
# Completion plus no sealed containers prevents overlapping an orphaned worker.
names = subprocess.run(['docker','ps','--format','{{.Names}}'], capture_output=True,
                       text=True, timeout=15, check=True).stdout.splitlines()
if any(name.startswith('tsr-m3-') for name in names):
    queue.save(status, dict(status='blocked', reason='M3 container remains after native queue completion'))
    raise SystemExit(1)
queue.save(status, dict(status='running-paired-pilot', started=queue.stamp()))

def paired(name):
    env = queue.env_for(name == 'agentid')
    env.update(TSR_M3_APPROVED_LIVE='100', TSR_M3_BROKER_SCRIPT=str(ROOT/'scripts/m3-cheaper-broker.py'),
               TSR_M3_PYTHON=sys.executable, TSR_M3_COST_LEDGER=str(ROOT/'target/m3-provider-cost.jsonl'))
    base = ROOT/'target/m3-live-pilot'/name
    results = []
    for mode in ('record','rules'):
        results.append(queue.run(name+'-'+mode,[queue.ENGINE,'task','run','--mode',mode,
            '--tasks',str(base/'tasks.jsonl'),'--driver','cheaper-inference','--profile',
            str(base/'profile.toml'),'--out',str(base/mode)],env))
    return results

with ThreadPoolExecutor(max_workers=2) as pool:
    futures = [pool.submit(queue.guarded,name,paired,name) for name in ('agentid','rts')]
    results = [future.result() for future in futures]
queue.save(queue.STATE/'complete.json',dict(finished=queue.stamp(),results=results))
