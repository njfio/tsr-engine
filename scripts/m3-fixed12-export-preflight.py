"""Copy bounded native/replay evidence, excluding candidate build directories."""
import hashlib
import json
from pathlib import Path
import shutil
ROOT=Path(__file__).resolve().parents[1]
def main():
    native=ROOT/'target/m3-fixed12-native-check';replay=ROOT/'target/m3-fixed12-v3-replay'
    original=json.loads((native/'summary.json').read_text());final=json.loads((replay/'summary.json').read_text())
    assert len(original['checks'])==len(final['checks'])==36
    dest=ROOT/'docs/m3-fixed12-native-evidence';dest.mkdir()
    for name in ('summary.json','boundary.jsonl','ledger.jsonl','ledger-diagnostics.jsonl','baseline.json'):
        shutil.copyfile(native/name,dest/name)
    shutil.copyfile(replay/'summary.json',dest/'v3-replay-summary.json')
    for a,b in zip(original['checks'],final['checks']):
        assert (a['index'],a['blocked'])==(b['index'],b['blocked'])
        label=str(a['index'])+('-blocked' if a['blocked'] else '')
        for folder,source,key in [('native',native,a['evidence_sha256']),('v3-replay',replay,b['v3_evidence_sha256'])]:
            source=source/label/'evidence'
            assert hashlib.sha256((source/'complete-sha256.json').read_bytes()).hexdigest()==key
            target=dest/folder/label;target.parent.mkdir(exist_ok=True);shutil.copytree(source,target)
            for name,h in json.loads((target/'complete-sha256.json').read_text()).items():
                assert hashlib.sha256((target/name).read_bytes()).hexdigest()==h
    hashes={str(p.relative_to(dest)):hashlib.sha256(p.read_bytes()).hexdigest() for p in dest.rglob('*') if p.is_file()}
    (dest/'sha256.json').write_text(json.dumps(hashes,indent=2)+'\n')
    print(json.dumps(dict(exported_files=len(hashes),native_checks=36,v3_replays=36,provider_calls=0)))
if __name__=='__main__':main()
