"""Re-export all distinct native cases with final controller, retaining the failed first classifier."""
import hashlib
import importlib.util
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
def main():
    first=ROOT/'target/m3-fixed12-native-check';second=ROOT/'target/m3-fixed12-native-check-v2'
    rows=[json.loads(l) for l in (ROOT/'docs/m3-validation/m3-fixed12-native.log').read_text().splitlines() if l.startswith('{')]
    assert len(rows)==24 and {r['index'] for r in rows}==set(range(24))
    old=first/'1-blocked/evidence'
    rows.append(dict(index=1,mode='rules',blocked=True,outcome='rejected',calls=1,evidence_sha256=hashlib.sha256((old/'complete-sha256.json').read_bytes()).hexdigest(),original_classifier_status='failed-proposal-diagnostic-alignment'))
    cases=[r|dict(source_base=str(first.relative_to(ROOT))) for r in rows]
    more=json.loads((second/'summary.json').read_text());assert more['provider_calls']==0 and more['shared_ledger_unchanged']
    cases.extend(r|dict(source_base=str(second.relative_to(ROOT))) for r in more['checks'])
    m=json.loads((first/'offline-manifest.json').read_text())
    expected={(i,False) for i in range(24)}|{(a['index'],True) for a in m['arms'] if a['mode']=='rules'}
    assert len(cases)==36 and {(r['index'],r['blocked']) for r in cases}==expected
    spec=importlib.util.spec_from_file_location('controller',ROOT/'scripts/m3-fixed12-v5-controller.py');c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)
    dest=ROOT/'target/m3-fixed12-v5-replay';dest.mkdir();previous={};results=[]
    for row in cases:
        label=str(row['index'])+('-blocked' if row['blocked'] else '')
        state=ROOT/row['source_base']/label;prior=state/'evidence';out=dest/label;out.mkdir()
        assert hashlib.sha256((prior/'complete-sha256.json').read_bytes()).hexdigest()==row['evidence_sha256']
        for name,h in json.loads((prior/'complete-sha256.json').read_text()).items():assert hashlib.sha256((prior/name).read_bytes()).hexdigest()==h
        ledger=(prior/'ledger.jsonl').read_bytes();(out/'ledger.jsonl').write_bytes(ledger)
        ds=json.loads((prior/'diagnostics.json').read_text());(out/'ledger-diagnostics.jsonl').write_text(''.join(json.dumps(d)+'\n' for d in ds))
        ops=c.Operations(m|dict(ledger=str((out/'ledger.jsonl').relative_to(ROOT))));ops.attempt_state=state;ops.diagnostic_before=b''
        arm=m['arms'][row['index']]|dict(input=str((state/'input').relative_to(ROOT)));assert row['mode']==arm['mode']
        checked=ops.export_and_verify(arm,json.loads((prior/'process.json').read_text()),out/'evidence')
        ids=c.bind_accounting(previous.get(row['source_base'],b''),ledger,checked['diagnostics']);previous[row['source_base']]=ledger
        assert c.classify(checked,{})==row['outcome'] and len(ids)==row['calls']
        results.append(row|dict(v5_evidence_sha256=checked['manifest_sha256']))
    (dest/'summary.json').write_text(json.dumps(dict(checks=results,provider_calls=0,new_native_workloads=0,original_preflight_status='failed-after-24-inspection-and-first-blocked-native-execution',original_failure_retained=True),indent=2)+'\n')
    print(json.dumps(dict(replayed=len(results),provider_calls=0)))
if __name__=='__main__':main()
