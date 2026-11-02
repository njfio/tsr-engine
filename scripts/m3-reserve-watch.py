"""One read-only disk watcher with a narrowly scoped coordinator pause; no deletion.

The current cohort remains supervised by its own timeout controller. Pausing only
its parent prevents the next cohort. This guard does not pause worker supervisors.
"""
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[1]
PID = 29246
EXPECTED = str(ROOT / 'scripts/m3-following-native-expansion.py')
STATE = ROOT / 'target/m3-reserve-watch-v2'
STARTED = 'Wed Oct 14 21:19:49 2026'
THRESHOLD = 200 * 1024**3


def main():
    STATE.mkdir()  # Exclusive ownership; no accidental duplicate watcher.
    while True:
        result = subprocess.run(['ps', '-p', str(PID), '-o', 'command='],
                                text=True, capture_output=True, timeout=10)
        command = result.stdout.strip()
        started = subprocess.run(['ps', '-p', str(PID), '-o', 'lstart='], text=True, capture_output=True, timeout=10).stdout.strip()
        free = shutil.disk_usage(ROOT).free
        row = dict(recorded_at=datetime.now(timezone.utc).isoformat(), free_bytes=free,
                   pause_below_bytes=THRESHOLD, coordinator_pid=PID, watcher_pid=os.getpid(), watcher_started=subprocess.check_output(['ps','-p',str(os.getpid()),'-o','lstart='],text=True).strip())
        if result.returncode or started != STARTED or command != '/opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python -B ' + EXPECTED:
            row['status'] = 'finished-or-identity-changed'
        elif free < THRESHOLD:
            os.kill(PID, signal.SIGSTOP)
            row['status'] = 'coordinator-paused-current-cohort-still-supervised'
        else:
            row['status'] = 'monitoring'
        temp = STATE / 'checkpoint.tmp'
        temp.write_text(json.dumps(row, indent=2) + '\n')
        temp.replace(STATE / 'checkpoint.json')
        if row['status'] != 'monitoring':
            return
        time.sleep(10)


if __name__ == '__main__':
    main()
