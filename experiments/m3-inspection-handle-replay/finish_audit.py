"""Read-only verification and durable export of completed synthetic evidence."""
from pathlib import Path
import json,subprocess,sqlite3,shutil,tempfile,hashlib
from replay import replay
ROOT=Path(__file__).resolve().parents[2];BASE=ROOT/'target/m3-handle-replay-native';DEST=ROOT/'docs/m3-inspection-handle-replay-evidence'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    DEST.mkdir();rows=json.loads((BASE/'summary.json').read_text());assert len(rows)==8
    audit=[]
    for row in rows:
        case=row['case'];src=BASE/case;dst=DEST/case;dst.mkdir()
        for name in ('events.json','boundary.jsonl','native.log','codec-rejection.json','issued-handle.txt'):
            if (src/name).exists():shutil.copyfile(src/name,dst/name)
        shutil.copytree(src/'input',dst/'input')
        (dst/'out').mkdir();shutil.copytree(src/'out/cas',dst/'out/cas')
        result=src/'out'/('results-'+row['mode']+'.jsonl')
        if result.exists():shutil.copyfile(result,dst/'out'/result.name)
        stores=list((src/'out/state').glob('*/*.sqlite'));assert len(stores)==1
        with sqlite3.connect(stores[0].as_uri()+'?mode=ro',uri=True) as db,sqlite3.connect(dst/'state.sqlite') as copy:
            for identity,payload in db.execute('select id,payload from events order by seq'):assert identity==json.loads(payload)['id']
            db.backup(copy)
        command=[str(ROOT/'target/m3-handle-replay-audit'),str(dst),row['mode']]
        check=subprocess.run(command,capture_output=True,text=True,timeout=30)
        (dst/'audit.log').write_text(check.stdout+check.stderr);assert check.returncode==0,case
        got=json.loads(check.stdout)
        got.update(replay(dst))
        if case in ('record','rules'):assert got['oracle_receipts_verified'] and got['bound_edits']==1
        if case in ('multiple','truncation'):assert got['bound_edits']==got['issuance_receipts']==0
        if case in ('foreign','wrong_path'):assert got['bound_edits']==0 and got['issuance_receipts']==1
        if case=='stale':assert got['bound_edits']==1
        if case=='protected_test':
            events=json.loads((dst/'events.json').read_text());closures=[e['kind']['reason'] for e in events if e['kind']['kind']=='task_closed']
            assert closures==['tamper:changed full-suite test resource `tests/answer.rs`']
            value=json.loads(result.read_text());assert value['oracle'] is None and value['outcome']=='rejected'
            got['protected_test_rejection']=closures[0]
        audit.append(row|got)
    # Tampered exported issuance text must fail canonical event-chain verification.
    with tempfile.TemporaryDirectory(prefix='m3-handle-tamper-') as tmp:
        d=Path(tmp);shutil.copytree(DEST/'record/input',d/'input');shutil.copyfile(DEST/'record/boundary.jsonl',d/'boundary.jsonl')
        events=json.loads((DEST/'record/events.json').read_text())
        evidence=next(e['kind']['observation']['evidence'] for e in events if e['kind'].get('observation',{}).get('evidence',{}).get('source','').startswith('broker/inspection-handle-receipt@v1:'))
        evidence['source']=evidence['source'].replace('src/lib.rs','src/wrong.rs');(d/'events.json').write_text(json.dumps(events))
        check=subprocess.run([str(ROOT/'target/m3-handle-replay-audit'),str(d),'record'],capture_output=True,text=True,timeout=30)
        assert check.returncode!=0
        (DEST/'tampered-issuance-audit.log').write_text(check.stdout+check.stderr)
    assert (BASE/'offline-ledger.jsonl').read_bytes()==b''
    freeze=json.loads((ROOT/'docs/m3-fixed12-freeze.json').read_text())['sha256'];assert all(sha(ROOT/p)==h for p,h in freeze.items())
    result=dict(groups=7,arms=8,checks=audit,tampered_issuance_rejected=True,frozen_pins_unchanged=len(freeze),paid_dispatches=0,free_bytes=shutil.disk_usage(ROOT).free,limits=['Synthetic fixtures only; no model reliability claim.','Freshness replay uses controller source-state checkpoints, not independent reexecution of candidate commands.','Protected-test tamper is refused before final oracle invocation.'])
    (DEST/'summary.json').write_text(json.dumps(result,indent=2)+'\n')
    pins={str(p.relative_to(DEST)):sha(p) for p in DEST.rglob('*') if p.is_file()};(DEST/'sha256.json').write_text(json.dumps(pins,indent=2)+'\n')
    print(json.dumps(result,indent=2))
if __name__=='__main__':main()
