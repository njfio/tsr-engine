"""Finite local qualification queue; status persists independently of terminal handles."""
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import signal
import shutil
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
        deadline = time.monotonic() + 1800
        errors = []
        try:
            while child.poll() is None and time.monotonic() < deadline:
                if shutil.disk_usage(ROOT).free < 200 * 1024**3:
                    raise RuntimeError('live disk guard')
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
                        raise RuntimeError('owned container remains: ' + name)
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
