"""One-time exact fixed12 build inventory cleanup, with immutable evidence snapshots."""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT/'docs/m3-fixed12-approved-cleanup'
INVENTORY = 'docs/m3-fixed12-reclaimable-inventory.json'

def digest(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def targets():
    raw = (ROOT/INVENTORY).read_bytes()
    assert raw == subprocess.check_output(['git','show','bed6045:'+INVENTORY],cwd=ROOT)
    rows = json.loads(raw)['rows']
    result = []
    pins = json.loads((ROOT/'docs/m3-fixed12-freeze.json').read_text())['sha256']
    for row in rows:
        if row['build'] is None:
            continue
        p = ROOT/row['build']
        expected = ROOT/row['workspace']/'target'
        assert p == expected and p.name == 'target'
        assert p.is_relative_to(ROOT/'target/m3-fixed12-inputs')
        for part in [p, *p.parents]:
            assert not part.is_symlink(), str(part)
            if part == ROOT: break
        assert p.resolve() == p and p.is_dir()
        decision = json.loads((ROOT/f"target/m3-fixed12-state/{row['index']}/decision.json").read_text())
        assert decision['outcome'] == row['outcome'] and decision['mode'] == row['mode']
        assert not any((ROOT/key).resolve().is_relative_to(p) for key in pins)
        result.append(p)
    assert len(result) == len(set(result)) == 20
    return result

def protected(paths):
    pins = json.loads((ROOT/'docs/m3-fixed12-freeze.json').read_text())['sha256']
    result = {}
    for name, expected in pins.items():
        p = ROOT/name
        assert digest(p) == expected, name
        result[name] = expected
    # Cover all experiment source, receipts, CAS, logs, ledgers and evidence; only approved builds excluded.
    bases = [ROOT/'target/m3-fixed12-inputs', ROOT/'target/m3-fixed12-state', ROOT/'docs/m3-fixed12-execution']
    for base in bases:
        for current, dirs, files in os.walk(base, followlinks=False):
            current = Path(current)
            dirs[:] = [d for d in dirs if current/d not in paths]
            for d in dirs: assert not (current/d).is_symlink()
            for name in files:
                p = current/name
                assert not p.is_symlink() and p.is_file()
                result[str(p.relative_to(ROOT))] = digest(p)
    for p in (ROOT/'target').glob('m3-provider-cost*.jsonl'):
        result[str(p.relative_to(ROOT))] = digest(p)
    return result

def save(path, value):
    with path.open('x') as f: json.dump(value,f,indent=2); f.write('\n')

def main():
    paths = targets()
    if sys.argv[1:] == ['prepare']:
        DEST.mkdir()
        save(DEST/'before.json',dict(free_bytes=shutil.disk_usage(ROOT).free,
            inventory_sha256=digest(ROOT/INVENTORY),targets=[str(p.relative_to(ROOT)) for p in paths],
            protected=protected(paths),identities={str(p.relative_to(ROOT)):[p.stat().st_dev,p.stat().st_ino] for p in paths}))
        print('Prepared exact20-path scope and protected hashes; nothing deleted.')
    elif sys.argv[1:] == ['delete-approved-inventory']:
        before=json.loads((DEST/'before.json').read_text())
        review=json.loads((DEST/'scope-review.json').read_text())
        active=json.loads((DEST/'active-use-check.json').read_text())
        assert review['approved_scope'] == before['targets'] and review['safe'] is True
        assert active['safe'] is True and active['targets'] == before['targets']
        age = (datetime.now(timezone.utc) - datetime.fromisoformat(active['at'])).total_seconds()
        assert 0 <= age <= 60, 'active-use check must be fresh within60seconds'
        assert not (DEST/'deletion-intent.json').exists(), 'one-shot cleanup already attempted'
        assert before['protected'] == protected(paths)
        assert before['identities'] == {str(p.relative_to(ROOT)):[p.stat().st_dev,p.stat().st_ino] for p in paths}
        save(DEST/'deletion-intent.json',dict(targets=before['targets'],free_bytes=shutil.disk_usage(ROOT).free))
        for p in paths: shutil.rmtree(p)
        assert before['protected'] == protected(paths)
        save(DEST/'after.json',dict(free_bytes=shutil.disk_usage(ROOT).free,protected_hashes_unchanged=len(before['protected']),removed=before['targets']))
        print('Removed exact20 approved build directories; protected hashes unchanged.')
    else: raise ValueError('explicit prepare or delete-approved-inventory required')

if __name__ == '__main__': main()
