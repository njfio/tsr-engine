"""Source-bound, independently reviewed contrast-derived contributor identities."""
import copy

def contract():
 return {'envelope':'{contrast_preference:{aspiration:r alias,contrast:c alias}}; one advisory candidate alone', 'meaning':'Optionally propose admitting a currently reviewed aspiration as a new generated contributor from the supplied authentic unmet Contrast. Preserve all original wants. No new criterion, fulfillment, user endorsement, score or action permission. No novelty is required; retention and ordinary action remain valid.'}
def bind(request,value):
 if not isinstance(value,dict) or set(value)!={'binding','aspiration','contrast'}:raise ValueError('contrast preference fields')
 return {'protocol':request['protocol'],'state_root':request['state_root'],'attention_binding':request['next_attention']['binding'],'proposals':[{'kind':'contrast_preference','choice':copy.deepcopy(value)}]}
def verify(adapter,request):
 values=request['coding']['relational']['inputs'].get('generated_preferences',[])
 pins=adapter.admission['origin']['public'].get('public_preferences',[])
 if not isinstance(pins,list) or len(pins)!=len(set(pins)):raise ValueError('generated preference origin pins')
 known={h['admission']:h for key,v in adapter.read()[2].items() if key.startswith('meaning_admission_') for h in v.get('generated_preferences',[])}
 matched=set()
 strip=lambda x:{k:v for k,v in x.items() if k!='eligible'}
 for h in values:
  imported={adapter_module_sha({**h,'eligible':x}) for x in (True,False)} & set(pins);matched.update(imported)
  if not imported and (h['admission'] not in known or strip(known[h['admission']])!=strip(h)):raise ValueError('unowned generated preference')
 if matched!=set(pins):raise ValueError('missing imported generated preference')
 expected=[x for x in values if x['eligible']]
 actual=[x for x in request['coding']['relational']['inputs']['parents'] if 'original_contributors' in x]
 if len(actual)!=len(expected):raise ValueError('generated preference eligibility drift')
 expected_by_id={x['id']:x for x in expected}
 if len(expected_by_id)!=len(expected) or {x['id'] for x in actual}!=set(expected_by_id):raise ValueError('generated preference identity drift')
 for current in actual:
  origin=expected_by_id[current['id']]
  if current==origin:continue
  retained=current.get('generated_origin')
  if not isinstance(retained,dict) or strip(retained)!=strip(origin):raise ValueError('generated preference origin drift')
  for key in set(origin)-{'meaning','parents','generator','reviewer','proposal'}:
   if current.get(key)!=origin.get(key):raise ValueError('generated preference immutable field drift: '+key)
  if current.get('preference_parents')!=origin['parents']:raise ValueError('generated preference ancestry drift')
  # Public-meaning ownership and exact latest overlay are checked separately.
  active=request['coding']['relational']['inputs'].get('public_desires',{}).get('active',[])
  if current not in active:raise ValueError('generated meaning lacks current public overlay')
def adapter_module_sha(value):
 import hashlib,json
 return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()


def success_contract():
 return {'envelope':'{success_preference:{aspiration:r alias,success:s alias}}; one advisory candidate alone','meaning':'Optional new generated wanting from a currently reviewed aspiration and authenticated scoped artifact success. The referenced reviewed aspiration supplies the explanation of how its contributing wants make this new wanting meaningful after this success; if that connection is absent, first propose an ordinary relational development rather than inventing extra fields in this reference-only envelope. The evidence establishes only its named met artifact criteria, not human fulfillment or acceptance. Every actual contributing root must be covered; unrelated unmet/unknown findings remain visible obligations, never erased or promoted. The complete assessment remains available; no model-selected evidence subset. Preserve all parents, history, obligations and permissions; successor fulfillment remains unknown. Independent review must reject overclaimed success or unsupported connection. Retention and no novelty remain valid.'}
def bind_success(request,value):
 if not isinstance(value,dict) or set(value)!={'binding','aspiration','success'}:raise ValueError('success preference fields')
 return {'protocol':request['protocol'],'state_root':request['state_root'],'attention_binding':request['next_attention']['binding'],'proposals':[{'kind':'success_preference','choice':copy.deepcopy(value)}]}


def provisional_contract():
 return {'envelope':'{provisional_preference:{parents:[current p aliases],meaning,rationale}}; independently reviewed advisory candidate', 'meaning':'Optional genuinely generated wanting grounded in one or more current contributors, including generated children. No prior aspiration, successful return or unmet Contrast is required. Explain the connection honestly; uncertainty and no novelty are valid. This is not a claim about a new user preference or an observed fulfillment. Preserve all ancestry, original obligations and permissions. Independent review precedes native identity admission; admission grants no action authority. Transport owns binding.'}

def bind_provisional(request,value):
 if not isinstance(value,dict) or set(value)!={'binding','parents','meaning','rationale'}:raise ValueError('provisional preference fields')
 inputs=request['coding']['relational']['inputs'];parents=value['parents'];eligible={p['id'] for p in inputs['parents']}
 if value['binding']!=request['coding']['relational']['binding']:raise ValueError('stale provisional binding')
 if not isinstance(parents,list) or not parents or any(not isinstance(p,str) for p in parents) or len(set(parents))!=len(parents) or not set(parents)<=eligible:raise ValueError('provisional contributors')
 if any(not isinstance(value[k],str) or not value[k].strip() for k in ('meaning','rationale')):raise ValueError('provisional wanting and rationale required')
 return {'protocol':request['protocol'],'state_root':request['state_root'],'attention_binding':request['next_attention']['binding'],'proposals':[{'kind':'provisional_preference','choice':copy.deepcopy(value)}]}
