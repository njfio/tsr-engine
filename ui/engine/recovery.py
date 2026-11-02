"""Read-only verification of an operator-recorded, pre-dispatch projection failure.
No HTTP endpoint creates this disposition. Old pending journals remain untouched.
"""
import contextlib,fcntl,json,os,sqlite3,types
from pathlib import Path

@contextlib.contextmanager
def guard(store,p):
 packet=store.meta(p).get('registered_packet')
 dispositions=[e for e in store.events(p) if e['type']=='operator_projection_recovery' and e.get('proof',{}).get('packet')==packet]
 if not dispositions:
  yield None;return
 with contextlib.ExitStack() as stack:
  for name in ('adapter/launch.lock','cycle/run.lock'):
   f=stack.enter_context(os.fdopen(os.open(Path(packet)/name,os.O_WRONLY|os.O_NOFOLLOW),'a'))
   fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
  yield packet

def native_head(packet):
 # Read the indexed ID and its payload together; the native request below still
 # performs authoritative chain/CAS verification. No timestamp-based inference.
 with contextlib.closing(sqlite3.connect((Path(packet)/'native/events.sqlite').resolve().as_uri()+'?mode=ro',uri=True)) as db:
  row=db.execute('SELECT seq,id,payload FROM events ORDER BY seq DESC LIMIT 1').fetchone()
 if row is None or json.loads(row[2])['id']!=row[1]:raise ValueError('Native head identity mismatch')
 return {'sequence':row[0],'event':row[1]}

def advance_capture(packet,proof,ad,cd,intent):
 name=intent['name'];index=name.removeprefix('advance_')
 if not name.startswith('advance_') or not index.isdigit():raise ValueError('Not an ordinary interrupted advance')
 captured=cd.get('callback_input_review_'+index);proposal=cd.get('proposal_'+index);review=cd.get('review_'+index)
 if not captured or captured['kind']!='review' or not proposal or captured['response']!=proposal['response'] or not review or review['judgment']['decision']!='approve':raise ValueError('Missing completed proposal/review prefix')
 continuations=[v['result']['continuation'] for k,v in ad.items() if k.startswith('session_continue_') and v.get('result',{}).get('continuation')]
 if not continuations or proof.get('native_head',{}).get('event')!=continuations[-1]:raise ValueError('Missing authenticated pre-advance head')
 if native_head(packet)!=proof['native_head']:raise ValueError('Native advance may have committed or history changed')
 return captured

def verify(store,p,packet,proof,lease=None):
 from server import load
 a=load('ui_projection_recovery','engine-browser-adapter.py');packet=Path(packet)
 if store.meta(p).get('registered_packet')!=str(packet):raise ValueError('Recovery packet is not current')
 if lease!=str(packet):
  # Direct diagnostic callers hold the same locks for the whole verification.
  with contextlib.ExitStack() as stack:
   for name in ('adapter/launch.lock','cycle/run.lock'):
    f=stack.enter_context(os.fdopen(os.open(packet/name,os.O_WRONLY|os.O_NOFOLLOW),'a'));fcntl.flock(f,fcntl.LOCK_EX|fcntl.LOCK_NB)
   return verify(store,p,packet,proof,str(packet))
 advance=proof.get('version')=='native-advance-not-committed@v1'
 if proof.get('version') not in ('local-projection-failure@v1','native-advance-not-committed@v1') or proof.get('packet')!=str(packet):raise ValueError('Recovery disposition identity')
 launches=[e for e in store.events(p) if e['type']=='run_started' and e.get('packet')==str(packet) and e.get('pid')]
 if not launches or launches[-1]['pid']!=proof.get('pid'):raise ValueError('Recovery worker provenance')
 if (packet/'cycle/STOP').exists() or any(c['kind']=='stop' and c.get('request_id')==launches[-1].get('request_id') for c in store.controls(p)):raise ValueError('Stop remains active; no recovery launch')
 if type(proof.get('pid')) is not int or proof['pid']<=1:raise ValueError('Recovery worker identity')
 try:os.kill(proof['pid'],0)
 except ProcessLookupError:pass
 else:raise ValueError('Recovery worker still exists')
 for name,digest in proof['files'].items():
  path=Path(name)
  if path.is_absolute() or '..' in path.parts or a.d.file_sha(packet/path)!=digest:raise ValueError('Recovery prefix changed')
 required={'adapter/adapter.jsonl','cycle/cycle.jsonl','adapter-admission.json','cycle-admission.json','prepared-manifest.json','scope-grant.json'}
 if set(proof['files'])!=required:raise ValueError('Recovery incomplete prefix')
 def journal(folder):
  ad=json.loads((packet/(folder+'-admission.json')).read_text())
  return a.d.Driver.read(types.SimpleNamespace(file=packet/folder/(folder+'.jsonl'),admission=ad,fixture=False,row=a.d.Driver.row,journal_limit=a.base.capacity.JOURNAL_BYTES))
 admission=json.loads((packet/'adapter-admission.json').read_text());cycle=json.loads((packet/'cycle-admission.json').read_text());manifest=json.loads((packet/'prepared-manifest.json').read_text());grant=json.loads((packet/'scope-grant.json').read_text())
 if admission['directory']!=str(packet/'adapter') or admission['origin']['directory']!=str(packet/'native') or cycle['journal']!=str(packet/'cycle'):raise ValueError('Recovery path binding')
 if manifest['adapter_sha256']!=a.e.sha(admission) or manifest['grant_sha256']!=a.e.sha(grant):raise ValueError('Recovery manifest binding')
 ar,ap,ad=journal('adapter');cr,cp,cd=journal('cycle')
 if ap or len(cp)!=1 or cr[-1]['event']!=next(iter(cp.values())):raise ValueError('Recovery unresolved stages')
 intent=next(iter(cp.values()));name=intent['name'];captured=cd.get('callback_input_'+name)
 if advance:captured=advance_capture(packet,proof,ad,cd,intent)
 elif not name.startswith('proposal_') or not captured or captured['kind']!='proposal' or captured['response'] is not None:raise ValueError('Recovery is not a captured proposal')
 # Every adapter side effect begins with a durable intent. None may follow this
 # captured outer stage; a provider failure is not classified by its HTTP code.
 if any(e['event'].get('at','')>=intent['at'] for e in ar):raise ValueError('Adapter activity after failed stage; reconcile instead')
 for key in ad:
  if key.startswith('dispatch_') and 'settle_'+key[len('dispatch_'):] not in ad:raise ValueError('Unsettled provider dispatch')
 _,_,ledger,_=store.ledger();status=ledger.status()
 if status['blocking_unknown_calls'] or status['blocking_unknown_executions']:raise ValueError('Unknown external effect')
 native=store.native(packet/'native');budget=native.budget()
 if any(budget[k] for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests')):raise ValueError('Unknown native effect')
 q=native.request();old=captured['context']
 current=native.cli('project','cycle','request')
 if current['resource_root']!=(proof.get('resource_root') if advance else old['resource_root']):raise ValueError('Resource history changed since captured callback')
 if q['state_root']!=old['state_root'] or q['coding']['snapshot']!=old['source_snapshot'] or q['next_attention']['scope']!=old['scope']:raise ValueError('Recovery source changed')
 origin=json.loads((packet/'adapter-admission.json').read_text())['origin']['public'];inputs=q['coding']['relational']['inputs']
 if q['next_attention']['scope']!=origin['scope'] or inputs['parents']!=origin['parents'] or inputs['requirements_revision']!=origin['requirements_revision']:raise ValueError('Contributors not restored under current policy')
 if proof.get('source_root')!=q['state_root'] or proof.get('snapshot')!=q['coding']['snapshot']:raise ValueError('Recovery source proof mismatch')
 if advance and native_head(packet)!=proof['native_head']:raise ValueError('Native history changed during verification')
 return {'state':'verified_native_advance_not_committed' if advance else 'verified_local_pre_dispatch_failure','old_stage':name,'old_journal_remains_pending':True,'redispatched':False}
