"""Create a separate build source; never alter the frozen checkout or engine."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
ROOT=Path(__file__).resolve().parents[2]
DEST=ROOT/'target/m3-inspection-handle-source'

def main():
    pins=json.loads((ROOT/'docs/m3-fixed12-freeze.json').read_text())['sha256']
    for name,want in pins.items():
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==want,name
    assert shutil.disk_usage(ROOT).free >= 200*2**30,'disk guard: free space below200GiB'
    assert not DEST.exists(),'refuse to overwrite an existing experimental source'
    DEST.mkdir()
    # Only tracked native source/build configuration, never private logs or credentials.
    files=subprocess.check_output(['git','ls-files','-z','Cargo.toml','Cargo.lock','crates'],cwd=ROOT).decode().split('\0')
    for name in filter(None,files):
        source=ROOT/name
        assert source.is_file() and not source.is_symlink(),name
        target=DEST/name;target.parent.mkdir(parents=True,exist_ok=True)
        shutil.copyfile(source,target)
    shutil.copyfile(Path(__file__).with_name('broker_v2.rs'),DEST/'crates/tsr-batch/src/broker_v2.rs')
    shutil.copyfile(Path(__file__).with_name('inspection_receipt.rs'),DEST/'crates/tsr-batch/src/inspection_receipt.rs')
    print(DEST)
if __name__=='__main__':main()
