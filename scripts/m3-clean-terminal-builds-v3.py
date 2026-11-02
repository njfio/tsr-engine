"""Exact approved cleanup; dry-run by default, no unrelated deletion."""
import argparse
from pathlib import Path
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[1]
PLAN = ROOT / 'docs/m3-terminal-build-cleanup-v3-plan.json'
EXPECTED = '6dde275186b872e76bcb95188b2f5b843d5e9c5c3d16c54a395696e9cbe4d00d'
RECEIPT = ROOT / 'docs/m3-terminal-build-cleanup-v3-result.json'


def real_directory(path):
    if not stat.S_ISDIR(path.lstat().st_mode):
        raise RuntimeError(f'Not a real directory: {path}')


def check_identity(path, row):
    real_directory(path)
    s = path.lstat()
    if (s.st_dev, s.st_ino) != (row['device'], row['inode']):
        raise RuntimeError(f'Directory identity changed: {path}')


def verify_target(row):
    name, task = row['attempt'], row['task']
    if not re.fullmatch(r'm3-qualified-[a-z0-9-]+', name) or not re.fullmatch(r'[a-f0-9]{64}', task):
        raise RuntimeError('Invalid target identity')
    expected = Path('target') / name / task / 'copy' / 'target'
    if row['path'] != str(expected):
        raise RuntimeError('Path does not match target identity')
    current = ROOT
    for part in expected.parts:
        current = current / part
        real_directory(current)
    source = ROOT / 'target' / name
    exported = ROOT / 'docs/m3-native-admission' / name
    reports = [json.loads(line) for line in (source / 'admission.jsonl').read_text().splitlines()]
    matches = [r for r in reports if r.get('task_id') == task]
    if len(matches) != 1 or not matches[0].get('accepted') or matches[0].get('protocol') != 'baseline-full-green/named-red/full-red/exact-reversal/full-green-v1':
        raise RuntimeError('Not one accepted strict qualification')
    count = 0
    for directory, dirs, files in os.walk(source, followlinks=False):
        dirs[:] = [n for n in dirs if n != 'copy']
        for n in dirs:
            real_directory(Path(directory) / n)
        for n in files:
            original = Path(directory) / n
            copied = exported / original.relative_to(source)
            if not stat.S_ISREG(original.lstat().st_mode) or not stat.S_ISREG(copied.lstat().st_mode):
                raise RuntimeError('Nonregular evidence')
            if hashlib.sha256(original.read_bytes()).digest() != hashlib.sha256(copied.read_bytes()).digest():
                raise RuntimeError('Evidence export differs')
            count += 1
    if count != row['verified_evidence_files']:
        raise RuntimeError('Evidence inventory changed')
    return ROOT / expected



def idle():
    outer = subprocess.check_output(['ps', '-p', '29246', '-o', 'state=,lstart=,command='], text=True).strip()
    expected = '/opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python -B ' + str(ROOT / 'scripts/m3-following-native-expansion.py')
    if not outer.startswith('T') or 'Wed Oct 14 21:19:49 2026' not in outer or not outer.endswith(expected):
        raise RuntimeError('Outer coordinator identity/state changed')
    rows = subprocess.check_output(['ps', '-axo', 'command='], text=True).splitlines()
    if any(x.startswith(str(ROOT / 'target/m3-qualification-engine-238ac39') + ' ') for x in rows):
        raise RuntimeError('Qualification process active')
    names = subprocess.check_output(['docker', 'ps', '--format', '{{.Names}}'], text=True, timeout=15).splitlines()
    if any(n.startswith('tsr-m3-') for n in names):
        raise RuntimeError('Task container active')
    if not (ROOT / 'target/m3-rts-structure-cohort-state/complete.json').exists():
        raise RuntimeError('Cohort not terminal')


def evidence():
    records = {}
    base = ROOT / 'docs/m3-native-admission'
    for attempt in base.iterdir():
        if not attempt.is_dir():
            continue
        for p in attempt.rglob('*'):
            if p.is_file():
                source = ROOT / 'target' / attempt.name / p.relative_to(attempt)
                if p.is_symlink() or source.is_symlink() or p.read_bytes() != source.read_bytes():
                    raise RuntimeError(f'Evidence mismatch: {p}')
                records[str(p.relative_to(ROOT))] = hashlib.sha256(p.read_bytes()).hexdigest()
    if len(records) != 5725:
        raise RuntimeError('Expected exactly 5725 evidence files')
    return records


def source_inventory(rows):
    result = {}
    for row in rows:
        copy = (ROOT / row['path']).parent
        for directory, dirs, files in os.walk(copy, followlinks=False):
            if Path(directory) == copy:
                dirs[:] = [x for x in dirs if x != 'target']
            for name in files + [x for x in dirs if (Path(directory) / x).is_symlink()]:
                p = Path(directory) / name
                result[str(p.relative_to(ROOT))] = ('symlink:' + os.readlink(p)) if p.is_symlink() else hashlib.sha256(p.read_bytes()).hexdigest()
    return result


def save(value):
    tmp = RECEIPT.with_suffix('.tmp')
    tmp.write_text(json.dumps(value, indent=2) + '\n')
    tmp.replace(RECEIPT)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--execute-approved', action='store_true')
    args = parser.parse_args()
    raw = PLAN.read_bytes()
    if hashlib.sha256(raw).hexdigest() != EXPECTED:
        raise RuntimeError('Approved manifest differs')
    rows = json.loads(raw)['targets']
    if len(rows) != 63 or len({x['path'] for x in rows}) != 63:
        raise RuntimeError('Expected 63 unique exact paths')
    idle()
    before_evidence = evidence()
    for row in rows:
        check_identity(verify_target(row), row)
    before_sources = source_inventory(rows)
    if not args.execute_approved:
        print(json.dumps(dict(status='preflight-passed', targets=63, evidence_files=len(before_evidence), source_entries=len(before_sources))))
        return
    if not shutil.rmtree.avoids_symlink_attacks or RECEIPT.exists():
        raise RuntimeError('Unsafe deletion platform or prior receipt; refuse replay')
    (ROOT / 'target/m3-cleanup-v3-owned').mkdir()
    receipt = dict(status='running', started=datetime.now(timezone.utc).isoformat(), manifest_sha256=EXPECTED,
                   before_free_bytes=shutil.disk_usage(ROOT).free, evidence_files=len(before_evidence),
                   source_entries=len(before_sources), results=[])
    save(receipt)
    try:
        for row in rows:
            idle()
            target = verify_target(row)
            check_identity(target, row)
            item = dict(path=row['path'], status='pending')
            receipt['results'].append(item)
            save(receipt)
            # Open every ancestor without following symlinks and delete relative
            # to the pinned parent, using Python's FD-safe rmtree implementation.
            fd = os.open(ROOT, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
            try:
                for part in Path(row['path']).parts[:-1]:
                    child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
                    os.close(fd)
                    fd = child
                s = os.stat('target', dir_fd=fd, follow_symlinks=False)
                if not stat.S_ISDIR(s.st_mode) or (s.st_dev, s.st_ino) != (row['device'], row['inode']):
                    raise RuntimeError('Pinned target identity changed')
                shutil.rmtree('target', dir_fd=fd)
                if os.path.lexists(target):
                    raise RuntimeError('Target still exists')
            finally:
                os.close(fd)
            item['status'] = 'removed'
            save(receipt)
        idle()
        if evidence() != before_evidence or source_inventory(rows) != before_sources:
            raise RuntimeError('Preservation verification failed')
        receipt.update(status='completed', evidence_unchanged=True, source_copies_unchanged=True)
    except Exception as error:
        receipt.update(status='stopped', error=repr(error))
        raise
    finally:
        receipt.update(finished=datetime.now(timezone.utc).isoformat(), after_free_bytes=shutil.disk_usage(ROOT).free)
        receipt['observed_reclaimed_bytes'] = receipt['after_free_bytes'] - receipt['before_free_bytes']
        save(receipt)
    print(json.dumps({k:v for k,v in receipt.items() if k != 'results'}))


if __name__ == '__main__':
    main()
