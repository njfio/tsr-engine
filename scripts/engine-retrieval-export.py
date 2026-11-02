"""Local-only preparation/projection. No transport, credentials or inspection dispatch.
Native retrieval, generated quarantine/admission and campaign effects remain separately gated.
"""
import copy,hashlib,importlib.util,json
from pathlib import Path
s=importlib.util.spec_from_file_location('qual_export',Path(__file__).with_name('engine-qualitative-export.py'));x=importlib.util.module_from_spec(s);s.loader.exec_module(x)
ROOT=x.ROOT
canonical=x.canonical
sha=x.sha
native=x.native
RUBRIC={'support':'Assess what selected fictional assertions actually support; access is observed, truth and usefulness are not.',
'parents':'Retain both original meanings and uncertainty. Explain parent contribution without claiming fulfillment.',
'development':'A useful possibility need not change wanting. A relationship can retain its wanted condition. No novelty requirement.',
'review':'Admit only justified advisory interpretation or useful possibility; reject/unknown/abstain are legitimate. No automatic truth or authority.',
'limits':'Missing or unselected bodies are unknown. No baseline, emergence or causal advantage claim.'}
def read(p):return json.loads(Path(p).read_text())
def save(p,v):
 with Path(p).open('x') as f:json.dump(v,f,sort_keys=True,ensure_ascii=False,allow_nan=False,indent=2);f.write('\n')
def frozen():
 x.f.p.check();return x.f.p.source()
def bootstrap():
 b=x.f.bootstrap();b['records']=[];return b

def prepare(out):
 out=Path(out).resolve();out.mkdir(parents=True,exist_ok=False);(out/'corpus').mkdir()
 source=frozen();save(out/'bootstrap.json',bootstrap());q=native(out/'state','init',out/'bootstrap.json')
 records=[];contents={};index=[]
 for r in source['records']:
  path=out/'corpus'/(r['id']+'.json');path.write_bytes(canonical(r)+b'\n');digest=native(out/'state','content-digest',path)
  records.append({'id':r['id'],'path':path.name,'digest':digest,'attribution':r['attributed_source']+'; evaluator-authored fiction','uncertainty':'Fictional assertion; world truth unknown'})
  contents[r['id']]={'digest':digest,'sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'bytes':list(path.read_bytes())}
  index.append({'id':r['id'],'topic':r['topic'],'attribution':records[-1]['attribution'],'uncertainty':records[-1]['uncertainty'],'digest':digest})
 config={'state_root':q['state_root'],'manifest':{'root':str(out/'corpus'),'records':records}};save(out/'config.json',config)
 native(out/'state','configure-retrieval',out/'config.json');q=native(out/'state','request-v2')
 o={'source':x.f.p.SOURCE_SHA,'initial_request':sha(q),'context':q['basis']['context'],'parents':q['parents'],'corpus':q['retrieval_contract']['corpus'],'index':index,'contents':contents,'config':sha(config),'constraints':q['constraints'],'source_parents':q['source_parents']}
 save(out/'origin-local-only.json',o);save(out/'initial-native-local-only.json',q)
 manifest={'version':'retrieval-export-plan@v1','approval':'pending','source':o['source'],'origin':sha(o),'pins':{p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in ['target/debug/tsr-qualitative','scripts/engine-retrieval-export.py','scripts/engine-qualitative-export.py','scripts/engine-complementarity-native-preflight.py','scripts/engine-complementarity-local.py','specs/experiments/complementarity-local-v1/source.json','scripts/engine-b91-campaign.py','scripts/engine-synthetic-transport.py']},'max_calls':4,'max_operations':1,'max_selected_records':4,'completion_tokens':8192,'request_bytes':32768,'seconds_per_call':240,'reservation_microusd':200000,'campaign':{'calls':192,'groups':57,'settled_microusd':161431,'caps':[200,60,5000000]},'model':'gpt-6-luna','provider':'https://api.cheaperinference.com/v1/chat/completions','rubric':sha(RUBRIC),'effects':{'calls':0,'campaign_inspections':0,'live_adoptions':0}}
 save(out/'manifest.json',manifest)
 ctx=project(q,o,expected=sha(q),stage='select');save(out/'context-select.json',ctx);save(out/'body-select.json',body(ctx,'select'))
 return {'manifest':sha(manifest),'body':sha(body(ctx,'select')),'bytes':len(canonical(body(ctx,'select'))),'directory':str(out)}

def project(q,o,*,expected,stage,receipt=None,pending=None,reviewed=None):
 if sha(q)!=expected:raise ValueError('stale native request')
 if stage not in ('select','reflect','review','next'):raise ValueError('stage')
 b=bootstrap()
 if o['source']!=x.f.p.SOURCE_SHA or q['source_parents']!=b['parents'] or q['constraints']!=b['constraints'] or q['visible_records'] or q['acceptance_references'] or q['observed_returns']:raise ValueError('scope drift')
 if q['authority']!={'executions':0,'acceptance_changes':False,'automatic_admission':False}:raise ValueError('authority drift')
 if q['retrieval_contract']['corpus']!=o['corpus'] or q['phase'] not in ('propose','review'):raise ValueError('corpus/stale context')
 if q['record_index']!=[{k:r[k] for k in ('id','digest','attribution','uncertainty')} for r in o['index']]:raise ValueError('index drift')
 reviewed=reviewed or {};pending=pending or {};used=set()
 parents=copy.deepcopy(q['parents']);originals={p['id']:p for p in o['parents']}
 for p in parents:
  if p['id'] not in originals or p['original_meaning']!=originals[p['id']]['original_meaning'] or p['active'] is not True:raise ValueError('parent origin/status')
  if p!=originals[p['id']]:
   if reviewed.get(p['revision'])!=sha(p):raise ValueError('unreviewed meaning')
   used.add(p['revision'])
 if len(parents)!=len(originals) or len({p['id'] for p in parents})!=len(originals):raise ValueError('parent set')
 evidence=[]
 obs=q.get('retrieval_observations',[])
 if stage=='select':
  if obs or q['basis']!={'kind':'initial_context','context':o['context']} or sha(q)!=o['initial_request']:raise ValueError('not fresh selection')
 else:
  if len(obs)!=1 or receipt is None or receipt['observation_sha256']!=sha(obs[0]) or receipt['native_receipt']!=obs[0]['receipt']:raise ValueError('receipt provenance')
  z=obs[0];req=receipt['request'];ids=req['ids']
  if req['state_root']!=receipt['pre_root'] or req['corpus']!=o['corpus'] or not 1<=len(ids)<=4 or len(ids)!=len(set(ids)) or z['selected_ids']!=ids or z['corpus']!=o['corpus']:raise ValueError('request/corpus identity')
  if z['request_digest']!=receipt['native_request_digest']:raise ValueError('native request identity')
  if [r['id'] for r in z['records']]!=ids:raise ValueError('record set')
  for r in z['records']:
   want=o['contents'].get(r['id']);meta=next((a for a in o['index'] if a['id']==r['id']),None)
   if want is None or r['bytes']!=want['bytes'] or r['digest']!=want['digest'] or r['attribution']!=meta['attribution'] or r['uncertainty']!=meta['uncertainty']:raise ValueError('altered/unallowlisted receipt bytes')
   raw=bytes(r['bytes'])
   if hashlib.sha256(raw).hexdigest()!=want['sha256']:raise ValueError('content hash')
   evidence.append({'id':r['id'],'digest':r['digest'],'content':json.loads(raw),'attribution':r['attribution'],'uncertainty':r['uncertainty']})
  if q['basis']!={'kind':'retrieval_observation','context':o['context'],'receipt':z['receipt']}:raise ValueError('reflection basis')
 for key in ('guidance','possibility_memory'):
  for item in q.get(key,[]):
   if reviewed.get(item['revision'])!=sha(item):raise ValueError('unreviewed advisory content')
   used.add(item['revision'])
 if used!=set(reviewed):raise ValueError('stale reviewed pins')
 pending_items=q['pending_reviews']
 for item in pending_items:
  if not item['current'] or pending.get(item['proposal'])!=sha(item):raise ValueError('pending content pin')
 if set(pending)!={p['proposal'] for p in pending_items} or (stage=='review')!=(len(pending_items)==1):raise ValueError('pending stage')
 if stage in ('select','reflect') and (reviewed or pending):raise ValueError('unexpected adoption')
 return {'profile':'fictional-retrieval-export@v1','stage':stage,'protocol':q['protocol'],'state_root':q['state_root'],'attention_binding':q['attention_binding'],'basis':q['basis'],'corpus':o['corpus'],'parents':parents,'constraints':b['constraints'],'index':o['index'],'retrieved':evidence,'receipt_facts':None if stage=='select' else {k:obs[0][k] for k in ('receipt','request_digest','corpus','selected_ids')},'guidance':copy.deepcopy(q['guidance']),'possibility_memory':copy.deepcopy(q.get('possibility_memory',[])),'pending_reviews':copy.deepcopy(pending_items),'scope':'Fictional assertions; receipt proves access/content identity only. No truth, fulfillment or action permission.'}

def body(ctx,stage):
 if ctx['stage']!=stage:raise ValueError('stage binding')
 if stage=='select':
  contract={'state_root':ctx['state_root'],'corpus':ctx['corpus'],'ids':'1..4 unique index IDs, or [] to abstain','reason':'bounded explanation, no body claims'}
  # Corpus is frozen by the index but explicitly supplied for exact selection binding.
  instruction='Choose which allowlisted fictional records to inspect, or abstain. Only index metadata is visible. Do not invent their contents or a desired answer. Return JSON with state_root, corpus, ids, reason.'
 else:
  legacy={'phase':'review' if stage=='review' else 'propose',**ctx};legacy['phase']='review' if stage=='review' else 'propose'
  base=x.body(legacy);payload=json.loads(base['messages'][1]['content']);contract=payload['response_contract']
  instruction=base['messages'][0]['content']
  if stage!='review':contract['suggestion_variants']['relational']['proposal']['aspiration']='bounded wanted condition; may retain existing meaning without invented novelty'
  instruction += ' This prospective profile permits reviewed useful possibilities without changed wanting. Retrieved fictional assertions remain unverified. Copy the supplied retrieval basis; no empirical outcome/fulfillment is established. Relationship revision may retain its wanted condition. A public meaning change can invalidate the current receipt context; do not force continuation.'
  if stage=='review':instruction='Independently assess the quarantined candidate against the exact retrieved assertions, parent meanings and uncertainty. Admit justified useful possibility or relationship/meaning development; no novelty requirement. Reject unsupported truth/fulfillment claims; unknown is legitimate. Return only the supplied Review object. Do not rewrite the candidate.'
  if stage=='next':instruction+=' Final response is preview only. No further adoption, action or review call follows.'
 result={'model':'gpt-6-luna','max_completion_tokens':8192,'stream':False,'messages':[{'role':'system','content':instruction},{'role':'user','content':canonical({'context':ctx,'response_contract':contract,'rubric':RUBRIC}).decode()}]}
 if len(json.dumps(result,ensure_ascii=False).encode())>32768:raise ValueError('payload limit')
 return result

def main():
 import argparse
 a=argparse.ArgumentParser();sub=a.add_subparsers(dest='cmd',required=True)
 p=sub.add_parser('prepare');p.add_argument('directory')
 p=sub.add_parser('project');p.add_argument('directory');p.add_argument('stage',choices=['select','reflect','review','next']);p.add_argument('--pins');p.add_argument('--output',required=True)
 args=a.parse_args()
 if args.cmd=='prepare':print(json.dumps(prepare(args.directory),indent=2));return
 d=Path(args.directory);q=native(d/'state','request-v2');o=read(d/'origin-local-only.json');pins=read(args.pins) if args.pins else {}
 ctx=project(q,o,expected=pins.get('expected_request',sha(q)),stage=args.stage,receipt=pins.get('receipt'),pending=pins.get('pending'),reviewed=pins.get('reviewed'));save(args.output,body(ctx,args.stage));print(sha(body(ctx,args.stage)))
if __name__=='__main__':main()
