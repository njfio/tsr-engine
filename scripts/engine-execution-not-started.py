"""Narrow operator-reviewed pre-exec disposition; never an execution receipt.
Historical folds validate embedded evidence. New admission additionally rechecks
its immutable artifacts. No launch, ledger mutation, or observation fabrication.
"""
import hashlib,json,os,re
from pathlib import Path

def digest(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
def identity(v):
 if not isinstance(v,str) or not re.fullmatch('[0-9a-f]{64}',v):raise ValueError('invalid evidence identity')
def validate_evidence(v):
 fields={'version','binding','proof','artifacts','missing_outputs','worker_pid','source','records'}
 if not isinstance(v,dict) or set(v)!=fields or v['version']!='execution-not-started-evidence@v1':raise ValueError('not-started evidence shape')
 b=v['binding']
 if set(b)!={'key','trial','admission_sha256','request_id','original_return_row'}:raise ValueError('not-started binding shape')
 for x in b.values():identity(x)
 p=v['proof']
 if set(p)!={'version','errno','executable','traceback','log_path'} or p['version']!='retained-python-preexec-error@v1' or p['errno']!=2 or p['executable']!='docker':raise ValueError('only reviewed Docker pre-exec ENOENT supported')
 t=p['traceback']
 if not isinstance(t,str) or len(t.encode())>32768 or not t.startswith('Traceback (most recent call last):\n') or t.count('Traceback (most recent call last):')!=1:raise ValueError('single complete traceback required')
 for marker in ('engine-browser-assessment.py','p=run();','subprocess.run(cmd,capture_output=True,timeout=90)','with Popen(*popenargs, **kwargs) as process:','self._execute_child(args, executable, preexec_fn, close_fds,','raise child_exception_type(errno_num, err_msg, err_filename)'):
  if marker not in t:raise ValueError('missing pre-exec process boundary')
 if not t.rstrip().endswith("FileNotFoundError: [Errno 2] No such file or directory: 'docker'"):raise ValueError('ambiguous/post-spawn failure')
 if not isinstance(v['artifacts'],dict) or p['log_path'] not in v['artifacts'] or len(v['artifacts'])<3:raise ValueError('missing immutable evidence artifacts')
 for path,h in v['artifacts'].items():
  if not Path(path).is_absolute():raise ValueError('absolute evidence paths required')
  identity(h)
 if type(v['worker_pid']) is not int or v['worker_pid']<=1:raise ValueError('worker identity required')
 if not isinstance(v['missing_outputs'],list) or not v['missing_outputs'] or any(not Path(x).is_absolute() for x in v['missing_outputs']):raise ValueError('assessment output absence required')
 if not isinstance(v['source'],dict) or not v['source']:raise ValueError('bound source required')
 for path,h in v['source'].items():
  if not Path(path).is_absolute():raise ValueError('source path')
  identity(h)
 if set(v['records'])!={'admission','adapter','cycle','ledger'} or any(path not in v['artifacts'] for path in v['records'].values()):raise ValueError('exact journal roles required')
 return b

def validate_current_evidence(v):
 validate_evidence(v)
 try:os.kill(v['worker_pid'],0)
 except ProcessLookupError:pass
 else:raise ValueError('worker present; cannot conclude launch failure')
 for path,expected in {**v['artifacts'],**v['source']}.items():
  p=Path(path)
  if p.is_symlink() or hashlib.sha256(p.read_bytes()).hexdigest()!=expected:raise ValueError('evidence artifact/source changed')
 raw=Path(v['proof']['log_path']).read_text()
 if not raw.endswith(v['proof']['traceback']):raise ValueError('failure is not exact final retained process traceback')
 if any(Path(p).exists() or Path(p).is_symlink() for p in v['missing_outputs']):raise ValueError('assessment output exists; reconcile its actual outcome')
 records=v['records'];ad=json.loads(Path(records['admission']).read_text());binding=v['binding']
 if digest(ad)!=binding['admission_sha256'] or digest({'trial':str(Path(ad['directory']).resolve()),'admission':digest(ad)})!=binding['trial']:raise ValueError('admission/trial substitution')
 def chain(path,style):
  rows=[json.loads(x) for x in Path(path).read_text().splitlines()]
  for i,r in enumerate(rows):
   hashrow=(lambda x:hashlib.sha256(json.dumps(x,sort_keys=True,separators=(',',':'),allow_nan=False).encode()).hexdigest()) if style=='ledger' else digest
   base={'sequence' if style=='ledger' else 'seq':i,'previous':hashrow(rows[i-1]) if i else None,'event':r['event']}
   if r!={**base,'hash':hashrow(base)}:raise ValueError('evidence journal chain changed')
  return rows
 rows=chain(records['adapter'],'driver');events=[r['event'] for r in rows];pending={}
 if events[0]!={'kind':'admission','value':ad,'fixture_only':False}:raise ValueError('adapter admission substitution')
 for e in events[1:]:
  if e['kind']=='intent':pending[e['name']]=e
  elif e['kind']=='result':
   if e['name'] not in pending or e['intent']!=digest(pending.pop(e['name'])):raise ValueError('unbound adapter result')
 if len(pending)!=1 or not next(iter(pending)).startswith('independent_assessment_'):raise ValueError('not exact lone assessment intent')
 executions=[e['value'] for e in events if e['kind']=='result' and e.get('name','').startswith('execution_')]
 if not executions:raise ValueError('missing native execution return')
 snapshot=executions[-1]['state']['goal_completion']['snapshot'];policy=ad['independent_assessment'];work=Path(ad['origin']['directory'])/'coding-work'
 expected={n:hashlib.sha256((work/n).read_bytes()).hexdigest() for n in ad['origin']['public']['editable']}
 if v['source']!={str(work/n):h for n,h in expected.items()}:raise ValueError('source proof substitution')
 cmd=['docker','run','--rm','--init','--pull=never','--network=none','--read-only','--cap-drop=ALL','--security-opt=no-new-privileges','--pids-limit=256','--memory=2g','--cpus=2','--user','501:20','--tmpfs=/tmp:rw,nosuid,nodev,exec,size=256m','--mount',f'type=bind,src={work},dst=/project,readonly','--mount',f"type=bind,src={policy['script']},dst=/verify.cjs,readonly",'--workdir','/project','--entrypoint=','sha256:58deb9d12b7291f6d2c697b335eb927557d707488df09d1503659d7abdd00883','node','/verify.cjs']
 request=digest({'script':policy['sha256'],'source':expected,'snapshot':snapshot,'command':cmd})
 if request!=binding['request_id'] or digest({'trial':binding['trial'],'request':request,'kind':'execution'})!=binding['key']:raise ValueError('execution request substitution')
 if next(iter(pending))!='independent_assessment_'+digest({'snapshot':snapshot,'source':expected,'script':policy['sha256']}):raise ValueError('assessment intent substitution')
 output=Path(ad['directory'])/(next(iter(pending))+'-files')
 if v['missing_outputs']!=[str(output/n) for n in ('receipt.json','stdout.bin','stderr.bin')]:raise ValueError('output absence substitution')
 cycle=chain(records['cycle'],'driver');cp={}
 ca=cycle[0]['event']
 if ca.get('kind')!='admission' or ca.get('fixture_only') is not False or Path(ca['value']['journal']).resolve()!=Path(records['cycle']).parent.resolve() or ca['value']['scope']!=ad['origin']['public']['scope']:raise ValueError('cycle admission substitution')
 for e in [r['event'] for r in cycle[1:]]:
  if e['kind']=='intent':
   if e['name'] in cp:raise ValueError('duplicate cycle intent')
   cp[e['name']]=e
  elif e['kind']=='result':
   if e['name'] not in cp or e['intent']!=digest(cp.pop(e['name'])):raise ValueError('unbound cycle result')
 if len(cp)!=1 or not next(iter(cp)).startswith('resume_'):raise ValueError('unexpected cycle stage')
 ledger=chain(records['ledger'],'ledger');starts=[r['event'] for r in ledger if r['event'].get('kind')=='execution' and r['event'].get('key')==binding['key']];returns=[r for r in ledger if r['event'].get('kind')=='return' and r['event'].get('key')==binding['key']]
 if len(starts)!=1 or starts[0]['trial']!=binding['trial'] or starts[0]['binding']!=binding['admission_sha256'] or len(returns)!=1 or returns[0]['hash']!=binding['original_return_row'] or returns[0]['event']['outcome']!='unknown':raise ValueError('reserved execution/unknown return substitution')
 return v['binding']
