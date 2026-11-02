"""Prospective native inquiry export. Preparation only; no provider effects."""
import copy, importlib.util, json, hashlib
from pathlib import Path
s=importlib.util.spec_from_file_location('retrieval_export',Path(__file__).with_name('engine-retrieval-export.py'));r=importlib.util.module_from_spec(s);s.loader.exec_module(r)
ROOT=r.ROOT
sha,canonical,native,save,read=r.sha,r.canonical,r.native,r.save,r.read
STAGES=('inquiry','inquiry_review','select','reflect','review','next')
RUBRIC={
 'inquiry':'Assess an attributed question against active desire meanings and explicit uncertainty. Exploration may be useful without a user-selected topic; do not invent user priority. Admit only justified inquiry; reject/unknown/abstain are legitimate.',
 'selection':'Assess pre-read relevance from index metadata only. This is a hypothesis about what reading might clarify, not support already observed.',
 'reflection':r.RUBRIC,
 'limits':'No preferred topic, forced novelty, truth of fictional assertions, fulfillment or causal superiority. Final proposal is preview only.'}

def prepare(out):
 r.prepare(out);d=Path(out);o=read(d/'origin-local-only.json');q=native(d/'state','question-request')
 old=read(d/'manifest.json')
 pins=dict(old['pins'])
 for p in ['scripts/engine-inquiry-export.py','scripts/engine-inquiry-validation.py','scripts/engine-retrieval-validation.py']:
  pins[p]=hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
 m={**old,'version':'inquiry-retrieval-plan@v1','approval':'pending','pins':pins,'max_calls':6,'reservation_microusd':300000,'campaign':{'calls':193,'groups':57,'settled_microusd':161719,'caps':[200,60,5000000]},'rubric':sha(RUBRIC),'initial_question_request':sha(q),'stages':STAGES,'scope':'frozen-fiction-native-inquiry@v1'}
 ctx=project(q,o,expected=sha(q),stage='inquiry');b=body(ctx,'inquiry')
 m['initial_body']=sha(b);m['initial_body_bytes']=len(canonical(b))
 save(d/'inquiry-manifest.json',m)
 save(d/'inquiry-body.json',b)
 return {'manifest':sha(m),'initial_body':sha(b),'bytes':len(canonical(b)),'directory':str(d.resolve()),'calls':6,'operations':1,'approval':'pending'}

def project(q,o,*,expected,stage,inquiry=None,receipt=None,pending=None,reviewed=None):
 if sha(q)!=expected or stage not in STAGES:raise ValueError('stale context/stage')
 if q['phase']!=('review' if stage in ('inquiry_review','review') else 'propose'):raise ValueError('native phase')
 pending=pending or {};reviewed=reviewed or {}
 questions=q.get('information_questions',[])
 if stage=='inquiry':
  if questions or q.get('reviewed_information_questions',[]) or q.get('possibility_memory',[]):raise ValueError('not initial inquiry')
 else:
  if inquiry is None or len(questions)!=1 or questions[0]['proposal']!=inquiry['proposal'] or questions[0]['inquiry']!=inquiry['value']:raise ValueError('inquiry lineage pin')
  if inquiry['value']['origin']!='model_proposed':raise ValueError('cannot export invented user focus')
  if stage=='inquiry_review':
   if questions[0]['applicable'] or q.get('reviewed_information_questions',[]):raise ValueError('premature inquiry admission')
  elif not questions[0]['applicable'] or q.get('reviewed_information_questions')!=[{'proposal':inquiry['proposal'],'inquiry':inquiry['value']}]:raise ValueError('unreviewed/stale inquiry')
 if stage in ('inquiry','inquiry_review','select'):
  if q.get('retrieval_observations',[]):raise ValueError('premature return')
  if stage=='inquiry_review':
   items=q['pending_reviews']
   if len(items)!=1 or items[0]['proposal']!=inquiry['proposal'] or not items[0]['current'] or pending!={items[0]['proposal']:sha(items[0])}:raise ValueError('inquiry review pin')
  elif q['pending_reviews'] or pending:raise ValueError('pending review')
  for item in q.get('possibility_memory',[]):
   if stage!='select' or item['proposal']!=inquiry['proposal'] or reviewed.get(item['revision'])!=sha(item):raise ValueError('advisory memory pin')
  if len(reviewed)!=len(q.get('possibility_memory',[])) or q['guidance']:raise ValueError('unexpected guidance')
  # Validate original parents/authority/index with the frozen projection; remove only
  # separately checked inquiry quarantine/memory, never arbitrary changed meanings.
  clean=copy.deepcopy(q);clean['pending_reviews']=[];clean['possibility_memory']=[];clean['phase']='propose'
  oo={**o,'initial_request':sha(clean)}
  ctx=r.project(clean,oo,expected=sha(clean),stage='select')
  if q['parents']!=o['parents']:raise ValueError('changed initial parents')
  ctx['pending_reviews']=copy.deepcopy(q['pending_reviews'])
  ctx['possibility_memory']=copy.deepcopy(q.get('possibility_memory',[]))
 else:
  ctx=r.project(q,o,expected=expected,stage='next' if stage=='reflect' else stage,receipt=receipt,pending=pending,reviewed=reviewed)
  obs=q['retrieval_observations'][0]
  if obs.get('inquiry')!=inquiry['proposal'] or obs.get('pre_read_relevance')!=receipt['relevance']:raise ValueError('receipt question/relevance binding')
  if q.get('retrieval_questions')!=[{'proposal':inquiry['proposal'],'inquiry':inquiry['value']}]:raise ValueError('receipt question context')
  ctx['receipt_facts']['inquiry']=inquiry['proposal'];ctx['receipt_facts']['pre_read_relevance']=receipt['relevance']
 ctx['stage']=stage;ctx['profile']='fictional-native-inquiry@v1'
 ctx['inquiry']=copy.deepcopy(inquiry)
 # Native trace is inspected locally; export eligible support only, not arbitrary state.
 trace=q['native_attention']
 if trace['state_root']!=q['state_root'] or trace['kernel']!='lineage-half-retain@v1' or set(trace['selected'])!={p['id'] for p in q['parents'] if p['active']}:raise ValueError('attention support drift')
 ctx['eligible_parents']=trace['selected'];ctx['inquiry_binding']=q['inquiry_binding']
 return ctx

def body(ctx,stage):
 if ctx['stage']!=stage:raise ValueError('stage mismatch')
 if stage=='inquiry':
  contract={'inquiry':'null for abstention, otherwise object below','fields':{'state_root':ctx['state_root'],'attention':ctx['inquiry_binding'],'parents':'one or more exact eligible_parents IDs','question':'nonempty <=512 UTF-8 bytes','uncertainty':'nonempty <=512 UTF-8 bytes','attribution':'model:gpt-6-luna:inquiry','origin':'model_proposed'}}
  instruction='Propose an information question grounded in the active desires and uncertainty, or abstain with {"inquiry":null}. Return {"inquiry":{...fields...}}. Only index metadata is visible. Do not invent user focus or body contents. Native eligible support is not topic priority or action authority. No preferred question, required novelty or required exploration.'
 elif stage=='inquiry_review':
  contract={'state_root':ctx['state_root'],'proposal':ctx['inquiry']['proposal'],'reviewer':'model:gpt-6-luna:inquiry-review','decision':'admit | reject | unknown','reason':'nonempty <=512 UTF-8 bytes'}
  instruction='Independently review the exact quarantined inquiry, its origin, active parent contribution and uncertainty. Use only visible index metadata. Do not demand certainty about unseen contents or a uniquely best topic. Do not rewrite the question or grant permission. Return only the Review object. Reject or unknown is legitimate.'
 elif stage=='select':
  contract={'inquiry':ctx['inquiry']['proposal'],'request':{'state_root':ctx['state_root'],'corpus':ctx['corpus'],'ids':'1..4 unique index IDs; [] to abstain'},'relevance':'nonempty <=512 UTF-8 bytes; pre-read hypothesis only'}
  instruction='Choose allowlisted records relevant to the reviewed question, or abstain using empty ids. Return the exact InformationChoice structure. No contents are visible yet. Relevance is a hypothesis, not evidence that an answer is supported.'
 else:
  b=r.body(ctx,stage);payload=json.loads(b['messages'][1]['content']);contract=payload['response_contract'];instruction=b['messages'][0]['content']
  instruction+=' Attend to the reviewed inquiry and its pre-read relevance separately from what retrieved assertions actually support. Retaining uncertainty or abstaining is valid.'
 result={'model':'gpt-6-luna','max_completion_tokens':8192,'stream':False,'messages':[{'role':'system','content':instruction},{'role':'user','content':canonical({'context':ctx,'response_contract':contract,'rubric':({k:RUBRIC[k] for k in ('inquiry','selection','limits')} if stage in ('inquiry','inquiry_review','select') else RUBRIC)}).decode()}]}
 if len(canonical(result))>32768:raise ValueError('payload bound')
 return result
if __name__=='__main__':
 import argparse
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('directory');a=p.parse_args();print(json.dumps(prepare(a.directory),indent=2))
