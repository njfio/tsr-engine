"""Serial, one-shot fresh comparison. Preparation and tests never invoke main."""
from pathlib import Path
import hashlib
import importlib.util
import json
import os
import shutil
import subprocess
import sys
from m3_smoke_budget import check

ROOT = Path(__file__).resolve().parents[1]


def preflight():
    freeze = json.loads((ROOT / 'docs/m3-smoke-freeze-v3.json').read_text())
    for relative, expected in freeze['sha256'].items():
        path = ROOT / relative
        if path.is_symlink() or hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError('frozen input changed: ' + relative)
    plan = json.loads((ROOT / 'docs/m3-smoke-comparison-plan.json').read_text())
    if len(plan['jobs']) != 2 or len({j['task_id'] for j in plan['jobs']}) != 2:
        raise ValueError('task scope changed')
    state = ROOT / plan['state']
    if state.exists():
        raise ValueError('one-shot state exists; no retry')
    for job in plan['jobs']:
        if job['modes'] != ['record', 'rules']:
            raise ValueError('mode scope changed')
        base = ROOT / job['input']
        for mode in job['modes']:
            if (base / mode).exists() or list(base.glob('startup-' + mode + '.sqlite*')):
                raise ValueError('arm state is not fresh')
    check((ROOT / plan['shared_ledger']).read_bytes(), json.loads((ROOT / plan['baseline']).read_text()))
    if shutil.disk_usage(ROOT).free < plan['initial_minimum_free_gib'] * 1024**3:
        raise ValueError('initial disk reserve')
    return plan


def require_idle():
    # Read-only process checks. Never resume or terminate the historical coordinator.
    import re
    rows = subprocess.check_output(['ps', '-axo', 'pid=,stat=,command='], text=True, timeout=15).splitlines()
    seen_historical = False
    expected_command = '/opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python -B ' + str(ROOT / 'scripts/m3-following-native-expansion.py')
    for row in rows:
        parts = row.strip().split(None, 2)
        if len(parts) != 3:
            continue
        pid, status, command = parts
        if int(pid) == os.getpid():
            continue
        if pid == '29246':
            seen_historical = True
            if 'T' not in status or command != expected_command:
                raise ValueError('historical coordinator is not the expected stopped process')
            start = subprocess.check_output(['ps', '-p', pid, '-o', 'lstart='], text=True, timeout=15).strip()
            if start != 'Wed Oct 14 21:19:49 2026':
                raise ValueError('historical coordinator identity changed')
            continue
        if 'T' not in status and (re.search(r'/m3-[^ /]+\.py(?: |$)', command) or 'm3-qualification-engine' in command or re.search(r'\b(?:task run|corpus (?:qualify|mutants))\b', command)):
            raise ValueError('another M3 controller or worker is active')
    if not seen_historical:
        raise ValueError('historical coordinator missing; review environment before launch')
    names = subprocess.check_output(['docker', 'ps', '--format', '{{.Names}}'], text=True, timeout=15).splitlines()
    if any(n.startswith('tsr-m3-') for n in names):
        raise ValueError('M3 container remains')


def main():
    if sys.argv[1:] != ['--approved-two-task-comparison']:
        raise ValueError('explicit launch flag required')
    plan = preflight()
    require_idle()
    spec = importlib.util.spec_from_file_location('queue', ROOT / 'scripts/m3-smoke-worker.py')
    queue = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(queue)
    queue.STATE = ROOT / plan['state']
    queue.STATE.mkdir()
    ledger = ROOT / plan['shared_ledger']
    baseline = json.loads((ROOT / plan['baseline']).read_text())
    shutil.copyfile(ledger, queue.STATE / 'cost-before.jsonl')
    queue.save(queue.STATE / 'plan.json', plan)
    results = []
    try:
        for job in plan['jobs']:
            base = ROOT / job['input']
            start_root = None
            for mode in job['modes']:
                require_idle()
                check(ledger.read_bytes(), baseline)
                if shutil.disk_usage(ROOT).free < plan['live_guard_gib'] * 1024**3:
                    raise ValueError('disk reserve')
                env = queue.env_for(False)
                env.update(TSR_M3_APPROVED_LIVE='100', TSR_M3_BROKER_SCRIPT=str(ROOT / plan['broker']),
                    TSR_M3_PYTHON=sys.executable, TSR_M3_COST_LEDGER=str(ledger),
                    TSR_DB=str(base / ('startup-' + mode + '.sqlite')))
                result = queue.run(job['task_id'][:12]+'-'+mode,
                    [str(ROOT / plan['engine']), 'task', 'run', '--mode', mode, '--tasks', str(base / 'tasks.jsonl'),
                     '--driver', 'cheaper-inference', '--profile', str(base / 'profile.toml'), '--out', str(base / mode)], env)
                results.append(result)
                if result.get('status') != 'completed' or result.get('exit') != 0 or result.get('errors'):
                    raise ValueError('arm process failed; no retry')
                check(ledger.read_bytes(), baseline)
                rows = [json.loads(line) for line in (base / mode / ('results-'+mode+'.jsonl')).read_text().splitlines()]
                if len(rows) != 1 or rows[0].get('outcome') not in ('accepted', 'rejected'):
                    raise ValueError('arm unavailable; preserve and stop')
                if rows[0].get('task') != job['task_id'] or rows[0].get('system') != mode or not rows[0].get('state_root_at_start'):
                    raise ValueError('result identity mismatch')
                if start_root is not None and rows[0]['state_root_at_start'] != start_root:
                    raise ValueError('paired starting roots differ')
                start_root = rows[0]['state_root_at_start']
        queue.save(queue.STATE / 'complete.json', dict(status='completed', results=results, finished=queue.stamp()))
    except Exception:
        queue.save(queue.STATE / 'complete.json', dict(status='halted-review-required', results=results, finished=queue.stamp()))
        raise

if __name__ == '__main__': main()
