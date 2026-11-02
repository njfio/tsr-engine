"""Explicit human decisions for a loopback-only, single-user application.

Origin/session tokens prevent cross-site requests, not same-user malware. The
visible confirmation and ephemeral challenge are interaction/replay safeguards,
not a claim of biometric or account authentication. No browse operation approves.
"""
import contextlib,datetime,fcntl,json,secrets,time
from pathlib import Path
from server import canonical,sha,now,ROOT

REVIEW={'decision':'dispose_output_preserve_financial_unknown','journal_sha256':'e0e18da1f1de90117f60230cbb866fa06bd45ff85bc2f408f7523d6962c990ab','request_body_sha256':'422a765aa0ae894138e3b9d3c4c70f824cec833bef9c12cb1f7dd89738afdd28','reviewer':'independent code/evidence review /root/campaign_extension; no user approval or financial settlement'}
REQUEST='831ea6c1a0a40273010d5a9f7d74fc882cabf93333150915c0ec01275efca7d8'
def bound_hash(value):
 import hashlib
 return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=True,allow_nan=False).encode()).hexdigest()

KEY='c7eeaa2812424e1f9555bd2aae9f9167e5bff1c6f363518076be27d51d42e5f4'
class Decisions:
 def __init__(self,store):self.store=store;self.challenges={}
 @contextlib.contextmanager
 def lock(self):
  with (self.store.root/'human-decisions.lock').open('a') as f:
   fcntl.flock(f,fcntl.LOCK_EX);yield
 def rows(self):
  file=self.store.root/'human-decisions.jsonl';rows=[];prior=None
  if file.exists():
   for line in file.read_text().splitlines():
    row=json.loads(line)
    if row['previous']!=prior or row['hash']!=sha({k:v for k,v in row.items() if k!='hash'}):raise ValueError('Human decision history integrity failure')
    prior=row['hash'];rows.append(row)
  return rows
 def append(self,event):
  import os
  rows=self.rows();row={'previous':rows[-1]['hash'] if rows else None,'event':event};row['hash']=sha(row)
  with (self.store.root/'human-decisions.jsonl').open('ab') as f:f.write(canonical(row)+b'\n');f.flush();os.fsync(f.fileno())
 def disclosure(self,p):
  store=self.store;campaign,m,ledger,_=store.ledger();status=ledger.status()
  if status.get('unknown_executions') or status.get('blocking_unknown_calls',status['unknown_calls'])!=[KEY] or status['overrun']:raise ValueError('This decision cannot authorize unknown execution effects or other unresolved calls.')
  proof=ledger.quarantine_generation(ROOT/'target/pond-beautiful-live-v1/adapter',REQUEST,REVIEW['journal_sha256'],REVIEW,dry_run=True)

  with ledger.locked() as (_,rows):head=rows[-1]["hash"]
  events=store.events(p)
  return {'version':'local-human-generation-disposition@v1','unknown_key':KEY,'proof':proof,'review':REVIEW,'project':p,'source':store.source_snapshot(p),'events_sha256':sha(events),'input_ids':[e['id'] for e in events if e['type']=='input'],'resume_input_id':next((e['id'] for e in reversed(events) if e['type']=='input'),None),'ledger_head':head,'manifest_sha256':ledger.hash,'max_calls':status['effective_max_calls'],'max_execution_groups':status['effective_max_execution_groups'],'max_committed_microusd':status['effective_cap_microusd'],'baseline_committed_microusd':status['committed_microusd'],'remaining_calls':status['effective_max_calls']-status['calls'],'remaining_operations':status['effective_max_execution_groups']-status['execution_groups'],'remaining_microusd':status['effective_cap_microusd']-status['committed_microusd'],'minutes':120,'text':store.recovery_disclosure()['text'],'resume':'Continue the saved messages in this project; preserve original games and contributing history.'}
 def present(self,p,body):
  with self.lock():
   d=self.disclosure(p);token=secrets.token_urlsafe(32);self.challenges[token]={'project':p,'disclosure':d,'until':time.monotonic()+600}
   return {'challenge':token,'disclosure':d,'digest':bound_hash(d),'expires_in_seconds':600}
 def decide(self,p,body):
  choice=body.get('choice');token=body.get('challenge')
  if choice not in ('approve','decline') or not isinstance(token,str):raise ValueError('Explicit approval or decline required')
  with self.lock():
   prior=next((r['event'] for r in self.rows() if r['event'].get('challenge_sha256')==sha(token)),None)
   if prior:
    if prior['project']!=p or prior['choice']!=choice or prior['disclosure_sha256']!=body.get('digest'):raise ValueError('Decision replay conflicts with recorded choice')
    return {'state':'recorded','message':'This decision is already recorded. Nothing is dispatched twice.'}
   c=self.challenges.get(token)
   if not c or c['project']!=p or c['until']<time.monotonic():raise ValueError('Decision expired. Review the current request again.')
   d=self.disclosure(p)
   if bound_hash(d)!=body.get('digest') or d!=c['disclosure']:raise ValueError('Project, messages or accounting changed. Review the current decision again.')
   at=datetime.datetime.now(datetime.timezone.utc);decision={'version':'local-human-decision@v1','project':p,'choice':choice,'challenge_sha256':sha(token),'disclosure_sha256':bound_hash(d),'disclosure':d,'at':at.isoformat(),'expires_at':(at+datetime.timedelta(minutes=120)).isoformat(),'trust':'explicit loopback single-user application interaction; not remote identity authentication'}
   self.append(decision);del self.challenges[token]
   if choice=='decline':return {'state':'declined','message':'Declined. Your messages, desires and history remain saved; nothing will run.'}
   self.apply(decision)
  # Durable send identity makes a repeated handler/reconnect inert.
  events=self.store.events(p);message=next((e for e in events if e['type']=='input' and e['id']==decision['disclosure']['resume_input_id']),None)
  if message:return self.store.send(p,{'id':message['id'],'text':message['text'],'confirm':'Send to engine within $10'})
  return {'state':'approved','message':'Decision recorded. Send a message to work within the approved allowance.'}
 def resume_recorded(self):
  # A durable explicit decision, never page load or a saved draft, permits recovery.
  for row in self.rows():
   d=row['event']
   if d.get('choice')!='approve' or datetime.datetime.fromisoformat(d['expires_at']).timestamp()<=time.time():continue
   try:
    with self.lock():self.apply(d)
    message=next((e for e in self.store.events(d['project']) if e['type']=='input' and e['id']==d['disclosure']['resume_input_id']),None)
    if message:self.store.send(d['project'],{'id':message['id'],'text':message['text'],'confirm':'Send to engine within $10'})
   except Exception as exc:
    with self.store.lock(d['project']):
     if not any(e.get('id')=='decision-resume-'+sha(d) for e in self.store.events(d['project'])):self.store.append(d['project'],{'id':'decision-resume-'+sha(d),'type':'interaction_error','role':'system','text':'Decision retained; safe resume blocked: '+str(exc)})
 def apply(self,decision):
  d=decision['disclosure'];campaign,m,ledger,_=self.store.ledger()
  with ledger.locked() as (_,rows):
   prefix=next((i for i,r in enumerate(rows) if r['hash']==d['ledger_head']),None)
   if prefix is None:raise ValueError('Approved ledger prefix missing')
   for r in rows[prefix+1:]:
    event=r['event']
    if event.get('kind')=='generation_risk_accepted' and event.get('value',{}).get('decision_sha256')==bound_hash(decision):break
    if event.get('kind')!='generation_quarantined' or event.get('key')!=KEY:raise ValueError('Accounting changed after decision; automatic application refused')
  ledger.quarantine_generation(ROOT/'target/pond-beautiful-live-v1/adapter',REQUEST,REVIEW['journal_sha256'],REVIEW)
  ledger.accept_generation_risk(KEY,decision,d)
 def approval(self,p):
  return next((r['event'] for r in reversed(self.rows()) if r['event'].get('project')==p and r['event'].get('choice')=='approve'),None)
