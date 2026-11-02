"""Scheduling only: drain an existing pair, revalidate admission, run frozen paid-2, resume original queues."""
from pathlib import Path
from datetime import datetime, timezone
import importlib.util
import json
import os
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
STATE = ROOT / 'target/m3-pair-boundary-2-state'


def ps_rows():
    output = subprocess.check_output(['ps', '-axo', 'pid=,pgid=,stat='], text=True)
    return [(int(p), int(g), s) for p, g, s in (line.split() for line in output.splitlines())]


def live_group(rows, pgid):
    return any(g == pgid and not s.startswith('Z') for _, g, s in rows)


def paused(identity):
    def field(key):
        return subprocess.check_output(['ps', '-p', str(identity['pid']), '-o', key+'='], text=True).strip()
    if field('command') != identity['command'] or field('lstart') != identity['start'] or not field('stat').startswith('T'):
        raise RuntimeError('Paused coordinator identity changed')


def verify_worker(identity, rows):
    found = [(g, state) for p, g, state in rows if p == identity['pid']]
    if found:
        group, state = found[0]
        def field(name):
            return subprocess.check_output(['ps', '-p', str(identity['pid']), '-o', name+'='], text=True).strip()
        if group != identity['pgid'] or field('lstart') != identity['start']:
            raise RuntimeError('Worker PID identity changed')
        if not state.startswith('Z') and field('command') != identity['command']:
            raise RuntimeError('Worker command identity changed')


def timeout_near(worker, now):
    return (now-datetime.fromisoformat(worker['receipt']['started'])).total_seconds() >= 2690


def names():
    return subprocess.check_output(['docker', 'ps', '--format', '{{.Names}}'], text=True, timeout=15).splitlines()


def save(value):
    temporary = STATE / 'status.tmp'
    temporary.write_text(json.dumps(value, indent=2)+'\n')
    temporary.replace(STATE / 'status.json')


def flatten(value):
    return [r for item in value for r in flatten(item)] if isinstance(value, list) else [value]


def main():
    identity = json.loads((ROOT / 'target/m3-pair-boundary-2-identities.json').read_text())
    ids = identity['identities']
    spec = importlib.util.spec_from_file_location('paired', ROOT / 'scripts/m3-paired-cohort-2.py')
    paired = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(paired)
    ledger = ROOT / 'target/m3-provider-cost.jsonl'
    for key in ('cohort', 'native'):
        paused(ids[key])
    if (ROOT / 'target/m3-paired-cohort-2-state').exists():
        raise RuntimeError('Paid cohort already has state')
    workers = []
    for key in ('worker1', 'worker2'):
        w = ids[key]
        matches = []
        for path in (ROOT / 'target/m3-rts-core-cohort-state').glob('*.json'):
            row = json.loads(path.read_text())
            if isinstance(row, dict) and row.get('pid') == w['pid']:
                matches.append((path, row))
        if len(matches) != 1:
            raise RuntimeError('Worker receipt ambiguous')
        path, row = matches[0]
        w = dict(w, task=path.stem, receipt=row)
        workers.append(w)
    (STATE / 'before.json').write_text(json.dumps(identity, indent=2)+'\n')
    while True:
        rows = ps_rows()
        for worker in workers:
            verify_worker(worker, rows)
        active = [w for w in workers if live_group(rows, w['pgid'])]
        if not active:
            break
        # The paused original parent owns the 2700s job timeout. Restore it before
        # that deadline so its existing timeout/cleanup/receipt logic remains authoritative.
        if any(timeout_near(w, datetime.now(timezone.utc)) for w in active):
            for key in ('cohort', 'native'):
                paused(ids[key])
                os.kill(ids[key]['pid'], signal.SIGCONT)
            save(dict(status='native-timeout-guard-resumed', paid_started=False, reason='Returned timeout enforcement to original controller before 2700s deadline'))
            return 2
        save(dict(status='waiting-for-active-pair', pid=os.getpid(), active=[w['task'] for w in active], heartbeat=time.time()))
        time.sleep(2)
    if any(n.startswith('tsr-m3-') for n in names()):
        raise RuntimeError('M3 container remains after pair exit')
    evidence = []
    for w in workers:
        base = ROOT / 'target' / ('m3-qualified-rts-core-'+w['task'][:12])
        reports = [json.loads(line) for line in (base/'admission.jsonl').read_text().splitlines()]
        if len(reports) != 1 or reports[0]['task_id'] != w['task']:
            raise RuntimeError('Worker lacks terminal admission record')
        report = reports[0]
        if report['accepted']:
            if report.get('protocol') != 'baseline-full-green/named-red/full-red/exact-reversal/full-green-v1':
                raise RuntimeError('Strict protocol missing')
            tasks = [json.loads(line) for line in (base/'tasks.jsonl').read_text().splitlines()]
            task = next(t for t in tasks if t['id'] == w['task'])
            named = task.get('failing_tests') or task.get('flipped_tests')
            if not named:
                raise RuntimeError('Named tests missing')
            expected = {'baseline','mutated-full','reversal-full'} | {f'{color}-{i}' for color in ('red','green') for i in range(len(named))}
            phases = {p.stem:json.loads(p.read_text()) for p in (base/w['task']).glob('*.json')}
            if not expected <= phases.keys():
                raise RuntimeError('Strict admission incomplete')
            if any(v['exit'] != (101 if k.startswith('red-') or k == 'mutated-full' else 0) for k,v in phases.items() if 'run' in v):
                raise RuntimeError('Unexpected qualification phase exit')
        evidence.append(dict(task=w['task'], admission=report, process_exit='pending original parent reap'))
    (STATE / 'pair-boundary.json').write_text(json.dumps(evidence, indent=2)+'\n')
    for key in ('cohort', 'native'):
        paused(ids[key])
    paired.require_reconciled(ledger)
    # Revalidate all strict admission phases with the repaired, frozen resolver.
    plan = json.loads((ROOT/'docs/m3-paired-cohort-2-plan.json').read_text())
    import hashlib
    import shutil
    for relative, digest in plan['hashes'].items():
        if hashlib.sha256((ROOT/relative).read_bytes()).hexdigest() != digest:
            raise RuntimeError('Frozen input changed: '+relative)
    spec = importlib.util.spec_from_file_location('revalidate', ROOT/'scripts/m3-detached-qualification.py')
    queue = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(queue)
    queue.STATE = ROOT/'target/m3-module-2-revalidation-state'
    queue.STATE.mkdir()
    if shutil.disk_usage(ROOT).free < 100*1024**3:
        raise RuntimeError('Disk floor reached before revalidation')
    job = plan['jobs'][3]
    out = ROOT/'target/m3-module-2-revalidation'
    save(dict(status='revalidating-admission', started=time.time(), pid=os.getpid()))
    result = queue.run('outline-revalidation', [str(ROOT/plan['engine']), 'corpus', 'qualify',
        '--tasks', str(ROOT/job['input']/'tasks.jsonl'), '--out', str(out)], queue.env_for())
    reports = [json.loads(line) for line in (out/'admission.jsonl').read_text().splitlines()] if (out/'admission.jsonl').exists() else []
    accepted = (result.get('exit') == 0 and not result.get('errors') and len(reports) == 1
        and reports[0].get('task_id') == job['task_id'] and reports[0].get('accepted') is True
        and reports[0].get('protocol') == 'baseline-full-green/named-red/full-red/exact-reversal/full-green-v1')
    evidence = ROOT/'docs/m3-module-2-revalidation'
    evidence.mkdir()
    for name in ('admission.jsonl', 'tasks.jsonl', 'preparation.json'):
        if (out/name).exists(): shutil.copyfile(out/name, evidence/name)
    if out.exists():
        task_evidence = evidence/job['task_id']
        task_evidence.mkdir()
        for phase in (out/job['task_id']).iterdir():
            if phase.is_file() and phase.suffix in ('.json', '.jsonl'):
                shutil.copyfile(phase, task_evidence/phase.name)
        if (out/job['task_id']/'cas').exists():
            shutil.copytree(out/job['task_id']/'cas', task_evidence/'cas')
        if (out/'cas').exists(): shutil.copytree(out/'cas', evidence/'cas')
    receipt = dict(status='accepted' if accepted else 'not-accepted', process=result, reports=reports,
                   engine_sha256=plan['hashes'][plan['engine']], finished=queue.stamp())
    (ROOT/plan['predecessor']).write_text(json.dumps(receipt, indent=2)+'\n')
    if live_group(ps_rows(), result['pid']) or any(n.startswith('tsr-m3-') for n in names()):
        raise RuntimeError('Revalidation worker remains')
    if not accepted:
        for key in ('cohort', 'native'):
            paused(ids[key]); os.kill(ids[key]['pid'], signal.SIGCONT)
        save(dict(status='revalidation-not-accepted', native_resumed=True, paid_started=False, finished=time.time()))
        return 2
    save(dict(status='running-approved-paired', started=time.time(), pid=os.getpid()))
    result = subprocess.run([sys.executable, '-B', str(ROOT/'scripts/m3-paired-cohort-2.py')], cwd=ROOT)
    complete = ROOT / 'target/m3-paired-cohort-2-state/complete.json'
    records = flatten(json.loads(complete.read_text())['results']) if complete.exists() else []
    if result.returncode != 0 or len(records) != 8 or any(r.get('status') != 'completed' or r.get('exit') != 0 or r.get('errors') or not r.get('pid') for r in records):
        raise RuntimeError('Paid cohort requires reconciliation before native resume')
    paired.require_reconciled(ledger)
    rows = ps_rows()
    if any(live_group(rows, row['pid']) for row in records) or any(n.startswith('tsr-m3-') for n in names()):
        raise RuntimeError('Paid worker remains')
    # Export terminal paid evidence before any native task resumes.
    subprocess.run([sys.executable, '-B', str(ROOT/'scripts/m3-paired-cohort-2-evidence.py')], cwd=ROOT, check=True)
    for key in ('cohort', 'native'):
        paused(ids[key])
        os.kill(ids[key]['pid'], signal.SIGCONT)
    save(dict(status='completed', paid_controller_exit=result.returncode, native_resumed=True, finished=time.time()))
    return 0


if __name__ == '__main__':
    STATE.mkdir()
    try:
        raise SystemExit(main())
    except Exception as error:
        save(dict(status='reconciliation-required', error_class=type(error).__name__, reason=str(error), finished=time.time()))
        raise
