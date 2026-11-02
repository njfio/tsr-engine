"""Read-only post-export hashes, stored roots and financial reconciliation."""
from pathlib import Path
from decimal import Decimal
import hashlib
import json
import sqlite3
import subprocess
import struct
import m3_smoke_budget as budget
ROOT = Path(__file__).resolve().parents[1]

def digest(data):
    return subprocess.check_output([str(ROOT/'target/m3-blake3-checksum')],input=data).decode().strip()

def main():
    summary=json.loads((ROOT/'docs/m3-progress-validation-result.json').read_text())
    export=ROOT/'docs/m3-progress-validation-execution'
    manifest=json.loads((ROOT/'docs/m3-progress-validation-export-sha256.json').read_text())
    assert {str(p.relative_to(export)) for p in export.rglob('*') if p.is_file()} == set(manifest)
    for name,want in manifest.items():
        assert hashlib.sha256((export/name).read_bytes()).hexdigest()==want, name
    cas_files=[p for directory in list(export.glob('*-input-cas'))+list(export.glob('*/cas')) for p in directory.iterdir() if p.is_file()]
    for path in cas_files:
        assert digest(path.read_bytes())==path.name, str(path)
    import importlib.util
    spec=importlib.util.spec_from_file_location('evidence',ROOT/'scripts/m3-progress-validation-evidence.py')
    evidence=importlib.util.module_from_spec(spec);spec.loader.exec_module(evidence)
    for arm in summary['arms']:
        checked=evidence.verify_receipts(export,arm['task'][:12]+'-'+arm['mode'],arm)
        assert checked==arm['receipt_verification']
        if arm['result'] and arm['result'].get('oracle') is not None:
            assert checked['status']=='checked', 'oracle receipt integrity unresolved'
    roots=[]
    empty=digest(b'')
    for arm in summary['arms']:
        if arm['result'] is None:continue
        result_file=export/(arm['task'][:12]+'-'+arm['mode']+'-results.jsonl')
        assert hashlib.sha256(result_file.read_bytes()).hexdigest()==arm['sha256']
        assert [json.loads(line) for line in result_file.read_text().splitlines()]==[arm['result']]
        store=export/(arm['task'][:12]+'-'+arm['mode']+'-state.sqlite')
        with sqlite3.connect('file:'+str(store)+'?mode=ro&immutable=1',uri=True) as db:
            assert db.execute('pragma integrity_check').fetchone()[0]=='ok'
            ids=[r[0].encode() for r in db.execute('select id from events order by seq')]
        root=digest(b''.join(struct.pack('<Q',len(part))+part for part in ids))
        assert root==arm['result']['state_root_at_end']
        assert arm['result']['state_root_at_start']==empty
        roots.append(dict(task=arm['task'],mode=arm['mode'],events=len(ids),start=empty,end=root))
    ledger=(export/'m3-provider-cost.jsonl').read_bytes()
    assert ledger==(ROOT/'target/m3-provider-cost.jsonl').read_bytes()
    original=(ROOT/'target/m3-progress-validation-state/cost-before.jsonl').read_bytes()
    assert original==(export/'state-cost-before.jsonl').read_bytes()
    assert ledger.startswith(original)
    baseline=json.loads((ROOT/'docs/m3-progress-validation-baseline.json').read_text())
    assert budget.freeze(original)==baseline
    assert budget.freeze(ledger[:baseline['bytes']])==baseline
    settled=budget.total(ledger);delta=settled-Decimal(baseline['settled_usd'])
    assert 0<=delta<=10 and settled<=100
    assert Decimal(summary['spending']['prior_usd'])==Decimal(baseline['settled_usd'])
    assert Decimal(summary['spending']['cohort_usd'])==delta
    assert Decimal(summary['spending']['total_usd'])==settled
    assert Decimal(summary['spending']['outstanding_reserved_usd'])==0
    frozen=json.loads((ROOT/'docs/m3-progress-validation-freeze.json').read_text())['sha256']
    for name,want in frozen.items():assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==want
    result=dict(status='passed',exported_files=len(manifest),cas_blake3_verified=len(cas_files),frozen_files_unchanged=len(frozen),roots=roots,total_settled_usd=str(settled),incremental_settled_usd=str(delta),unreconciled_reservations=0,limitations=['Stored root verification hashes event IDs in order; it is not a new canonical validation of each event payload.','Oracle receipts are checked from exported controller evidence; no independent semantic proof or test replay is claimed.'])
    (ROOT/'docs/m3-progress-validation-terminal-audit.json').write_text(json.dumps(result,indent=2)+'\n')
    print(json.dumps(result,indent=2))

if __name__=='__main__':main()
