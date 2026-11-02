"""Prospectively pinned independent browser assessment, separate from native cases.
No script generation, source editing, player preferences or authority creation.
"""
import base64,hashlib,json,subprocess,os,shutil
from pathlib import Path
IMAGE='sha256:58deb9d12b7291f6d2c697b335eb927557d707488df09d1503659d7abdd00883'
def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def validate(policy,root):
 if policy is None:return
 if not isinstance(policy,dict) or set(policy)!={'version','script','sha256','limitations'} or policy['version']!='source-bound-browser-assessment@v1':raise ValueError('assessment admission shape')
 path=Path(policy['script'])
 if not path.is_absolute() or '..' in path.parts or not path.resolve().is_relative_to(Path(root).resolve()) or any(x.is_symlink() for x in (path,*path.parents)) or not path.is_file() or sha(path)!=policy['sha256']:raise ValueError('assessment script pin/path')
 if not isinstance(policy['limitations'],str) or not policy['limitations'].strip():raise ValueError('assessment limitations required')
def sources(work,paths):return {name:sha(Path(work)/name) for name in paths}
def decode(raw,expected):
 def unique(pairs):
  v={}
  for k,x in pairs:
   if k in v:raise ValueError('duplicate assessment key')
   v[k]=x
  return v
 data=json.loads(raw,object_pairs_hook=unique)
 if data.get('source')!=expected:raise ValueError('assessment source mismatch')
 if not isinstance(data.get('checks'),list) or not data['checks'] or any(not isinstance(x,dict) or not isinstance(x.get('name'),str) or type(x.get('pass')) is not bool for x in data['checks']):raise ValueError('assessment checks shape')
 return data
def collect(adapter,request):
 policy=adapter.admission.get('independent_assessment')
 if policy is None:return None
 validate(policy,adapter.e_root if hasattr(adapter,'e_root') else Path(__file__).resolve().parents[1])
 coding=request['coding'];work=adapter.controller.directory/'coding-work';expected=sources(work,adapter.admission['origin']['public']['editable']);snapshot=coding['snapshot'];key='independent_assessment_'+adapter.recovery_sha({'snapshot':snapshot,'source':expected,'script':policy['sha256']})
 done=adapter.read()[2]
 if key in done:
  record=done[key]
  if record['source']!=expected or record['snapshot']!=snapshot or record['script_sha256']!=policy['sha256']:raise ValueError('assessment replay binding')
  authenticate(adapter,record)
  return record
 if adapter.ledger.status()['stopped']:raise ValueError('unresolved effects before assessment')
 from datetime import datetime
 deadline=adapter.clock_deadline()
 if __import__('time').time()+120>deadline:raise ValueError('assessment cannot fit session deadline')
 output=adapter.root/(key+'-files')
 if output.is_symlink():raise ValueError('assessment output symlink')
 output.mkdir(exist_ok=True)
 script=Path(policy['script']);cmd=['docker','run','--rm','--init','--pull=never','--network=none','--read-only','--cap-drop=ALL','--security-opt=no-new-privileges','--pids-limit=256','--memory=2g','--cpus=2','--user','501:20','--tmpfs=/tmp:rw,nosuid,nodev,exec,size=256m','--mount',f'type=bind,src={work},dst=/project,readonly','--mount',f'type=bind,src={script},dst=/verify.cjs,readonly','--workdir','/project','--entrypoint=',IMAGE,'node','/verify.cjs']
 binding=adapter.recovery_sha({'script':policy['sha256'],'source':expected,'snapshot':snapshot,'command':cmd})
 if getattr(adapter,'assessment_runner',None) is None and shutil.which(cmd[0]) is None:raise ValueError('Browser runtime executable unavailable before assessment reservation')
 def worker():
  if getattr(adapter,'assessment_runner',None) is not None and not adapter.ledger.manifest.get('fixture_only'):raise ValueError('authored assessment runner requires fixture ledger')
  run=getattr(adapter,'assessment_runner',None) or (lambda:subprocess.run(cmd,capture_output=True,timeout=90))
  try:p=run()
  except FileNotFoundError as exc:
   if exc.errno!=2 or exc.filename!=cmd[0]:raise
   # Popen's executable ENOENT is a received pre-exec failure, never a test pass.
   record={'version':'source-bound-browser-assessment@v1','snapshot':snapshot,'source':expected,'script_sha256':policy['sha256'],'execution_binding':binding,'status':'not_started','observations':None,'attribution':'OS pre-exec failure; no browser launched','limitations':policy['limitations'],'failure':{'phase':'pre_exec','errno':exc.errno,'executable':exc.filename}}
   with (output/'receipt.json').open('x') as stream:json.dump(record,stream,sort_keys=True);stream.flush();os.fsync(stream.fileno())
   return record
  (output/'stdout.bin').write_bytes(p.stdout);(output/'stderr.bin').write_bytes(p.stderr)
  record={'version':'source-bound-browser-assessment@v1','snapshot':snapshot,'source':expected,'script_sha256':policy['sha256'],'execution_binding':binding,'returncode':p.returncode,'stdout_sha256':hashlib.sha256(p.stdout).hexdigest(),'stderr_sha256':hashlib.sha256(p.stderr).hexdigest(),'attribution':'independent authored control-driven browser checks; not player preference, native case receipt or final acceptance','limitations':policy['limitations'],'status':'unavailable','observations':None}
  if p.returncode==0 and len(p.stdout)<=4*1048576:
   try:
    data=decode(p.stdout,expected)
    for field,name in [('screenshot_base64','screenshot.png'),('mobile_screenshot_base64','mobile-screenshot.png')]:
     if field in data:(output/name).write_bytes(base64.b64decode(data.pop(field),validate=True))
    record.update(status='observed',observations=data)
   except (ValueError,TypeError,KeyError) as error:record['diagnostic']=str(error)
  if sources(work,list(expected))!=expected:raise ValueError('source altered during read-only assessment')
  receipt=output/'receipt.json'
  with receipt.open('x') as stream:json.dump(record,stream,sort_keys=True);stream.flush();os.fsync(stream.fileno())
  return record
 return adapter.op(key,lambda:adapter.ledger.run_execution(adapter.trial,binding,adapter.recovery_sha(adapter.admission),worker))

def recover(adapter):
 rows,pending,done=adapter.read()
 if not pending:return 0
 if len(pending)!=1:return 0
 name,intent=next(iter(pending.items()))
 if not name.startswith('independent_assessment_'):return 0
 policy=adapter.admission.get('independent_assessment');validate(policy,Path(__file__).resolve().parents[1])
 if not policy or adapter.ledger.status()['stopped']:raise ValueError('unresolved assessment; no replay')
 output=adapter.root/(name+'-files');record=json.loads((output/'receipt.json').read_text())
 authenticate(adapter,record)
 request=adapter.controller.request();expected=sources(adapter.controller.directory/'coding-work',adapter.admission['origin']['public']['editable'])
 if record['source']!=expected or record['snapshot']!=request['coding']['snapshot'] or record['script_sha256']!=policy['sha256'] or (record.get('status')!='not_started' and (sha(output/'stdout.bin')!=record['stdout_sha256'] or sha(output/'stderr.bin')!=record['stderr_sha256'])):raise ValueError('assessment source/output drift')
 if name!='independent_assessment_'+adapter.recovery_sha({'snapshot':record['snapshot'],'source':expected,'script':policy['sha256']}):raise ValueError('assessment intent drift')
 adapter.append({'kind':'result','name':name,'intent':adapter.recovery_sha(intent),'value':record});return 1

def authenticate(adapter,record):
 ledger_digest=adapter.recovery_load('assessment_campaign_digest','engine-b91-campaign.py').digest
 key=ledger_digest({'trial':adapter.trial,'request':record['execution_binding'],'kind':'execution'})
 with adapter.ledger.locked() as (_,ledger_rows):
  events=[r['event'] for r in ledger_rows];starts=[x for x in events if x.get('kind')=='execution' and x.get('key')==key];returns=[x for x in events if x.get('kind')=='return' and x.get('key')==key]
 if len(starts)!=1 or starts[0]['trial']!=adapter.trial or starts[0]['binding']!=adapter.recovery_sha(adapter.admission) or len(returns)!=1 or returns[0].get('outcome')!='returned' or returns[0].get('result_sha256')!=ledger_digest(record):raise ValueError('assessment result not authenticated by settled ledger')


def reflection_evidence(adapter,request,context):
    """Promote visibility of authenticated reports, never their trust class."""
    reports=context.get('independent_assessments',[])
    if not reports:return []
    coding=request['coding'];snapshot=coding['snapshot']
    expected={f['path']:hashlib.sha256(f['source'].encode()).hexdigest() for f in coding['files'] if f['editable']}
    owned=[v for k,v in adapter.read()[2].items() if k.startswith('independent_assessment_')]
    result=[]
    for projected in reports:
        record={k:v for k,v in projected.items() if k!='applicability'}
        if record not in owned:raise ValueError('unowned reflection assessment')
        current=record['snapshot']==snapshot
        if projected.get('applicability')!=('current_source' if current else 'historical_source_only'):raise ValueError('reflection assessment applicability mismatch')
        if not current:continue  # Historical reports remain explicitly historical in context.
        authenticate(adapter,record)
        if record['source']!=expected:raise ValueError('reflection assessment source digest mismatch')
        if record['status']=='observed' and (not isinstance(record['observations'],dict) or record['observations'].get('source')!=expected):raise ValueError('reflection observation source mismatch')
        result.append({'version':'attributed-reflection-assessment@v1','channel':'attributed_assessment','report_sha256':adapter.recovery_sha(record),'source_snapshot':snapshot,'source_files':expected,'report':record,'native_observation':False,'instruction':'This is an authenticated source-bound report, not a native case receipt or player feedback. Consider its actual contents, coverage and limitations separately from the smoke receipt. Attribute claims to this report; do not call reported text absent merely because the native smoke check did not assert it. Do not infer untested behavior, enjoyment, fulfillment or authority.'})
    return result
