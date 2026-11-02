"""Wait for the current native cohort, run approved paired-1, then resume native queue."""
from pathlib import Path
import json
import os
import signal
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
receipt = ROOT / 'target/m3-approved-paired-launch.json'


def verify_paused(identity):
    def field(name):
        return subprocess.check_output(['ps', '-p', str(identity['pid']), '-o', name+'='], text=True).strip()
    if field('command') != identity['command'] or field('lstart') != identity['start'] or not field('stat').startswith('T'):
        raise RuntimeError('Expected paused coordinator identity changed')


def flatten(value):
    if isinstance(value, list):
        return [row for item in value for row in flatten(item)]
    return [value]


def main():
    identity = json.loads((ROOT / 'target/m3-native-coordinator-identity.json').read_text())
    verify_paused(identity)
    deadline = time.monotonic() + 7200
    terminal = ROOT / 'target/m3-rts-core-cohort-state/complete.json'
    while not terminal.exists():
        receipt.write_text(json.dumps(dict(status='waiting-for-current-native-cohort', pid=os.getpid(), heartbeat=time.time()))+'\n')
        if time.monotonic() > deadline:
            raise RuntimeError('Native completion wait expired; paid launch not attempted')
        time.sleep(10)
    native = json.loads(terminal.read_text())
    if native.get('status') != 'completed' or any(r.get('status') != 'completed' or r.get('exit') != 0 or r.get('errors') for r in native.get('results', [])):
        raise RuntimeError('Native cohort needs reconciliation before paid launch')
    verify_paused(identity)
    receipt.write_text(json.dumps(dict(status='running-approved-paired', pid=os.getpid(), started=time.time()))+'\n')
    result = subprocess.run([sys.executable, '-B', str(ROOT / 'scripts/m3-paired-cohort.py')], cwd=ROOT)
    complete = ROOT / 'target/m3-paired-cohort-1-state/complete.json'
    rows = flatten(json.loads(complete.read_text())['results']) if complete.exists() else []
    clean = result.returncode == 0 and len(rows) == 8 and all(r.get('status') == 'completed' and r.get('exit') == 0 and not r.get('errors') and r.get('pid') for r in rows)
    names = subprocess.check_output(['docker','ps','--format','{{.Names}}'], text=True, timeout=15).splitlines()
    groups = {int(p) for p in subprocess.check_output(['ps','-axo','pgid='], text=True).split()}
    resumed = False
    if clean and not any(name.startswith('tsr-m3-') for name in names) and not any(r['pid'] in groups for r in rows):
        verify_paused(identity)
        os.kill(identity['pid'], signal.SIGCONT)
        resumed = True
    receipt.write_text(json.dumps(dict(status='terminal', paired_controller_exit=result.returncode,
        native_resumed=resumed, reconciliation_required=not resumed, finished=time.time()))+'\n')
    return result.returncode


if __name__ == '__main__':
    receipt.open('x').close()
    try:
        raise SystemExit(main())
    except Exception as error:
        receipt.write_text(json.dumps(dict(status='controller-error', error_class=type(error).__name__, native_resumed=False, finished=time.time()))+'\n')
        raise
