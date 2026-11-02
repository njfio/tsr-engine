"""Local design projection only. No transport, subprocess, credentials or admission."""
import copy
import hashlib
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
PACKET=ROOT/'specs/experiments/complementarity-local-v1'
SOURCE_SHA='9e6205e264e716c6dc3b2864a8125c482e5955d1216cd4d08db86a45aeebed9a'
def canonical(x):return json.dumps(x,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
def source(path=PACKET/'source.json'):
    raw=path.read_bytes()
    if hashlib.sha256(raw).hexdigest()!=SOURCE_SHA:raise ValueError('frozen source changed')
    return json.loads(raw)
def project(arm):
    if arm not in ('A','B','AB','AB-repeat','withdraw-B'):raise ValueError('unknown arm')
    x=copy.deepcopy(source())
    active={'A':['A'],'B':['B'],'AB':['A','B'],'AB-repeat':['A','B'],'withdraw-B':['A']}[arm]
    # Single-parent ablations remove the other meaning; withdrawal retains history.
    if arm in ('A','B'):x['parents']=[p for p in x['parents'] if p['id'] in active]
    x['active_parents']=active
    x['withdrawn_parents']=['B'] if arm=='withdraw-B' else []
    x['active_guidance']=[]
    return x

def prepared():
    outputs={a:canonical(project(a))+b'\n' for a in ('A','B','AB','AB-repeat','withdraw-B')}
    manifest={'source_sha256':SOURCE_SHA,'status':'local_design_only_no_admission','files':{a+'.json':hashlib.sha256(b).hexdigest() for a,b in outputs.items()},'guidance_ablation':'blocked: no generated reviewed aspiration; do not substitute authored guidance','return_control':'not instantiated: no proposed view, probe or actual receipt'}
    return outputs,manifest

def prepare():
    outputs,manifest=prepared()
    for a,b in outputs.items():(PACKET/(a+'.json')).write_bytes(b)
    (PACKET/'manifest.json').write_bytes(canonical(manifest)+b'\n')
    return manifest

def check():
    manifest=json.loads((PACKET/'manifest.json').read_text())
    expected={a+'.json':hashlib.sha256(canonical(project(a))+b'\n').hexdigest() for a in ('A','B','AB','AB-repeat','withdraw-B')}
    if manifest!=prepared()[1]:raise ValueError('manifest drift')
    for name,digest in expected.items():
        if hashlib.sha256((PACKET/name).read_bytes()).hexdigest()!=digest:raise ValueError('projection drift')
    return {'status':'local_projection_checked','arms':5,'dispatches':0,'executions':0,'adoptions':0}
if __name__=='__main__':
    import argparse
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('command',choices=['prepare','check']);a=parser.parse_args()
    print(json.dumps(prepare() if a.command=='prepare' else check(),indent=2))
