"""One prospective retry of the three affected arms; originals remain untouched."""
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def startup_store(base, mode):
    if mode not in ('record', 'rules'):
        raise ValueError('Unsupported arm')
    return str((base / ('startup-' + mode + '.sqlite')).resolve())


def main():
    spec = importlib.util.spec_from_file_location('queue', ROOT / 'scripts/m3-detached-qualification.py')
    queue = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(queue)
    queue.STATE = ROOT / 'target/m3-live-pilot-retry-v2-state'
    output = ROOT / 'target/m3-live-pilot-retry-v2'
    plan = json.loads((ROOT / 'docs/m3-pilot-retry-v2-plan.json').read_text())
    broker = ROOT / 'target/m3-pilot-retry-v2-broker.py'
    if hashlib.sha256(broker.read_bytes()).hexdigest() != plan['broker_sha256']:
        raise RuntimeError('Broker differs from reviewed frozen version')
    if shutil.disk_usage(ROOT).free < 100 * 1024**3:
        raise RuntimeError('Less than100GiB free')
    names = subprocess.run(['docker', 'ps', '--format', '{{.Names}}'],
                           capture_output=True, text=True, timeout=15, check=True).stdout.splitlines()
    if any(name.startswith('tsr-m3-') for name in names):
        raise RuntimeError('Prior M3 container remains')
    ledger = ROOT / 'target/m3-provider-cost.jsonl'
    events = [json.loads(line) for line in ledger.read_text().splitlines()]
    if any(e['kind'] == 'halt' for e in events):
        raise RuntimeError('Cost reconciliation required')
    if {e['id'] for e in events if e['kind'] == 'reserve'} != {e['id'] for e in events if e['kind'] == 'settled'}:
        raise RuntimeError('Outstanding charge requires reconciliation')
    for relative, expected in plan['input_sha256'].items():
        if hashlib.sha256((ROOT / relative).read_bytes()).hexdigest() != expected:
            raise RuntimeError('Reviewed input changed: ' + relative)
    for source, expected in plan['task_ids'].items():
        rows = [json.loads(line) for line in (ROOT / 'target/m3-live-pilot' / source / 'tasks.jsonl').read_text().splitlines()]
        if len(rows) != 1 or rows[0]['id'] != expected:
            raise RuntimeError('Pilot task scope changed')
    queue.STATE.mkdir()
    output.mkdir()
    queue.save(queue.STATE / 'plan.json', plan)
    for source in ('agentid', 'rts'):
        base = output / source
        base.mkdir()
        original = ROOT / 'target/m3-live-pilot' / source
        for file in ('tasks.jsonl', 'profile.toml', 'context.json'):
            if (original / file).exists():
                shutil.copyfile(original / file, base / file)
        # Copy only corpus input CAS and trusted context; no earlier candidate edits/stores.
        shutil.copytree(original / 'cas', base / 'cas')
        for path in original.glob('*context*.json'):
            if not (base / path.name).exists():
                shutil.copyfile(path, base / path.name)
    queue.save(queue.STATE / 'progress.json', dict(status='running', started=queue.stamp(), pid=os.getpid()))

    def run_source(source, modes):
        base = output / source
        env = queue.env_for(source == 'agentid')
        env.update(TSR_M3_APPROVED_LIVE='100', TSR_M3_BROKER_SCRIPT=str(broker),
                   TSR_M3_PYTHON=sys.executable, TSR_M3_COST_LEDGER=str(ledger))
        results = []
        for mode in modes:
            if shutil.disk_usage(ROOT).free < 100 * 1024**3:
                raise RuntimeError('Disk reserve reached')
            env['TSR_DB'] = startup_store(base, mode)
            results.append(queue.run(source + '-' + mode, [queue.ENGINE, 'task', 'run',
                '--mode', mode, '--tasks', str(base / 'tasks.jsonl'), '--driver',
                'cheaper-inference', '--profile', str(base / 'profile.toml'),
                '--out', str(base / mode)], env))
            # A genuine provider refusal is terminal; never use another arm to retry it.
            diagnostic = ledger.with_name(ledger.stem + '-diagnostics.jsonl')
            if diagnostic.exists() and any(json.loads(line).get('code') == 'provider_refusal'
                                           for line in diagnostic.read_text().splitlines()):
                queue.HALT.set()
                break
        return results

    with ThreadPoolExecutor(max_workers=2) as pool:
        futures = [pool.submit(queue.guarded, 'agentid', run_source, 'agentid', ('record',)),
                   pool.submit(queue.guarded, 'rts', run_source, 'rts', ('record', 'rules'))]
        results = [future.result() for future in futures]
    queue.save(queue.STATE / 'complete.json', dict(finished=queue.stamp(), results=results))


if __name__ == '__main__':
    main()
