"""Finite local-only AgentID policy campaign after the RTS metrics cohort."""
from concurrent.futures import ThreadPoolExecutor
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('queue', ROOT/'scripts/m3-detached-qualification.py')
queue = importlib.util.module_from_spec(spec)
spec.loader.exec_module(queue)
queue.STATE = ROOT/'target/m3-agentid-policy-cohort-state'
queue.STATE.mkdir()  # Exclusive launch ownership.
plan_bytes = (ROOT/'docs/m3-agentid-policy-cohort-plan.json').read_bytes()
plan = json.loads(plan_bytes)
# Preserve exact operator inputs before starting any source command.
configuration = {'recorded_at': queue.stamp(), 'sha256': {}}
for label, path in [('controller.py', Path(__file__)),
                    ('queue-helper.py', Path(queue.__file__)),
                    ('plan.json', ROOT/'docs/m3-agentid-policy-cohort-plan.json'),
                    ('engine', Path(queue.ENGINE))]:
    digest = hashlib.sha256()
    if label == 'engine':
        with path.open('rb') as stream:
            for block in iter(lambda: stream.read(1024 * 1024), b''):
                digest.update(block)
    else:
        data = plan_bytes if label == 'plan.json' else path.read_bytes()
        digest.update(data)
        (queue.STATE/label).write_bytes(data)
    configuration['sha256'][label] = digest.hexdigest()
queue.save(queue.STATE/'configuration.json', configuration)

assert plan['max_workers'] == 2 and plan['max_total_candidates'] <= 2
status = queue.STATE/'progress.json'
deadline = time.monotonic() + 7200
while not (ROOT/'target/m3-rts-metrics-cohort-state/complete.json').exists():
    queue.save(status, dict(status='waiting', heartbeat=queue.stamp()))
    if time.monotonic() > deadline:
        queue.save(status, dict(status='waiting-timeout', finished=queue.stamp()))
        raise SystemExit(1)
    time.sleep(10)
names = subprocess.run(['docker', 'ps', '--format', '{{.Names}}'], capture_output=True,
                       text=True, timeout=15, check=True).stdout.splitlines()
if any(name.startswith('tsr-m3-') for name in names):
    queue.save(status, dict(status='blocked', reason='Task container remains after predecessor'))
    raise SystemExit(1)
inputs = ROOT/'target/m3-agentid-policy-cohort-input'
inputs.mkdir()
tasks = []

def require_headroom(label):
    if shutil.disk_usage(ROOT).free < 100 * 1024**3:
        queue.HALT.set()
        queue.save(queue.STATE/('resource-halt-'+label+'.json'), dict(status='blocked',
            reason='Less than 100 GiB free; no further jobs started', observed=queue.stamp()))
        raise RuntimeError('Disk headroom below 100 GiB')

for group in plan['screen_groups']:
    label = group['label']
    require_headroom(label)
    out = ROOT/'target'/('m3-agentid-policy-'+label)
    argv = [queue.ENGINE, 'corpus', 'mutants', str(ROOT/'target/m3-second-source/repo'),
            '--package', group['package'], '--screen-target', group['screen_target'],
            '--mutant', '('+'|'.join(c['filter'] for c in group['candidates'])+')', '--out', str(out)]
    queue.save(status, dict(status='screening', group=label, started=queue.stamp()))
    result = queue.guarded(label, queue.run, label, argv, queue.env_for(True))
    if result.get('exit') != 0 or not (out/'tasks.jsonl').exists():
        queue.save(queue.STATE/'complete.json', dict(status='screen-failed', result=result))
        raise SystemExit(1)
    caught = [json.loads(line) for line in (out/'tasks.jsonl').read_text().splitlines()]
    if len(caught) > group['max_candidates']:
        queue.save(queue.STATE/'complete.json', dict(status='screen-count-refused', group=label))
        raise SystemExit(1)
    for task in caught:
        directory = inputs/task['id']
        directory.mkdir()  # Refuse duplicate IDs, never overwrite prior evidence.
        (directory/'tasks.jsonl').write_text(json.dumps(task)+'\n')
        shutil.copytree(out/'cas', directory/'cas')
    tasks.extend(caught)
if len(tasks) > plan['max_total_candidates']:
    raise RuntimeError('Total candidate bound exceeded')

def qualify(task):
    require_headroom(task['id'])
    return queue.run(task['id'], [queue.ENGINE, 'corpus', 'qualify', '--tasks',
        str(inputs/task['id']/'tasks.jsonl'), '--out',
        str(ROOT/'target'/('m3-qualified-agentid-policy-'+task['id'][:12]))], queue.env_for(True))

queue.save(status, dict(status='qualifying', candidates=len(tasks), started=queue.stamp()))
with ThreadPoolExecutor(max_workers=2) as pool:
    futures = [pool.submit(queue.guarded, task['id'], qualify, task) for task in tasks]
    results = [future.result() for future in futures]
queue.save(queue.STATE/'complete.json', dict(status='completed', finished=queue.stamp(), results=results))
