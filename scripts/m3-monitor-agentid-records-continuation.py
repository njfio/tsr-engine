"""Finite observer of the existing recovery cohort; never launch or cancel work.

Writes durable progress and final audit evidence, then exits. No deletion,
provider calls, scheduler changes, or qualification commands.
"""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
STATE = ROOT / 'target/m3-agentid-records-continuation-state'
MONITOR = ROOT / 'target/m3-agentid-records-continuation-monitor-state'
FINAL = ROOT / 'docs/m3-agentid-records-continuation-final.json'
OWNED = False


def stamp():
    return datetime.now(timezone.utc).isoformat()


def save(path, value):
    temp = path.with_suffix('.tmp')
    temp.write_text(json.dumps(value, indent=2) + '\n')
    temp.replace(path)


def snapshot():
    receipts = []
    for path in STATE.glob('*.json'):
        row = json.loads(path.read_text())
        if 'argv' in row or path.name.endswith(('-cancelled.json', '-error.json')):
            receipts.append(dict(receipt=path.name, **row))
    guard = json.loads((STATE / 'guard.json').read_text())
    processes = subprocess.check_output(['ps', '-axo', 'pid=,state=,command='], text=True, timeout=15).splitlines()
    workers = []
    controller = []
    for row in processes:
        columns = row.strip().split(None, 2)
        if len(columns) != 3:
            continue
        if columns[2].startswith(str(ROOT / 'target/m3-qualification-engine-238ac39') + ' '):
            workers.append(row.strip())
        if columns[0] == '98954':
            controller.append(row.strip())
    containers = [n for n in subprocess.check_output(['docker', 'ps', '--format', '{{.Names}}'], text=True, timeout=15).splitlines() if n.startswith('tsr-m3-')]
    complete = json.loads((STATE / 'complete.json').read_text()) if (STATE / 'complete.json').exists() else None
    anomalies = []
    if len(workers) > 2 or len(containers) > 2:
        anomalies.append('Concurrency exceeds two; manual inspection required, no automatic intervention')
    age = (datetime.now(timezone.utc) - datetime.fromisoformat(guard['recorded_at'])).total_seconds()
    if complete is None and (age > 60 or not controller):
        anomalies.append('Controller absent or guard stale; preserve state and inspect, no retry')
    return dict(recorded_at=stamp(), free_bytes=shutil.disk_usage(ROOT).free, guard=guard,
                workers=workers, controller=controller, task_containers=containers,
                receipts=receipts, terminal=complete, anomalies=anomalies)


def export_and_audit(index):
    summary_path = ROOT / 'docs/m3-native-admission/summary.json'
    before = json.loads(summary_path.read_text())
    subprocess.run([sys.executable, '-B', str(ROOT / 'scripts/m3-admission-evidence.py')], cwd=ROOT, check=True, timeout=180)
    after = json.loads(summary_path.read_text())
    now = {a['attempt']: a for a in after['attempts']}
    if any(now.get(a['attempt']) != a for a in before['attempts']):
        raise RuntimeError('Prior attempt records changed')
    count = 0
    base = ROOT / 'docs/m3-native-admission'
    for attempt in base.iterdir():
        if not attempt.is_dir():
            continue
        for path in attempt.rglob('*'):
            if path.is_file():
                source = ROOT / 'target' / attempt.name / path.relative_to(attempt)
                if path.is_symlink() or source.is_symlink() or path.read_bytes() != source.read_bytes():
                    raise RuntimeError('Export/source evidence differs: ' + str(path))
                count += 1
    out = ROOT / 'target' / ('m3-agentid-records-monitor-audit-' + str(index))
    subprocess.run([sys.executable, '-B', str(ROOT / 'scripts/m3-audit-admissions.py'), '--out', str(out)], cwd=ROOT, check=True, timeout=180)
    report = json.loads((out / 'report.json').read_text())
    if report['exit'] != 0:
        raise RuntimeError('Native audit failed')
    return dict(admissions=after['admitted_production_tasks'], verified_export_files=count,
                audit=str(out.relative_to(ROOT)), report=report)


def main():
    global OWNED
    MONITOR.mkdir()  # Exclusive observer ownership, no duplicate exporters.
    OWNED = True
    previous = None
    audit_index = 0
    deadline = time.monotonic() + 12 * 3600
    while time.monotonic() < deadline:
        current = snapshot()
        signature = sorted((r['receipt'], r.get('status'), r.get('exit')) for r in current['receipts'] if r.get('status') != 'running')
        terminal = current['terminal'] is not None
        if current['anomalies']:
            save(MONITOR / 'attention.json', current)
            print(json.dumps(dict(status='attention-required', anomalies=current['anomalies'])), flush=True)
            return
        if signature != previous or terminal:
            audit_index += 1
            audit = export_and_audit(audit_index)
            current['audit'] = audit
            save(MONITOR / ('checkpoint-' + str(audit_index) + '.json'), current)
            print(json.dumps(dict(recorded_at=stamp(), admissions=audit['admissions'], terminal=terminal,
                                 active_workers=len(current['workers']), free_gib=current['free_bytes'] / 1024**3)), flush=True)
            previous = signature
        save(MONITOR / 'progress.json', current)
        if terminal:
            # A completed controller must have reaped its children/containers.
            current = snapshot()
            if current['workers'] or current['task_containers']:
                raise RuntimeError('Terminal controller has surviving workers or task containers')
            summary = json.loads((ROOT / 'docs/m3-native-admission/summary.json').read_text())
            attempts = [a for a in summary['attempts'] if a['attempt'].startswith(('m3-qualified-agentid-records-continuation-',))]
            reports = [v for a in attempts for v in a['reports']]
            final = dict(current, audit=audit, accepted=sum(bool(v['accepted']) for v in reports),
                         rejected=sum(not v['accepted'] for v in reports), attempts=attempts,
                         next_eligible_work='If AgentID records completed without controller errors: RTS behavior, at most five candidates, using fresh continuation state and reconciliation of this predecessor. Do not launch an old controller blindly: expected predecessor paths differ. If halted: reconcile unfinished records work first. No automatic next launch.',
                         no_paid_calls=True, no_further_cleanup=True)
            if FINAL.exists():
                raise RuntimeError('Final report exists; refuse overwrite')
            save(FINAL, final)
            shutil.copyfile(ROOT / audit['audit'] / 'report.json', ROOT / 'docs/m3-agentid-records-continuation-final-audit.json')
            print(json.dumps(dict(status='final-evidence-written', accepted=final['accepted'], rejected=final['rejected'], admissions=audit['admissions'])), flush=True)
            return
        time.sleep(30)
    save(MONITOR / 'attention.json', dict(status='monitor-deadline', recorded_at=stamp(), reason='12-hour observer bound reached; no workload was altered'))


if __name__ == '__main__':
    try:
        main()
    except Exception as error:
        if OWNED:
            save(MONITOR / 'error.json', dict(recorded_at=stamp(), error=repr(error), action='Observation stopped; qualification unchanged'))
        raise
