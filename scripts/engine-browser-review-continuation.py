"""Exact failed-zero review continuation; existing browser controller owns all later choices.
No historical journal is reopened. A retained proposal is authenticated under its
original transport admission, never replaced with operator-written content.
"""
import argparse,copy,fcntl,hashlib,importlib.util,json,os,shutil,time
from pathlib import Path
S=Path(__file__).parent
s=importlib.util.spec_from_file_location('browser_continuation_adapter',S/'engine-browser-adapter.py');a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
e=a.e;d=a.d;c=a.load('continuation_campaign','engine-b91-campaign.py');o=a.load('continuation_zero','engine-objective-continuation.py');launch=a.load('continuation_launch','engine-native-cycle.py');recovery=a.load('continuation_tree','engine-browser-recovery.py')
def read(p):return json.loads(Path(p).read_text())
def save(p,v):e.private_save(Path(p),v)
def history(path):
 rows=[read_line for read_line in (e.strict(x) for x in Path(path).read_bytes().splitlines())];pending={};done={}
 for i,row in enumerate(rows):
  if row!=d.Driver.row(rows[:i],row['event']):raise ValueError('historical chain mismatch')
  v=row['event'];name=v.get('name')
  if v['kind']=='intent':
   if name in pending or name in done:raise ValueError('duplicate historical intent')
   pending[name]=v
  elif v['kind']=='result':
   if name not in pending or v['intent']!=e.sha(pending.pop(name)):raise ValueError('historical result mismatch')
   done[name]=v['value']
 return rows,pending,done

def inspect(parent,ledger):
 parent=Path(parent).resolve();old=read(parent/'adapter-admission.json');ca=read(parent/'cycle-admission.json');ar,ap,ad=history(parent/'adapter/adapter.jsonl');cr,cp,cd=history(parent/'cycle/cycle.jsonl')
 if ar[0]['event']['value']!=old or cr[0]['event']['value']!=ca:raise ValueError('historical admission substitution')
 if ap or len(cp)!=1:raise ValueError('exact settled adapter and one missing review required')
 stage=next(iter(cp))
 if not stage.startswith('review_') or not stage[7:].isdigit() or cr[-1]['event']!=cp[stage]:raise ValueError('only missing ordinary review supported')
 index=stage[7:];captured=cd['request_'+index];proposal=cd['proposal_'+index]
 calls=[(k,v) for k,v in ad.items() if k.startswith('dispatch_')];name,failed=calls[-1];ident=name[9:];lookup=read(parent/'provider-exact-lookup.json');intent=next(x['event'] for x in ar if x['event'].get('name')==name and x['event']['kind']=='intent');pid=o.failed_zero(failed['receipt'],lookup,earliest=d.stamp(intent['at']))
 if ar[-1]['event'].get('name')!='settle_'+ident or any(k.startswith('execution_') for k in ad):raise ValueError('unsupported effects after failed review')
 matches=[k[10:] for k,v in ad.items() if k.startswith('generated_') and v['response']==proposal['response']]
 if len(matches)!=1:raise ValueError('ambiguous retained proposal')
 generated_id=matches[0];source={k:captured[k] for k in ('state_root','source_snapshot','scope')}
 bound=a.base.bound_output.verify_envelope(ad['bound_output_'+generated_id],admission=old,ident=generated_id,dispatch=ad['dispatch_'+generated_id],settled=ad['settle_'+generated_id],source=source)
 if bound['association']['kind']!='proposal' or bound['decoded_sha256']!=e.sha([proposal['response'],ad['generated_'+generated_id]['return_basis']]):raise ValueError('retained decoded proposal/basis mismatch')
 trial=e.sha({'trial':str(parent/'adapter'),'admission':e.sha(old)});key=e.sha({'trial':trial,'request':ident,'kind':'call'})
 with ledger.locked() as (_,rows):
  status,trials,lc,le=ledger.fold(rows);proofs=[x['event'] for x in rows if x['event']['kind']=='bill_reconciled' and x['event']['key']==key]
  expected=read(parent/'billing-reconciled.json')['event']
  if proofs!=[expected] or expected['provider_request_id']!=pid or expected['lookup_sha256']!=recovery.sha(parent/'provider-exact-lookup.json') or expected['provider_journal_sha256']!=recovery.sha(parent/'adapter/adapter.jsonl') or expected['native_journal_sha256']!=recovery.sha(parent/'cycle/cycle.jsonl') or expected['native_result_sha256']!=e.sha(failed):raise ValueError('failed-zero ledger/source proof mismatch')
  if status['stopped'] or sum(v['trial']==trial for v in lc.values())!=len(calls) or any(v['trial']==trial for v in le.values()):raise ValueError('unknown or substituted prefix')
 if read(parent/'CLOSED.json')['remaining_batch_authority'] is not False:raise ValueError('parent must remain closed')
 parent_grant=read(parent/'scope-grant.json');corrections=a.base.correction.Budget(parent/'adapter/corrections.jsonl',old,old['correction_allowance']).events()
 files={n:recovery.sha(parent/n) for n in ('adapter-admission.json','cycle-admission.json','adapter/adapter.jsonl','cycle/cycle.jsonl','CLOSED.json','provider-exact-lookup.json','billing-review.json','billing-reconciled.json','scope-grant.json')}
 if corrections:files['adapter/corrections.jsonl']=recovery.sha(parent/'adapter/corrections.jsonl')
 return {'parent':str(parent),'files':files,'native_files':recovery.tree(parent/'native'),'source':source,'proposal':proposal,'generated_id':generated_id,'original_admission':old,'inherited':{k:ad[k] for k in ('generated_'+generated_id,'bound_output_'+generated_id,'dispatch_'+generated_id,'settle_'+generated_id)},'failed_request':ident,'provider_request_id':pid,'max_calls':old['max_calls'],'max_operations':old['max_executions'],'cap_microusd':parent_grant['session_cap_microusd'],'used_corrections':len(corrections),'used_continuations':sum(k.startswith('session_continue_') for k in cd),'prior_settlement':{'request_id':ident,'prompt_tokens':None,'output_tokens':None,'duration_ns':int(failed['receipt']['elapsed_seconds']*1e9),'external_cost_microusd':None,'outcome':'unknown'},'used_calls':len(calls),'used_operations':0,'used_microusd':status['known_cost_microusd']-read(parent/'scope-grant.json')['baseline']['settled_microusd'],'expires_at':old['expires_at'],'ledger_head':rows[-1]['hash']}

def verify_parent(p):
 parent=Path(p['parent'])
 if any(recovery.sha(parent/n)!=h for n,h in p['files'].items()):raise ValueError('parent evidence drift')
 # Regenerated facts must remain equal; inspect() additionally verifies ledger joins at preparation/check.
 _,pending,done=history(parent/'adapter/adapter.jsonl')
 if len(a.base.correction.Budget(parent/'adapter/corrections.jsonl',p['original_admission'],p['original_admission']['correction_allowance']).events())!=p['used_corrections']:raise ValueError('correction history drift')
 if pending or any(done[k]!=v for k,v in p['inherited'].items()):raise ValueError('inherited proposal substitution')

def mapping(prefix,request,response):
 return {'original_model_content_sha256':e.sha(prefix['inherited']['bound_output_'+prefix['generated_id']]['model_content']),'original_native_response_sha256':e.sha(prefix['proposal']['response']),'derivative_context_sha256':e.sha({'state_root':request['state_root'],'attention':request['next_attention'],'coding':request['coding']}),'implementation_sha256':recovery.sha(__file__),'materialized_response_sha256':e.sha(response)}

def materialize(prefix,request):
 if {f['path']:f['source'] for f in request['coding']['files'] if f['editable']}!=prefix['original_admission']['origin']['initial_sources']:raise ValueError('materialization source bytes changed')
 if request['coding']['relational']['inputs']['parents']!=prefix['original_admission']['origin']['public']['parents']:raise ValueError('materialization parents changed')
 original=prefix['inherited']['bound_output_'+prefix['generated_id']]['model_content']
 if not isinstance(original,dict) or set(original)!={'alternatives'} or len(original['alternatives'])!=1 or set(original['alternatives'][0])!={'edits','rationale'}:raise ValueError('only authenticated edit-only rematerialization supported')
 native=a.web.bind_edit(request,original['alternatives'][0],lambda _:None)
 prior=prefix['proposal']['response'];basis=prefix['inherited']['generated_'+prefix['generated_id']]['return_basis']
 if basis or native['state_root']!=prior['state_root'] or native['attention_binding']!=prior['attention_binding'] or a.semantic_candidate(native,{})!=a.semantic_candidate(prior,{}):raise ValueError('rematerialization changed meaning or attention')
 return native

class Adapter(a.Adapter):
 def __init__(self,*args,prefix,**kwargs):self.prefix=prefix;super().__init__(*args,**kwargs)
 def read(self):
  rows,pending,done=super().read();inherited={'generated_'+self.prefix['generated_id']:{**self.prefix['inherited']['generated_'+self.prefix['generated_id']],'response':self.prefix['materialized_response']}}
  return rows,pending,{**inherited,**done}
 def validate_current_admission(self):verify_parent(self.prefix);return super().validate_current_admission()
 def verify_owned_output(self,ident):
  if ident!=self.prefix['generated_id']:return super().verify_owned_output(ident)
  verify_parent(self.prefix);v=self.native('request',None);source={k:v[k] for k in ('state_root','source_snapshot','scope')};p=self.prefix;h=p['inherited']
  bound=a.base.bound_output.verify_envelope(h['bound_output_'+ident],admission=p['original_admission'],ident=ident,dispatch=h['dispatch_'+ident],settled=h['settle_'+ident],source=source)
  request=self.controller.request();derived=materialize(p,request)
  if mapping(p,request,derived)!=p['materialization']:raise ValueError('host materialization mapping substituted')
  if derived!=p['materialized_response']:raise ValueError('checkout-local transport materialization drift')
  return {**bound,'decoded_sha256':e.sha([derived,h['generated_'+ident]['return_basis']]),'historical_host_materialization':{'original_decoded_sha256':bound['decoded_sha256'],'original_request':ident,'native_response_sha256':e.sha(derived)}}
 def proposal(self,view):
  # Reuse exactly once, with no generation or new usage charge. Restart returns
  # the same durable result; after a subsequent choice ordinary generation resumes.
  if not any(k.startswith(('reviewed_','review_rejection_')) for k in self.read()[2]):
   if {k:view[k] for k in ('state_root','source_snapshot','scope')}!=self.prefix['source']:raise ValueError('stale retained proposal')
   self.verify_owned_output(self.prefix['generated_id'])
   self.op('historical_proposal_reuse',lambda:{'generated_id':self.prefix['generated_id'],'parent':self.prefix['parent'],'proposal_sha256':e.sha(self.prefix['proposal']),'materialized_response_sha256':e.sha(self.prefix['materialized_response']),'new_provider_call':False})
   return {'response':copy.deepcopy(self.prefix['materialized_response']),'receipt':{**copy.deepcopy(self.prefix['proposal']['receipt']),'mode':self.prefix['proposal']['callback_mode'],'status':'settled'}}
  return super().proposal(view)

def prepare(root,parent,ledger):
 p=inspect(parent,ledger);root=Path(root).resolve();root.mkdir(exist_ok=False);shutil.copytree(Path(parent)/'native',root/'native')
 if recovery.tree(root/'native')!=p['native_files']:raise ValueError('clone mismatch')
 native=d.Native(root/'native')
 remaining=p['max_calls']-p['used_calls'];operations=p['max_operations']-p['used_operations'];cost=p['cap_microusd']-p['used_microusd']
 if remaining<2 or cost<=0 or time.time()+480>=d.stamp(p['expires_at']):raise ValueError('remaining original envelope cannot fit')
 reconciliation={'state_root':p['source']['state_root'],'prior':p['prior_settlement'],'proof':p['files']['provider-exact-lookup.json'],'review':p['files']['billing-review.json']}
 save(root/'native-reconciliation.json',reconciliation);native.json_cli(('project-resource-reconcile-failed-zero',),reconciliation,'--proof')
 v=native.cli('project','cycle','request')
 if {k:v[k] for k in ('state_root','source_snapshot','scope')}!=p['source'] or v['branch']!='propose':raise ValueError('native identity/choice changed during separation')
 materialization_request=native.request();p['materialized_response']=materialize(p,materialization_request);p['materialization']=mapping(p,materialization_request,p['materialized_response'])
 origin=copy.deepcopy(p['original_admission']['origin']);origin['directory']=str(root/'native');q=native.request()['coding'];origin['public'].update(parents=q['relational']['inputs'].get('acceptance_parents',q['relational']['inputs']['parents']),public_preferences=[e.sha(x) for x in q['relational']['inputs'].get('generated_preferences',[])],public_meanings=[e.sha(x) for x in q['relational']['inputs'].get('public_desires',{}).get('history',[])],source_snapshots=[q['snapshot']],public_returns=[e.sha(x) for x in q['relational']['inputs']['returns']],public_relationships=[e.sha(x) for x in q['relational']['history']])
 corrections=min(p['original_admission']['correction_allowance']-p['used_corrections'],remaining-2);corrections-=((remaining-corrections)%2);decisions=(remaining-corrections)//2
 policy=copy.deepcopy(p['original_admission']['feedback_cycle']);policy['max_continuations']-=p['used_continuations']
 if policy['max_continuations']<1:raise ValueError('original continuation allowance exhausted')
 ad=a.admission(root/'adapter',origin,native.cli('project','config'),ledger.manifest,p['expires_at'],max_decisions=decisions,max_executions=operations,correction_allowance=corrections,feedback_cycle=policy)
 ca=read(Path(parent)/'cycle-admission.json');ca.update(journal=str(root/'cycle'),max_decisions=decisions,max_executions=operations,feedback_cycle=policy)
 save(root/'prefix.json',p);save(root/'adapter-admission.json',ad);save(root/'cycle-admission.json',ca)
 adapter=Adapter(ad['directory'],ad,native,ledger,prefix=p);authority={'purpose':'single retry of exact confirmed failed independent review then original cycle','original_parent':str(parent),'remaining_calls':ad['max_calls'],'remaining_operations':operations,'remaining_microusd':cost,'expires_at':p['expires_at'],'additional_provider_failure_retries':0,'implementation_sha256':recovery.sha(__file__)}
 grant=adapter.scope_grant(e.sha(authority),'independent-agent:/root/handoff_audit',cost,extend_authority=True);save(root/'authority.json',authority);save(root/'scope-grant.json',grant)
 save(root/'manifest.json',{'script_sha256':recovery.sha(__file__),'files':{n:recovery.sha(root/n) for n in ('prefix.json','adapter-admission.json','cycle-admission.json','authority.json','scope-grant.json')},'admission_sha256':e.sha(ad),'grant_sha256':c.digest(grant),'original_deadline':p['expires_at']})
 return {'admission':e.sha(ad),'grant':c.digest(grant),'remaining_calls':ad['max_calls'],'remaining_operations':operations,'remaining_microusd':cost,'provider_calls':0}

def main():
 parser=argparse.ArgumentParser();parser.add_argument('mode',choices=['prepare','check','run']);parser.add_argument('--parent',type=Path,required=True);parser.add_argument('--directory',type=Path,required=True);args=parser.parse_args();root=args.directory.resolve();parent=args.parent.resolve()
 man=read(e.ROOT/'target/tsr-integrated-campaign-1/manifest.json');up=read(parent/'campaign-upgrade.json');ledger=c.Ledger(man['ledger_path'],man,c.digest(man),upgrade=up,upgrade_hash=c.digest(up))
 if args.mode=='prepare':print(json.dumps(prepare(root,parent,ledger),indent=2));return
 manifest=read(root/'manifest.json')
 if manifest['script_sha256']!=recovery.sha(__file__) or any(recovery.sha(root/n)!=h for n,h in manifest['files'].items()):raise ValueError('continuation pin drift')
 p=read(root/'prefix.json');verify_parent(p)
 if recovery.tree(parent/'native')!=p['native_files']:raise ValueError('original native state changed')
 ad=read(root/'adapter-admission.json');ca=read(root/'cycle-admission.json');grant=read(root/'scope-grant.json');native=d.Native(root/'native');adapter=Adapter(ad['directory'],ad,native,ledger,prefix=p,before_call=lambda:launch.refresh_price(ledger,c));runner=a.loop.Cycle(ca['journal'],ca,adapter.native,adapter.proposal,adapter.review)
 lock=os.fdopen(os.open(root/'launch.lock',os.O_CREAT|os.O_WRONLY|os.O_NOFOLLOW,0o600),'a');fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
 if args.mode=='run':adapter.recover_session_transition()
 if adapter.read()[1] or ledger.status()['stopped']:raise d.Blocked('uncertain continuation effect')
 runner.recover_pending();done=runner.read()[2]
 if 'terminal' in done:print(json.dumps({'replay':True,'result':done['terminal'],'campaign':ledger.status()}));return
 status=launch.ensure_scope_grant(ledger,c,adapter,grant,manifest['grant_sha256'],check_only=True);launch.require_headroom(ad,adapter.read()[2],d.stamp);adapter.project(adapter.native('request',None))
 if args.mode=='check':
  adapter.verify_owned_output(p['generated_id']);adapter.native('preview',{'response':p['materialized_response']})
  print(json.dumps({'ready':True,'status':status,'provider_calls':0,'operations':0}));return
 launch.ensure_scope_grant(ledger,c,adapter,grant,manifest['grant_sha256']);adapter.op('campaign_envelope',lambda:{'original_scope':read(root/'authority.json'),'campaign_before':ledger.status()})
 print(json.dumps({'result':runner.run(),'campaign':ledger.status()},indent=2))
if __name__=='__main__':main()
