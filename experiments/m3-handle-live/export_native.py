"""Durable local integration evidence, excluding compiled candidate workspaces."""
from pathlib import Path
import hashlib,json,shutil
ROOT=Path(__file__).resolve().parents[2];BASE=ROOT/'target/m3-handle-live-offline-native-v3';DEST=ROOT/'docs/m3-handle-live-offline-evidence'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def main():
    DEST.mkdir()
    for name in ('summary.json','controller-summary.json'):shutil.copyfile(BASE/name,DEST/name)
    wire=[]
    inventory=json.loads((ROOT/'docs/m3-handle-live-payload-inventory.json').read_text())
    for i in (0,1):
        src=BASE/str(i);dst=DEST/str(i);dst.mkdir()
        shutil.copytree(src/'controller-evidence',dst/'controller-evidence')
        for name in ('native.log','broker-stderr.log'):shutil.copyfile(src/name,dst/name)
        source=src/'input/record/copies'/inventory['task']/'record'/inventory['file']
        assert sha(source)==inventory['mutated_source_sha256'],'offline fixture accidentally changed source'
        for p in sorted(src.glob('body-*.json')):
            body=json.loads(p.read_text());request=json.loads(body['messages'][1]['content'])
            wire.append(dict(interface='old' if i==0 else 'new',path=str(p.relative_to(ROOT)),sha256=sha(p),serialized_bytes=len(p.read_bytes()),request_keys=sorted(request),observation_bytes=len(request['observation'].encode()),model=body['model'],max_output_tokens=body['max_tokens']))
    (DEST/'wire-inventory.json').write_text(json.dumps(wire,indent=2)+'\n')
    for source,name in (('/tmp/m3-handle-live-tests.log','unit-tests.log'),('/tmp/m3-handle-replay-regression.log','replay-regressions.log'),('/tmp/m3-fixed12-v6-regression.log','controller-regressions.log')):shutil.copyfile(source,DEST/name)
    (DEST/'sha256.json').write_text(json.dumps({str(p.relative_to(DEST)):sha(p) for p in DEST.rglob('*') if p.is_file()},indent=2)+'\n')
    print('Exported native/controller evidence; source unchanged; no provider response bodies.')
if __name__=='__main__':main()
