"""Exact one-arm checkpoint launcher for the approved pair; no extra retry logic."""
import hashlib,json,os,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def canonical(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False,allow_nan=False).encode()).hexdigest()
if len(sys.argv)!=3 or sys.argv[1] not in ('check','run') or sys.argv[2] not in ('tsr','ordinary'):raise SystemExit('check|run tsr|ordinary')
mode,arm=sys.argv[1:];base=ROOT/'target/game-wants-live-v3';run=base/arm
manifest=json.loads((run/'launch-manifest.json').read_text())
for n,h in manifest['files'].items():assert sha(ROOT/n)==h,n
ad=json.loads((run/'adapter-admission.json').read_text());grant=json.loads((run/'scope-grant.json').read_text())
args=[sys.executable,str(ROOT/'scripts/engine-native-cycle.py'),mode,'--browser-v1','--admission',str(run/'adapter-admission.json'),'--expected-sha256',canonical(ad),'--cycle-admission',str(run/'cycle-admission.json'),'--campaign-manifest',str(ROOT/'target/tsr-integrated-campaign-1/manifest.json'),'--campaign-upgrade',str(base/'campaign-upgrade.json'),'--scope-grant',str(run/'scope-grant.json'),'--scope-grant-sha256',canonical(grant)]
os.execv(sys.executable,args)
