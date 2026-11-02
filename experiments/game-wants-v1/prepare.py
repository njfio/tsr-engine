"""Offline packet helpers; no native admission or provider transport."""
import copy,hashlib,json
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def sha(v):return hashlib.sha256(v).hexdigest()
def protocol():return json.loads((HERE/'protocol.json').read_text())
def semantic_input(history=None):
 p=protocol();return {'user_input':p['user_input'],'operational_roots':p['operational_roots'],'criteria':p['criteria'],'obligations':p['minimum_obligations'],'history':copy.deepcopy(history or []),'feedback':p['feedback'],'unknown_rule':p['unknown_rule'],'shared_consequences':p['shared_consequences'],'excluded_claims':p['excluded_claims'],'observation_policy':p['observation_policy']}
def arm(ordinary=False,history=None):
 facts=semantic_input(history);return {'model':protocol()['model'],'semantic_sha256':sha(json.dumps(facts,sort_keys=True,ensure_ascii=False).encode()),'user_prompt_sha256':sha(facts['user_input']['text'].encode()),'facts':facts,'process':'ordinary-memory@v1' if ordinary else 'native-tsr','formal_transitions':[] if ordinary else ['reviewed relationship','reviewed native Preference','reviewed meaning development'],'paid_dispatch':False}
def sequences(kind):
 growth=['d','w','d'] if kind=='separate' else ['ArrowRight','ArrowUp','ArrowRight']
 return {'crossing-first':['ArrowUp']*4,'growth-first':growth+['ArrowUp']*3,'mixed-pursuit':growth+(['ArrowUp','a','ArrowUp'] if kind=='separate' else ['ArrowLeft']),'partial':['ArrowUp']}
def search_plan(mapping,batch):
 import itertools
 if batch not in ('crossing-first','growth-first','mixed-pursuit'):raise ValueError('unknown batch')
 order=('Up','Right','Down','Left')
 sets={k:[mapping[k][direction] for direction in order if direction in mapping[k]] for k in ('crossing','growth')}
 if any(not actions or any(not isinstance(k,str) or not k for k in actions) for actions in sets.values()):raise ValueError('unmapped controls: unknown coverage')
 result=[]
 for length in (1,2,4,8,16,32):
  choices=[sets['growth' if batch=='growth-first' or batch=='mixed-pursuit' and i%2 else 'crossing'] for i in range(length)]
  candidates=[tuple(c[j%len(c)] for c in choices) for j in range(max(map(len,choices)))]
  candidates+=list(itertools.islice(itertools.product(*choices),64))
  unique=list(dict.fromkeys(candidates))[:64]
  result.extend([list(keys) for keys in unique])
 return result
def probe(keys):
 return {'version':'browser-probe@v1','assets':['index.html','app.js','style.css'],'entry':'index.html','actions':[{'kind':'click','value':'button'},*[{'kind':'key','value':k} for k in keys]],'checks':[{'selector':'h1','text':'Authored evidence fixture'},{'selector':'output','text':''}]}
def validate_observation(value,sources):
 if value.get('version')!='browser-observation@v1' or value.get('files')!=sources:raise ValueError('stale or foreign browser observation')
 if not isinstance(value.get('observations'),list):raise ValueError('missing observed trace')
 # Protocol success is not gameplay criterion success. Never infer not_met from
 # an incomplete path, nor native receipt status from this standalone result.
 return {'evidence_class':'authored control-driven browser observation','source':sources,'observations':value['observations'],'coverage':'finite trace only; unestablished reachability is unknown','criterion_verdict':None,'native_receipt':False}
def build():
 files={str(p.relative_to(HERE)):sha(p.read_bytes()) for p in sorted(HERE.rglob('*')) if p.is_file() and '__pycache__' not in p.parts and p.name!='manifest.json'}
 implementation={str(p.relative_to(ROOT)):sha(p.read_bytes()) for p in [ROOT/'scripts/engine-browser-harness.cjs',ROOT/'scripts/engine-browser-ordinary.py',ROOT/'scripts/engine-proposal-references.py',ROOT/'scripts/engine-criterion-return.py',ROOT/'scripts/test_game_wants_packet.py']}
 value={'version':'game-wants-offline@v1','status':'not_admitted','files':files,'implementation':implementation,'arms':{k:arm(k=='ordinary') for k in ('tsr','ordinary')},'runtime_image':'sha256:58deb9d12b7291f6d2c697b335eb927557d707488df09d1503659d7abdd00883','limits':'No paid authority; future equal uncapped policy remains subject to explicit approval. Existing per-request and Stop safeguards remain.'}
 (HERE/'manifest.json').write_text(json.dumps(value,indent=2)+'\n');return value
if __name__=='__main__':
 m=build();print(json.dumps({'manifest_sha256':sha((HERE/'manifest.json').read_bytes()),'paid_calls':0,'live_admissions':0}))
