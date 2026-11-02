"""Close an interrupted assessment as unavailable, never as a passing receipt.
Requires an exact independently reviewed ledger disposition. Preserves native
state and original journal prefixes; the next run needs fresh pins/admission.
"""
import importlib.util,json,fcntl,os,contextlib,hashlib
from pathlib import Path

def load(name,file):
 s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m

def close(packet,ledger,evidence):
 proof=load('close_proof','engine-execution-not-started.py');proof.validate_evidence(evidence)
 d=load('close_driver','engine-b91-driver.py');e=d.e;packet=Path(packet)
 class Journal:
  row=staticmethod(d.Driver.row);read=d.Driver.read;append=d.Driver.append
  def __init__(self,path,ad):self.file=path;self.admission=ad;self.fixture=False;self.journal_limit=128*1024*1024
 ad=json.loads((packet/'adapter-admission.json').read_text());ca=json.loads((packet/'cycle-admission.json').read_text())
 if e.sha(ad)!=evidence['binding']['admission_sha256'] or Path(ad['directory']).resolve()!=(packet/'adapter').resolve() or Path(ca['journal']).resolve()!=(packet/'cycle').resolve():raise ValueError('close packet substitution')
 with contextlib.ExitStack() as stack:
  for path in (packet/'adapter/launch.lock',packet/'cycle/run.lock'):
   lock=stack.enter_context(os.fdopen(os.open(path,os.O_CREAT|os.O_WRONLY|os.O_NOFOLLOW,0o600),'a'));fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
  for path,expected in evidence['source'].items():
   if hashlib.sha256(Path(path).read_bytes()).hexdigest()!=expected:raise ValueError('source changed before close')
  with ledger.locked() as (_,rows):
   records=[r['event'] for r in rows if r['event'].get('kind')=='execution_not_started' and r['event'].get('key')==evidence['binding']['key']]
   if len(records)!=1 or records[0]['evidence_sha256']!=ledger_digest(evidence):raise ValueError('missing exact reviewed not-started disposition')
   disposition=records[0]
  a=Journal(packet/'adapter/adapter.jsonl',ad);c=Journal(packet/'cycle/cycle.jsonl',ca)
  ar,ap,done=a.read();cr,cp,cdone=c.read()
  def prefix(journal,role):
   expected=evidence['artifacts'][evidence['records'][role]];data=b''
   for i,line in enumerate(journal.file.read_bytes().splitlines(keepends=True)):
    data+=line
    if hashlib.sha256(data).hexdigest()==expected:return i+1
   raise ValueError('reviewed journal prefix changed')
  ai=prefix(a,'adapter');ci=prefix(c,'cycle');base_done={r['event']['name']:r['event']['value'] for r in ar[:ai] if r['event']['kind']=='result'}
  executions=[v for k,v in base_done.items() if k.startswith('execution_')];resume='resume_'+str(len(executions)-1)
  name='independent_assessment_'+e.sha({'snapshot':executions[-1]['state']['goal_completion']['snapshot'],'source':{Path(k).name:v for k,v in evidence['source'].items()},'script':ad['independent_assessment']['sha256']})
  value={'version':'assessment-not-started-disposition@v1','status':'not_started','observations':None,'execution_key':evidence['binding']['key'],'disposition_sha256':ledger_digest(disposition),'meaning':'Reviewed pre-exec failure; no browser observations or successful test receipt'}
  terminal={'status':'execution_not_started','disposition_sha256':ledger_digest(disposition),'execution_key':evidence['binding']['key'],'completed_feedback_cycle':False,'operator_recovery':True,'next':'Fresh source-bound session may retain native history; do not replay the closed assessment.'}
  aprefix=ar[ai-1]['event'];cprefix=cr[ci-1]['event']
  if aprefix.get('kind')!='intent' or aprefix.get('name')!=name or cprefix.get('kind')!='intent' or cprefix.get('name')!=resume:raise ValueError('reviewed pending stages changed')
  expected_a={'kind':'result','name':name,'intent':e.sha(aprefix),'value':value};expected_c={'kind':'result','name':resume,'intent':e.sha(cprefix),'value':executions[-1]}
  ax=[r['event'] for r in ar[ai:]];cx=[r['event'] for r in cr[ci:]]
  if ax not in ([],[expected_a]):raise ValueError('unreviewed adapter suffix')
  if len(cx)>3 or (cx and cx[0]!=expected_c):raise ValueError('unreviewed cycle suffix')
  if len(cx)>=2 and (set(cx[1])!={'kind','name','at'} or cx[1]['kind']!='intent' or cx[1]['name']!='terminal'):raise ValueError('unreviewed terminal intent')
  if len(cx)==3 and cx[2]!={'kind':'result','name':'terminal','intent':e.sha(cx[1]),'value':terminal}:raise ValueError('unreviewed terminal result')
  if 'terminal' in cdone:return terminal
  if ap:
   if list(ap)!=[name]:raise ValueError('different pending adapter effect')
   a.append({'kind':'result','name':name,'intent':e.sha(ap[name]),'value':value})
  elif done.get(name)!=value:raise ValueError('different assessment result')
  executions=[v for k,v in done.items() if k.startswith('execution_')]
  resume='resume_'+str(len(executions)-1)
  terminal_pending=cp.pop('terminal',None)
  if terminal_pending and cp:raise ValueError('mixed terminal effects')
  if cp:
   if list(cp)!=[resume]:raise ValueError('different pending outer effect')
   c.append({'kind':'result','name':resume,'intent':e.sha(cp[resume]),'value':executions[-1]})
  elif cdone.get(resume)!=executions[-1]:raise ValueError('different existing native return')
  terminal={'status':'execution_not_started','disposition_sha256':ledger_digest(disposition),'execution_key':evidence['binding']['key'],'completed_feedback_cycle':False,'operator_recovery':True,'next':'Fresh source-bound session may retain native history; do not replay the closed assessment.'}
  intent=terminal_pending or {'kind':'intent','name':'terminal','at':d.iso_now()}
  if terminal_pending is None:c.append(intent)
  c.append({'kind':'result','name':'terminal','intent':e.sha(intent),'value':terminal});return terminal

def ledger_digest(v):return __import__('hashlib').sha256(json.dumps(v,sort_keys=True,separators=(',',':'),allow_nan=False).encode()).hexdigest()
