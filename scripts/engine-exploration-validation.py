"""Bounded admitted retrieval trial. No effects without a fresh exact local grant file.
Call/effect intents are durable; any interruption stops for reconciliation, never retries.
"""
import fcntl,hashlib,importlib.util,json,time,argparse,subprocess
from pathlib import Path
def load(n,f):
 s=importlib.util.spec_from_file_location(n,Path(__file__).with_name(f));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
x=load('rx','engine-exploration-export.py');c=load('camp','engine-b91-campaign.py');t=load('transport','engine-synthetic-transport.py')
correction=load('correction','engine-bounded-correction.py')
ROOT=x.ROOT

def ledger(upgrade_path=None):
 m=c.strict((ROOT/'target/tsr-integrated-campaign-1/manifest.json').read_bytes());u=c.strict((Path(upgrade_path) if upgrade_path is not None else ROOT/'target/qualitative-exploration-live-2/campaign-upgrade.json').read_bytes());return c.Ledger(m['ledger_path'],m,c.digest(m),upgrade=u,upgrade_hash=c.digest(u))
def run(d,grant_path):
 d=Path(d).resolve();grant=c.strict(Path(grant_path).read_bytes());plan=x.read(d/'inquiry-manifest.json')
 with (d/'run.lock').open('a') as lock:
  fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
  if (d/'terminal.json').exists():return {'terminal':x.read(d/'terminal.json'),'dispatch':False}
  if list(d.glob('intent-*.json')) or (d/'admission.json').exists():raise ValueError('Existing run; reconcile only, no redispatch')
  allowance=grant.get('correction_allowance',0)
  if type(allowance) is not int or not 0<=allowance<=2:raise ValueError('invalid correction allowance')
  maximum=6+allowance
  contract_grant={k:v for k,v in grant.items() if k!='correction_allowance'}
  if set(contract_grant)!={'plan','max_calls','max_operations','reservation_microusd','deadline','scope','runner_sha256'} or grant['plan']!=x.sha(plan) or grant['max_calls']!=maximum or grant['max_operations']!=1 or grant['reservation_microusd']!=maximum*50000 or grant['scope']!=plan.get('scope') or grant['scope'] not in ('frozen-fiction-native-exploration@v1',x.continuation.SCOPE) or grant['runner_sha256']!=hashlib.sha256(Path(__file__).read_bytes()).hexdigest() or not time.time()+maximum*240+160<grant['deadline']<time.time()+7200:raise ValueError('fresh exact grant required')
  for p,h in plan['pins'].items():
   if hashlib.sha256((ROOT/p).read_bytes()).hexdigest()!=h:raise ValueError('implementation drift')
  o=x.read(d/'exploration-origin-local-only.json')
  if x.sha(o)!=plan['origin']:raise ValueError('origin drift')
  if (grant['scope']==x.continuation.SCOPE)!=('continuation' in o):raise ValueError('profile binding')
  if 'continuation' in o:x.continuation.validate_source(x,o)
  initial=x.native(d/'state','question-request')
  if x.sha(initial)!=plan['initial_question_request']:raise ValueError('initial state drift')
  initial_body=x.body(x.project(initial,o,expected=plan['initial_question_request'],stage='inquiry'),'inquiry',x.model_contract(d/'state','inquiry'))
  if x.sha(initial_body)!=plan['initial_body'] or x.sha(x.read(d/'inquiry-body.json'))!=plan['initial_body']:raise ValueError('initial payload drift')
  l=ledger(d/'campaign-upgrade.json');before=l.preflight_run(calls=maximum,execution_groups=1)
  if (before['status']['calls'],before['status']['execution_groups'],before['status']['known_cost_microusd'])!=tuple(plan['campaign'][k] for k in ('calls','groups','settled_microusd')):raise ValueError('campaign changed')
  quote=c.official_preflight();l.record_price(quote)
  admission={'grant':grant,'quote':quote,'before':before,'created':time.time()};x.save(d/'admission.json',admission);trial=x.sha(admission);l.enroll(trial,trial)
  corrections=correction.Budget(d/'corrections.jsonl',grant,allowance)
  count=0;operations=0;pin=None;pending={};reviewed={};inquiry=None
  def terminal(reason):
   v={'reason':reason,'attempted_call_slots':count,'attempted_operation_slots':operations,'unused_calls_expired':maximum-count,'campaign':l.status()};x.save(d/'terminal.json',v);return v
  def call(stage,feedback=None,source=None):
   nonlocal count
   if count>=maximum or time.time()+240>grant['deadline']:raise ValueError('call/time bound')
   if 'continuation' in o:x.continuation.validate_source(x,o)
   q=x.native(d/'state','question-request');ctx=x.project(q,o,expected=x.sha(q),stage=stage,inquiry=inquiry,receipt=pin,pending=pending,reviewed=reviewed);body=x.body(ctx,stage,x.model_contract(d/'state',stage,inquiry_focus=inquiry['proposal'] if stage=='select' else None))
   base=correction.digest(body)
   if source is not None and source!=base:raise ValueError('correction source/context drift')
   if feedback is not None:body=corrections.body(body,feedback)
   if len(x.canonical(body))>32768:raise ValueError('correction payload bound')
   if time.time()+240>grant['deadline']:raise ValueError('dispatch deadline headroom')
   n=count+1;x.save(d/f'context-{n}.json',ctx);x.save(d/f'body-{n}.json',body)
   req=x.sha({'trial':trial,'number':n,'stage':stage,'body':x.sha(body)})
   x.save(d/f'intent-call-{n}.json',{'request':req,'body':x.sha(body),'stage':stage,'native_request':x.sha(q)})
   count=n
   evidence=t.FailureEvidence(d,f'failed-response-{n}')
   r=l.run_call(trial,req,body,240,lambda b,s:t.invoke(b,s,expected_model='gpt-6-luna',payload_limit=32768,failure_evidence=evidence),payload_limit=32768)
   x.save(d/f'result-{n}.json',r)
   if r.get('resource_stop') or r.get('value') is None:
    evidence.clear()
    feedback=corrections.feedback(stage,base,r,req)
    return call(stage,feedback,base)
   try:
    checked=x.contract_command(d/'state','check-model-response',stage,r['value'],inquiry_focus=inquiry['proposal'] if stage=='select' else None)
   except subprocess.CalledProcessError as error:
    try:retained=evidence.retain('native_contract')
    except Exception:retained={'status':'retention_failed'}
    evidence.clear()
    x.save(d/f'contract-rejection-{n}.json',{'stage':stage,'diagnostic':(error.stderr or '')[:2048],'admitted':False,'failure_evidence':retained})
    try:feedback=corrections.feedback(stage,base,r,req,native_diagnostic=error.stderr or '')
    except ValueError as stop:raise ValueError(stage+' response contract rejected; '+str(stop)) from error
    return call(stage,feedback,base)
   evidence.clear()
   x.save(d/f'contract-check-{n}.json',checked)
   return r['value'],q
  try:
   value,q=call('inquiry')
   x.r.x.exact(value,['inquiry'])
   if value['inquiry'] is None:return terminal('inquiry_abstention')
   generated=value['inquiry']
   if generated.get('origin') not in ('model_proposed','engine_exploration') or generated.get('attribution')!='model:gpt-6-luna:inquiry':raise ValueError('inquiry origin')
   x.save(d/'inquiry.json',generated);pid=x.native(d/'state','propose-inquiry',d/'inquiry.json')
   inquiry={'proposal':pid,'value':generated}
   q=x.native(d/'state','question-request');pending={pid:x.sha(q['pending_reviews'][0])}
   value,q=call('inquiry_review')
   if value.get('proposal')!=pid or value.get('reviewer')!='model:gpt-6-luna:inquiry-review':raise ValueError('inquiry reviewer binding')
   x.save(d/'inquiry-review.json',value);x.native(d/'state','review',d/'inquiry-review.json');pending={}
   if value['decision']!='admit':return terminal('inquiry_review_'+value['decision'])
   q=x.native(d/'state','question-request')
   if any(r['route']['state']!='corpus_candidate' for r in q['information_routes'] if r['inquiry']==pid):return terminal('inquiry_waiting_for_supported_answer_source')
   reviewed={p['revision']:x.sha(p) for p in q['possibility_memory']}
   value,q=call('select')
   x.r.x.exact(value,['inquiry','request','relevance']);x.r.x.exact(value['request'],['state_root','corpus','ids'])
   request=value['request'];ids=request['ids']
   if value['inquiry']!=pid or request['state_root']!=q['state_root'] or request['corpus']!=o['corpus'] or not isinstance(ids,list) or not isinstance(value['relevance'],str) or not value['relevance'].strip() or len(value['relevance'].encode())>512:raise ValueError('invalid selection')
   if not ids:return terminal('selection_abstention')
   if len(ids)>1 or any(not isinstance(i,str) for i in ids) or len(set(ids))!=len(ids) or any(i not in o['contents'] for i in ids):raise ValueError('unallowlisted selection')
   x.save(d/'retrieval-request.json',value)
   if time.time()+30>grant['deadline']:raise ValueError('inspection deadline headroom')
   group=x.sha({'trial':trial,'choice':value});x.save(d/'intent-inspection.json',{'group':group,'choice':value})
   operations=1
   rid=l.run_execution(trial,group,x.sha(value),lambda:x.native(d/'state','retrieve-for-question',d/'retrieval-request.json'))
   q=x.native(d/'state','question-request');obs=next(z for z in q['retrieval_observations'] if z['receipt']==rid)
   pin={'request':request,'pre_root':request['state_root'],'native_receipt':rid,'native_request_digest':obs['request_digest'],'observation_sha256':x.sha(obs),'relevance':value['relevance']};x.save(d/'receipt-pin.json',pin)
   value,q=call('reflect');x.save(d/'reflection.json',value);x.native(d/'state','preview',d/'reflection.json')
   suggestions=value['response']['proposals']
   if not suggestions:return terminal('reflection_abstention')
   if len(suggestions)!=1:raise ValueError('one reflection candidate')
   kind=suggestions[0]['kind']
   if kind=='public_desire' and suggestions[0]['proposal']['change']=='clarification':return terminal('clarification_only')
   if kind not in ('new_candidate','relational','public_desire'):raise ValueError('reflection kind')
   pid=x.native(d/'state','submit-possibility' if kind=='new_candidate' else 'submit',d/'reflection.json')
   q=x.native(d/'state','question-request');pending={pid:x.sha(q['pending_reviews'][0])}
   value,q=call('review');x.save(d/'review.json',value)
   # Native review binds the full head, candidate, distinct reviewer and bounded decision.
   if value.get('proposal')!=pid or value.get('reviewer')!='gpt-6-luna/independent-context-review':raise ValueError('reflection reviewer candidate binding')
   x.native(d/'state','review',d/'review.json');pending={}
   if value['decision']!='admit':return terminal('review_'+value['decision'])
   q=x.native(d/'state','question-request')
   if q['phase']!='propose':return terminal('reviewed_meaning_requires_fresh_context_no_extra_inspection')
   for item in q['parents']:
    if item['revision'] is not None:reviewed[item['revision']]=x.sha(item)
   for key in ('guidance','possibility_memory'):
    for item in q.get(key,[]):reviewed[item['revision']]=x.sha(item)
   value,q=call('next');x.save(d/'next.json',value);preview=x.native(d/'state','preview',d/'next.json');x.save(d/'next-preview.json',preview)
   return terminal('completed_next_preview_no_further_adoption')
  except Exception as e:
   return terminal('stopped:'+type(e).__name__+':'+str(e)[:300])
if __name__=='__main__':
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('directory');a.add_argument('--grant',required=True);args=a.parse_args();print(json.dumps(run(args.directory,args.grant),indent=2))
