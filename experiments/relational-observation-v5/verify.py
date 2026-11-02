"""Offline representation audit and authored Stop/restart; no provider/ledger writes."""
import copy,hashlib,importlib.util,json,sys,unittest
from pathlib import Path
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1];sys.path.insert(0,str(ROOT));sys.path.insert(0,str(ROOT/'scripts'))
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
p=load('prospective_packet',HERE/'prepare.py');p.check()
refs=load('prospective_refs',ROOT/'scripts/engine-proposal-references.py')
adapter=load('prospective_adapter',ROOT/'scripts/engine-browser-adapter.py')
from scripts import test_relational_observation_packet as tests
tests.PACKET=HERE;tests.p=p
suite=unittest.TestSuite(tests.Packet(n) for n in ('test_frozen_projection_has_only_eligible_condition_and_uncapped_policy','test_three_condition_session_stop_resume_and_idempotence'))
if not unittest.TextTestRunner().run(suite).wasSuccessful():raise SystemExit(1)
old=ROOT/'target/relational-observation-live-v4/adapter/adapter.jsonl';before=hashlib.sha256(old.read_bytes()).hexdigest()
records=[json.loads(line)['event'] for line in old.read_text().splitlines()];done={e['name']:e['value'] for e in records if e.get('kind')=='result'}
native={refs.digest(v):v for k,v in done.items() if k.startswith('proposal_native_')}
ids=['ef5f2ca97628b4a6ed711c14c34d552b25833f4e21cf73ab16adeb5006f88221','5d9ba5cb7d0e70a97f8875e8e8257963ea8a0e2d89c9172c24401132162b5cdb','d080443b9223a034a641b765e1d244ec748b6be7ff45eeba42d75ff04ed34161']
results=[]
conditions=json.loads((HERE/'conditions.json').read_text())['conditions']
for index,ident in enumerate(ids):
 body=done['request_body_'+ident]['body'];wire=json.loads(body['messages'][1]['content']);saved=copy.deepcopy(wire)
 request=native[wire['proposal_references']['request_sha256']]
 def strings(value):
  if isinstance(value,str):return [value]
  if isinstance(value,dict):return [s for v in value.values() for s in strings(v)]
  if isinstance(value,list):return [s for v in value for s in strings(v)]
  return []
 exposed='\n'.join(strings(wire))
 for fact in conditions[index]['facts']:assert fact in exposed,('missing current fact',index,fact)
 known={fact for condition in conditions[:index+1] for fact in condition['facts']}
 for condition in conditions[index+1:]:
  for fact in condition['facts']:
   if fact not in known:assert fact not in exposed,('future condition leak',index,fact)
 wire['choice_summary']=refs.choice_summary(wire,request,wire['proposal_references'])
 assert 'success_preference' in wire['choice_summary']['forms']
 assert wire['choice_summary']['reviewed_aspiration_references']==[]
 assert 'retain' in wire['choice_summary']['forms']
 brief=wire['public_brief'];assert hashlib.sha256(brief.encode()).hexdigest()==refs.LEGACY_HOST_BRIEF_SHA256
 wire['public_brief']=brief.replace(refs.LEGACY_OPTIONS,'Current proposal forms and prerequisites are in choice_summary.')
 wire['brief_presentation']={'original_sha256':refs.LEGACY_HOST_BRIEF_SHA256,'source':'exact frozen host-authored synthetic noticeboard brief','replaced_host_capability_text':refs.LEGACY_OPTIONS,'original_preserved':'unchanged admitted origin; no task facts, user wording, evidence or obligations removed'}
 assert all(wire[k]==v for k,v in saved.items() if k!='public_brief')
 # Exercise the actual current system/body constructor, with a pure serializer.
 from types import SimpleNamespace
 out=adapter.Adapter.proposal_body(SimpleNamespace(body=lambda system,context:{**body,'messages':[{'role':'system','content':system},{'role':'user','content':json.dumps(context)}]}),{'branch':'propose'},wire)
 assert 'choice_summary' in out['messages'][0]['content']
 results.append({'retained_request':ident,'native_request_sha256':refs.digest(request),'original_bytes':len(json.dumps(body).encode()),'prospective_representation_bytes':len(json.dumps(out).encode()),'unchanged_evidence_keys':sorted(k for k in saved if k!='public_brief'),'summary_sha256':refs.digest(wire['choice_summary'])})
assert hashlib.sha256(old.read_bytes()).hexdigest()==before
print(json.dumps({'kind':'read_only_representation_fixtures_not_live_requests','checks':results,'live_admissions':0,'paid_calls':0},indent=2))
