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
PLAN = ROOT / 'docs/m3-terminal-build-cleanup-expanded-plan.json'
EXPECTED = 'd79ed06404b4eaf9255ec2db695ed951be0768b515f386b51982350e06be4363'
RECEIPT = ROOT / 'docs/m3-terminal-build-cleanup-expanded-result.json'


def real_directory(path):
    if not stat.S_ISDIR(path.lstat().st_mode):
        raise RuntimeError(f'Not a real directory: {path}')


def check_identity(path, row):
    real_directory(path)
    s = path.lstat()
    if (s.st_dev, s.st_ino) != (row['device'], row['inode']):
        raise RuntimeError(f'Directory identity changed: {path}')


def verify_target(row):
    relative = row['path']
    parts = Path(relative).parts
    if row['kind'] == 'terminal-qualification-build-output':
        if len(parts) != 5 or parts[0] != 'target' or parts[3:] != ('copy', 'target') or not re.fullmatch(r'm3-qualified-[a-z0-9-]+', parts[1]) or not re.fullmatch(r'[a-f0-9]{64}', parts[2]):
            raise RuntimeError('Unexpected qualification build path')
    elif row['kind'] == 'terminal-offline-resolver-probe-build-output':
        if relative != 'target/m3-supplementary-compatibility/target':
            raise RuntimeError('Unexpected probe path')
    else:
        raise RuntimeError('Unsupported target kind')
    current = ROOT
    for part in parts:
        current = current / part
        real_directory(current)
    check_identity(current, row)
    if current.parent.stat().st_ino != row['parent_inode']:
        raise RuntimeError('Parent identity changed')
    receipt = ROOT / row['receipt']
    if hashlib.sha256(receipt.read_bytes()).hexdigest() != row['receipt_sha256']:
        raise RuntimeError('Terminal receipt changed')
    recorded = json.loads(receipt.read_text())
    if row['kind'] == 'terminal-qualification-build-output':
        complete = ROOT / row['cohort_complete']
        if hashlib.sha256(complete.read_bytes()).hexdigest() != row['cohort_sha256'] or json.loads(complete.read_text()).get('status') != 'completed':
            raise RuntimeError('Cohort not cleanly terminal')
        argv = recorded.get('argv', [])
        if recorded.get('status') != 'completed' or recorded.get('exit') != 0 or recorded.get('errors') != [] or 'qualify' not in argv or '--out' not in argv:
            raise RuntimeError('Qualification not terminal')
        if argv[argv.index('--out') + 1] != str(ROOT / 'target' / parts[1]):
            raise RuntimeError('Qualification output mismatch')
    elif recorded.get('probe_exit') != 0:
        raise RuntimeError('Probe not terminal')
    if not (current / 'debug').is_dir():
        raise RuntimeError('Build directory shape changed')
    return current


def protected_snapshot():
    plan = json.loads(PLAN.read_text())
    path = ROOT / plan['protected_snapshot']
    if hashlib.sha256(path.read_bytes()).hexdigest() != plan['protected_snapshot_sha256']:
        raise RuntimeError('Protected snapshot changed')
    snapshot = json.loads(path.read_text())
    for relative, want in snapshot['source_entries'].items():
        path = ROOT / relative
        if 'symlink' in want:
            if not path.is_symlink() or os.readlink(path) != want['symlink']:
                raise RuntimeError('Protected source link changed: ' + relative)
        elif path.is_symlink() or not path.is_file() or hashlib.sha256(path.read_bytes()).hexdigest() != want['sha256']:
            raise RuntimeError('Protected source changed: ' + relative)
    for relative, expected in snapshot['export_sha256'].items():
        if hashlib.sha256((ROOT / relative).read_bytes()).hexdigest() != expected:
            raise RuntimeError('Historical export changed: ' + relative)
    return len(snapshot['source_entries'])


def idle():
    outer = subprocess.check_output(['ps', '-p', '29246', '-o', 'state=,lstart=,command='], text=True).strip()
    expected = '/opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python -B ' + str(ROOT / 'scripts/m3-following-native-expansion.py')
    if not outer.startswith('T') or 'Wed Oct 14 21:19:49 2026' not in outer or not outer.endswith(expected):
        raise RuntimeError('Outer coordinator identity/state changed')
    rows = subprocess.check_output(['ps', '-axo', 'command='], text=True).splitlines()
    if any(x.startswith(str(ROOT / 'target/m3-qualification-engine-')) for x in rows):
        raise RuntimeError('Qualification process active')
    names = subprocess.check_output(['docker', 'ps', '--format', '{{.Names}}'], text=True, timeout=15).splitlines()
    if any(n.startswith('tsr-m3-') for n in names):
        raise RuntimeError('Task container active')
    if json.loads((ROOT / 'target/m3-fuzzy-continuation-state/complete.json').read_text()).get('status') != 'completed':
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
    if len(records) != 9424:
        raise RuntimeError('Expected exactly 9424 evidence files')
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
    if len(rows) != 37 or len({x['path'] for x in rows}) != 37:
        raise RuntimeError('Expected 37 unique exact paths')
    audit = json.loads((ROOT / 'docs/m3-fuzzy-continuation-final-audit.json').read_text())
    if audit.get('exit') != 0 or audit.get('admissions_checked') != 107 or hashlib.sha256((ROOT / 'docs/m3-native-admission/summary.json').read_bytes()).hexdigest() != audit.get('summary_sha256'):
        raise RuntimeError('107-admission audit is not current')
    idle()
    protected_count = protected_snapshot()
    before_evidence = evidence()
    for row in rows:
        check_identity(verify_target(row), row)
    before_sources = source_inventory(rows)
    if not args.execute_approved:
        print(json.dumps(dict(status='preflight-passed', targets=37, evidence_files=len(before_evidence), source_entries=len(before_sources), protected_entries=protected_count)))
        return
    if not shutil.rmtree.avoids_symlink_attacks or RECEIPT.exists():
        raise RuntimeError('Unsafe deletion platform or prior receipt; refuse replay')
    (ROOT / 'target/m3-cleanup-expanded-owned').mkdir()
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
                if os.fstat(fd).st_ino != row['parent_inode']:
                    raise RuntimeError('Pinned parent identity changed')
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
        protected_snapshot()
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
