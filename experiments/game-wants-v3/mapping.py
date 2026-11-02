"""Offline reviewed mapping/probe compiler. No dispatcher or authority creation."""
import copy,hashlib,json

def digest(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
def source(files):return {p:hashlib.sha256(v.encode()).hexdigest() for p,v in files.items()}
def validate(mapping,files,review):
 if set(mapping)!={'version','source','entry','start','controls','readouts','documentation','limitations'} or mapping['version']!='reviewed-control-map@v1':raise ValueError('mapping contract')
 if mapping['source']!=source(files):raise ValueError('stale mapping source')
 if mapping['entry'] not in files or not mapping['entry'].endswith('.html'):raise ValueError('mapping entry')
 if set(review)!={'mapping_sha256','decision','attribution','reviewer','author','limitations'} or review['mapping_sha256']!=digest(mapping):raise ValueError('mapping review binding')
 if review['decision']!='accept':return {'status':'unknown','reason':'Mapping not independently accepted'}
 if review['reviewer']==review['author'] or not all(isinstance(review[k],str) and review[k].strip() for k in ('attribution','reviewer','author','limitations')):raise ValueError('mapping independent review required')
 # This is a record contract, not authentication. The caller must establish
 # reviewer provenance through its existing review/admission mechanism.
 if set(mapping['controls'])!={'crossing','growth'} or set(mapping['readouts'])!={'crossing','growth'}:raise ValueError('mapping roots')
 if not isinstance(mapping['documentation'],dict) or set(mapping['documentation'])!={'path','quote'} or mapping['documentation']['path'] not in files or not mapping['documentation']['quote'] or mapping['documentation']['quote'] not in files[mapping['documentation']['path']]:raise ValueError('mapping documentation absent')
 if not mapping['limitations']:raise ValueError('mapping limitations required')
 selectors=[mapping['start'],*mapping['readouts'].values()]
 if any(not isinstance(v,str) or not v or len(v.encode())>128 for v in selectors):return {'status':'unknown','reason':'Missing or unsupported control/readout mapping'}
 for keys in mapping['controls'].values():
  if not isinstance(keys,dict) or set(keys)!={'Up','Right','Down','Left'} or any(not isinstance(k,str) or not k or len(k.encode())>128 for k in keys.values()) or len(set(keys.values()))!=4:return {'status':'unknown','reason':'Ambiguous or unsupported discrete directions'}
 return {'status':'mapped','mapping_sha256':digest(mapping),'review_sha256':digest(review),'source':copy.deepcopy(mapping['source'])}

def plan(mapping,files,review,batch,keys):
 valid=validate(mapping,files,review)
 if valid['status']!='mapped':return valid
 if batch not in ('crossing-first','growth-first','mixed-pursuit'):raise ValueError('unknown observation batch')
 if not 1<=len(keys)<=6:raise ValueError('unsupported sequence length; no truncation')
 for i,key in enumerate(keys):
  root='growth' if batch=='growth-first' or batch=='mixed-pursuit' and i%2 else 'crossing'
  if key not in mapping['controls'][root].values():raise ValueError('input outside reviewed batch controls')
 actions=[{'kind':'click','value':mapping['start']}]
 for i,key in enumerate(keys):
  if i:actions.append({'kind':'wait','value':100})
  actions.append({'kind':'key','value':key})
 probe={'version':'browser-probe@v1','assets':sorted(files),'entry':mapping['entry'],'actions':actions,'checks':[{'selector':v,'text':''} for v in dict.fromkeys(mapping['readouts'].values())]}
 assert len(actions)<=12
 body={**valid,'batch':batch,'probe':probe,'coverage':'Finite ordered sequence and final DOM readouts only; no inferred success or failure. Every probe starts a fresh page and uses advertised restart. No cross-probe state continuation.'}
 return {**body,'plan_sha256':digest(body)}

def search(mapping,batch):
 if batch not in ('crossing-first','growth-first','mixed-pursuit'):raise ValueError('unknown observation batch')
 # Six predeclared partial traces, not an exhaustive search or the old 32-input policy.
 directions=('Up','Right','Down','Left')
 words=[(d,)*6 for d in directions]+[('Up','Right','Down','Left','Up','Right'),('Up','Left','Down','Right','Up','Left')]
 return [[mapping['controls']['growth' if batch=='growth-first' or batch=='mixed-pursuit' and i%2 else 'crossing'][d] for i,d in enumerate(word)] for word in words]

def bind_observation(plan,value,files):
 if plan.get('status')!='mapped':raise ValueError('no executable mapped plan')
 if digest({k:v for k,v in plan.items() if k!='plan_sha256'})!=plan['plan_sha256']:raise ValueError('plan changed')
 if source(files)!=plan['source'] or value.get('files')!=plan['source']:raise ValueError('stale observed source')
 if value.get('version')!='browser-observation@v1' or not isinstance(value.get('observations'),list):raise ValueError('observation shape')
 actions=[{'kind':v['kind'],'value':v['value']} for v in value['observations'] if v.get('kind') in ('key','click','wait')]
 if actions!=plan['probe']['actions']:raise ValueError('missing or reordered observed inputs')
 return {'version':'mapped-control-observation@v1','plan_sha256':plan['plan_sha256'],'mapping_sha256':plan['mapping_sha256'],'review_sha256':plan['review_sha256'],'source':plan['source'],'observation':copy.deepcopy(value),'criterion_verdict':None,'attribution':'source-bound standalone probe observation; not a native receipt or player feedback','coverage':plan['coverage']}
