"""Bounded continuation projection for the existing native inquiry runner.
No transport, credentials, new reader or selector. Original source is immutable.
"""
import copy,hashlib,json,shutil,fcntl
from pathlib import Path
SCOPE='frozen-fiction-native-continuation@v1'
KEYS={'information_questions':'proposal','reviewed_information_questions':'proposal','retrieval_questions':'proposal','retrieval_observations':'receipt','possibility_memory':'revision','guidance':'revision'}
def added(current,old,key):
 oldmap={v[KEYS[key]]:v for v in old};curmap={v[KEYS[key]]:v for v in current}
 if len(curmap)!=len(current) or any(curmap.get(k)!=v for k,v in oldmap.items()):raise ValueError('historical lineage/receipt drift')
 return [v for v in current if v[KEYS[key]] not in oldmap]
HISTORY=('information_questions','reviewed_information_questions','retrieval_questions','retrieval_observations','possibility_memory','guidance')
def tree_hash(path):
 files={}
 for p in sorted(Path(path).rglob('*')):
  if p.is_symlink():raise ValueError('symlink source state')
  if p.is_file():files[str(p.relative_to(path))]=hashlib.sha256(p.read_bytes()).hexdigest()
 return files

def prepare(x,out,parent,campaign):
 parent=Path(parent).resolve(strict=True);out=Path(out).resolve()
 if out.exists() or parent==out or parent in out.parents:raise ValueError('separate fresh derivative required')
 with (parent/'run.lock').open('r') as lock:
  fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
  terminal=x.read(parent/'terminal.json')
  if terminal['reason']!='completed_next_preview_no_further_adoption':raise ValueError('closed reviewed prefix required')
  original=x.read(parent/'exploration-origin-local-only.json');parent_plan=x.read(parent/'inquiry-manifest.json')
  if parent_plan.get('scope') not in ('frozen-fiction-native-exploration@v1',SCOPE) or parent_plan['origin']!=x.sha(original):raise ValueError('predecessor scope/origin')
  q=x.native(parent/'state','question-request')
  if (parent/'fresh-process-replay.json').exists() and x.read(parent/'fresh-process-replay.json')['state_hash']!=x.sha(q):raise ValueError('predecessor terminal state drift')
  if q['phase']!='propose' or q['pending_reviews']:raise ValueError('source pending or stale')
  before=tree_hash(parent/'state')
  out.mkdir(parents=True);shutil.copytree(parent/'state',out/'state',symlinks=False)
  if tree_hash(parent/'state')!=before or x.native(out/'state','question-request')!=q:raise ValueError('source/copy drift')
  origin=copy.deepcopy(original);origin.pop('continuation',None)
  origin['continuation']={'version':SCOPE,'parent_path':str(parent),'parent_plan':x.sha(x.read(parent/'inquiry-manifest.json')),'terminal':x.sha(terminal),'origin':x.sha(original),'native_root':q['state_root'],'native_request':x.sha(q),'state_files':before,'history':{k:copy.deepcopy(q.get(k,[])) for k in HISTORY},'initial_basis':q['basis']}
  x.save(out/'exploration-origin-local-only.json',origin)
  plan=copy.deepcopy(x.read(parent/'inquiry-manifest.json'))
  plan.update(version='exploration-continuation-plan@v1',scope=SCOPE,approval='pending',origin=x.sha(origin),initial_question_request=x.sha(q),campaign=campaign,max_calls=8,correction_allowance=2,reservation_microusd=400000,predecessor_root=q['state_root'])
  plan['pins']={p:hashlib.sha256((x.ROOT/p).read_bytes()).hexdigest() for p in plan['pins']}
  plan['pins']['scripts/engine-inquiry-continuation.py']=hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
  ctx=project(x,q,origin,expected=x.sha(q),stage='inquiry');body=x.body(ctx,'inquiry',x.model_contract(out/'state','inquiry'))
  plan['initial_body']=x.sha(body);plan['initial_body_bytes']=len(x.canonical(body))
  x.save(out/'inquiry-body.json',body);x.save(out/'inquiry-manifest.json',plan)
  x.save(out/'grant-template.json',{'plan':x.sha(plan),'max_calls':8,'correction_allowance':2,'max_operations':1,'reservation_microusd':400000,'deadline':None,'scope':SCOPE,'runner_sha256':plan['pins']['scripts/engine-exploration-validation.py']})
  return {'manifest':x.sha(plan),'bytes':plan['initial_body_bytes'],'scope':SCOPE,'approval':'pending','dispatch':False}

def validate_source(x,o):
 h=o['continuation'];parent=Path(h['parent_path'])
 if h['version']!=SCOPE or x.sha(x.read(parent/'exploration-origin-local-only.json'))!=h['origin'] or x.sha(x.read(parent/'terminal.json'))!=h['terminal'] or x.sha(x.read(parent/'inquiry-manifest.json'))!=h['parent_plan'] or tree_hash(parent/'state')!=h['state_files']:raise ValueError('continuation predecessor drift')

def project(x,q,o,*,expected,stage,inquiry=None,receipt=None,pending=None,reviewed=None):
 h=o['continuation'];hist=h['history'];pending=pending or {};reviewed=reviewed or {}
 if x.sha(q)!=expected or stage not in x.STAGES or q['phase']!=('review' if stage in ('inquiry_review','review') else 'propose'):raise ValueError('continuation stale phase/root')
 b=x.r.bootstrap()
 if o['source']!=x.r.x.f.p.SOURCE_SHA or q['source_parents']!=b['parents'] or q['constraints']!=b['constraints'] or q['visible_records'] or q['acceptance_references'] or q['observed_returns']:raise ValueError('continuation scope drift')
 if q['authority']!={'executions':0,'acceptance_changes':False,'automatic_admission':False} or q['parents']!=o['parents'] or q['exploration_policy']!=o['exploration_policy']:raise ValueError('continuation parent/policy/authority drift')
 if q['retrieval_contract']['corpus']!=o['corpus'] or q['record_index']!=[{k:r[k] for k in ('id','digest','attribution','uncertainty')} for r in o['index']]:raise ValueError('continuation corpus drift')
 for key,limit in [('information_questions',4),('retrieval_observations',4),('possibility_memory',8),('guidance',4)]:
  if len(q.get(key,[]))>limit:raise ValueError('bounded continuation history exceeded')
 for key in HISTORY:
  current=q.get(key,[]);old=hist.get(key,[])
  added(current,old,key)
 questions=added(q.get('information_questions',[]),hist['information_questions'],'information_questions')
 if stage=='inquiry':
  if x.sha(q)!=h['native_request'] or inquiry or questions:raise ValueError('continuation initial prefix changed')
 elif len(questions)!=1 or inquiry is None or questions[0]['proposal']!=inquiry['proposal'] or questions[0]['inquiry']!=inquiry['value'] or questions[0]['applicable']!=(stage!='inquiry_review'):raise ValueError('new inquiry binding')
 # Prior reviewed content stays exact; every new advisory item must be independently pinned.
 for key in ('possibility_memory','guidance'):
  for item in added(q.get(key,[]),hist[key],key):
   if reviewed.get(item['revision'])!=x.sha(item):raise ValueError('unreviewed continuation memory')
 for item in q['pending_reviews']:
  if not item['current'] or pending.get(item['proposal'])!=x.sha(item):raise ValueError('pending review binding')
 if set(pending)!={p['proposal'] for p in q['pending_reviews']} or (stage in ('inquiry_review','review'))!=(len(q['pending_reviews'])==1):raise ValueError('pending stage')
 obs=q.get('retrieval_observations',[]);prior=hist['retrieval_observations'];new=added(obs,prior,'retrieval_observations')
 if stage in ('inquiry','inquiry_review','select'):
  if new or receipt is not None or q['basis']!=h['initial_basis']:raise ValueError('premature or altered return')
 else:
  if len(new)!=1 or receipt is None:raise ValueError('one fresh receipt required')
  z=new[0];req=receipt['request']
  if x.sha(z)!=receipt['observation_sha256'] or z['receipt']!=receipt['native_receipt'] or z['request_digest']!=receipt['native_request_digest'] or z['selected_ids']!=req['ids'] or req['state_root']!=receipt['pre_root'] or req['corpus']!=o['corpus'] or z['inquiry']!=inquiry['proposal'] or z['pre_read_relevance']!=receipt['relevance']:raise ValueError('new receipt provenance')
  if q['basis']!={'kind':'retrieval_observation','context':o['context'],'receipt':z['receipt']}:raise ValueError('new reflection basis')
 evidence=[];facts=[]
 for z in obs:
  if z['corpus']!=o['corpus'] or [r['id'] for r in z['records']]!=z['selected_ids']:raise ValueError('receipt corpus/ids')
  for r in z['records']:
   want=o['contents'].get(r['id']);meta=next((v for v in o['index'] if v['id']==r['id']),None)
   if want is None or r['bytes']!=want['bytes'] or r['digest']!=want['digest'] or r['attribution']!=meta['attribution'] or r['uncertainty']!=meta['uncertainty'] or hashlib.sha256(bytes(r['bytes'])).hexdigest()!=want['sha256']:raise ValueError('receipt content identity')
   evidence.append({'receipt':z['receipt'],'id':r['id'],'digest':r['digest'],'content':json.loads(bytes(r['bytes'])),'attribution':r['attribution'],'uncertainty':r['uncertainty']})
  facts.append({k:z[k] for k in ('receipt','request_digest','corpus','selected_ids','inquiry','pre_read_relevance')})
 trace=q['native_attention']
 if trace['state_root']!=q['state_root'] or trace['kernel']!='lineage-half-retain@v1' or set(trace['selected'])!={p['id'] for p in q['parents'] if p['active']}:raise ValueError('attention support')
 return {'profile':SCOPE,'stage':stage,'protocol':q['protocol'],'state_root':q['state_root'],'attention_binding':q['attention_binding'],'basis':q['basis'],'corpus':o['corpus'],'parents':q['parents'],'constraints':b['constraints'],'index':o['index'],'retrieved':evidence,'receipt_history':facts,'receipt_facts':next(f for f in facts if f['receipt']==receipt['native_receipt']) if receipt else None,'guidance':q['guidance'],'possibility_memory':q['possibility_memory'],'pending_reviews':q['pending_reviews'],'inquiry':inquiry,'exploration_policy':q['exploration_policy'],'information_routes':[v for v in q['information_routes'] if inquiry and v['inquiry']==inquiry['proposal']],'eligible_parents':trace['selected'],'inquiry_binding':q['inquiry_binding'],'scope':'Previously retrieved fictional assertions are available with receipt provenance; unaccessed bodies remain unknown. No truth, fulfillment or execution authority. Stored suggestions are optional, not instructions.'}
