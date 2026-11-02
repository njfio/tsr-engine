"""Run the frozen four-task cohort once; no adaptive retries or outcome selection."""
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


def require_reconciled(ledger):
    events = [json.loads(line) for line in ledger.read_text().splitlines()]
    reserves = [e['id'] for e in events if e['kind'] == 'reserve']
    settled = [e['id'] for e in events if e['kind'] == 'settled']
    if any(e['kind'] not in ('reserve', 'settled') for e in events) or len(set(reserves)) != len(reserves) or len(set(settled)) != len(settled) or set(reserves) != set(settled):
        raise RuntimeError('Ledger requires reconciliation')


def main():
    plan = json.loads((ROOT / 'docs/m3-paired-cohort-1-plan.json').read_text())
    for relative, expected in plan['hashes'].items():
        if hashlib.sha256((ROOT / relative).read_bytes()).hexdigest() != expected:
            raise RuntimeError('Reviewed input changed: ' + relative)
    if len(plan['jobs']) != 4 or len({j['task_id'] for j in plan['jobs']}) != 4:
        raise RuntimeError('Four distinct tasks required')
    for job in plan['jobs']:
        rows = [json.loads(line) for line in (ROOT / job['input'] / 'tasks.jsonl').read_text().splitlines()]
        if len(rows) != 1 or rows[0]['id'] != job['task_id'] or job['modes'] != ['record', 'rules']:
            raise RuntimeError('Task scope changed')
    predecessor = json.loads((ROOT / plan['predecessor']).read_text())
    if predecessor.get('status') not in ('completed', 'reconciled-evidence-complete'):
        raise RuntimeError('Qualification predecessor not completed')
    names = subprocess.check_output(['docker', 'ps', '--format', '{{.Names}}'], text=True, timeout=15).splitlines()
    if any(name.startswith('tsr-m3-') for name in names):
        raise RuntimeError('Prior M3 container remains')
    spec = importlib.util.spec_from_file_location('queue', ROOT / 'scripts/m3-detached-qualification.py')
    queue = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(queue)
    queue.STATE = ROOT / 'target/m3-paired-cohort-1-state'
    ledger = ROOT / plan['shared_ledger']
    require_reconciled(ledger)
    diagnostic = ledger.with_name(ledger.stem + '-diagnostics.jsonl')
    previous_lines = len(diagnostic.read_text().splitlines()) if diagnostic.exists() else 0
    queue.STATE.mkdir()
    shutil.copyfile(ledger, queue.STATE / 'cost-before.jsonl')
    queue.save(queue.STATE / 'plan.json', plan)
    queue.save(queue.STATE / 'progress.json', dict(status='running', started=queue.stamp(), pid=os.getpid()))

    def paired(job):
        base = ROOT / job['input']
        env = queue.env_for(job['source'] == 'agentid')
        env.update(TSR_M3_APPROVED_LIVE='100', TSR_M3_BROKER_SCRIPT=str(ROOT / plan['broker']),
                   TSR_M3_PYTHON=sys.executable, TSR_M3_COST_LEDGER=str(ledger))
        results = []
        for mode in job['modes']:
            try:
                require_reconciled(ledger)
            except RuntimeError:
                queue.HALT.set()
                raise
            if shutil.disk_usage(ROOT).free < plan['minimum_free_gib'] * 1024**3:
                queue.HALT.set()
                raise RuntimeError('Disk reserve reached')
            env['TSR_DB'] = str(base / ('startup-' + mode + '.sqlite'))
            label = job['task_id'][:12] + '-' + mode
            results.append(queue.run(label, [str(ROOT / plan['engine']), 'task', 'run', '--mode', mode,
                '--tasks', str(base / 'tasks.jsonl'), '--driver', 'cheaper-inference',
                '--profile', str(base / 'profile.toml'), '--out', str(base / mode)], env))
            if results[-1].get('exit') != 0 or results[-1].get('errors'):
                queue.HALT.set()
            # Stop queued work on a refusal or an uncertain dispatched charge; no implicit retry.
            if diagnostic.exists():
                recent = [json.loads(line) for line in diagnostic.read_text().splitlines()[previous_lines:]]
                if any(d.get('code') == 'provider_refusal' or
                       (d.get('reservation_id') and d.get('status') == 'failed' and not d.get('settled')) for d in recent):
                    queue.HALT.set()
        return results

    with ThreadPoolExecutor(max_workers=1) as pool:
        futures = [pool.submit(queue.guarded, j['task_id'], paired, j) for j in plan['jobs']]
        results = [future.result() for future in futures]
    queue.save(queue.STATE / 'complete.json', dict(finished=queue.stamp(), results=results))


if __name__ == '__main__':
    main()
