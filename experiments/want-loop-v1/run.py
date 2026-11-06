"""One launch of the want-driven loop (spec #11). Runs until it stops by itself; no stages."""
import hashlib,json,os,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def canonical(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False,allow_nan=False).encode()).hexdigest()
REVIEWED={'launch_manifest_sha256':'launch-manifest.json','adapter_admission_file_sha256':'adapter-admission.json','cycle_admission_file_sha256':'cycle-admission.json','scope_grant_file_sha256':'scope-grant.json'}
def check_review(run):
 """The one independent review must accept exactly the files this launch will use."""
 path=run/'binding-review.json'
 if not path.exists():raise SystemExit('independent binding review required; run refused')
 review=json.loads(path.read_text())
 if review.get('decision')!='accept':raise SystemExit('binding review did not accept; run refused')
 for field,name in REVIEWED.items():
  if review.get(field)!=sha(run/name):raise SystemExit('binding review does not match '+name+'; run refused')
def main(argv):
 if len(argv)!=2 or argv[1] not in ('check','run'):raise SystemExit('check|run')
 mode=argv[1];base=ROOT/('target/want-loop-'+os.environ.get('TSR_LIVE_LABEL','r1'));run=base/'tsr'
 if mode=='run':
  if json.loads((base/'authorization.json').read_text())['approved'] is not True:raise SystemExit('operator approval required; run refused')
  check_review(run)
 manifest=json.loads((run/'launch-manifest.json').read_text())
 for n,h in manifest['files'].items():assert sha(ROOT/n)==h,n
 ad=json.loads((run/'adapter-admission.json').read_text());grant=json.loads((run/'scope-grant.json').read_text())
 label=os.environ.get('TSR_LIVE_LABEL','r1')
 args=[sys.executable,str(ROOT/'scripts/engine-native-cycle.py'),mode,'--browser-v1','--admission',str(run/'adapter-admission.json'),'--expected-sha256',canonical(ad),'--cycle-admission',str(run/'cycle-admission.json'),'--campaign-manifest',str(ROOT/('target/want-loop-campaign-'+label+'/manifest.json')),'--campaign-upgrade',str(base/'campaign-upgrade.json'),'--scope-grant',str(run/'scope-grant.json'),'--scope-grant-sha256',canonical(grant)]
 os.execv(sys.executable,args)
if __name__=='__main__':main(sys.argv)
