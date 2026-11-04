"""Exact reviewed observation-continuation launcher; no recovery or retry policy."""
import hashlib,json,os,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];LABEL=os.environ.get('TSR_LIVE_LABEL','r1')
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def canonical(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False,allow_nan=False).encode()).hexdigest()
if len(sys.argv)!=4 or sys.argv[1] not in ('check','run') or sys.argv[2] not in ('tsr',) or sys.argv[3] not in ('crossing-first','growth-first','mixed-pursuit'):raise SystemExit('check|run tsr|ordinary batch')
mode,arm,batch=sys.argv[1:];base=ROOT/('target/continuous-game-live-'+LABEL);run=base/arm/('continuation-'+batch)
if mode=='run' and (json.loads((base/'authorization.json').read_text())['approved'] is not True or not (run/'binding-review.json').exists()):raise SystemExit('operator approval and independent binding review required; run refused')
manifest=json.loads((base/arm/'launch-manifest.json').read_text());assert sha(ROOT/'experiments/continuous-game-v4/protocol.json')==json.loads((base/'prepared-test.json').read_text())['packet_sha256']
for n,h in manifest['files'].items():assert sha(ROOT/n)==h,n
ad=json.loads((run/'adapter-admission.json').read_text());grant=json.loads((run/'scope-grant.json').read_text())
args=[sys.executable,str(ROOT/'scripts/engine-native-cycle.py'),mode,'--browser-v1','--admission',str(run/'adapter-admission.json'),'--expected-sha256',canonical(ad),'--cycle-admission',str(run/'cycle-admission.json'),'--campaign-manifest',str(ROOT/('target/continuous-game-campaign-'+LABEL+'/manifest.json')),'--campaign-upgrade',str(base/'campaign-upgrade.json'),'--scope-grant',str(run/'scope-grant.json'),'--scope-grant-sha256',canonical(grant)]
os.execv(sys.executable,args)
