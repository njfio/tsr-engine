"""Independently reread receipts from the offline native-boundary checks."""
from pathlib import Path
import hashlib
import importlib.util
import json
import shutil
import sqlite3
ROOT=Path(__file__).resolve().parents[1]
def main():
    s=importlib.util.spec_from_file_location('c',ROOT/'scripts/m3-two-model-v2-controller.py');c=importlib.util.module_from_spec(s);s.loader.exec_module(c)
    m=json.loads((ROOT/'docs/m3-two-model-v2-freeze.json').read_text());ops=c.Operations(m);c.old.verify_freeze(m)
    base=ROOT/'target/m3-two-model-v2-native-check';dest=ROOT/'docs/m3-two-model-v2-native-evidence';dest.mkdir();results=[]
    for i,arm in enumerate(m['arms']):
        source=base/f'out-{i}';d=dest/str(i);d.mkdir();label=arm['task'][:12]+'-record'
        shutil.copytree(source/'cas',d/label/'cas')
        for p in (d/label/'cas').iterdir():
            if p.is_symlink() or ops.digest(p.read_bytes())!=p.name:raise ValueError('CAS mismatch')
        stores=list((source/'state').glob('*/*.sqlite'));assert len(stores)==1
        with sqlite3.connect(stores[0].as_uri()+'?mode=ro',uri=True) as src,sqlite3.connect(d/(label+'-state.sqlite')) as dst:src.backup(dst);dst.execute('pragma journal_mode=DELETE')
        shutil.copyfile(ROOT/arm['input']/'tasks.jsonl',d/(arm['task'][:12]+'-tasks.jsonl'))
        shutil.copyfile(source/'results-record.jsonl',d/'results.jsonl')
        shutil.copyfile(base/f'arm-{i}-boundary.jsonl',d/'boundary.jsonl')
        result=json.loads((d/'results.jsonl').read_text());checked=ops.receipts.verify_receipts(d,label,dict(result=result,task=arm['task'],mode='record'))
        assert checked['status']=='checked' and checked['verified_success']
        (d/'receipt-verification.json').write_text(json.dumps(checked,indent=2)+'\n');results.append(dict(index=i,model=arm['model'],**checked))
    (dest/'summary.json').write_text(json.dumps(results,indent=2)+'\n')
    (ROOT/'docs/m3-two-model-v2-native-evidence-sha256.json').write_text(json.dumps({str(p.relative_to(dest)):hashlib.sha256(p.read_bytes()).hexdigest() for p in dest.rglob('*') if p.is_file()},indent=2)+'\n')
    print(json.dumps(dict(verified_scripted_receipts=len(results),model_inference=False)))
if __name__=='__main__':main()
