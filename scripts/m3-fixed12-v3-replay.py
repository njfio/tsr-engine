"""Re-export completed offline native cases with the final controller; no workloads/network."""
import hashlib
import importlib.util
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'target/m3-fixed12-native-check'
def main():
    summary=json.loads((BASE/'summary.json').read_text())
    assert len(summary['checks'])==36 and summary['provider_calls']==0
    expected={(i,False) for i in range(24)}
    plan=json.loads((BASE/'offline-manifest.json').read_text())
    expected|={(a['index'],True) for a in plan['arms'] if a['mode']=='rules'}
    assert {(r['index'],r['blocked']) for r in summary['checks']}==expected
    spec=importlib.util.spec_from_file_location('controller',ROOT/'scripts/m3-fixed12-v3-controller.py')
    c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)
    m=json.loads((BASE/'offline-manifest.json').read_text())
    dest=ROOT/'target/m3-fixed12-v3-replay';dest.mkdir()
    ledger_before=b'';rows=[]
    for row in summary['checks']:
        label=str(row['index'])+('-blocked' if row['blocked'] else '')
        state=BASE/label;prior=state/'evidence';out=dest/label;out.mkdir()
        hashes=json.loads((prior/'complete-sha256.json').read_text())
        assert hashlib.sha256((prior/'complete-sha256.json').read_bytes()).hexdigest()==row['evidence_sha256']
        for p,h in hashes.items():assert hashlib.sha256((prior/p).read_bytes()).hexdigest()==h
        ledger=(prior/'ledger.jsonl').read_bytes();(out/'ledger.jsonl').write_bytes(ledger)
        ds=json.loads((prior/'diagnostics.json').read_text())
        (out/'ledger-diagnostics.jsonl').write_text(''.join(json.dumps(d)+'\n' for d in ds))
        ops=c.Operations(m|dict(ledger=str((out/'ledger.jsonl').relative_to(ROOT))))
        ops.attempt_state=state;ops.diagnostic_before=b''
        arm=m['arms'][row['index']]|dict(input=str((state/'input').relative_to(ROOT)))
        assert row['mode']==arm['mode']
        checked=ops.export_and_verify(arm,json.loads((prior/'process.json').read_text()),out/'evidence')
        ids=c.bind_accounting(ledger_before,ledger,checked['diagnostics']);ledger_before=ledger
        outcome=c.classify(checked,{})
        assert outcome==row['outcome'] and len(ids)==row['calls']
        rows.append(row|dict(v3_evidence_sha256=checked['manifest_sha256']))
    (dest/'summary.json').write_text(json.dumps(dict(checks=rows,provider_calls=0,new_native_workloads=0),indent=2)+'\n')
    print(json.dumps(dict(replayed=len(rows),provider_calls=0)))
if __name__=='__main__':main()
