"""Bounded local history validation; executes argv only in a sealed Docker worker."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import threading
import time
import uuid

p = argparse.ArgumentParser()
p.add_argument('--image-file', required=True)
p.add_argument('--copy', required=True)
p.add_argument('--out', required=True)
p.add_argument('--seconds', type=int, default=180)
p.add_argument('--tmp-exec', action='store_true', help='permit generated test executables on container-local tmpfs')
p.add_argument('argv', nargs=argparse.REMAINDER)
a = p.parse_args()
image = Path(a.image_file).read_text().strip()
if not re.fullmatch(r'sha256:[0-9a-f]{64}', image):
    raise ValueError('immutable local image ID required')
copy = Path(a.copy).resolve(strict=True)
if ',' in str(copy) or not a.argv or not 0 < a.seconds <= 600:
    raise ValueError('invalid bounded probe')
argv = a.argv[1:] if a.argv[0] == '--' else a.argv
name = 'tsr-m3-probe-' + uuid.uuid4().hex[:16]
command = ['docker', 'run', '--rm', '--init', '--pull=never', '--name', name,
    '--network=none', '--read-only', '--cap-drop=ALL', '--security-opt=no-new-privileges',
    '--pids-limit=256', '--memory=2g', '--cpus=2', '--user', '65534:65534',
    '--tmpfs=/tmp:rw,nosuid,nodev,' + ('exec,' if a.tmp_exec else '') + 'size=256m', '--mount', f'type=bind,src={copy},dst={copy}',
    '--workdir', str(copy), '--env', 'CARGO_INCREMENTAL=0', '--entrypoint=', image] + argv
output = bytearray()
excess = threading.Event()
def capture(stream):
    while chunk := stream.read(4096):
        if len(output) + len(chunk) > 1024 * 1024:
            excess.set()
            return
        output.extend(chunk)
start = time.monotonic()
child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL)
reader = threading.Thread(target=capture, args=(child.stdout,), daemon=True)
reader.start()
while child.poll() is None and time.monotonic() - start < a.seconds and not excess.is_set():
    time.sleep(.1)
code = child.poll()
if code is None or excess.is_set():
    code = 125 if excess.is_set() else 124
    subprocess.run(['docker', 'rm', '--force', name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=5)
    child.kill()
child.wait(timeout=5)
reader.join(timeout=5)
record = {'command': command, 'exit': code, 'seconds': time.monotonic()-start,
    'output': output.decode(errors='replace'), 'output_truncated': excess.is_set(),
    'qualifying_tasks': 0}
lock = copy/'Cargo.lock'
if lock.is_file(): record['prepared_lock_sha256'] = hashlib.sha256(lock.read_bytes()).hexdigest()
Path(a.out).write_text(json.dumps(record, indent=2)+'\n')
print(json.dumps({k:v for k,v in record.items() if k not in ('output','command')}))
print(record['output'][-3000:])
