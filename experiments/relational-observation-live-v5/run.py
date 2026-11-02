"""Exact existing launcher binding; no new execution/recovery semantics."""
import hashlib,json,os,sys
from datetime import datetime,timezone
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];RUN=ROOT/'target/relational-observation-live-v5';FROZEN=ROOT/'experiments/relational-observation-v5'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def canonical(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False,allow_nan=False).encode()).hexdigest()
mode=sys.argv[1] if len(sys.argv)==2 else None
if mode not in ('check','run'):raise SystemExit('check or run required')
m=json.loads((RUN/'launch-manifest.json').read_text())
assert sha(FROZEN/'manifest.json')==m['frozen_packet_sha256']
for n,h in m['binding_files'].items():assert sha(RUN/n)==h,n
for n,h in m['launcher_files'].items():assert sha(ROOT/n)==h,n
ad=json.loads((RUN/'adapter-admission.json').read_text());grant=json.loads((RUN/'scope-grant.json').read_text())
args=[sys.executable,str(ROOT/'scripts/engine-native-cycle.py'),mode,'--browser-v1','--admission',str(RUN/'adapter-admission.json'),'--expected-sha256',canonical(ad),'--cycle-admission',str(RUN/'cycle-admission.json'),'--campaign-manifest',str(ROOT/'target/tsr-integrated-campaign-1/manifest.json'),'--campaign-upgrade',str(RUN/'campaign-upgrade.json'),'--scope-grant',str(RUN/'scope-grant.json'),'--scope-grant-sha256',canonical(grant)]
if mode=='run' and not (RUN/'launch-metadata.json').exists():
 with (RUN/'launch-metadata.json').open('x') as f:json.dump({'launch_requested_at':datetime.now(timezone.utc).isoformat(),'deadline':None,'frozen_packet_sha256':m['frozen_packet_sha256'],'scope':'one separately approved three-condition control-observation evaluation'},f,indent=2)
os.execv(sys.executable,args)
