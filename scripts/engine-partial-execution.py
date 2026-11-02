"""Operator-reviewed partial execution disposal. No game/model execution or replay.
Hashes bind retained evidence, not human identity. Native records attest the known
mutation prefix; missing read-only test outcomes stay unavailable forever.
"""
import hashlib,json,os,subprocess,fcntl
from contextlib import contextmanager
from pathlib import Path

def digest(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False,allow_nan=False).encode()).hexdigest()
def identity(v):
 if not isinstance(v,str) or len(v)!=64 or any(c not in '0123456789abcdef' for c in v):raise ValueError('partial identity')
def validate_evidence(v):
 if not isinstance(v,dict) or set(v)!={'version','binding','native_directory','native_receipt','binary','binary_sha256','execution_binary','historical_implementation','artifacts'} or v['version']!='partial-execution-disposition@v1':raise ValueError('partial evidence shape')
 b=v['binding'];fields={'key','trial','admission_sha256','request_id','original_return_row'}
 if not isinstance(b,dict) or set(b)!=fields:raise ValueError('partial binding')
 for x in b.values():identity(x)
 if b['key']!=digest({'trial':b['trial'],'request':b['request_id'],'kind':'execution'}):raise ValueError('partial execution key')
 r=v['native_receipt']
 if set(r)!={'disposition','commitment','source_after','receipts','evidence','review','observation','duration','replayed'} or r['observation']!='unavailable' or r['duration']!='unmeasured_full_reservation_charged' or r['replayed'] is not False or len(r['receipts'])!=2:raise ValueError('partial observation must remain unavailable')
 for x in [r['disposition'],r['commitment'],r['source_after'],*r['receipts']]:identity(x)
 e=r['evidence'];review=r['review']
 if e['version']!='partial-readonly-tail-proof@v1' or e['directory']!=v['native_directory'] or e['commitment']!=r['commitment'] or e['source_after']!=r['source_after']:raise ValueError('native partial evidence binding')
 if review['decision']!='partial_observation_unavailable' or not review['reviewer'].strip() or not review['report'].strip():raise ValueError('independent partial review required')
 a=e['attestation']
 if a['processes_absent'] is not True or a['containers_absent'] is not True or a['container_prefix']!=f"tsr-m3-{a['native_pid']}-":raise ValueError('partial quiescence evidence')
 if not isinstance(v['artifacts'],dict) or len(v['artifacts'])<3:raise ValueError('partial retained artifacts')
 for p,h in v['artifacts'].items():
  if not Path(p).is_absolute():raise ValueError('partial artifact path')
  identity(h)
 identity(v['binary_sha256'])
 if not Path(v['native_directory']).is_absolute() or not Path(v['binary']).is_absolute():raise ValueError('partial native path')
 return b

@contextmanager
def locked_source(v):
 root=Path(v['native_directory']).parent
 handles=[]
 try:
  for p in (root/'adapter/launch.lock',root/'cycle/run.lock'):
   f=p.open('a+');handles.append(f);fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
  yield
 finally:
  for f in reversed(handles):f.close()

def validate_current_evidence(v):
 b=validate_evidence(v);root=Path(v['native_directory']).parent
 if Path(v['binary']).resolve()!=Path(__file__).resolve().parents[1]/'target/debug/tsr':raise ValueError('unapproved recovery binary path')
 for p,h in {**v['artifacts'],v['binary']:v['binary_sha256']}.items():
  p=Path(p)
  if p.is_symlink() or hashlib.sha256(p.read_bytes()).hexdigest()!=h:raise ValueError('partial artifacts changed')
 ad=json.loads((root/'adapter-admission.json').read_text())
 old=Path(v['execution_binary'])
 if old.is_symlink() or hashlib.sha256(old.read_bytes()).hexdigest()!=ad['binary_sha256']:raise ValueError('historical executed binary mismatch')
 required_code={'engine-project-cycle-adapter.py','engine-browser-adapter.py','engine-b91-campaign.py'}
 if set(v['historical_implementation'])!=required_code:raise ValueError('historical implementation scope')
 for name,path in v['historical_implementation'].items():
  p=Path(path)
  if p.is_symlink() or hashlib.sha256(p.read_bytes()).hexdigest()!=ad['implementation'][name]:raise ValueError('historical execution implementation mismatch')
 if Path(ad['origin']['directory']).resolve()!=Path(v['native_directory']).resolve():raise ValueError('partial native directory substitution')
 required={str(root/'adapter-admission.json'),str(root/'cycle-admission.json'),str(root/'adapter/adapter.jsonl'),str(root/'cycle/cycle.jsonl')}
 if not required<=set(v['artifacts']):raise ValueError('partial source journals missing')
 if digest(ad)!=b['admission_sha256'] or digest({'trial':str(root/'adapter'),'admission':digest(ad)})!=b['trial']:raise ValueError('partial admission/trial mismatch')
 rows=[json.loads(s) for s in (root/'adapter/adapter.jsonl').read_text().splitlines()]
 pending={}
 for i,row in enumerate(rows):
  expected={'seq':i,'previous':digest(rows[i-1]) if i else None,'event':row['event']}
  if row!={**expected,'hash':digest(expected)}:raise ValueError('partial adapter chain')
  event=row['event']
  if event['kind']=='intent':pending[event['name']]=event
  elif event['kind']=='result':
   if event['name'] not in pending or event['intent']!=digest(pending.pop(event['name'])):raise ValueError('partial adapter association')
 if set(pending)!={'execution_'+b['request_id']}:raise ValueError('not exact isolated unknown execution')
 cycle=[json.loads(s) for s in (root/'cycle/cycle.jsonl').read_text().splitlines()];cp={};cd={}
 for i,row in enumerate(cycle):
  expected={'seq':i,'previous':digest(cycle[i-1]) if i else None,'event':row['event']}
  if row!={**expected,'hash':digest(expected)}:raise ValueError('partial cycle chain')
  event=row['event']
  if event['kind']=='intent':cp[event['name']]=event
  elif event['kind']=='result':
   if event['name'] not in cp or event['intent']!=digest(cp.pop(event['name'])):raise ValueError('partial cycle association')
   cd[event['name']]=event['value']
 if len(cp)!=1 or not next(iter(cp)).startswith('resume_'):raise ValueError('partial cycle stage')
 index=next(iter(cp)).removeprefix('resume_');before=cd.get('advance_'+index,{}).get('next',{})
 if before.get('branch')!='resume' or digest({'admission':digest(ad),'binding':before.get('binding')})!=b['request_id']:raise ValueError('partial pre-execution binding')
 plan=before.get('plan',{});r=v['native_receipt']
 if plan.get('commitment')!=r['commitment'] or plan.get('expected_after')!=r['source_after']:raise ValueError('partial cross-execution disposition')

 a=v['native_receipt']['evidence']['attestation']
 for pid in (a['native_pid'],a['outer_pid']):
  try:os.kill(pid,0)
  except ProcessLookupError:pass
  else:raise ValueError('execution process remains active')
 found=subprocess.run(['/opt/homebrew/bin/docker','ps','-a','--filter','name='+a['container_prefix'],'--format','{{.Names}}'],capture_output=True,check=True,timeout=15)
 if found.stdout.strip():raise ValueError('execution container remains')
 result=subprocess.run([v['binary'],'engine','--directory',v['native_directory'],'project','partial-execution-status'],capture_output=True,check=True,timeout=120)
 if json.loads(result.stdout).count(v['native_receipt'])!=1:raise ValueError('native disposition missing or substituted')
 return b
