"""Finite local qualification queue; status persists independently of terminal handles."""
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import signal
import subprocess
import time
from threading import Event

ROOT = Path(__file__).resolve().parents[1]
STATE = ROOT / 'target/m3-detached-state'
ENGINE = str(ROOT / 'target/m3-qualification-engine-238ac39')
HALT = Event()

def stamp():
    return datetime.now(timezone.utc).isoformat()

def save(path, value):
    temporary = path.with_suffix('.tmp')
    temporary.write_text(json.dumps(value, indent=2) + '\n')
    temporary.replace(path)

def env_for(agentid=False):
    env = os.environ.copy()
    image = Path('/tmp/m3-agentid-v2-image-id').read_text().strip() if agentid else 'sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6'
    prep = 'agentid-head-v2' if agentid else 'rts-head-v2'
    env.update(TSR_M3_RUNTIME='docker', TSR_M3_USER='65534:65534', TSR_M3_IMAGE=image,
               TSR_M3_PREPARATION=str(ROOT / 'docs/m3-preparations' / prep / 'preparation.json'))
    return env

def run(label, argv, env):
    if HALT.is_set():
        return dict(status='skipped-after-controller-error', label=label)
    env = env.copy()
    env.setdefault('TSR_DB', str((STATE / (label + '-startup.sqlite')).resolve()))
    path = STATE / (label + '.json')
    record = dict(status='running', started=stamp(), argv=argv)
    with (STATE / (label + '.log')).open('x') as log:
        child = subprocess.Popen(argv, cwd=ROOT, env=env, stdin=subprocess.DEVNULL,
                                 stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        record['pid'] = child.pid
        deadline = time.monotonic() + 2700
        errors = []
        try:
            while child.poll() is None and time.monotonic() < deadline:
                record['heartbeat'] = stamp()
                save(path, record)
                time.sleep(10)
            record['status'] = 'timeout' if child.poll() is None else 'completed'
        except Exception as error:
            record['status'] = 'controller-error'
            errors.append(repr(error))
        finally:
            if child.poll() is None:
                try:
                    os.killpg(child.pid, signal.SIGTERM)
                except ProcessLookupError:
                    pass
                try:
                    child.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    try:
                        os.killpg(child.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    try:
                        child.wait(timeout=10)
                    except subprocess.TimeoutExpired as error:
                        errors.append('reap: ' + repr(error))
            # Inspect even after normal exit; no next job if cleanup cannot be verified.
            try:
                names = subprocess.run(['docker', 'ps', '--format', '{{.Names}}'],
                    capture_output=True, text=True, timeout=15, check=True).stdout.splitlines()
                for name in names:
                    if name.startswith(f'tsr-m3-{child.pid}-'):
                        subprocess.run(['docker', 'rm', '--force', name], timeout=20, check=True)
            except Exception as error:
                errors.append('cleanup: ' + repr(error))
            record.update(exit=child.returncode, finished=stamp(), heartbeat=stamp(), errors=errors)
            try:
                save(path, record)
            except Exception as error:
                errors.append('terminal-status: ' + repr(error))
            if errors:
                # Other active work may finish; queued work must not start.
                HALT.set()
                print(json.dumps(dict(label=label, **record)), flush=True)
    return record

def native(task):
    identifier = task['id']
    return run(identifier, [ENGINE, 'corpus', 'qualify', '--tasks',
        str(ROOT / 'target/m3-resume-input' / identifier / 'tasks.jsonl'), '--out',
        str(ROOT / 'target' / ('m3-qualified-resume-' + identifier[:12] + '-v2'))], env_for())

def agentid():
    screen = ROOT / 'target/m3-agentid-screen-v2'
    result = run('agentid-screen-v2', [ENGINE, 'corpus', 'mutants',
        str(ROOT / 'target/m3-second-source/repo'), '--package', 'agentid-core',
        '--file', 'crates/agentid-core/src/validation.rs', '--mutant',
        r'(replace >= with < in is_valid_e164|replace \|\| with && in is_valid_email_address)$',
        '--screen-target', 'lib', '--out', str(screen)], env_for(True))
    manifest = screen / 'tasks.jsonl'
    if result.get('exit') == 0 and manifest.exists() and manifest.read_text().strip():
        return run('agentid-qualify-v2', [ENGINE, 'corpus', 'qualify', '--tasks', str(manifest),
            '--out', str(ROOT / 'target/m3-qualified-agentid-v2')], env_for(True))
    return result

def guarded(label, operation, *args):
    if HALT.is_set():
        return dict(status='skipped-after-controller-error', label=label)
    try:
        return operation(*args)
    except Exception as error:
        HALT.set()
        record = dict(status='controller-error', label=label, reason=repr(error), finished=stamp())
        try:
            save(STATE / (label + '-error.json'), record)
        except Exception:
            print(json.dumps(record), flush=True)
        return record

if __name__ == '__main__':
    STATE.mkdir()  # Refuse duplicate launches.
    tasks = json.loads((ROOT / 'target/m3-resume-plan.json').read_text())
    tasks = [t for t in tasks if not t['id'].startswith('a552be6a9355')]
    save(STATE / 'plan.json', dict(started=stamp(), pid=os.getpid(), workers=2,
                                 job_timeout_seconds=2700, native_tasks=tasks))
    with ThreadPoolExecutor(max_workers=2) as pool:
        futures = [pool.submit(guarded, 'agentid', agentid)] + [pool.submit(guarded, task['id'], native, task) for task in tasks]
        results = []
        for future in futures:
            try:
                results.append(future.result())
            except Exception as error:
                results.append(dict(status='controller-error', reason=repr(error)))
    save(STATE / 'complete.json', dict(finished=stamp(), results=results))
