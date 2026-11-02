"""Read-only filesystem inventory; writes a new proposal, never deletes anything."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess

ROOT = Path(__file__).resolve().parents[1]
PLAN = ROOT / 'docs/m3-post-context-cleanup-plan.json'
SNAPSHOT = ROOT / 'docs/m3-post-context-cleanup-protected.json'
FAMILIES = ('syntax', 'typed', 'reference', 'context')

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def target_shape(relative):
    p = Path(relative).parts
    return len(p) == 5 and p[0] == 'target' and p[3:] == ('copy', 'target') and bool(re.fullmatch(r'm3-qualified-rts-(syntax|typed|reference|context)-continuation-[a-f0-9]{12}', p[1])) and bool(re.fullmatch('[a-f0-9]{64}', p[2]))

def real_ancestors(path):
    current = ROOT
    for part in path.relative_to(ROOT).parts:
        current /= part
        if not stat.S_ISDIR(current.lstat().st_mode):
            raise RuntimeError('Non-directory or symlink ancestor: ' + str(current))

def inventory_tree(root, excluded, records):
    for directory, dirs, files in os.walk(root, followlinks=False):
        dirs[:] = [n for n in dirs if Path(directory) / n not in excluded]
        for n in files + [n for n in dirs if (Path(directory) / n).is_symlink()]:
            p = Path(directory) / n
            records[str(p.relative_to(ROOT))] = dict(symlink=os.readlink(p)) if p.is_symlink() else dict(sha256=digest(p))

def main():
    if PLAN.exists() or SNAPSHOT.exists():
        raise RuntimeError('Refuse to replace frozen proposal')
    prior = ROOT / 'docs/m3-terminal-build-cleanup-expanded-result.json'
    result = json.loads(prior.read_text())
    assert result['status'] == 'completed'
    cutoff = datetime.fromisoformat(result['finished']).timestamp()
    rows = []
    for family in FAMILIES:
        state = ROOT / ('target/m3-' + family + '-continuation-state')
        complete = state / 'complete.json'
        assert json.loads(complete.read_text())['status'] == 'completed'
        for receipt in sorted(state.glob('*.json')):
            record = json.loads(receipt.read_text());argv = record.get('argv', [])
            if 'qualify' not in argv:
                continue
            assert record['status'] == 'completed' and record['exit'] == 0 and record['errors'] == []
            out = Path(argv[argv.index('--out') + 1])
            targets = list(out.glob('*/copy/target'));assert len(targets) == 1
            target = targets[0];relative = str(target.relative_to(ROOT));assert target_shape(relative)
            real_ancestors(target);s = target.stat();assert s.st_birthtime > cutoff
            assert receipt.stem == target.parent.parent.name and out.name.endswith(receipt.stem[:12])
            assert (target / 'debug').is_dir()
            size = int(subprocess.check_output(['du', '-sk', str(target)], text=True).split()[0]) * 1024
            rows.append(dict(path=relative,kind='terminal-qualification-build-output',device=s.st_dev,inode=s.st_ino,parent_inode=target.parent.stat().st_ino,birth_utc=datetime.fromtimestamp(s.st_birthtime,timezone.utc).isoformat(),allocated_bytes_estimate=size,receipt=str(receipt.relative_to(ROOT)),receipt_sha256=digest(receipt),cohort_complete=str(complete.relative_to(ROOT)),cohort_sha256=digest(complete)))
    assert len(rows) == 35 and len({v['path'] for v in rows}) == 35
    excluded = {ROOT / v['path'] for v in rows};records = {}
    for v in rows:
        # Entire attempt except its compiler output: includes source, CAS and oracle evidence.
        inventory_tree((ROOT / v['path']).parents[2], excluded, records)
    for family in FAMILIES:
        inventory_tree(ROOT / ('target/m3-' + family + '-continuation-state'), excluded, records)
    for repo in ['target/m3-history/repo', 'target/m3-second-source/repo']:
        assert not subprocess.check_output(['git','-C',str(ROOT/repo),'status','--porcelain'],text=True).strip()
        inventory_tree(ROOT/repo, excluded, records)
    exports = {}
    for attempt in (ROOT/'docs/m3-native-admission').iterdir():
        if not attempt.is_dir():continue
        for p in attempt.rglob('*'):
            if p.is_file():
                q=ROOT/'target'/attempt.name/p.relative_to(attempt)
                assert not p.is_symlink() and not q.is_symlink() and p.read_bytes()==q.read_bytes()
                exports[str(p.relative_to(ROOT))]=digest(p)
    assert len(exports)==11950
    assert not any(Path(p)==Path(v['path']) or Path(v['path']) in Path(p).parents for p in [*records,*exports] for v in rows)
    snap=dict(source_and_evidence_entries=records,export_sha256=exports)
    SNAPSHOT.write_text(json.dumps(snap,indent=2,sort_keys=True)+'\n')
    total=sum(v['allocated_bytes_estimate'] for v in rows)
    plan=dict(recorded_at=datetime.now(timezone.utc).isoformat(),status='proposal-only-unapproved-unexecuted',cutoff_utc=result['finished'],prior_cleanup_receipt=str(prior.relative_to(ROOT)),prior_cleanup_receipt_sha256=digest(prior),targets=rows,target_count=len(rows),allocated_bytes_estimate=total,estimated_reclaim_gib=total/2**30,free_gib=shutil.disk_usage(ROOT).free/2**30,protected_snapshot=str(SNAPSHOT.relative_to(ROOT)),protected_snapshot_sha256=digest(SNAPSHOT),protected_entries=len(records),export_files=len(exports),summary_sha256=digest(ROOT/'docs/m3-native-admission/summary.json'),audit='docs/m3-context-continuation-final-audit.json',audit_sha256=digest(ROOT/'docs/m3-context-continuation-final-audit.json'),excluded=['All source and source copies outside exact copy/target paths','All oracle evidence, CAS, logs, rejection and unviable records','Dependency caches, container images, original repositories, prior completed cleanup targets','All build directories born before the latest fulfilled cleanup cutoff'],limitations=['Allocated sizes from du; APFS shared extents can reduce actual reclaimed space.','Approval applies only to exact frozen paths; future executor must recheck quiescence, identities, terminal receipts and protected bytes.','No deletion implementation or execution is part of this proposal.'])
    PLAN.write_text(json.dumps(plan,indent=2)+'\n')
    print(json.dumps(dict(targets=len(rows),estimated_reclaim_gib=total/2**30,protected_entries=len(records),export_files=len(exports),manifest_sha256=digest(PLAN))))

if __name__ == '__main__':
    main()
