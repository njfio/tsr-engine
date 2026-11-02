"""Bounded admitted retrieval trial. No effects without a fresh exact local grant file.
Call/effect intents are durable; any interruption stops for reconciliation, never retries.
"""
import fcntl,hashlib,importlib.util,json,time,argparse
from pathlib import Path
def load(n,f):
 s=importlib.util.spec_from_file_location(n,Path(__file__).with_name(f));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
x=load('rx','engine-inquiry-export.py');c=load('camp','engine-b91-campaign.py');t=load('transport','engine-synthetic-transport.py')
ROOT=x.ROOT

def ledger():
 m=c.strict((ROOT/'target/tsr-integrated-campaign-1/manifest.json').read_bytes());u=c.strict((ROOT/'target/heldout-evaluation/campaign-upgrade.json').read_bytes());return c.Ledger(m['ledger_path'],m,c.digest(m),upgrade=u,upgrade_hash=c.digest(u))
def run(d,grant_path):
 d=Path(d).resolve();grant=c.strict(Path(grant_path).read_bytes());plan=x.read(d/'inquiry-manifest.json')
 with (d/'run.lock').open('a') as lock:
  fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
  if (d/'terminal.json').exists():return {'terminal':x.read(d/'terminal.json'),'dispatch':False}
  if list(d.glob('intent-*.json')) or (d/'admission.json').exists():raise ValueError('Existing run; reconcile only, no redispatch')
  if set(grant)!={'plan','max_calls','max_operations','reservation_microusd','deadline','scope','runner_sha256'} or grant['plan']!=x.sha(plan) or grant['max_calls']!=6 or grant['max_operations']!=1 or grant['reservation_microusd']!=300000 or grant['scope']!='frozen-fiction-native-inquiry@v1' or grant['runner_sha256']!=hashlib.sha256(Path(__file__).read_bytes()).hexdigest() or not time.time()+1600<grant['deadline']<time.time()+7200:raise ValueError('fresh exact grant required')
  for p,h in plan['pins'].items():
   if hashlib.sha256((ROOT/p).read_bytes()).hexdigest()!=h:raise ValueError('implementation drift')
  o=x.read(d/'origin-local-only.json')
  if x.sha(o)!=plan['origin']:raise ValueError('origin drift')
  initial=x.native(d/'state','question-request')
  if x.sha(initial)!=plan['initial_question_request']:raise ValueError('initial state drift')
  initial_body=x.body(x.project(initial,o,expected=plan['initial_question_request'],stage='inquiry'),'inquiry')
  if x.sha(initial_body)!=plan['initial_body'] or x.sha(x.read(d/'inquiry-body.json'))!=plan['initial_body']:raise ValueError('initial payload drift')
  l=ledger();before=l.preflight_run(calls=6,execution_groups=1)
  if (before['status']['calls'],before['status']['execution_groups'],before['status']['known_cost_microusd'])!=(193,57,161719):raise ValueError('campaign changed')
  quote=c.official_preflight();l.record_price(quote)
  admission={'grant':grant,'quote':quote,'before':before,'created':time.time()};x.save(d/'admission.json',admission);trial=x.sha(admission);l.enroll(trial,trial)
  count=0;operations=0;pin=None;pending={};reviewed={};inquiry=None
  def terminal(reason):
   v={'reason':reason,'attempted_call_slots':count,'attempted_operation_slots':operations,'unused_calls_expired':6-count,'campaign':l.status()};x.save(d/'terminal.json',v);return v
  def call(stage):
   nonlocal count
   if count>=6 or time.time()+240>grant['deadline']:raise ValueError('call/time bound')
   q=x.native(d/'state','question-request');ctx=x.project(q,o,expected=x.sha(q),stage=stage,inquiry=inquiry,receipt=pin,pending=pending,reviewed=reviewed);body=x.body(ctx,stage)
   if time.time()+240>grant['deadline']:raise ValueError('dispatch deadline headroom')
   n=count+1;x.save(d/f'context-{n}.json',ctx);x.save(d/f'body-{n}.json',body)
   req=x.sha({'trial':trial,'number':n,'stage':stage,'body':x.sha(body)})
   x.save(d/f'intent-call-{n}.json',{'request':req,'body':x.sha(body),'stage':stage,'native_request':x.sha(q)})
   count=n
   r=l.run_call(trial,req,body,240,lambda b,s:t.invoke(b,s,expected_model='gpt-6-luna',payload_limit=32768),payload_limit=32768)
   x.save(d/f'result-{n}.json',r)
   if r.get('resource_stop') or r.get('value') is None:raise ValueError('provider failure/unknown; stop and reconcile')
   return r['value'],q
  try:
   value,q=call('inquiry')
   x.r.x.exact(value,['inquiry'])
   if value['inquiry'] is None:return terminal('inquiry_abstention')
   generated=value['inquiry']
   if generated.get('origin')!='model_proposed' or generated.get('attribution')!='model:gpt-6-luna:inquiry':raise ValueError('inquiry origin')
   x.save(d/'inquiry.json',generated);pid=x.native(d/'state','propose-inquiry',d/'inquiry.json')
   inquiry={'proposal':pid,'value':generated}
   q=x.native(d/'state','question-request');pending={pid:x.sha(q['pending_reviews'][0])}
   value,q=call('inquiry_review')
   if value.get('proposal')!=pid or value.get('reviewer')!='model:gpt-6-luna:inquiry-review':raise ValueError('inquiry reviewer binding')
   x.save(d/'inquiry-review.json',value);x.native(d/'state','review',d/'inquiry-review.json');pending={}
   if value['decision']!='admit':return terminal('inquiry_review_'+value['decision'])
   q=x.native(d/'state','question-request')
   reviewed={p['revision']:x.sha(p) for p in q['possibility_memory']}
   value,q=call('select')
   x.r.x.exact(value,['inquiry','request','relevance']);x.r.x.exact(value['request'],['state_root','corpus','ids'])
   request=value['request'];ids=request['ids']
   if value['inquiry']!=pid or request['state_root']!=q['state_root'] or request['corpus']!=o['corpus'] or not isinstance(ids,list) or not isinstance(value['relevance'],str) or not value['relevance'].strip() or len(value['relevance'].encode())>512:raise ValueError('invalid selection')
   if not ids:return terminal('selection_abstention')
   if len(ids)>4 or any(not isinstance(i,str) for i in ids) or len(set(ids))!=len(ids) or any(i not in o['contents'] for i in ids):raise ValueError('unallowlisted selection')
   x.save(d/'retrieval-request.json',value)
   if time.time()+30>grant['deadline']:raise ValueError('inspection deadline headroom')
   group=x.sha({'trial':trial,'choice':value});x.save(d/'intent-inspection.json',{'group':group,'choice':value})
   operations=1
   rid=l.run_execution(trial,group,x.sha(value),lambda:x.native(d/'state','retrieve-for-question',d/'retrieval-request.json'))
   q=x.native(d/'state','question-request');obs=q['retrieval_observations'][0]
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
if __name__ == '__main__':
    import runpy, sys
    runpy.run_path(str(Path(__file__).with_name('engine-output-release.py')))['check_cli'](__file__,sys.argv[1:])

if __name__=='__main__':
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('directory');a.add_argument('--grant',required=True);args=a.parse_args();print(json.dumps(run(args.directory,args.grant),indent=2))
