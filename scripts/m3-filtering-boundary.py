"""Finite local extension at an idle boundary; the outer coordinator is paused by launch.

No paid calls. Frozen historical evaluators remain untouched. Fail closed and leave
outer coordinator paused on any incomplete predecessor, mismatch, or resource limit.
"""
from pathlib import Path
import hashlib
import json
import os
import signal
import subprocess
import sys
import time
from datetime import datetime, timezone
import shutil
ROOT = Path(__file__).resolve().parents[1]
STATE = ROOT / 'target/m3-filtering-boundary-state'
PID = 29246
STARTED = 'Wed Oct 14 21:19:49 2026'

def require_paused():
    command = subprocess.check_output(['ps','-p',str(PID),'-o','command='],text=True).strip()
    started = subprocess.check_output(['ps','-p',str(PID),'-o','lstart='],text=True).strip()
    state = subprocess.check_output(['ps','-p',str(PID),'-o','stat='],text=True).strip()
    if started != STARTED or not state.startswith('T') or command != '/opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python -B ' + str(ROOT/'scripts/m3-following-native-expansion.py'):
        raise RuntimeError('Expected exact outer coordinator paused')


def main():
    STATE.mkdir()
    require_paused()
    plan = json.loads((ROOT / 'docs/m3-filtering-extension-freeze.json').read_text())
    deadline = time.monotonic() + 3 * 3600
    terminal = ROOT / 'target/m3-rts-core-cohort-state/complete.json'
    while not terminal.exists():
        (STATE / 'waiting.json').write_text(json.dumps(dict(status='waiting-for-core-boundary', at=datetime.now(timezone.utc).isoformat())))
        if time.monotonic() > deadline:
            raise RuntimeError('Boundary wait expired; outer remains paused')
        time.sleep(10)
    require_paused()
    detail = json.loads(terminal.read_text())
    if detail.get('status') != 'completed' or any(r.get('exit') != 0 or r.get('status') != 'completed' or r.get('errors') for r in detail['results']):
        raise RuntimeError('Core completion requires reconciliation')
    for path, digest in plan['hashes'].items():
        if hashlib.sha256((ROOT / path).read_bytes()).hexdigest() != digest:
            raise RuntimeError('Frozen extension input changed: ' + path)
    repo = ROOT / 'target/m3-history/repo'
    if subprocess.check_output(['git','-C',str(repo),'rev-parse','HEAD'],text=True).strip() != plan['source_revision'] or subprocess.check_output(['git','-C',str(repo),'status','--porcelain'],text=True).strip():
        raise RuntimeError('Prepared source changed')
    if shutil.disk_usage(ROOT).free < 200 * 1024**3:
        raise RuntimeError('Conservative disk reserve prevents extension')
    require_paused()
    with (STATE / 'extension.log').open('x') as log:
        result = subprocess.run([sys.executable,'-B',str(ROOT / 'scripts/m3-rts-filtering-cohort.py')],cwd=ROOT,stdout=log,stderr=subprocess.STDOUT)
    final = json.loads((ROOT / 'target/m3-rts-filtering-cohort-state/complete.json').read_text())
    if result.returncode or final.get('status') != 'completed' or any(r.get('exit') != 0 or r.get('status') != 'completed' or r.get('errors') for r in final['results']):
        raise RuntimeError('Extension requires reconciliation; outer remains paused')
    command = subprocess.check_output(['ps','-p',str(PID),'-o','command='],text=True).strip()
    names = subprocess.check_output(['docker','ps','--format','{{.Names}}'],text=True).splitlines()
    if command != '/opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python -B ' + str(ROOT/'scripts/m3-following-native-expansion.py') or any(n.startswith('tsr-m3-') for n in names):
        raise RuntimeError('Resume identity or idle check failed')
    watch = json.loads((ROOT/'target/m3-reserve-watch-v2/checkpoint.json').read_text())
    require_paused()
    watcher_command = subprocess.check_output(['ps','-p',str(watch['watcher_pid']),'-o','command='],text=True).strip()
    watcher_start = subprocess.check_output(['ps','-p',str(watch['watcher_pid']),'-o','lstart='],text=True).strip()
    age = (datetime.now(timezone.utc) - datetime.fromisoformat(watch['recorded_at'])).total_seconds()
    if watcher_command != '/opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python -B ' + str(ROOT/'scripts/m3-reserve-watch.py') or watcher_start != watch['watcher_started'] or not 0 <= age <= 30:
        raise RuntimeError('Disk watcher absent, changed or stale')
    if shutil.disk_usage(ROOT).free < 200*1024**3 or watch['status'] != 'monitoring':
        raise RuntimeError('Resource watcher disallows resume')
    os.kill(PID,signal.SIGCONT)
    (STATE/'complete.json').write_text(json.dumps(dict(status='completed-outer-resumed', at=datetime.now(timezone.utc).isoformat()),indent=2)+'\n')

if __name__ == '__main__':
    try:
        main()
    except Exception as error:
        if STATE.exists():
            (STATE/'error.json').write_text(json.dumps(dict(status='blocked-outer-remains-paused', reason=str(error)),indent=2)+'\n')
        raise
