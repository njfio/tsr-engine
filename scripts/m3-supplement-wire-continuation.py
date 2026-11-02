"""Prepared, unlaunched finite RTS supplement-wire continuation (maximum 4).

Requires explicit --execute-approved. Uses fresh state/output paths, the
unchanged qualification engine, two workers and a live 200 GiB launch guard.
"""
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
from threading import Event, Thread
import time

ROOT = Path(__file__).resolve().parents[1]
STATE = ROOT / 'target/m3-supplement-wire-continuation-state'
FREEZE = ROOT / 'docs/m3-supplement-wire-continuation-launch-freeze.json'
PREPARED = ROOT / 'docs/m3-supplement-wire-continuation-freeze.json'
THRESHOLD = 200 * 1024**3
OUTER_STARTED = 'Wed Oct 14 21:19:49 2026'
OUTER_COMMAND = '/opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python -B ' + str(ROOT / 'scripts/m3-following-native-expansion.py')


def valid_outer(state, started, command):
    return state.startswith('T') and started == OUTER_STARTED and command == OUTER_COMMAND


def outer_stopped():
    values = [subprocess.check_output(['ps', '-p', '29246', '-o', field + '='], text=True, timeout=10).strip()
              for field in ['state', 'lstart', 'command']]
    return valid_outer(*values)


def terminal_status(results, halted):
    bad = any(r.get('status') != 'completed' or r.get('exit') != 0 or r.get('errors') for r in results)
    return 'halted' if halted or bad else 'completed'



def permitted(free_bytes, halted, watcher_alive):
    return free_bytes >= THRESHOLD and not halted and watcher_alive


def gated_launch(check, operation):
    if not check():
        return dict(status='cancelled-before-launch', reason='resource guard')
    return operation()


def initial_headroom(free_bytes):
    return free_bytes >= THRESHOLD + 40 * 1024**3


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def configure_engine(queue, plan):
    expected = 'target/m3-qualification-engine-module-v2'
    if plan.get('engine') != expected or digest(ROOT / expected) != plan.get('engine_sha256'):
        raise RuntimeError('Nested-module engine is not pinned and verified')
    queue.ENGINE = str(ROOT / expected)


def verify_activation(plan, prepared):
    if plan.get('status') != 'reviewed-ready-for-launch':
        raise RuntimeError('Activation is not reviewed')
    for relative, expected in prepared['hashes'].items():
        if relative != 'docs/m3-native-admission/summary.json' and plan.get('hashes', {}).get(relative) != expected:
            raise RuntimeError('Activation changed a prepared immutable input: ' + relative)
    predecessor = prepared['predecessor']
    family = predecessor.removeprefix('target/m3-').removesuffix('-continuation-state/complete.json')
    required = [predecessor, 'docs/m3-' + family + '-continuation-final.json',
                'docs/m3-' + family + '-continuation-final-audit.json', 'docs/m3-native-admission/summary.json']
    if any(not plan.get('hashes', {}).get(path) for path in required):
        raise RuntimeError('Activation lacks terminal predecessor or current summary hashes')
    if type(plan.get('predecessor_admissions')) is not int or plan['predecessor_admissions'] < 142:
        raise RuntimeError('Activation lacks reconciled admission count')


def verify(plan):
    prepared = json.loads(PREPARED.read_text())
    verify_activation(plan, prepared)
    if plan.get('prepared_freeze_sha256') != digest(PREPARED):
        raise RuntimeError('Activation does not bind prepared freeze')
    for relative, expected in plan['hashes'].items():
        if digest(ROOT / relative) != expected:
            raise RuntimeError('Frozen input differs: ' + relative)
    repo = str(ROOT / 'target/m3-history/repo')
    if subprocess.check_output(['git', '-C', repo, 'rev-parse', 'HEAD'], text=True).strip() != plan['source_revision']:
        raise RuntimeError('Source revision changed')
    if subprocess.check_output(['git', '-C', repo, 'status', '--porcelain'], text=True).strip():
        raise RuntimeError('Source is dirty')


def main():
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute-approved', action='store_true', required=True)
    parser.parse_args()
    plan = json.loads(FREEZE.read_text())
    if plan.get('status') != 'reviewed-ready-for-launch':
        raise RuntimeError('Future activation freeze has not been reviewed')
    verify(plan)
    if not outer_stopped():
        raise RuntimeError('Original outer coordinator identity/state changed')
    predecessor = json.loads((ROOT / 'target/m3-supplement-daemon-continuation-state/complete.json').read_text())
    if predecessor.get('status') != 'completed' or terminal_status(predecessor.get('results', []), False) != 'completed':
        raise RuntimeError('RTS supplement-daemon predecessor is not cleanly terminal')
    final_audit = json.loads((ROOT / 'docs/m3-supplement-daemon-continuation-final-audit.json').read_text())
    if final_audit.get('exit') != 0 or final_audit.get('admissions_checked') != plan['predecessor_admissions']:
        raise RuntimeError('RTS supplement-daemon evidence audit is not reconciled')
    if final_audit.get('summary_sha256') != digest(ROOT / 'docs/m3-native-admission/summary.json'):
        raise RuntimeError('Predecessor audit does not bind current corpus')
    post_cleanup = json.loads((ROOT / 'docs/m3-admission-audit-post-expanded-cleanup.json').read_text())
    if post_cleanup.get('exit') != 0 or post_cleanup.get('admissions_checked') != 107:
        raise RuntimeError('Post-cleanup admission audit failed')
    result = json.loads((ROOT / 'docs/m3-terminal-build-cleanup-expanded-result.json').read_text())
    if result.get('status') != 'completed' or not result.get('evidence_unchanged') or not result.get('source_copies_unchanged'):
        raise RuntimeError('Cleanup preservation not verified')
    names = subprocess.check_output(['docker', 'ps', '--format', '{{.Names}}'], text=True).splitlines()
    if any(n.startswith('tsr-m3-') for n in names):
        raise RuntimeError('Prior task container remains')
    commands = subprocess.check_output(['ps', '-axo', 'command='], text=True).splitlines()
    if any(x.startswith(str(ROOT / 'target/m3-qualification-engine-')) for x in commands):
        raise RuntimeError('Prior qualification process remains')
    # Entire finite continuation has a conservative 40 GiB allocation budget.
    if not initial_headroom(shutil.disk_usage(ROOT).free):
        raise RuntimeError('Insufficient headroom for finite continuation')
    STATE.mkdir()
    spec = importlib.util.spec_from_file_location('queue', ROOT / 'scripts/m3-detached-qualification.py')
    queue = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(queue)
    configure_engine(queue, json.loads((ROOT / 'docs/m3-supplement-wire-cohort-plan.json').read_text()))
    queue.STATE = STATE
    queue.save(STATE / 'plan.json', dict(plan=plan, pid=os.getpid(), started=queue.stamp(), max_workers=2))
    stop = Event()
    ready = Event()
    last_tick = [0.0]

    def watch():
        try:
            while not stop.is_set():
                free = shutil.disk_usage(ROOT).free
                if free < THRESHOLD:
                    queue.HALT.set()
                queue.save(STATE / 'guard.json', dict(recorded_at=queue.stamp(), pid=os.getpid(),
                    free_bytes=free, pause_below_bytes=THRESHOLD,
                    status='cancel-pending-active-supervised' if queue.HALT.is_set() else 'monitoring'))
                last_tick[0] = time.monotonic()
                ready.set()
                stop.wait(10)
        except Exception as error:
            queue.HALT.set()
            queue.save(STATE / 'guard-error.json', dict(error=repr(error), recorded_at=queue.stamp()))

    watcher = Thread(target=watch, daemon=True)
    watcher.start()
    if not ready.wait(15):
        queue.HALT.set()
        raise RuntimeError('Resource watcher did not become ready')

    def check():
        live = watcher.is_alive() and time.monotonic() - last_tick[0] <= 20
        okay = permitted(shutil.disk_usage(ROOT).free, queue.HALT.is_set(), live) and outer_stopped()
        if not okay:
            queue.HALT.set()
        return okay

    def run(label, argv):
        def operation():
            return queue.guarded(label, queue.run, label, argv, queue.env_for())
        row = gated_launch(check, operation)
        if row.get('status') == 'cancelled-before-launch':
            queue.save(STATE / (label + '-cancelled.json'), dict(row, label=label, finished=queue.stamp()))
        return row

    def qualify(task, manifest, family):
        out = ROOT / 'target' / ('m3-qualified-' + family + '-continuation-' + task['id'][:12])
        if out.exists():
            queue.HALT.set()
            raise RuntimeError('Output already exists')
        return run(task['id'], [queue.ENGINE, 'corpus', 'qualify', '--tasks', str(manifest), '--out', str(out)])

    outcomes = []
    try:
        verify(plan)
        indexing = json.loads((ROOT / 'docs/m3-supplement-wire-cohort-plan.json').read_text())
        inputs = ROOT / 'target/m3-supplement-wire-continuation-input'
        inputs.mkdir()
        tasks = []
        for group in indexing['screen_groups']:
            out = ROOT / 'target' / ('m3-rts-supplement-wire-continuation-' + group['label'])
            if out.exists():
                raise RuntimeError('Screen output already exists')
            argv = [queue.ENGINE, 'corpus', 'mutants', str(ROOT / 'target/m3-history/repo'),
                '--package', group['package'], '--screen-target', group['screen_target'],
                '--mutant', '(' + '|'.join(c['filter'] for c in group['candidates']) + ')', '--out', str(out)]
            row = run('supplement-wire-' + group['label'], argv)
            outcomes.append(row)
            if row.get('status') != 'completed' or row.get('exit') != 0 or row.get('errors'):
                queue.HALT.set()
                return
            caught = [json.loads(l) for l in (out / 'tasks.jsonl').read_text().splitlines()]
            if len(caught) > group['max_candidates']:
                raise RuntimeError('Screen count exceeded')
            for task in caught:
                directory = inputs / task['id']
                directory.mkdir()
                (directory / 'tasks.jsonl').write_text(json.dumps(task) + '\n')
                shutil.copytree(out / 'cas', directory / 'cas')
                tasks.append(task)
        if len(tasks) > 4:
            raise RuntimeError('Extraction candidate bound exceeded')
        with ThreadPoolExecutor(max_workers=2) as pool:
            futures = [pool.submit(qualify, t, inputs / t['id'] / 'tasks.jsonl', 'rts-supplement-wire') for t in tasks]
            outcomes.extend(f.result() for f in futures)
    except Exception as error:
        queue.HALT.set()
        outcomes.append(dict(status='controller-error', reason=repr(error)))
        raise
    finally:
        stop.set()
        watcher.join(timeout=15)
        queue.save(STATE / 'complete.json', dict(status=terminal_status(outcomes, queue.HALT.is_set()),
            finished=queue.stamp(), results=outcomes, free_bytes=shutil.disk_usage(ROOT).free))


if __name__ == '__main__':
    main()
