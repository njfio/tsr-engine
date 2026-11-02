"""Verify the frozen cleanup proposal without deleting or modifying any target."""
from pathlib import Path
import hashlib
import importlib.util
import json
import os

ROOT=Path(__file__).resolve().parents[1]
PLAN=ROOT/'docs/m3-post-context-cleanup-plan.json'
EXPECTED='d04839063db8d829e28ad27fda1418ae83275322638cabaa932bdaa7415643b4'

def main():
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    assert sha(PLAN)==EXPECTED,'Manifest differs'
    plan=json.loads(PLAN.read_text());assert len(plan['targets'])==35
    spec=importlib.util.spec_from_file_location('inventory',ROOT/'scripts/m3-prepare-post-context-cleanup.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
    assert sha(ROOT/plan['prior_cleanup_receipt'])==plan['prior_cleanup_receipt_sha256']
    assert sha(ROOT/plan['audit'])==plan['audit_sha256']
    assert sha(ROOT/'docs/m3-native-admission/summary.json')==plan['summary_sha256']
    audit=json.loads((ROOT/plan['audit']).read_text());assert audit['exit']==0 and audit['admissions_checked']==142 and audit['summary_sha256']==plan['summary_sha256']
    for row in plan['targets']:
        p=ROOT/row['path'];assert m.target_shape(row['path']);m.real_ancestors(p);s=p.stat()
        assert (s.st_dev,s.st_ino,p.parent.stat().st_ino)==(row['device'],row['inode'],row['parent_inode'])
        assert sha(ROOT/row['receipt'])==row['receipt_sha256'] and sha(ROOT/row['cohort_complete'])==row['cohort_sha256']
        receipt=json.loads((ROOT/row['receipt']).read_text());assert receipt['status']=='completed' and receipt['exit']==0 and not receipt['errors']
        assert Path(receipt['argv'][receipt['argv'].index('--out')+1])==p.parents[2]
    snap=ROOT/plan['protected_snapshot'];assert sha(snap)==plan['protected_snapshot_sha256'];data=json.loads(snap.read_text())
    for rel,want in data['source_and_evidence_entries'].items():
        p=ROOT/rel
        if 'symlink' in want:assert p.is_symlink() and os.readlink(p)==want['symlink'],rel
        else:assert not p.is_symlink() and sha(p)==want['sha256'],rel
    for rel,want in data['export_sha256'].items():
        p=ROOT/rel;assert not p.is_symlink() and sha(p)==want,rel
        parts=Path(rel).parts;source=ROOT/'target'/parts[2]/Path(*parts[3:]);assert not source.is_symlink() and source.read_bytes()==p.read_bytes(),rel
    print(json.dumps(dict(status='preservation-and-identity-check-passed',targets=35,protected_entries=len(data['source_and_evidence_entries']),verified_export_files=len(data['export_sha256']),manifest_sha256=EXPECTED,no_deletion=True)))

if __name__=='__main__':main()
