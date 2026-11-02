"""Complete production controller export/classification checks on offline native outputs."""
from pathlib import Path
import hashlib,json,shutil
import controller as c
ROOT=c.ROOT;BASE=ROOT/'target/m3-handle-live-offline-native-v3'
def main():
    rows=[]
    for index in (0,1):
        directory=BASE/str(index);m=json.loads((BASE/f'freeze-{index}.json').read_text());arm=m['arms'][index]
        ops=c.Operations(m);ops.diagnostic_before=b'';ops.attempt_state=directory
        shutil.copyfile(directory/'native.log',directory/(arm['task'][:12]+'-record.log'))
        process=dict(exit=0,status='completed',started='2026-10-18T00:00:00+00:00',finished='2026-10-18T00:00:01+00:00',note='Export-check metadata only; native outcome and ledger come from actual offline run.')
        checked=ops.export_and_verify(arm,process,directory/'controller-evidence')
        baseline=json.loads((ROOT/m['baseline']).read_text());after=ops.ledger();ids=c.bind_accounting(after[:baseline['bytes']],after,checked['diagnostics'])
        outcome=c.classify(checked,process)
        row=dict(interface=arm['interface'],classification=outcome,settled_fake_requests=len(ids),export_manifest_sha256=checked['manifest_sha256'])
        if index==1:row['freshness']=c.verify_handle_evidence(arm,checked,directory/'controller-evidence'/'freshness')
        assert outcome=='rejected' and checked['result']['oracle'] is not None and len(ids)==3
        rows.append(row)
    (BASE/'controller-summary.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(rows,indent=2))
if __name__=='__main__':main()
