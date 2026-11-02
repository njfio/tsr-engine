"""Copy bounded original and v6 evidence, preserving failed-preflight provenance."""
import hashlib
import json
from pathlib import Path
import shutil
ROOT=Path(__file__).resolve().parents[1]
def main():
    replay=ROOT/'target/m3-fixed12-v6-replay';summary=json.loads((replay/'summary.json').read_text());assert len(summary['checks'])==36
    dest=ROOT/'docs/m3-fixed12-native-evidence';dest.mkdir();shutil.copyfile(replay/'summary.json',dest/'summary.json')
    for name in ('m3-fixed12-native-check','m3-fixed12-native-check-v2'):
        group=dest/name;group.mkdir()
        for file in ('boundary.jsonl','ledger.jsonl','ledger-diagnostics.jsonl','baseline.json'):
            shutil.copyfile(ROOT/'target'/name/file,group/file)
    for row in summary['checks']:
        label=str(row['index'])+('-blocked' if row['blocked'] else '')
        for folder,source,key in [('native',ROOT/row['source_base'],row['evidence_sha256']),('v6-replay',replay,row['v6_evidence_sha256'])]:
            source=source/label/'evidence';assert hashlib.sha256((source/'complete-sha256.json').read_bytes()).hexdigest()==key
            target=dest/folder/label;target.parent.mkdir(exist_ok=True);shutil.copytree(source,target)
            for name,h in json.loads((target/'complete-sha256.json').read_text()).items():assert hashlib.sha256((target/name).read_bytes()).hexdigest()==h
    zero=ROOT/'target/m3-fixed12-zero-native'
    z=json.loads((zero/'summary.json').read_text());assert z['status']=='passed' and z['provider_calls']==0
    shutil.copytree(zero/'evidence',dest/'zero-native'/'evidence')
    shutil.copyfile(zero/'summary.json',dest/'zero-native'/'summary.json')
    hashes={str(p.relative_to(dest)):hashlib.sha256(p.read_bytes()).hexdigest() for p in dest.rglob('*') if p.is_file()}
    (dest/'sha256.json').write_text(json.dumps(hashes,indent=2)+'\n');print(json.dumps(dict(files=len(hashes),native_cases=36,replays=36,provider_calls=0)))
if __name__=='__main__':main()
