#!/usr/bin/env python3
"""Loopback UI bridge. Fixed native commands; no model calls on browse/input.
Live dispatch requires an independently prepared, source-bound registered packet.
"""
import io,zipfile,sys
import argparse,contextlib,datetime,fcntl,hashlib,http.server,importlib.util,json,os,re,secrets,shutil,subprocess,threading,time,urllib.parse,uuid,types
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];STATIC=Path(__file__).parent

def load(name,file):
 s=importlib.util.spec_from_file_location(name,ROOT/'scripts'/file);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m

def canonical(x):return json.dumps(x,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()
def sha(x):return hashlib.sha256(canonical(x)).hexdigest()
def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def save(path,x):
 path.parent.mkdir(parents=True,exist_ok=True);tmp=path.with_suffix('.tmp');tmp.write_bytes(canonical(x));os.replace(tmp,path)
def checked_id(value):
 if not isinstance(value,str) or not re.fullmatch(r'[a-zA-Z0-9_-]{1,100}',value):raise ValueError('Invalid identity')
 return value

def accounting_snapshot(module,manifest,upgrade):
 """Validate one observed complete ledger prefix, without acquiring writer locks.
 Display only: every dispatch still rereads under the campaign's exclusive lock.
 """
 path=Path(manifest['ledger_path'])
 with os.fdopen(os.open(path,os.O_RDONLY|os.O_NOFOLLOW),'rb') as f:
  size=os.fstat(f.fileno()).st_size
  if not 0<size<=4*1024*1024:raise ValueError('campaign ledger bound')
  raw=f.read(size)
  if len(raw)!=size:raise ValueError('campaign snapshot shortened during read')
 complete=raw.endswith(b'\n')
 prefix=raw if complete else raw[:raw.rfind(b'\n')+1]
 rows=[module.strict(line.decode('utf8')) for line in prefix.splitlines()]
 for i,row in enumerate(rows):
  if row!=module.Ledger.row(rows[:i],row['event']):raise ValueError('campaign chain changed')
 if not rows or rows[0]['event']!={'kind':'manifest','value':manifest}:raise ValueError('wrong campaign history')
 if upgrade and not any(r['hash']==upgrade['ledger_prefix_head'] for r in rows):raise ValueError('upgrade ledger prefix missing')
 module.validate(manifest,upgrade,module.digest(upgrade) if upgrade else None)
 holder=object.__new__(module.Ledger);holder.manifest=manifest;holder.hash=module.digest(manifest)
 status=holder.fold(rows)[0]
 return {**status,'projection':'observed_complete_prefix_not_dispatch_authority','observed_at':now(),'observed_ledger_head':rows[-1]['hash'],'observed_rows':len(rows),'append_in_progress':not complete,'stopped':status['stopped'] or not complete}

# Fresh installations do not inherit private projects or conversation authority.
CATALOG=[]
def new_project_scope():
 """Public browser template; no historical project, authority or provider state."""
 adapter=load('ui_browser_scope','engine-browser-adapter.py')
 probe={'version':'browser-probe@v1','assets':['index.html','style.css','app.js'],'entry':'index.html','actions':[{'kind':'click','value':'[data-tsr="start"]'},{'kind':'key','value':'ArrowLeft'},{'kind':'key','value':'Space'},{'kind':'wait','value':250}],'checks':[{'selector':'[data-tsr="status"]','text':'Playing'}]}
 return {'goal':'User-requested original browser project','image':adapter.IMAGE,'user':'501:20','browser':True,'browser_container_only':True,'protected':{'browser-harness.cjs':(ROOT/'scripts/engine-browser-harness.cjs').read_text(),'probe.json':json.dumps(probe)},'probe_entry':'browser-harness.cjs','cases':[{'id':'browser_smoke','args':['probe.json'],'expected_stdout':'browser-observation@v1'}],'timeout_ms':30000}

class Store:
 def __init__(self,state=None,catalog=None,native_factory=None):
  self.root=Path(state or ROOT/'target/engine-ui');self.root.mkdir(parents=True,exist_ok=True);self.catalog=CATALOG if catalog is None else catalog;self.native_factory=native_factory;self.cache={};self.jobs={};self.preparing={};self.active_attempts={};self.preparation_pending=set();self.guard=threading.RLock();self.csrf=secrets.token_urlsafe(32)
  (self.root/'projects').mkdir(exist_ok=True)
  from decisions import Decisions
  self.decisions=Decisions(self)
  for p in self.catalog:
   d=self.root/'projects'/p['id'];d.mkdir(parents=True,exist_ok=True)
   if not (d/'project.json').exists():save(d/'project.json',{**p,'created_at':now(),'source':str((ROOT/p['source']).resolve()),'native':None,'registered_packet':None})
 def path(self,p):return self.root/'projects'/checked_id(p)
 @contextlib.contextmanager
 def lock(self,p):
  d=self.path(p);d.mkdir(parents=True,exist_ok=True)
  with (d/'lock').open('a') as f:
   fcntl.flock(f,fcntl.LOCK_EX);yield
 @contextlib.contextmanager
 def control(self,p):
  d=self.path(p);d.mkdir(parents=True,exist_ok=True)
  with (d/'control.lock').open('a') as f:
   fcntl.flock(f,fcntl.LOCK_EX);yield
 def controls(self,p):
  path=self.path(p)/'controls.jsonl'
  rows=[json.loads(x) for x in path.read_text().splitlines()] if path.exists() else [];previous=None;events=[]
  for row in rows:
   if row.get('previous')!=previous or row.get('hash')!=sha({k:v for k,v in row.items() if k!='hash'}):raise ValueError('Stop-control history integrity failure')
   previous=row['hash'];events.append(row['event'])
  return events
 def meta(self,p):return json.loads((self.path(p)/'project.json').read_text())
 def events(self,p):
  f=self.path(p)/'events.jsonl'
  if not f.exists():return []
  out=[];previous=None
  for line in f.read_text().splitlines():
   row=json.loads(line)
   if row['previous']!=previous or row['hash']!=sha({k:v for k,v in row.items() if k!='hash'}):raise ValueError('UI event history integrity failure')
   out.append(row['event']);previous=row['hash']
  return out
 def append(self,p,e):
  f=self.path(p)/'events.jsonl';rows=[json.loads(s) for s in f.read_text().splitlines()] if f.exists() else [];e={**e,'at':now(),'sequence':len(rows)};row={'previous':rows[-1]['hash'] if rows else None,'event':e};row['hash']=sha(row)
  with f.open('a') as out:out.write(json.dumps(row,ensure_ascii=False)+'\n');out.flush();os.fsync(out.fileno())
  return e
 def native(self,path):return self.native_factory(Path(path)) if self.native_factory else load('ui_browser','engine-browser-adapter.py').d.Native(Path(path))
 def source_snapshot(self,p):
  try:
   meta=self.meta(p);q=self.native(self.project_native(p)).request()
   result={'native_state_root':q['state_root'],'native_snapshot':q['coding']['snapshot'],'editable_sha256':{f['path']:hashlib.sha256(f['source'].encode()).hexdigest() for f in q['coding'].get('files',[]) if f.get('editable')}}
   if meta.get('artifact'):
    result['artifact_sha256']={n:hashlib.sha256((ROOT/meta['artifact']/n).read_bytes()).hexdigest() for n in ('index.html','style.css','app.js')}
    result['artifact_matches_native']=result['artifact_sha256']==result['editable_sha256']
   return result
  except (OSError,KeyError,ValueError):return {'status':'unavailable'}
 def project_native(self,p,create=False):
  meta=self.meta(p)
  if meta.get('native'):return Path(meta['native'])
  source=Path(meta['source'])/'native'
  if not create:return source
  dest=self.path(p)/'native'
  if not dest.exists():
   staging=self.path(p)/'native-staging'
   if staging.exists():shutil.rmtree(staging)
   shutil.copytree(source,staging);os.rename(staging,dest)
  meta['native']=str(dest.resolve());meta['forked_at']=now();save(self.path(p)/'project.json',meta);return dest
 def ledger(self):
  a=load('ui_account_browser','engine-browser-adapter.py');m=a.load('ui_campaign','engine-b91-campaign.py');campaign=json.loads((ROOT/'target/tsr-integrated-campaign-1/manifest.json').read_text());rows=list(map(json.loads,Path(campaign['ledger_path']).read_text().splitlines()));base=json.loads((ROOT/'target/pond-beautiful-live-v1/campaign-upgrade.json').read_text());base.update(new_implementation_sha256=a.d.file_sha(Path(m.__file__)),ledger_prefix_head=rows[-1]['hash']);ledger=m.Ledger(campaign['ledger_path'],campaign,m.digest(campaign),upgrade=base,upgrade_hash=m.digest(base));return campaign,m,ledger,base
 def budget(self):
  try:
   status=self.ledger()[2].status();unlimited=bool(status.get('unlimited_policy_sha256'))
   return {**status,'available':True,'provider':'CheaperInference','model':'gpt-6-luna','budget_mode':'unlimited' if unlimited else 'limited','per_run_max_usd':None if unlimited else 10,'per_run_max_calls':None if unlimited else 200,'per_run_max_operations':None if unlimited else 100}
  except Exception:return {'available':False,'stopped':True,'reason':'Accounting unavailable. No dispatch is permitted.','per_run_max_usd':10,'per_run_max_calls':200,'per_run_max_operations':100}
 def budget_snapshot(self):
  # Authored test servers provide an entirely local fake budget.
  if getattr(self,'fixture_only',False):return self.budget()
  try:
   m=load('ui_readonly_campaign','engine-b91-campaign.py')
   campaign=json.loads((ROOT/'target/tsr-integrated-campaign-1/manifest.json').read_text())
   base=json.loads((ROOT/'target/pond-beautiful-live-v1/campaign-upgrade.json').read_text())
   # Keep the reviewed historical prefix; do not invent a head from a second read.
   base['new_implementation_sha256']=hashlib.sha256(Path(m.__file__).read_bytes()).hexdigest()
   status=accounting_snapshot(m,campaign,base);unlimited=bool(status.get('unlimited_policy_sha256'))
   return {**status,'available':True,'provider':'CheaperInference','model':'gpt-6-luna','budget_mode':'unlimited' if unlimited else 'limited','per_run_max_usd':None if unlimited else 10,'per_run_max_calls':None if unlimited else 200,'per_run_max_operations':None if unlimited else 100}
  except Exception:return {'available':False,'stopped':True,'projection':'observed_complete_prefix_not_dispatch_authority','reason':'Accounting snapshot unavailable; dispatch uses fresh authoritative checks.'}
 def input(self,p,body):
  mid=checked_id(body.get('id'));text=body.get('text')
  if not isinstance(text,str) or not text.strip() or len(text.encode())>65536:raise ValueError('Input must be nonblank and fit the native text envelope (64 KiB).')
  with self.lock(p):
   events=self.events(p);prior=next((e for e in events if e['id']==mid and e['type']=='input'),None)
   if prior and prior['text']!=text:raise ValueError('This submission identity belongs to different text.')
   if prior and prior.get('state')=='native_admitted':return {'state':'native_admitted','model_consumed':False}
   if not prior:self.append(p,{'id':mid,'type':'input','role':'user','text':text,'attribution':'local_user_input','state':'queued','source_snapshot':self.source_snapshot(p)})
   completed=next((e for e in self.events(p) if e['id']==mid and e['type']=='native_input'),None)
   if completed:return completed
   if p in self.preparation_pending:return {'state':'queued','reason':'Saved for the next turn; current preparation keeps its frozen input.'}
   if p in self.jobs and self.jobs[p].poll() is None:return {'state':'queued','reason':'Run active; input will not mutate its frozen context.'}
   if any(e['type']=='run_started' for e in self.events(p)):
    try:self.require_prior_terminal(p)
    except (ValueError,OSError):return {'state':'queued','reason':'Prior run needs reconciliation; native state was not changed.'}
   return self.admit_input(p,mid,text)
 def admit_input(self,p,mid,text):
  # Only a private UI derivative changes; original game/run remains immutable.
  c=self.native(self.project_native(p,create=True));
  if not self.native_factory:
   budget=c.budget()
   if any(budget[k] for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests')):raise ValueError('Native effects unresolved; input remains queued without changing native guidance')
  request=c.request();parents=request['coding']['relational']['inputs']['parents'];marker='\nUser input ['+mid+']: '+text
  if not parents:return {'state':'queued','reason':'Saved safely. This source currently exposes no eligible native guidance. A compatible scoped continuation must be admitted before this input can enter the engine. No model has consumed this input.'}
  if all(marker in x['meaning'] for x in parents):result={'recovered':True}
  else:
   completion=c.cli('project','acceptance','status');guidance={x['requirement']:x['meaning']+marker for x in parents}
   result=c.json_cli(('project','acceptance','feedback'),{'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],'guidance':guidance})
  after=c.request();entry=self.append(p,{'id':mid,'type':'native_input','role':'system','state':'native_admitted','text':'Your input is recorded as attributed native guidance, alongside the original contributing meanings. This is not an inferred desire or a model response.','original_contributors':parents,'native_root':after['state_root'],'snapshot':after['coding']['snapshot'],'source':'native acceptance feedback API; attributed guidance, not desire evolution','model_consumed':False});self.cache.pop(p,None);return entry
 def create(self,body):
  mid=checked_id(body.get('id'));text=body.get('text');title=body.get('title','New browser project')
  if not isinstance(text,str) or not text.strip() or len(text.encode())>65536 or not isinstance(title,str) or len(title)>120:raise ValueError('A title and bounded nonblank browser-project brief are required.')
  pid='task-'+mid
  with self.lock(pid):
   if (self.path(pid)/'project.json').exists():
    if self.meta(pid).get('initial_text')!=text:raise ValueError('Task identity conflict')
    return {'id':pid,'replayed':True}
   native=self.path(pid)/('native-'+str(uuid.uuid4()));native.mkdir()
   base=new_project_scope();base['editable']={'index.html':'<!doctype html><html><head><meta charset="utf-8"><title>New project</title><link rel="stylesheet" href="style.css"></head><body><button data-tsr="start">Start</button><p data-tsr="status">Not started</p><canvas width="640" height="480"></canvas><script src="app.js"></script></body></html>','style.css':'/* Awaiting engine design. */','app.js':'// Awaiting engine implementation.'};base['goal']='User-requested original browser project';save(self.path(pid)/'scope.json',base);save(self.path(pid)/'requirements.json',{'requested_outcome':text})
   c=self.native(native);c.cli('project','init','--spec',str(self.path(pid)/'scope.json'),'--requirements',str(self.path(pid)/'requirements.json'))
   completion=c.cli('project','acceptance','status');c.json_cli(('project','acceptance','feedback'),{'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],'guidance':{'requested_outcome':text}})
   save(self.path(pid)/'project.json',{'id':pid,'title':title,'subtitle':'New native browser project','source':str(self.path(pid)),'native':str(native),'artifact':None,'initial_text':text,'created_at':now(),'registered_packet':None});self.append(pid,{'id':mid,'type':'input','role':'user','text':text,'state':'native_admitted','attribution':'local_user_input','model_consumed':False});return {'id':pid}
 def timeline(self,p):
  meta=self.meta(p);source=Path(meta.get('registered_packet') or meta.get('watch') and ROOT/meta['watch'] or meta['source']);file=source/'adapter/adapter.jsonl';out=[]
  if file.exists():
   a=load('ui_timeline_adapter','engine-browser-adapter.py');ad=json.loads((source/'adapter-admission.json').read_text())
   rows=[json.loads(x) for x in file.read_text().splitlines()];fixture=bool(rows and rows[0]['event'].get('fixture_only'))
   holder=types.SimpleNamespace(file=file,admission=ad,fixture=fixture,row=a.d.Driver.row,journal_limit=a.base.capacity.JOURNAL_BYTES)
   _,_,done=a.d.Driver.read(holder)
   for name,v in done.items():
    kind=None;text=None
    if name.startswith('generated_'):
     kind='proposal';response=v.get('response',{});text=response.get('retain') or response.get('abstain') or response.get('reason')
     if not text:text='\n\n'.join(x.get('rationale') or x.get('proposal',{}).get('relationship','') for x in response.get('proposals',[]))
    elif name.startswith('reviewed_'):kind='review';text=v.get('report',{}).get('verdict','Review recorded')+': '+' '.join(x.get('explanation','') for x in v.get('report',{}).get('findings',{}).values())
    elif name.startswith('independent_assessment_'):kind='evidence';text='Independent browser observations: '+str(sum(x.get('pass',False) for x in (v.get('observations') or {}).get('checks',[])))+'/'+str(len((v.get('observations') or {}).get('checks',[])))+' checks passed. Bound to snapshot '+str(v.get('snapshot','unavailable'))
    elif name.startswith('execution_'):kind='execution';text='Native isolated execution returned. Review its receipt before inferring fulfillment.'
    elif name.startswith('session_continue_'):kind='continuation';text='Administrative native continuation: '+str(v.get('purpose',v.get('reason','recorded')))
    if kind:out.append({'id':name,'kind':kind,'text':text or 'Recorded; no readable rationale supplied.','source':str(file.relative_to(ROOT)) if file.is_relative_to(ROOT) else 'UI session journal','recorded':True,'current_session':bool(meta.get('registered_packet'))})
  cycle_file=source/'cycle/cycle.jsonl'
  if cycle_file.exists():
   a=load('ui_timeline_cycle','engine-browser-adapter.py');ca=json.loads((source/'cycle-admission.json').read_text())
   rows=[json.loads(x) for x in cycle_file.read_text().splitlines()];fixture=bool(rows and rows[0]['event'].get('fixture_only'))
   holder=types.SimpleNamespace(file=cycle_file,admission=ca,fixture=fixture,row=a.d.Driver.row,journal_limit=a.base.capacity.JOURNAL_BYTES)
   _,_,done=a.d.Driver.read(holder)
   if 'terminal' in done:out.append({'id':'terminal','kind':'decision','outcome':done['terminal'].get('status'),'text':done['terminal'].get('reason',done['terminal'].get('status','Stopped')),'source':str(cycle_file.relative_to(ROOT)),'recorded':True,'current_session':bool(meta.get('registered_packet'))})
  if meta.get('registered_packet'):
   binding=json.loads((source/'scope-authority.json').read_text());admission=json.loads((source/'adapter-admission.json').read_text())
   for item in out:item.update(input_ids=binding.get('input_ids',[]),admission_sha256=sha(admission),packet=str(source))
  return out
 def delivery(self,p):
  packet=self.meta(p).get('registered_packet')
  if not packet:return {}
  file=Path(packet)/'adapter/adapter.jsonl'
  if not file.exists():return {}
  ad=json.loads((Path(packet)/'adapter-admission.json').read_text());a=load('ui_delivery','engine-browser-adapter.py')
  rows=[json.loads(x) for x in file.read_text().splitlines()];holder=types.SimpleNamespace(file=file,admission=ad,fixture=bool(rows and rows[0]['event'].get('fixture_only')),row=a.d.Driver.row,journal_limit=a.base.capacity.JOURNAL_BYTES)
  _,_,done=a.d.Driver.read(holder);result={}
  frozen=json.loads((Path(packet)/'scope-authority.json').read_text()).get('input_ids',[])
  def strings(value):
   if isinstance(value,str):return [value]
   if isinstance(value,list):return [s for x in value for s in strings(x)]
   if isinstance(value,dict):return [s for x in value.values() for s in strings(x)]
   return []
  for key,value in done.items():
   if not key.startswith('request_body_') or value['body_sha256']!=a.e.sha(value['body']):continue
   ident=key.removeprefix('request_body_');receipt=done.get('dispatch_'+ident,{}).get('receipt',{})
   if receipt.get('http_status')!=200 or receipt.get('response_status')!='received' or 'bound_output_'+ident not in done:continue
   bound=done['bound_output_'+ident];association=bound.get('association',{})
   if association.get('request')!=ident or association.get('request_body_sha256')!=value['body_sha256'] or association.get('source')!=value.get('source') or association.get('admission_sha256')!=a.e.sha(ad) or association.get('receipt_sha256')!=a.e.sha(receipt):continue
   texts=[]
   for message in value['body'].get('messages',[]):
    content=message.get('content','')
    try:texts+=strings(json.loads(content))
    except (ValueError,TypeError):texts.append(content)
   for event in self.events(p):
    if event['type']!='input' or event['id'] not in frozen:continue
    # The public brief contains structured JSON; decode that layer, never rewrite it.
    found=False
    for text in texts:
     for line in text.split('Recorded input:\n')[1:]:
      try:items=json.loads(line);found=any(x.get('id')==event['id'] and x.get('text')==event['text'] for x in items)
      except (ValueError,TypeError,AttributeError):pass
     if found:break
    if found:result[event['id']]={'state':'provider_delivered','request_body_sha256':value['body_sha256'],'source':value.get('source'),'meaning':'Exact frozen input present in a successful authenticated provider response; not proof of reliance or learning'}
  return result
 def view(self,p):
  meta=self.meta(p);cached=self.cache.get(p)
  if cached and time.time()-cached[0]<6:state=cached[1]
  else:
   try:
    c=self.native(self.project_native(p));q=c.request();rel=q['coding']['relational'];state={'root':q['state_root'],'snapshot':q['coding']['snapshot'],'parents':rel['inputs']['parents'],'relationships':rel.get('history',[]),'returns':rel['inputs'].get('returns',[]),'semantic_return':rel['inputs'].get('semantic_return'),'expectation':None,'native':True}
    try:
     history=c.cli('project','acceptance','published-history')
     if history.get('version')!='published-guidance-history@v1' or history.get('state_root')!=q['state_root'] or history.get('current_source')!=q['coding']['snapshot']:raise ValueError('Published history changed during read')
     state['published_history']=history
    except Exception:state['history_error']='Retained contributor history is unavailable or changed during this read; no missing desires are inferred.'
    self.cache[p]=(time.time(),state)
   except Exception:state={'native':False,'error':'Native state unavailable; recorded inputs are preserved.'}
  try:timeline=self.timeline(p);timeline_error=None
  except Exception:timeline=[];timeline_error='Work journal is unavailable or failed integrity verification. No response is inferred.'
  try:delivery=self.delivery(p)
  except Exception:delivery={}
  worker=self.jobs.get(p);exit_code=worker.poll() if worker else None
  worker_error=('Engine worker stopped before recording a result (exit '+str(exit_code)+'). Your message is saved. No automatic replay of uncertain work; inspect the activity record.' if exit_code not in (None,0) and not any(e['kind']=='decision' and e.get('current_session') for e in timeline) else None)
  return {'project':{k:v for k,v in meta.items() if k in ('id','title','subtitle','created_at','artifact')},'state':state,'messages':self.events(p),'artifacts':[{'id':e['id'],'title':e.get('title','Verified candidate'),'url':'/preview/'+p+'/'+e['id']+'/index.html','snapshot':e['source_snapshot']} for e in self.events(p) if e.get('type')=='artifact_delivered'],'delivery':delivery,'timeline':timeline,'timeline_error':timeline_error,'worker_error':worker_error,'running':p in self.jobs and self.jobs[p].poll() is None,'preparing':p in self.preparing and self.preparing[p].is_alive(),'dispatch_ready':True,'limits':'General new-desire identity succession and hidden model reasoning are not available. These are recorded wants, relationships, rationale summaries and observations.'}
 def delivered_preview(self,p,revision,name):
  checked_id(p);checked_id(revision)
  if name not in ('index.html','style.css','app.js'):raise ValueError('Unknown preview file')
  entries=[e for e in self.events(p) if e.get('type')=='artifact_delivered' and e['id']==revision]
  if len(entries)!=1:raise ValueError('Unknown delivered revision')
  entry=entries[0];relative=Path(entry['artifact'])
  if relative.is_absolute() or '..' in relative.parts:raise ValueError('Invalid delivery path')
  base=ROOT/relative
  if not base.resolve().is_relative_to((ROOT/'artifacts').resolve()) or any(x.is_symlink() for x in (base,*base.parents)):raise ValueError('Invalid delivery boundary')
  file=base/name
  if file.is_symlink():raise ValueError('Invalid preview file')
  data=file.read_bytes()
  if hashlib.sha256(data).hexdigest()!=entry['source_sha256'].get(name):raise ValueError('Delivered source changed')
  return data
 def download(self,p):
  c=self.native(self.project_native(p));budget=c.budget()
  if any(budget.get(k) for k in ('unresolved_tool_requests','unknown_compute_requests')):raise ValueError('An execution outcome is unresolved; no current-source artifact is certified.')
  q=c.request();files=[f for f in q['coding']['files'] if f.get('editable')]
  out=io.BytesIO()
  with zipfile.ZipFile(out,'w',zipfile.ZIP_DEFLATED) as archive:
   for f in files:
    path=Path(f['path'])
    if path.is_absolute() or '..' in path.parts:raise ValueError('Invalid artifact path')
    archive.writestr(f['path'],f['source'])
   archive.writestr('SOURCE.json',json.dumps({'snapshot':q['coding']['snapshot'],'state_root':q['state_root'],'note':'Current editable source. This download is not independent acceptance or proof of playability.'},indent=2))
  return out.getvalue()
 def register(self,p,packet):
  # CLI operator registration only; browser clients cannot choose paths or commands.
  packet=Path(packet).resolve();allowed=(ROOT/'target').resolve()
  if not packet.is_relative_to(allowed) or any(x.is_symlink() for x in (packet,*packet.parents)):raise ValueError('Packet path boundary')
  ad=json.loads((packet/'adapter-admission.json').read_text());g=json.loads((packet/'scope-grant.json').read_text());m=json.loads((packet/'prepared-manifest.json').read_text())
  unlimited=self.budget().get('unlimited_policy_sha256')
  if ad.get('unlimited_policy_sha256'):
   if ad['unlimited_policy_sha256']!=unlimited or g.get('unlimited_policy_sha256')!=unlimited or any(ad[k] is not None for k in ('max_calls','max_executions','expires_at')) or g['session_cap_microusd'] is not None:raise ValueError('Unlimited packet policy mismatch')
  if ad['model']!='gpt-6-luna' or (not ad.get('unlimited_policy_sha256') and (ad['max_calls']>200 or ad['max_executions']>100 or g['session_cap_microusd']>10000000)):raise ValueError('Run budget exceeds UI policy')
  if Path(ad['origin']['directory']).resolve()!=self.project_native(p).resolve():raise ValueError('Packet does not own this UI native project')
  if m['adapter_sha256']!=sha(ad) or m['grant_sha256']!=hashlib.sha256(json.dumps(g,sort_keys=True,separators=(',',':')).encode()).hexdigest():raise ValueError('Packet identity mismatch')
  meta=self.meta(p);meta['registered_packet']=str(packet);save(self.path(p)/'project.json',meta)
 def require_prior_terminal(self,p,recovery_lease=None):
  launches=[e for e in self.events(p) if e['type']=='run_started']
  if not launches:return
  packet=Path(launches[-1]['packet']);a=load('ui_resume_adapter','engine-browser-adapter.py')
  ad=json.loads((packet/'adapter-admission.json').read_text());ca=json.loads((packet/'cycle-admission.json').read_text())
  campaign,m,ledger,upgrade=self.ledger()
  adapter=types.SimpleNamespace(file=Path(ad['directory'])/'adapter.jsonl',admission=ad,fixture=bool(campaign.get('fixture_only')),row=a.d.Driver.row,journal_limit=a.base.capacity.JOURNAL_BYTES)
  _,pending,_=a.d.Driver.read(adapter)
  cycle=types.SimpleNamespace(file=Path(ca['journal'])/'cycle.jsonl',admission=ca,fixture=False,row=a.d.Driver.row,journal_limit=a.base.capacity.JOURNAL_BYTES)
  _,cycle_pending,done=a.d.Driver.read(cycle)
  if pending or cycle_pending:
   dispositions=[e for e in self.events(p) if e['type']=='operator_projection_recovery' and e.get('proof',{}).get('packet')==str(packet)]
   if len(dispositions)==1:
    from recovery import verify
    return verify(self,p,packet,dispositions[0]['proof'],recovery_lease)
   raise ValueError('Prior session has unresolved effects; no redispatch occurred.')
  if 'terminal' in done:return
  # A cooperative operator checkpoint is resumable through a NEW admission only.
  # Never relabel it completed or reopen the paused journal.
  with contextlib.ExitStack() as stack:
   for path in (packet/'adapter/launch.lock',packet/'cycle/run.lock'):
    lock=stack.enter_context(os.fdopen(os.open(path,os.O_WRONLY|os.O_NOFOLLOW),'a'));fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
   stop=json.loads((packet/'cycle/STOP').read_text());pid=launches[-1].get('pid')
   if stop.get('kind')!='operator_checkpoint_pause' or stop.get('admission_sha256')!=a.e.sha(ad) or stop.get('worker_pid')!=pid or type(pid) is not int or pid<=1 or stop.get('final_user_acceptance') is not False:raise ValueError('Unverified prior checkpoint; no redispatch occurred.')
   try:os.kill(pid,0)
   except ProcessLookupError:pass
   else:raise ValueError('Prior checkpoint worker still present; no duplicate launch.')
   if a.d.Driver.read(adapter)[1] or a.d.Driver.read(cycle)[1]:raise ValueError('Checkpoint has unresolved journal effects.')
   status=ledger.status()
   if status.get('blocking_unknown_calls') or status.get('blocking_unknown_executions'):raise ValueError('Checkpoint has unresolved external effects.')
   return {'state':'settled_operator_checkpoint','completed':False}

 def runtime_preflight(self):
  if getattr(self,'fixture_only',False):return {'fixture_only':True}
  docker=shutil.which('docker')
  if not docker:raise ValueError('Docker executable unavailable. No model or execution reservation started.')
  result=subprocess.run([docker,'image','inspect','sha256:58deb9d12b7291f6d2c697b335eb927557d707488df09d1503659d7abdd00883','--format','{{.Id}}'],capture_output=True,text=True,timeout=15)
  if result.returncode or result.stdout.strip()!='sha256:58deb9d12b7291f6d2c697b335eb927557d707488df09d1503659d7abdd00883':raise ValueError('Existing isolated browser runtime unavailable; no model dispatch started.')
  return {'docker':docker,'image':result.stdout.strip()}
 def start(self,p,body):
  from recovery import guard
  from context_refresh import guard as refresh_guard
  with refresh_guard(self,p,body.get('id')) as refresh:
   with guard(self,p) as lease:return self._start(p,body,lease,refresh)
 def _start(self,p,body,recovery_lease=None,refresh=None):
  if body.get('confirm')!=('Run under the active unlimited policy' if self.budget().get('unlimited_policy_sha256') else 'Run this reviewed session with up to $10'):raise ValueError('Explicit run confirmation required')
  with self.lock(p):
   request_id=checked_id(body.get('id'));prior=next((e for e in self.events(p) if e['type']=='run_request' and e['id']==request_id),None)
   if prior:return {'state':'already_requested','replayed':True,'text':'This Start request was already recorded. No redispatch.'}
   budget=self.budget()
   if budget.get('stopped') or not budget.get('available'):raise ValueError('Accounting recovery is required. Nothing dispatched.')
   if p in self.preparation_pending:return {'state':'preparing','replayed':True}
   if p in self.jobs and self.jobs[p].poll() is None:return {'state':'running','replayed':True}
   self.require_prior_terminal(p,recovery_lease)
   predecessor=None;refresh_inputs=None
   if refresh:
    from context_refresh import verify_successor
    verified_refresh=verify_successor(self,p,refresh);predecessor=verified_refresh['predecessor'];refresh_inputs=set(verified_refresh['input_ids'])
   self.runtime_preflight()
   frozen=[e for e in self.events(p) if e['type'] in ('input','assessment') and (refresh_inputs is None or e['id'] in refresh_inputs)]
   approved=self.decisions.approval(p)
   if approved and request_id=='send-'+str(approved['disclosure'].get('resume_input_id')):
    if self.source_snapshot(p)!=approved['disclosure']['source']:raise ValueError('Project guidance changed after this approval. Saved input is retained; submit the intended current message explicitly.')
    frozen=[e for e in frozen if e['id'] in approved['disclosure']['input_ids']]
   with self.control(p):
    stopped_ids={request_id,predecessor}
    if recovery_lease:
     prior_launch=next((e for e in reversed(self.events(p)) if e['type']=='run_started' and e.get('packet')==recovery_lease and e.get('pid')),None)
     if not prior_launch:raise ValueError('Recovery launch provenance missing')
     stopped_ids.add(prior_launch.get('request_id'))
     if (Path(recovery_lease)/'cycle/STOP').exists():raise ValueError('Stop remains active; no recovery preparation')
    if any(e.get('request_id') in stopped_ids and e['kind']=='stop' for e in self.controls(p)):return {'state':'stopped','text':'Stopped before preparation; nothing dispatched.'}
    self.append(p,{'id':request_id,'type':'run_request','role':'system','text':'Preparing the submitted messages under the recorded engine policy.','state':'preparing','input_ids':[e['id'] for e in frozen]})
    self.active_attempts[p]=request_id;self.preparation_pending.add(p)
  try:
   from prepare import prepare
   packet=Path(prepare(self,p,input_events=frozen))
   with self.lock(p):
    self.register(p,str(packet));m=json.loads((packet/'prepared-manifest.json').read_text());ad=json.loads((packet/'adapter-admission.json').read_text());expires=None if ad.get('unlimited_policy_sha256') else datetime.datetime.fromisoformat(ad['expires_at']).timestamp()
    if expires is not None and (expires-time.time()<240 or expires-time.time()>7200):raise ValueError('Session deadline is expired or outside policy')
    cmd=self.run_command(packet)
    with self.control(p):
     if any(e.get('request_id') in stopped_ids and e['kind']=='stop' for e in self.controls(p)):raise ValueError('Stopped before model dispatch. Messages and history are saved.')
     self.append(p,{'id':str(uuid.uuid4()),'type':'run_started','role':'system','text':'Launch reserved; an uncertain restart cannot redispatch it.','packet':str(packet),'state':'launch_reserved','request_id':request_id})
     with (self.path(p)/'run.log').open('ab') as log:proc=self.launch(cmd,log,packet)
     self.jobs[p]=proc
     self.append(p,{'id':str(uuid.uuid4()),'type':'run_started','role':'system','text':'Engine session started. Every model call and action is accounted.','pid':proc.pid,'packet':str(packet),'state':'running','request_id':request_id})
    return {'state':'running'}
  finally:self.preparation_pending.discard(p)
 def run_command(self,packet):
  m=json.loads((packet/'prepared-manifest.json').read_text())
  return [sys.executable,str(ROOT/'scripts/engine-native-cycle.py'),'--browser-v1','run','--admission',str(packet/'adapter-admission.json'),'--expected-sha256',m['adapter_sha256'],'--cycle-admission',str(packet/'cycle-admission.json'),'--campaign-manifest',str(ROOT/'target/tsr-integrated-campaign-1/manifest.json'),'--campaign-upgrade',str(packet/'campaign-upgrade.json'),'--scope-grant',str(packet/'scope-grant.json'),'--scope-grant-sha256',m['grant_sha256']]
 def unstarted_packet(self,p,rid):
  """Read-only proof: no campaign enrollment or durable effect intent, dead child."""
  events=self.events(p)
  if any(e['type']=='startup_retry' and e.get('request_id')==rid for e in events):raise ValueError('Startup recovery already used; inspect its exact result, no automatic repeat.')
  launches=[e for e in events if e['type']=='run_started' and e.get('request_id')==rid and e.get('pid')]
  if not launches:raise ValueError('No known launched process to reconcile')
  launch=launches[-1];pid=launch['pid']
  if type(pid) is not int or pid<=1:raise ValueError('No authentic child identity')
  try:os.kill(pid,0)
  except ProcessLookupError:pass
  else:raise ValueError('Prior process is still present; no duplicate launch')
  packet=Path(launch['packet'])
  if str(packet)!=self.meta(p).get('registered_packet'):raise ValueError('Startup packet changed')
  ad=json.loads((packet/'adapter-admission.json').read_text());m=load('startup_proof','engine-browser-adapter.py')
  manifest=json.loads((packet/'prepared-manifest.json').read_text());ca=json.loads((packet/'cycle-admission.json').read_text());grant=json.loads((packet/'scope-grant.json').read_text())
  if manifest['adapter_sha256']!=m.e.sha(ad) or manifest['grant_sha256']!=m.e.sha(grant):raise ValueError('Startup manifest binding changed')
  if Path(ca['journal']).resolve()!=(packet/'cycle').resolve():raise ValueError('Startup cycle journal path changed')
  holder=types.SimpleNamespace(file=Path(ad['directory'])/'adapter.jsonl',admission=ad,fixture=bool(self.ledger()[0].get('fixture_only')),row=m.d.Driver.row,journal_limit=m.base.capacity.JOURNAL_BYTES)
  rows,pending,done=m.d.Driver.read(holder)
  if len(rows)!=1 or pending or done:raise ValueError('Prior durable stages exist; startup replay is forbidden')
  for journal in (packet/'cycle').rglob('*.jsonl'):
   if any(json.loads(line)['event']['kind']!='admission' for line in journal.read_text().splitlines()):raise ValueError('Prior cycle effects exist; startup replay is forbidden')
  _,campaign,ledger,_=self.ledger();trial=campaign.digest({'trial':str(Path(ad['directory']).resolve()),'admission':m.e.sha(ad)})
  with ledger.locked() as (_,entries):
   if any(e['event'].get('key')==trial or e['event'].get('value',{}).get('session_trial_sha256')==trial for e in entries):raise ValueError('Prior campaign enrollment exists; no startup replay')
  authority=json.loads((packet/'scope-authority.json').read_text());q=self.native(self.project_native(p)).request()
  if q['state_root']!=authority['source_root'] or q['coding']['snapshot']!=authority['source_snapshot']:raise ValueError('Startup native source changed')
  return packet,{'pid':pid,'packet':str(packet),'admission_sha256':m.e.sha(ad),'adapter_journal_sha256':m.d.file_sha(holder.file),'ledger_head':entries[-1]['hash'],'source_root':q['state_root'],'finding':'dead child; no durable dispatch stages or campaign enrollment'}
 def settled_packet(self,p,rid):
  """Ordinary journal replay only: retained results, no unresolved effect or reset."""
  launches=[e for e in self.events(p) if e['type']=='run_started' and e.get('request_id')==rid and e.get('pid')]
  if not launches:raise ValueError('No prior worker')
  last=launches[-1];pid=last['pid']
  if type(pid) is not int or pid<=1:raise ValueError('Invalid worker identity')
  try:os.kill(pid,0)
  except ProcessLookupError:pass
  else:raise ValueError('Worker still present; no concurrent resume')
  if any(e['type']=='settled_resume' and e.get('prior_launch')==last['id'] for e in self.events(p)):raise ValueError('Resume already reserved for this worker')
  packet=Path(last['packet'])
  if str(packet)!=self.meta(p).get('registered_packet'):raise ValueError('Registered packet changed')
  a=load('settled_resume','engine-browser-adapter.py');ad=json.loads((packet/'adapter-admission.json').read_text());ca=json.loads((packet/'cycle-admission.json').read_text());manifest=json.loads((packet/'prepared-manifest.json').read_text());grant=json.loads((packet/'scope-grant.json').read_text())
  if manifest['adapter_sha256']!=a.e.sha(ad) or manifest['grant_sha256']!=a.e.sha(grant) or Path(ca['journal']).resolve()!=(packet/'cycle').resolve():raise ValueError('Resume packet binding changed')
  fixture=bool(self.ledger()[0].get('fixture_only'))
  for file,admission in [(Path(ad['directory'])/'adapter.jsonl',ad),(Path(ca['journal'])/'cycle.jsonl',ca)]:
   h=types.SimpleNamespace(file=file,admission=admission,fixture=fixture,row=a.d.Driver.row,journal_limit=a.base.capacity.JOURNAL_BYTES)
   rows,pending,done=a.d.Driver.read(h)
   if pending:raise ValueError('Unresolved journal effect; no ordinary resume')
   if 'terminal' in done or 'session_completed' in done:raise ValueError('Completed checkpoint; do not reopen')
  if self.budget().get('stopped',True):raise ValueError('Accounting blocks resume')
  return packet,{'prior_launch':last['id'],'pid':pid,'admission_sha256':a.e.sha(ad),'adapter_sha256':a.d.file_sha(Path(ad['directory'])/'adapter.jsonl'),'cycle_sha256':a.d.file_sha(Path(ca['journal'])/'cycle.jsonl'),'finding':'dead worker; authenticated settled journals; ordinary native replay retains completed responses'}
 def resume_settled(self,p,rid):
  with self.lock(p):
   packet,proof=self.settled_packet(p,rid)
   self.runtime_preflight()
   with self.control(p):
    if any(e.get('request_id')==rid and e['kind']=='stop' for e in self.controls(p)):raise ValueError('Stop remains active')
    self.append(p,{'id':str(uuid.uuid4()),'type':'settled_resume','role':'system','state':'resuming','text':'Resuming retained results after a local interruption. No completed model call or action is repeated.','request_id':rid,'prior_launch':proof['prior_launch'],'proof':proof})
    with (self.path(p)/'run.log').open('ab') as log:proc=self.launch(self.run_command(packet),log,packet)
    self.jobs[p]=proc;self.active_attempts[p]=rid
    self.append(p,{'id':str(uuid.uuid4()),'type':'run_started','role':'system','state':'running','text':'Engine resumed its existing source-bound journal.','request_id':rid,'packet':str(packet),'pid':proc.pid})
   return {'state':'running'}
 def resume_unstarted(self,p,rid):
  with self.lock(p):
   packet,proof=self.unstarted_packet(p,rid)
   self.runtime_preflight()
   with self.control(p):
    if any(e.get('request_id')==rid and e['kind']=='stop' for e in self.controls(p)):raise ValueError('Stop remains active; no startup retry')
    self.append(p,{'id':'startup-retry-'+rid,'type':'startup_retry','role':'system','state':'retrying_startup','text':'Recovering a confirmed pre-dispatch startup failure; no model call or action is repeated.','request_id':rid,'proof':proof})
    with (self.path(p)/'run.log').open('ab') as log:proc=self.launch(self.run_command(packet),log,packet)
    self.jobs[p]=proc;self.active_attempts[p]=rid
    self.append(p,{'id':str(uuid.uuid4()),'type':'run_started','role':'system','state':'running','text':'Engine worker resumed before its first effect.','request_id':rid,'packet':str(packet),'pid':proc.pid})
   return {'state':'running'}
 def launch(self,cmd,log,packet):
  return subprocess.Popen(cmd,cwd=ROOT,stdout=log,stderr=log,start_new_session=True)
 def recovery_disclosure(self,observed_budget=None):
  b=self.budget() if observed_budget is None else observed_budget;unknown=b.get('unknown_calls',[]);tools=b.get('unknown_executions',[])
  kind='accounting_unavailable' if not b.get('available') else 'unknown_execution' if tools else 'budget_exceeded' if b.get('overrun') else 'unknown_charge' if b.get('blocking_unknown_calls',unknown) else 'ready'
  text=("Permanently abandon the interrupted tool-free model output and never execute it. Retain its uncertain historical charge: the $0.05 reservation is an estimate, not a verified maximum. The combined historical and new total cannot be guaranteed under $10. Permit only fresh requests within the displayed remaining accounting allowance; preserve prior spending and all history. This does not approve retrying unknown tool or execution effects.")
  value={'version':'ui-recovery-disclosure@v1','blocker':kind,'unknown_calls':unknown,'unknown_executions':tools,'known_microusd':b.get('known_cost_microusd'),'committed_microusd':b.get('committed_microusd'),'text':text}
  return {**value,'digest':sha(value),'required':bool(b.get('stopped')),'can_request':kind=='unknown_charge','authentication':'loopback_single_user_interaction','action':'Review the exact recovery action and approve or decline here. No decision is made by opening this page.'}
 def send(self,p,body):
  if body.get('confirm')!=('Send to engine under unlimited policy' if self.budget().get('unlimited_policy_sha256') else 'Send to engine within $10'):raise ValueError('Send confirmation required')
  mid=checked_id(body.get('id'));saved=self.input(p,body)
  with self.lock(p):
   if not any(e['type']=='input_submitted' and e.get('input_id')==mid for e in self.events(p)):self.append(p,{'id':'submitted-'+mid,'type':'input_submitted','input_id':mid,'role':'system','state':'queued','text':'Input submitted to the engine.'})
   budget=self.budget()
   if budget.get('stopped') or not budget.get('available'):
    disclosure=self.recovery_disclosure();message='Saved. Review the recovery decision here before new spending.' if disclosure['can_request'] else 'Saved. '+{'accounting_unavailable':'Accounting is unavailable; retry after service recovery.','unknown_execution':'An execution outcome needs reconciliation; financial approval alone cannot clear it.','budget_exceeded':'The budget is exhausted or exceeded; no new spending is permitted.'}.get(disclosure['blocker'],'The session is blocked.')
    return {'state':'approval_required' if disclosure['can_request'] else 'blocked','saved':True,'message':message,'recovery':disclosure}
   if p in self.preparing and self.preparing[p].is_alive():return {'state':'preparing','saved':True,'message':'Your message is saved. Session preparation is already in progress.'}
   if p in self.jobs and self.jobs[p].poll() is None:return {'state':'queued','saved':True,'message':'Saved for the next turn. The current run keeps its original context.'}
   rid='send-'+mid
   retry_startup=any(e['type']=='run_request' and e['id']==rid for e in self.events(p))
   resume_settled=False
   if retry_startup:
    try:self.unstarted_packet(p,rid)
    except ValueError:
     try:self.settled_packet(p,rid);resume_settled=True
     except ValueError:return {'state':'already_requested','saved':True,'message':'This message already has a run request. Its result or blocker is shown below; nothing was dispatched twice.'}
   with self.control(p):
    if not any(e['type']=='run_scheduled' and e['id']==rid for e in self.events(p)):self.append(p,{'id':rid,'type':'run_scheduled','state':'scheduled','role':'system','text':'Message scheduled for preparation; no model call yet.'})
    self.active_attempts[p]=rid
   def work():
    try:
     if resume_settled:self.resume_settled(p,rid)
     elif retry_startup:self.resume_unstarted(p,rid)
     else:self.start(p,{'id':rid,'confirm':'Run under the active unlimited policy' if self.budget().get('unlimited_policy_sha256') else 'Run this reviewed session with up to $10'})
    except Exception as exc:
     with self.lock(p):self.append(p,{'id':rid,'type':'interaction_error','role':'system','state':'blocked','text':str(exc),'source':'local session controller; not a model response'})
   thread=threading.Thread(target=work,daemon=True);self.preparing[p]=thread;thread.start()
   return {'state':'preparing','saved':True,'message':'Preparing your engine session. You can follow its progress here.'}
 def refuse_recovery(self,p,body):
  d=self.recovery_disclosure()
  if body.get('disclosure')!=d['digest']:raise ValueError('Recovery disclosure changed; review it again.')
  mid=checked_id(body.get('id'))
  with self.lock(p):
   if not any(e['type']=='recovery_refused' and e['id']==mid for e in self.events(p)):self.append(p,{'id':mid,'type':'recovery_refused','role':'system','state':'saved','text':'Recovery not approved. Messages and drafts remain available; no spending is enabled.','disclosure':d})
  return {'state':'not_approved','message':'No problem. Your messages stay saved; nothing will run.'}
 def stop(self,p):
  with self.control(p):
   rid=self.active_attempts.get(p)
   if rid is None:rid=next((e['id'] for e in reversed(self.events(p)) if e['type'] in ('run_request','run_scheduled')),None)
   event={'kind':'stop','request_id':rid,'at':now()};self.controls(p);control_file=self.path(p)/'controls.jsonl'
   rows=[json.loads(x) for x in control_file.read_text().splitlines()] if control_file.exists() else []
   row={'previous':rows[-1]['hash'] if rows else None,'event':event};row['hash']=sha(row)
   with control_file.open('a') as f:f.write(json.dumps(row)+'\n');f.flush();os.fsync(f.fileno())
   proc=self.jobs.get(p);active=proc is not None and proc.poll() is None
   if active:proc.terminate()
  with self.lock(p):self.append(p,{'id':str(uuid.uuid4()),'type':'stop_requested','role':'system','text':'Stop requested. Completed work, desires and history remain. In-flight effects may still require reconciliation.','state':'stop_requested'})
  return {'state':'stop_requested' if active or p in self.preparation_pending else 'not_running','text':'Stop recorded. Work not yet launched will stay stopped; any already dispatched effects must settle safely.'}

class Handler(http.server.BaseHTTPRequestHandler):
 server_version='TSRLocal/1'
 def log_message(self,*args):pass
 def send(self,status,value,content_type='application/json',preview=False):
  data=json.dumps(value,ensure_ascii=False).encode() if content_type=='application/json' else value;self.send_response(status);self.send_header('Content-Type',content_type);self.send_header('Content-Length',str(len(data)));
  if content_type=='application/zip':self.send_header('Content-Disposition','attachment; filename=tsr-project.zip')
  self.send_header('Cache-Control','no-store');self.send_header('X-Content-Type-Options','nosniff');self.send_header('Content-Security-Policy',(f"sandbox allow-scripts; default-src 'none'; script-src {preview}; style-src {preview}; img-src {preview} data:; connect-src 'none'; form-action 'none'; frame-ancestors {self.server.origin}" if preview else "default-src 'self'; script-src 'self'; style-src 'self'; img-src 'self' data:; connect-src 'self'; object-src 'none'; frame-ancestors 'none'"));self.end_headers();self.wfile.write(data)
 def valid_host(self):return self.headers.get('Host')==self.server.expected_host
 def do_GET(self):
  if not self.valid_host():return self.send(403,{'error':'Invalid host'})
  path=urllib.parse.urlparse(self.path).path;store=self.server.store
  try:
   if path.startswith('/preview/'):
    parts=path.split('/')
    if len(parts)==5:
     pid=checked_id(parts[2]);revision=checked_id(parts[3]);name=parts[4]
     data=store.delivered_preview(pid,revision,name)
     return self.send(200,data,{'.html':'text/html; charset=utf-8','.css':'text/css','.js':'text/javascript'}[Path(name).suffix],preview=self.server.origin+'/preview/'+pid+'/'+revision+'/')
    if len(parts)!=4 or parts[3] not in ('index.html','style.css','app.js'):raise ValueError('Unknown preview file')
    pid=checked_id(parts[2]);entry=next((x for x in store.catalog if x['id']==pid),None)
    if not entry or not entry.get('artifact'):raise ValueError('No delivered preview')
    original=ROOT/entry['artifact'];base=original.resolve();candidate=original/parts[3];file=candidate.resolve()
    if file.parent!=base or candidate.is_symlink() or any(x.is_symlink() for x in (original,*original.parents)):raise ValueError('Invalid preview path')
    return self.send(200,file.read_bytes(),{'.html':'text/html; charset=utf-8','.css':'text/css','.js':'text/javascript'}[file.suffix],preview=self.server.origin+'/preview/'+pid+'/')
   if path=='/api/bootstrap':
    observed=store.budget_snapshot()
    return self.send(200,{'csrf':store.csrf,'fixture_only':getattr(store,'fixture_only',False),'projects':[{'id':x.name,**{k:v for k,v in store.meta(x.name).items() if k in ('title','subtitle','accent')}} for x in (store.root/'projects').iterdir() if (x/'project.json').exists()],'budget':observed,'recovery':store.recovery_disclosure(observed)})
   if path.startswith('/api/project/') and path.endswith('/download'):return self.send(200,store.download(checked_id(path.split('/')[-2])),'application/zip')
   if path.startswith('/api/project/'):return self.send(200,store.view(checked_id(path.split('/')[-1])))
   files={'/':'index.html','/app.js':'app.js','/style.css':'style.css','/vendor/react.js':'vendor/react.js','/vendor/react-dom.js':'vendor/react-dom.js'}
   if path not in files:return self.send(404,{'error':'Not found'})
   file=STATIC/files[path];return self.send(200,file.read_bytes(),{'.html':'text/html; charset=utf-8','.css':'text/css','.js':'text/javascript'}[file.suffix])
  except (ValueError,KeyError,OSError) as e:return self.send(400,{'error':str(e)})
 def do_POST(self):
  if not self.valid_host() or self.headers.get('Origin')!=self.server.origin or self.headers.get('X-TSR-CSRF')!=self.server.store.csrf:return self.send(403,{'error':'Local origin and session token required'})
  try:
   size=int(self.headers.get('Content-Length','0'))
   if not 0<size<=131072:raise ValueError('Request size outside local input envelope')
   if self.headers.get('Content-Type')!='application/json':raise ValueError('JSON required')
   body=json.loads(self.rfile.read(size));parts=urllib.parse.urlparse(self.path).path.strip('/').split('/');store=self.server.store
   if parts==['api','projects']:return self.send(200,store.create(body))
   if len(parts)!=4 or parts[:2]!=['api','project']:raise ValueError('Unknown operation')
   p=checked_id(parts[2]);action=parts[3]
   result=store.decisions.present(p,body) if action=='decision-present' else store.decisions.decide(p,body) if action=='decision' else store.send(p,body) if action=='send' else store.refuse_recovery(p,body) if action=='recovery-refuse' else store.input(p,body) if action=='input' else store.start(p,body) if action=='run' else store.resume_settled(p,checked_id(body.get('id'))) if action=='resume-settled' and set(body)=={'id'} else store.stop(p) if action=='stop' else None
   if result is None:raise ValueError('Unknown operation')
   return self.send(200,result)
  except (ValueError,KeyError,OSError,json.JSONDecodeError) as e:return self.send(409,{'error':str(e)})
def serve(port=8766,state=None):
 server=http.server.ThreadingHTTPServer(('127.0.0.1',port),Handler);server.store=Store(state);from context_refresh import monitor;threading.Thread(target=monitor,args=(server.store,),daemon=True).start();server.expected_host=f'127.0.0.1:{server.server_port}';server.origin='http://'+server.expected_host;threading.Thread(target=server.store.decisions.resume_recorded,daemon=True).start();print(server.origin,flush=True);server.serve_forever()
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--port',type=int,default=8766);p.add_argument('--state',type=Path);p.add_argument('--register',nargs=2,metavar=('PROJECT','PACKET'));args=p.parse_args();
 if args.register:Store(args.state).register(*args.register)
 else:serve(args.port,args.state)
