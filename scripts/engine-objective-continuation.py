#!/usr/bin/env python3
"""Source-bound objective recovery. Preparation is local; fresh reviewed authority required.

Historical journals are never edited or replayed. A new Cycle starts at the missing
round review. Prefix ownership is evidence, not additional operation allowance.
"""
import argparse
import copy
import datetime
import decimal
import importlib.util
import json
from pathlib import Path
import shutil
import signal
import threading
import tempfile
import time
import urllib.request
import uuid

spec=importlib.util.spec_from_file_location('objective_recovery_base',Path(__file__).with_name('engine-objective-adapter.py'))
a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)
e,d,loop=a.e,a.d,a.loop
c=a.load('objective_recovery_campaign','engine-b91-campaign.py')
b=a.load('objective_recovery_copy','engine-b91-continuation.py')
VERSION='objective-continuation@v1'

def read(path):return e.strict(e.bounded_file(Path(path),8*1024*1024))
def tree(path):
 path=Path(path)
 if not path.is_dir() or any(p.is_symlink() for p in (path,*path.parents)):raise ValueError('missing/unsafe tree')
 return b.tree(path)
def history(path):
 rows=[e.strict(x) for x in e.bounded_file(Path(path),8*1024*1024).splitlines()]
 if not rows:raise ValueError('empty journal')
 v=object.__new__(d.Driver);v.root=Path(path).parent;v.file=Path(path);v.admission=rows[0]['event']['value'];v.fixture=rows[0]['event']['fixture_only']
 return v.admission,v.read()
def pins():return {**a.pins(),Path(__file__).name:d.file_sha(Path(__file__)),'engine-b91-continuation.py':d.file_sha(Path(b.__file__))}

def failed_zero(receipt,lookup,*,earliest):
 """Positive exact provider record required; missing/null billing is not zero."""
 ids=receipt.get('response_request_ids',[])
 if receipt.get('http_status')!=503 or receipt.get('response_status')!='received_http_error' or len(ids)!=1:raise ValueError('not exact terminal HTTP503')
 pid=ids[0]
 if str(uuid.UUID(pid))!=pid:raise ValueError('provider identity')
 endpoint='https://api.cheaperinference.com/v1/usage/requests?request_ids='+pid
 if lookup.get('request_id')!=pid or lookup.get('endpoint')!=endpoint:raise ValueError('cross-request lookup')
 page=lookup['record']
 if page.get('has_more') is not False or len(page.get('data',[]))!=1:raise ValueError('exact singleton lookup required')
 v=page['data'][0]
 if (v.get('request_id')!=pid or v.get('model')!=c.MODEL or v.get('endpoint')!='/v1/chat/completions' or v.get('status')!='failed' or v.get('currency')!='USD' or v.get('billing',{}).get('status')!='failed'):raise ValueError('terminal provider record mismatch')
 charge=decimal.Decimal(str(v['billing'].get('billed_cost_usd')))
 if not charge.is_finite() or charge!=0 or (v.get('billed_cost_usd') is not None and decimal.Decimal(str(v['billed_cost_usd']))!=0):raise ValueError('positive failed-zero evidence required')
 if any(type(v.get(k)) is not int or v[k]!=0 for k in ('prompt_tokens','completion_tokens','total_tokens')):raise ValueError('failed usage not zero')
 if not earliest-5<=d.stamp(v['created_at'])<=d.stamp(lookup['retrieved_at']):raise ValueError('lookup chronology')
 return pid

def lookup_failed(pid):
 """One authenticated read, fixed endpoint, no redirects/proxies or polling."""
 if str(uuid.UUID(pid))!=pid:raise ValueError('request id')
 broker=a.base.transport_module.broker
 endpoint='https://api.cheaperinference.com/v1/usage/requests?request_ids='+pid
 if threading.current_thread() is not threading.main_thread() or signal.getitimer(signal.ITIMER_REAL)[0]:raise ValueError('exclusive lookup deadline required')
 previous=signal.getsignal(signal.SIGALRM)
 try:
  signal.signal(signal.SIGALRM,a.base.transport_module._alarm);signal.setitimer(signal.ITIMER_REAL,20)
  request=urllib.request.Request(endpoint,headers={'Authorization':'Bearer '+broker.credential()})
  opener=urllib.request.build_opener(urllib.request.ProxyHandler({}),broker.NoRedirect())
  with opener.open(request,timeout=20) as response:raw=response.read(1024*1024+1)
 finally:
  signal.setitimer(signal.ITIMER_REAL,0);signal.signal(signal.SIGALRM,previous)
 if len(raw)>1024*1024:raise ValueError('lookup response limit')
 return {'request_id':pid,'endpoint':endpoint,'retrieved_at':d.iso_now(),'record':e.strict(raw)}

def ledger_prefix(ledger):
 with ledger.locked() as (_,rows):return {'rows':len(rows),'head':rows[-1]['hash'],'status':ledger.fold(rows)[0]}
def check_ledger_prefix(ledger,prefix):
 with ledger.locked() as (_,rows):
  n=prefix['rows']
  if type(n) is not int or n<1 or len(rows)<n or rows[n-1]['hash']!=prefix['head'] or ledger.fold(rows[:n])[0]!=prefix['status']:raise ValueError('ledger prefix drift')
 return ledger.status()

def inspect_prefix(parent,ledger):
 """Read original state through a disposable copy (CLI may materialize caches)."""
 parent=Path(parent).resolve();files={name:d.file_sha(parent/name) for name in ('admission.json','cycle-admission.json','adapter/adapter.jsonl','cycle/cycle.jsonl','CLOSED.json','provider-exact-lookup.json','billing-review.json','billing-reconciled.json')}
 old=read(parent/'admission.json');ca=read(parent/'cycle-admission.json');closed=read(parent/'CLOSED.json')
 aa,(ar,ap,ad)=history(parent/'adapter/adapter.jsonl');cc,(_,cp,cd)=history(parent/'cycle/cycle.jsonl')
 if aa!=old or cc!=ca or old['version']!='synthetic-objective-adapter@v2' or ca['version']!='local-project-cycle@v5' or closed.get('remaining_batch_authority') is not False:raise ValueError('closed objective prefix required')
 if old['directory']!=str(parent/'adapter') or ca['journal']!=str(parent/'cycle') or old['origin']['directory']!=str(parent/'state'):raise ValueError('cross-root prefix')
 if ap or len(cp)!=1 or 'terminal' in cd:raise ValueError('unknown or terminal prefix')
 calls=[(k,v) for k,v in ad.items() if k.startswith('dispatch_')]
 if len(calls)!=closed['actual_calls'] or not calls:raise ValueError('prefix call inventory')
 failed,result=calls[-1];ident=failed.removeprefix('dispatch_');lookup=read(parent/'provider-exact-lookup.json')
 intent=next(r['event'] for r in ar if r['event'].get('kind')=='intent' and r['event'].get('name')==failed)
 pid=failed_zero(result['receipt'],lookup,earliest=d.stamp(intent['at']))
 if result['candidate_sha256']!=e.sha(None) or any(k=='generated_'+ident or k=='reviewed_'+ident for k in ad):raise ValueError('failed response was applied')
 if any(v['receipt'].get('usage_status')!='reported' for _,v in calls[:-1]):raise ValueError('other unknown provider effects')
 review=read(parent/'billing-review.json');reconciled=read(parent/'billing-reconciled.json')['event']
 if review.get('decision')!='exact_failed_zero' or not review.get('reviewer') or not review.get('artifacts'):raise ValueError('independent billing review required')
 for name,digest in review['artifacts'].items():
  path=e.ROOT/name
  if not path.resolve().is_relative_to(e.ROOT) or path.is_symlink() or d.file_sha(path)!=digest:raise ValueError('reviewed billing artifact drift')
 trial=e.sha({'trial':str(parent/'adapter'),'admission':e.sha(old)});key=c.digest({'trial':trial,'request':ident,'kind':'call'})
 with ledger.locked() as (_,rows):
  status,trials,lc,le=ledger.fold(rows)
  bills=[r for r in rows if r['event'].get('kind')=='bill' and r['event'].get('key')==key]
  matches=[r['event'] for r in rows if r['event'].get('kind')=='bill_reconciled' and r['event'].get('key')==key]
  if (trials.get(trial,{}).get('admission')!=e.sha(old) or len(bills)!=1 or bills[0]['event']['microusd'] is not None or matches!=[reconciled] or reconciled['original_bill_row']!=bills[0]['hash'] or reconciled['provider_request_id']!=pid or reconciled['microusd']!=0 or reconciled['lookup_sha256']!=files['provider-exact-lookup.json'] or reconciled['review_sha256']!=e.sha(review) or reconciled['native_journal_sha256']!=files['cycle/cycle.jsonl'] or reconciled['provider_journal_sha256']!=files['adapter/adapter.jsonl'] or reconciled['native_result_sha256']!=e.sha(result)):raise ValueError('exact reconciled ledger join required')
  if sum(v['trial']==trial for v in lc.values())!=len(calls) or sum(v['trial']==trial for v in le.values())!=closed['actual_execution_groups'] or status['stopped']:raise ValueError('unreconciled campaign prefix')
  returned={row['event']['key']:row['event']['outcome'] for row in rows if row['event']['kind']=='return'}
  operations=[k for k in ad if k.startswith(('service_execution_','execution_'))]
  if len(operations)!=closed['actual_execution_groups']:raise ValueError('execution inventory mismatch')
  for name in operations:
   request=name.removeprefix('service_execution_') if name.startswith('service_execution_') else name.removeprefix('execution_')
   operation_key=c.digest({'trial':trial,'request':request,'kind':'execution'})
   if le.get(operation_key,{}).get('binding')!=e.sha(old) or le.get(operation_key,{}).get('trial')!=trial or returned.get(operation_key)!='returned':raise ValueError('execution receipt ledger identity/outcome mismatch')
  for name,value in calls:
   request=name.removeprefix('dispatch_');call_key=c.digest({'trial':trial,'request':request,'kind':'call'})
   if lc.get(call_key,{}).get('trial')!=trial or 'reserve_'+request not in ad or 'settle_'+request not in ad:raise ValueError('call reservation/settlement identity mismatch')
 inventory=tree(parent/'state')
 with tempfile.TemporaryDirectory(prefix='objective-prefix-') as tmp:
  state=Path(tmp)/'state';shutil.copytree(parent/'state',state);native=d.Native(state)
  request=native.request();obj=native.cli('project','objective-status');budget=native.budget();config=native.cli('project','config')
 if tree(parent/'state')!=inventory:raise ValueError('original state changed during read')
 context=obj['context'];i=context['round']
 if context['next']!='round_review' or i<0 or list(cp)!=['objective_round_review_'+str(i)+'_review']:raise ValueError('only missing round review supported')
 if (budget['unresolved_tool_requests'] or budget['unsettled_requests'] or budget['unknown_external_cost_requests']!=1 or budget['unknown_compute_requests'] or budget['unknown_usage']!=1 or budget['model_calls']!=len(calls) or request.get('pending')):raise ValueError('unknown action or additional native effects')
 if (budget['measured_prompt_tokens']!=sum(v['receipt']['usage']['prompt_tokens'] for _,v in calls[:-1]) or budget['measured_output_tokens']!=sum(v['receipt']['usage']['completion_tokens'] for _,v in calls[:-1])):raise ValueError('native usage does not match recorded requests')
 receipts=[v.get('result',v)['next']['objective']['context']['receipts'][-1] for k,v in ad.items() if k.startswith('service_execution_')]
 reviews=[v['record'] for k,v in ad.items() if k.startswith('objective_admitted_')]
 if context['receipts']!=receipts or len(receipts)!=i+1 or context['round_reviews']!=reviews or len(reviews)!=i or context['final_review'] is not None or any(not r['complete'] for r in receipts):raise ValueError('completed ordered prefix required')
 origin=old['origin'];coding=request['coding'];inputs=coding['relational']['inputs']
 if (context['admission']!=origin['objective_admission'] or context['schedule']!=origin['schedule'] or inputs['parents']!=origin['parents'] or inputs['requirements_revision']!=origin['requirements_revision'] or e.sha(config)!=origin['config_sha256']):raise ValueError('prefix source/criteria/schedule mismatch')
 # Check exact response/review ownership and source projection through the old journal,
 # without requiring current code pins to equal the frozen historical implementation.
 view=object.__new__(a.Adapter);view.root=parent/'adapter';view.file=view.root/'adapter.jsonl';view.admission=old;view.fixture=ledger.manifest['fixture_only']
 generated=a.base.Adapter.generated(view)
 allowed={origin['initial_source']}|{p['new_text'] for r in generated for q in r.get('proposals',[]) if q['kind']=='edit_set' for p in q['patches']}
 if [f['source'] for f in coding['files'] if f.get('editable')][0] not in allowed or {f['path']:f['source_digest'] for f in coding['files'] if not f.get('editable')}!=origin['protected']:raise ValueError('prefix source inventory/lineage')
 a.receipt_checker(origin)
 return {'parent':str(parent),'files':files,'state_files':inventory,'state_root':request['state_root'],'snapshot':coding['snapshot'],'requirements_revision':inputs['requirements_revision'],'objective':obj,'failed_request':ident,'provider_request_id':pid,'ledger':ledger_prefix(ledger),'inherited':{'generated':generated,'reviews':reviews,'receipts':receipts},'origin':origin,'fixture_only':ledger.manifest['fixture_only']}

def verify_parent(prefix,ledger):
 parent=Path(prefix['parent'])
 if any(d.file_sha(parent/name)!=digest for name,digest in prefix['files'].items()) or tree(parent/'state')!=prefix['state_files']:raise ValueError('historical prefix drift')
 check_ledger_prefix(ledger,prefix['ledger'])
 old,(rows,pending,done)=history(parent/'adapter/adapter.jsonl')
 inherited={'generated':[v['response'] for k,v in done.items() if k.startswith('generated_')],
            'reviews':[v['record'] for k,v in done.items() if k.startswith('objective_admitted_')],
            'receipts':[v.get('result',v)['next']['objective']['context']['receipts'][-1] for k,v in done.items() if k.startswith('service_execution_')]}
 if pending or prefix['origin']!=old['origin'] or prefix['inherited']!=inherited:raise ValueError('substituted inherited evidence')
 _,(_,cp,cd)=history(parent/'cycle/cycle.jsonl')
 saved=cd.get('objective_round_review_'+str(prefix['objective']['context']['round'])+'_context',{}).get('objective')
 if saved!=prefix['objective']:raise ValueError('substituted objective review context')
 if prefix['snapshot']!=saved['context']['source_snapshot'] or prefix['requirements_revision']!=saved['context']['requirements_revision']:raise ValueError('substituted source identity')

def admission(root,prefix,campaign,expires_at,*,retry_allowance=0):
 if type(retry_allowance) is not int or retry_allowance not in (0,1):raise ValueError('explicit retry allowance 0 or 1')
 root=Path(root).resolve();origin=copy.deepcopy(prefix['origin']);origin['directory']=str(root/'state')
 return {'version':VERSION,'directory':str(root/'adapter'),'origin':origin,'prefix':prefix,'campaign_sha256':e.sha(campaign),'model':c.MODEL,'max_calls':17+retry_allowance,'max_decisions':6,'max_executions':3,'max_service_rounds':1,'correction_allowance':2,'terminal_retry_allowance':retry_allowance,'retry_backoff_seconds':60,'expires_at':expires_at,'implementation':pins(),'binary_sha256':d.file_sha(e.ROOT/'target/debug/tsr'),'output_profile':'strict-json-bound@v2'}
def validate(ad,ledger):
 root=Path(ad['directory']).parent;p=ad['prefix'];parent=Path(p['parent'])
 if ad!=admission(root,p,ledger.manifest,ad['expires_at'],retry_allowance=ad['terminal_retry_allowance']):raise ValueError('continuation admission drift')
 if time.time()>=d.stamp(ad['expires_at']) or root==parent or root.is_relative_to(parent) or parent.is_relative_to(root) or any(q.is_symlink() for q in (root,*root.parents)):raise ValueError('expired/unsafe derivative')
 if not ledger.manifest['fixture_only'] and (not root.is_relative_to(e.ROOT/'target') or root==e.ROOT/'target'):raise ValueError('live derivative must be inside repository target')
 verify_parent(p,ledger)

def prepare(root,prefix,ledger,expires_at,*,retry_allowance=0):
 """Local separate derivative only; no campaign enrollment or launch grant."""
 root=Path(root).resolve();ad=admission(root,prefix,ledger.manifest,expires_at,retry_allowance=retry_allowance);validate(ad,ledger)
 root.mkdir(parents=True,exist_ok=False);parent=Path(prefix['parent'])
 shutil.copytree(parent/'state',root/'state')
 if tree(root/'state')!=prefix['state_files']:raise ValueError('clone mismatch')
 (root/'state/shadow').rename(root/'parent-shadow')
 archive=root/'parent-evidence';archive.mkdir()
 for name in prefix['files']:
  dst=archive/name;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(parent/name,dst)
 native=d.Native(root/'state')
 if native.request()['state_root']!=prefix['state_root']:raise ValueError('clone state identity changed')
 native.json_cli(('project-lifecycle-open',),{'state_root':native.request()['state_root'],'training':None,'tool_ms_ceiling':300000,'reason':'Separately admitted remaining objective continuation; completed actions remain historical','non_tool':{'model_calls':ad['max_calls'],'model_tokens':ad['max_calls']*40960,'wall_ms':max(1,int((d.stamp(expires_at)-time.time())*1000)),'patch_bytes':4096,'external_cost_microusd':0}},'--admission')
 native.cli('project','resource-unpriced-admit','--state-root',native.request()['state_root'])
 if native.cli('project','objective-status')!=prefix['objective']:raise ValueError('objective changed during resource separation')
 ca={'version':'local-project-cycle@v5','journal':str(root/'cycle'),'scope':ad['origin']['scope'],'max_decisions':6,'max_executions':3,'max_service_rounds':1,'expires_at':expires_at,'implementation_sha256':d.file_sha(Path(loop.__file__))}
 e.private_save(root/'admission.json',ad);e.private_save(root/'cycle-admission.json',ca)
 e.private_save(root/'PREPARED_ONLY.json',{'external_launch_authorized':False,'provider_calls':0,'operations':0,'admission_sha256':e.sha(ad),'prefix_sha256':e.sha(prefix)})
 return ad,ca

class Adapter(a.Adapter):
 release_profiles=a.Adapter.release_profiles+(VERSION,)
 def __init__(self,*args,failure_lookup=None,**kwargs):
  self.failure_lookup=failure_lookup or lookup_failed
  super().__init__(*args,**kwargs)
 def validate_current_admission(self):
  validate(self.admission,self.ledger);root=self.root.parent;p=self.admission['prefix']
  if str(self.controller.directory)!=self.admission['origin']['directory']:raise ValueError('derivative root mismatch')
  if any(d.file_sha(root/'parent-evidence'/n)!=h for n,h in p['files'].items()) or tree(root/'parent-shadow')!={k.removeprefix('shadow/'):v for k,v in p['state_files'].items() if k.startswith('shadow/')}:raise ValueError('historical archive drift')
 def inherited(self):return copy.deepcopy(self.admission['prefix']['inherited'])
 def _wire(self,body,timeout):
  start=time.time();result=super()._wire(body,timeout);receipt=result.get('receipt',{})
  if receipt.get('http_status')!=503 or result.get('value') is not None:return result
  if not self.admission['terminal_retry_allowance'] or any(k.startswith('terminal_retry_') for k in self.read()[2]):return result
  # No call is repeated while outcome or charge is unknown. One exact lookup may
  # establish terminal failure before settlement; no polling or response repair.
  try:
   ids=receipt['response_request_ids']
   used={self.admission['prefix']['provider_request_id']}|{pid for k,v in self.read()[2].items() if k.startswith('dispatch_') for pid in v.get('receipt',{}).get('response_request_ids',[])}
   if len(ids)!=1 or ids[0] in used or time.time()-start>timeout-25:return result
   lookup=self.failure_lookup(ids[0]);failed_zero(receipt,lookup,earliest=start)
   if time.time()-start>timeout:return result
  except (ValueError,KeyError,TypeError,OSError,decimal.InvalidOperation):return result
  index=sum(k.startswith('dispatch_') for k in self.read()[2]);ident=e.sha({'admission':e.sha(self.admission),'kind':self.wire_kind,'index':index,'body':e.sha(body)})
  self.op('failed_zero_'+ident,lambda:{'original_receipt':receipt,'lookup':lookup,'body_sha256':e.sha(body),'basis':'authenticated exact terminal failed-zero lookup; original retained'})
  return {**result,'receipt':{**receipt,'status':'settled','usd':'0.000000','usage':{'prompt_tokens':0,'completion_tokens':0,'total_tokens':0},'usage_status':'reported','elapsed_seconds':time.time()-start}}
 def call(self,kind,body):
  self.wire_kind=kind
  return super().call(kind,body)
 def retry_terminal_failure(self,ident,receipt):
  done=self.read()[2]
  if 'failed_zero_'+ident not in done:return False
  if self.admission['terminal_retry_allowance']!=1 or any(k.startswith('terminal_retry_') for k in done):raise d.Blocked('terminal retry allowance exhausted')
  if sum(k.startswith('dispatch_') for k in done)>=self.admission['max_calls'] or time.time()+300>d.stamp(self.admission['expires_at']):raise d.Blocked('retry cannot fit remaining admission')
  self.op('terminal_retry_'+ident,lambda:{'failed_request':ident,'proof_sha256':e.sha(done['failed_zero_'+ident]),'not_before':time.time()+self.admission['retry_backoff_seconds'],'new_counted_request':True})
  wait=max(0,self.read()[2]['terminal_retry_'+ident]['not_before']-time.time())
  if wait:time.sleep(wait)
  return True

def cycle(ad):
 return {'version':'local-project-cycle@v5','journal':str(Path(ad['directory']).parent/'cycle'),'scope':ad['origin']['scope'],'max_decisions':ad['max_decisions'],'max_executions':ad['max_executions'],'max_service_rounds':ad['max_service_rounds'],'expires_at':ad['expires_at'],'implementation_sha256':d.file_sha(Path(loop.__file__))}

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('mode',choices=('prepare','check','run'));parser.add_argument('--parent',type=Path);parser.add_argument('--directory',type=Path,required=True);parser.add_argument('--campaign-manifest',type=Path,required=True);parser.add_argument('--campaign-upgrade',type=Path,required=True);parser.add_argument('--admission-sha256');parser.add_argument('--terminal-retry-allowance',type=int,default=0);parser.add_argument('--allow-approved-effects',action='store_true');args=parser.parse_args()
 manifest=read(args.campaign_manifest);upgrade=read(args.campaign_upgrade);ledger=c.Ledger(manifest['ledger_path'],manifest,c.digest(manifest),upgrade=upgrade,upgrade_hash=c.digest(upgrade))
 if args.mode=='prepare':
  if not args.parent:parser.error('parent required')
  prefix=inspect_prefix(args.parent,ledger);expires=(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(hours=2)).isoformat()
  ad,_=prepare(args.directory,prefix,ledger,expires,retry_allowance=args.terminal_retry_allowance)
  print(json.dumps({'prepared_only':True,'admission_sha256':e.sha(ad),'calls':ad['max_calls'],'groups':3,'launch_authorized':False}));return
 ad=read(args.directory/'admission.json')
 if args.admission_sha256!=e.sha(ad):raise ValueError('explicit admission identity required')
 if args.mode=='run' and (not args.allow_approved_effects or manifest['fixture_only']):raise ValueError('fresh approved live effects required; fixtures cannot launch')
 validate(ad,ledger)
 ca=read(args.directory/'cycle-admission.json')
 if ca!=cycle(ad):raise ValueError('cycle admission mismatch')
 launch=a.load('objective_continuation_launcher','engine-native-cycle.py')
 adapter=Adapter(ad['directory'],ad,d.Native(Path(ad['origin']['directory'])),ledger,before_call=lambda:launch.refresh_price(ledger,c))
 runner=loop.Cycle(ca['journal'],ca,adapter.native,adapter.proposal,adapter.review)
 if adapter.read()[1] or runner.read()[1]:raise d.Blocked('uncertain derivative effect; no replay')
 done=runner.read()[2]
 if 'terminal' in done:
  print(json.dumps({'terminal':done['terminal'],'replayed_without_dispatch':True}));return
 status=ledger.status()
 if status['stopped']:raise d.Blocked('campaign stopped')
 if 'campaign_envelope' not in adapter.read()[2] and (status['remaining_calls']<ad['max_calls'] or status['remaining_execution_groups']<ad['max_executions'] or status['committed_microusd']+ad['max_calls']*c.RESERVE>c.CAP):raise d.Blocked('whole envelope does not fit')
 launch.require_headroom(ad,adapter.read()[2],d.stamp)
 current=adapter.native('request',None);adapter.project(current)
 if args.mode=='check':print(json.dumps({'ready_for_scope_review':True,'branch':current['branch'],'status':status,'provider_calls':0,'operations':0}));return
 adapter.op('campaign_envelope',lambda:{'admission_sha256':e.sha(ad),'calls':ad['max_calls'],'execution_groups':ad['max_executions'],'reservation_microusd':ad['max_calls']*c.RESERVE,'campaign_before':status})
 ledger.record_price(c.official_preflight())
 print(json.dumps({'result':runner.run(),'campaign':ledger.status()}))

if __name__=='__main__':main()
