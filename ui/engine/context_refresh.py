"""Queued context crosses a settled boundary through a fresh admission, never a patch to a live request."""
import contextlib,fcntl,json,os,time,types
from pathlib import Path

SAFE_TERMINALS={'proposal_retention','proposal_abstention','idle','advisory_reconsidered','feedback_cycle_complete','post_action_advisory_admitted'}

def launch_event(store,p):
 return next((e for e in reversed(store.events(p)) if e['type']=='run_started' and e.get('pid')),None)

def queued(store,p,launch):
 events=store.events(p)
 prior=next(e for e in events if e['type']=='run_request' and e['id']==launch['request_id'])
 used=set(prior['input_ids']);submitted={e['input_id'] for e in events if e['type']=='input_submitted'}
 return [e for e in events if e['id'] not in used and (e['type']=='assessment' and e.get('state')=='queued' or e['type']=='input' and e['id'] in submitted)]

def inspect(store,p,launch):
 from server import load
 a=load('refresh_adapter','engine-browser-adapter.py');packet=Path(launch['packet'])
 if store.meta(p).get('registered_packet')!=str(packet):raise ValueError('Refresh predecessor changed')
 ad=json.loads((packet/'adapter-admission.json').read_text());ca=json.loads((packet/'cycle-admission.json').read_text());manifest=json.loads((packet/'prepared-manifest.json').read_text())
 if ad['directory']!=str(packet/'adapter') or ca['journal']!=str(packet/'cycle') or ad['origin']['directory']!=str(packet/'native') or manifest['adapter_sha256']!=a.e.sha(ad):raise ValueError('Refresh admission binding changed')
 if not ad.get('unlimited_policy_sha256') or ad['unlimited_policy_sha256']!=store.budget().get('unlimited_policy_sha256'):raise ValueError('Automatic context refresh requires the active ongoing policy')
 def journal(folder,admission):
  h=types.SimpleNamespace(file=packet/folder/(folder+'.jsonl'),admission=admission,fixture=False,row=a.d.Driver.row,journal_limit=a.base.capacity.JOURNAL_BYTES)
  return a.d.Driver.read(h)
 return a,packet,ca,journal('adapter',ad),journal('cycle',ca)

def verify_successor(store,p,request_id):
 """Called under project lock before source copy; locks remain held through launch."""
 from server import sha
 events=store.events(p);record=next((e for e in events if e['type']=='context_refresh_scheduled' and e['id']==request_id),None)
 if record is None:raise ValueError('Missing durable context refresh schedule')
 launch=launch_event(store,p)
 if not launch or launch['id']!=record['predecessor_launch']:raise ValueError('Refresh launch changed')
 a,packet,ca,ar,cr=inspect(store,p,launch)
 if ar[1] or cr[1]:raise ValueError('Unresolved predecessor effects')
 terminal=cr[2].get('terminal',{})
 if a.e.sha(terminal)!=record['terminal_sha256']:raise ValueError('Terminal changed')
 if terminal.get('status')=='context_refresh_required':request=terminal['refresh'];source=terminal
 elif terminal.get('status') in SAFE_TERMINALS:
  request=json.loads((packet/'cycle/CONTEXT_REFRESH').read_text());source=record['settled_source']
 else:raise ValueError('Not a safe completed context boundary')
 if a.e.sha(request)!=record['refresh_sha256']:raise ValueError('Missing bound refresh terminal')
 if request.get('request_id')!=launch['request_id'] or request.get('cycle_admission_sha256')!=a.e.sha(ca) or a.e.sha(request)!=record['refresh_sha256']:raise ValueError('Refresh terminal provenance changed')
 selected=[e for e in events if e['type'] in ('input','assessment') and e['id'] in request['input_ids']]
 if [e['id'] for e in selected]!=request['input_ids'] or sha(selected)!=request['inputs_sha256']:raise ValueError('Queued context changed')
 if (packet/'cycle/STOP').exists():raise ValueError('Stop remains active')
 budget=store.budget()
 if budget.get('stopped') or not budget.get('available'):raise ValueError('Accounting blocks refresh')
 native=store.native(packet/'native');b=native.budget()
 if any(b.get(k) for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests')):raise ValueError('Unresolved native effects')
 q=native.cli('project','cycle','request')
 if any(q[k]!=source[k] for k in ('state_root','source_snapshot','scope')):raise ValueError('Refresh source changed')
 prior=next(e for e in events if e['type']=='run_request' and e['id']==launch['request_id'])
 return {'predecessor':launch['request_id'],'input_ids':list(dict.fromkeys(prior['input_ids']+request['input_ids']))}

@contextlib.contextmanager
def guard(store,p,request_id):
 record=next((e for e in store.events(p) if e['type']=='context_refresh_scheduled' and e['id']==request_id),None)
 if record is None:yield None;return
 launch=launch_event(store,p)
 if launch is None or launch['id']!=record['predecessor_launch']:raise ValueError('Refresh predecessor changed')
 pid=launch['pid']
 if type(pid) is not int or pid<=1:raise ValueError('Invalid predecessor process')
 try:os.kill(pid,0)
 except ProcessLookupError:pass
 else:raise ValueError('Predecessor still active; no duplicate launch')
 with contextlib.ExitStack() as stack:
  for name in ('adapter/launch.lock','cycle/run.lock'):
   f=stack.enter_context(os.fdopen(os.open(Path(launch['packet'])/name,os.O_WRONLY|os.O_NOFOLLOW),'a'));fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
  yield request_id

def tick(store,p):
 from server import sha,save
 with store.lock(p):
  if p in store.preparation_pending or p in store.preparing and store.preparing[p].is_alive():return
  launch=launch_event(store,p)
  if not launch:return
  if any(c['kind']=='stop' and c.get('request_id')==launch['request_id'] for c in store.controls(p)):return
  entries=queued(store,p,launch)
  if not entries:return
  a,packet,ca,ar,cr=inspect(store,p,launch)
  if (packet/'cycle/STOP').exists():return
  if not (packet/'cycle/cycle.jsonl').exists():return # Do not prepopulate a fresh journal.
  path=packet/'cycle/CONTEXT_REFRESH'
  if not path.exists():
   request={'version':'context-refresh@v1','cycle_admission_sha256':a.e.sha(ca),'request_id':launch['request_id'],'input_ids':[e['id'] for e in entries],'inputs_sha256':sha(entries)}
   save(path,request)
   store.append(p,{'id':'refresh-request-'+a.e.sha(request),'type':'context_refresh_requested','role':'system','state':'queued','text':'New context is queued. The current action and its assessment finish before a fresh decision.','request':request})
  terminal=cr[2].get('terminal',{})
  if ar[1] or cr[1] or terminal.get('status') not in SAFE_TERMINALS|{'context_refresh_required'}:return
  proc=store.jobs.get(p)
  if proc is not None:proc.poll() # Reap our own exited child before PID check.
  try:os.kill(launch['pid'],0)
  except ProcessLookupError:pass
  else:return
  request=json.loads(path.read_text());digest=a.e.sha(request);rid='refresh-'+digest
  if any(e['type']=='interaction_error' and e['id']==rid for e in store.events(p)):return
  if any(e['type']=='run_request' and e['id']==rid for e in store.events(p)):
   # An interrupted preparation is not permission to retry a possible launch.
   store.append(p,{'id':rid,'type':'interaction_error','role':'system','state':'blocked','text':'Context preparation was interrupted. Its durable run request is retained; reconcile the launch record before resuming. No duplicate dispatch was attempted.'})
   return
  if not any(e['type']=='context_refresh_scheduled' and e['id']==rid for e in store.events(p)):
   source=terminal if terminal.get('status')=='context_refresh_required' else store.native(packet/'native').cli('project','cycle','request')
   if source.get('branch') in ('resume','information_resume','service_resume','blocked','reconcile'):raise ValueError('Completed boundary has pending native work')
   store.append(p,{'id':rid,'type':'context_refresh_scheduled','role':'system','state':'scheduled','text':'Preparing a new source-bound decision with queued context; no completed effect is repeated.','predecessor_launch':launch['id'],'refresh_sha256':digest,'terminal_sha256':a.e.sha(terminal),'settled_source':{k:source[k] for k in ('state_root','source_snapshot','scope')}})
 # start owns durable idempotency and repeats Stop/budget/source checks. The
 # predecessor locks are held through preparation/launch, not the Stop lock.
 try:store.start(p,{'id':rid,'confirm':'Run under the active unlimited policy'})
 except Exception as exc:
  with store.lock(p):store.append(p,{'id':rid,'type':'interaction_error','role':'system','state':'blocked','text':'Queued-context handoff blocked: '+str(exc),'source':'local controller; not a model response'})

def monitor(store):
 while True:
  for d in (store.root/'projects').iterdir():
   if not (d/'project.json').exists():continue
   try:tick(store,d.name)
   except Exception as exc:
    # Visible once per unchanged error, never a blind dispatch retry.
    with store.lock(d.name):
     text='Queued-context handoff blocked: '+str(exc)
     if not any(e['type']=='context_refresh_error' and e['text']==text for e in store.events(d.name)):
      store.append(d.name,{'id':__import__('uuid').uuid4().hex,'type':'context_refresh_error','role':'system','state':'blocked','text':text})
  time.sleep(3)
