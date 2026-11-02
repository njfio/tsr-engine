"""Authored lifecycle faults only; not model behavior or gameplay evidence."""
import copy,hashlib,importlib.util,json,unittest,tempfile,types
from pathlib import Path
s=importlib.util.spec_from_file_location('pursuit',Path(__file__).with_name('engine-pursuit-reflection.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
def sha(x):return hashlib.sha256(json.dumps(x,sort_keys=True,separators=(',',':')).encode()).hexdigest()
class Fake:
 def __init__(self,crash=None,verdict='accept'):
  self.done={};self.calls=0;self.applies=[];self.crash=crash;self.verdict=verdict;self.stopped=False;self.unknown=False
  self.h={'id':'a'*64,'frozen':{'expectation':{'anticipated':'Authored label appears','grounds':'Authored source','uncertainty':'No people tested'},'source_before':'b'*64},'return':{'receipt':'c'*64,'source_after':'d'*64,'observations':['Authored receipt']}}
  self.controller=self;self.admission={'fixture_only':True};self.narrative_limit=131072;self.recovery_sha=sha
 def read(self):return [],{},self.done
 def cli(self,*args):return {'state_root':str(self.calls),'binding':'e'*64,'snapshot':'d'*64,'pending':None if self.applies else copy.deepcopy(self.h)}
 def native(self,*args):return {'state_root':str(self.calls),'scope':'f'*64,'source_snapshot':'d'*64}
 def project(self,view):return {},{'binding':view,'authored':'Public fixture, no private state'}
 def body(self,system,payload):return {'messages':[{'role':'system','content':system},{'role':'user','content':json.dumps(payload)}]}
 def op(self,name,fn):
  if name not in self.done:self.done[name]=fn()
  if self.crash and name.startswith(self.crash):self.crash=None;raise RuntimeError('authored interruption after durable record')
  return self.done[name]
 def verify_owned_output(self,ident):
  assert ident in {str(n) for n in range(1,self.calls+1)}
 def checked(self,name,body,validate,review=False):
  if self.stopped:raise ValueError('Stopped; desires retained')
  payload=json.loads(body['messages'][1]['content']);self.calls+=1
  self.done['request_body_'+str(self.calls)]={'body':body,'body_sha256':sha(body)}
  if self.unknown:raise ValueError('Unknown response; no retry')
  verdict='accept' if name.startswith('pursuit_interpretation_') else self.verdict
  finding={'accept':'met','reject':'not_met','unknown':'unknown','abstain':'unknown'}[verdict]
  v={'review_id':payload['binding']['review_id'],'verdict':verdict,'findings':{k:{'finding':finding,'basis':'static_source','evidence':[payload['binding']['evidence_ids'][0]],'explanation':'Authored assessment','limitations':'Not live evidence'} for k in payload['criteria']}}
  if 'findings' not in payload['response_schema']['properties']:v={'review_id':payload['binding']['review_id']}
  if 'interpretation' in payload['response_schema']['properties']:v['interpretation']={'conclusion':'unknown','observed':'Authored observation','revised_expectation':None,'uncertainty':'Authored uncertainty','contrast':'Goal fulfillment is unobserved','possibilities':[]}
  validate(v);return str(self.calls),v,{'mode':'fixture'}
 def json_cli(self,args,v):
  assert v['reflection']['receipt']==self.h['return']['receipt'];self.applies.append(v)
  if self.crash=='apply':self.crash=None;raise RuntimeError('authored interruption after native commit')
  return {'authored':'applied'}
class Lifecycle(unittest.TestCase):
 def test_session_pending_return_precedes_guidance_and_checkpoint_stop_blocks(self):
  spec=importlib.util.spec_from_file_location('session_test',Path(__file__).with_name('engine-browser-session.py'));session=importlib.util.module_from_spec(spec);spec.loader.exec_module(session)
  for stop in (False,True):
   with tempfile.TemporaryDirectory() as folder:
    root=Path(folder).resolve();events=[]
    a=types.SimpleNamespace(root=root/'adapter',admission={'feedback_events':[{'id':'fixture'}]},read=lambda:([],{},{}),ledger=types.SimpleNamespace(status=lambda:{'stopped':False}),complete_pending_return=lambda:events.append('return'),apply_feedback=lambda index:events.append('feedback'),native=lambda *args:{'branch':'propose'},proposal=None,review=None)
    class Cycle:
     def __init__(self,*args):pass
     def run(self):events.append('choice');return {'status':'proposal_retention'}
    if stop:
     (root/'checkpoint-0').mkdir();(root/'checkpoint-0/STOP').write_text('authored Stop')
     with self.assertRaisesRegex(ValueError,'session stopped'):session.run(a,{'journal':str(root)},cycle_type=Cycle)
     self.assertEqual(events,[])
    else:
     self.assertEqual(session.run(a,{'journal':str(root)},cycle_type=Cycle)['status'],'completed_checkpoints')
     self.assertEqual(events,['return','feedback','choice'])
 def test_interpretation_uses_transport_binding_without_review_fields(self):
  a=Fake();m.reflect(a)
  spec=importlib.util.spec_from_file_location('bound_test',Path(__file__).with_name('engine-bound-output.py'));bound=importlib.util.module_from_spec(spec);spec.loader.exec_module(bound)
  body=a.done['request_body_1']['body'];wire,metadata=bound.prepare_review_body(body,aliases=True)
  semantic={'interpretation':a.done['pursuit_interpretation_'+'a'*64]['value']['interpretation']}
  expanded=bound.expand_review(bound.attach_review(semantic,metadata),wire)
  self.assertEqual(expanded,a.done['pursuit_interpretation_'+'a'*64]['value'])
  with self.assertRaises(ValueError):bound.attach_review({**semantic,'review_id':'wrong'},metadata)
 def test_generator_has_no_self_review_but_assessor_remains_strict(self):
  a=Fake();m.reflect(a)
  bodies=[json.loads(v['body']['messages'][1]['content']) for k,v in a.done.items() if k.startswith('request_body_')]
  self.assertEqual(set(bodies[0]['response_schema']['properties']),{'review_id','interpretation'})
  self.assertEqual(set(bodies[1]['response_schema']['properties']),{'review_id','verdict','findings'})
  spec=importlib.util.spec_from_file_location('shape_test',Path(__file__).with_name('engine-project-output.py'));shape=importlib.util.module_from_spec(spec);spec.loader.exec_module(shape)
  value=copy.deepcopy(a.done['pursuit_assessment_'+'a'*64]['value']);value['verdict']='unknown'
  with self.assertRaisesRegex(ValueError,'reflection uncertainty missing'):m._validate(value,bodies[1]['binding'],shape,bodies[1]['response_schema'])
  old={**a.done['pursuit_interpretation_'+'a'*64]['value'],'verdict':'unknown','findings':value['findings']}
  with self.assertRaises(ValueError):m._validate_interpretation(old,bodies[0]['binding'],shape,bodies[0]['response_schema'])
 def test_legacy_capture_keeps_stage_prompt_and_criteria_on_resume(self):
  for point in ('pursuit_interpretation_','pursuit_assessment_'):
   a=Fake('pursuit_input_')
   with self.assertRaises(RuntimeError):m.reflect(a)
   capture=a.done['pursuit_input_'+'a'*64];capture.pop('contract_version');capture['payload']['criteria'].pop('evidence_coverage')
   a.crash=point
   with self.assertRaises(RuntimeError):m.reflect(a)
   m.reflect(a);self.assertEqual(a.calls,2);self.assertEqual(len(a.applies),1)
   bodies=[v['body'] for k,v in a.done.items() if k.startswith('request_body_')]
   self.assertEqual(bodies[0]['messages'][0]['content'],m.LEGACY_GENERATION_SYSTEM)
   self.assertEqual(json.loads(bodies[1]['messages'][1]['content'])['criteria'],capture['payload']['criteria'])
   self.assertNotIn('evidence_coverage',json.loads(bodies[1]['messages'][1]['content'])['response_schema']['properties']['findings']['properties'])
 def test_exact_public_import_preserves_history_and_rejects_substitution(self):
  a=Fake();h={'id':'a'*64,'frozen':{'source_before':'b'*64,'index':0,'expectation':{'parents':['parent'],'anticipated':'Authored prior expectation'}},'return':None,'reflection_reviews':[]}
  a.admission={'origin':{'public':{'source_snapshots':['b'*64],'public_pursuits':[sha(h)]}}}
  request={'coding':{'pursuit':{'history':[h]}}};context={'parents':[{'id':'parent'}]}
  m.project(a,request,context);self.assertEqual(context['pursuit_history']['history'],[h])
  changed=copy.deepcopy(request);changed['coding']['pursuit']['history'][0]['frozen']['expectation']['anticipated']='Substituted'
  with self.assertRaisesRegex(ValueError,'unowned generated'):m.project(a,changed,{'parents':[{'id':'parent'}]})
 def test_imported_unreflected_base_accepts_only_exact_local_review_addition(self):
  a=Fake();base={'id':'a'*64,'frozen':{'source_before':'b'*64,'index':0,'expectation':{'parents':['parent'],'anticipated':'Authored prior expectation'}},'return':{'receipt':'c'*64},'reflection_reviews':[],'reviewed_reflections':[]}
  a.admission={'origin':{'public':{'source_snapshots':[],'public_pursuits':[sha(base)]}}}
  candidate={'observed':'Authored return interpretation'};assessment={'verdict':'accept'}
  a.done={'pursuit_interpretation_'+'a'*64:{'request':'g','value':{'interpretation':candidate}},'pursuit_assessment_'+'a'*64:{'request':'r','value':assessment},'bound_output_g':{'native_content_sha256':sha({'interpretation':candidate})},'bound_output_r':{'native_content_sha256':sha(assessment)}}
  h=copy.deepcopy(base);review={'generator':'separate-runtime-invocation:g','reviewer':'separate-runtime-invocation:r','reflection':candidate,'decision':'accept','review_note':json.dumps(assessment,sort_keys=True)};h['reflection_reviews']=[review];h['reviewed_reflections']=[review]
  request={'coding':{'pursuit':{'history':[h]}}};m.project(a,request,{'parents':[{'id':'parent'}]})
  h['reflection_reviews'][0]['reflection']={'observed':'Substituted meaning'}
  with self.assertRaisesRegex(ValueError,'substituted interpretation'):m.project(a,request,{'parents':[{'id':'parent'}]})
 def test_resume_after_each_settled_stage_reuses_exact_input(self):
  for point in ('pursuit_interpretation_','pursuit_assessment_','apply'):
   with self.subTest(point=point):
    a=Fake(point)
    with self.assertRaises(RuntimeError):m.reflect(a)
    m.reflect(a);self.assertEqual(a.calls,2);self.assertEqual(len(a.applies),1)
    self.assertIsNone(m.reflect(a));self.assertEqual(a.calls,2)
    capture=a.done['pursuit_input_'+'a'*64]
    self.assertEqual(next(iter(capture['payload']['evidence'].values()))['binding']['state_root'],'0')
 def test_rejection_uncertainty_and_abstention_do_not_shop(self):
  for verdict in ('reject','unknown','abstain'):
   a=Fake(verdict=verdict);m.reflect(a);self.assertEqual(a.applies[0]['decision'],verdict)
   self.assertIsNone(m.reflect(a));self.assertEqual(a.calls,2)
 def test_stale_return_blocks_retained_generation(self):
  a=Fake('pursuit_interpretation_')
  with self.assertRaises(RuntimeError):m.reflect(a)
  a.h['return']['receipt']='9'*64
  with self.assertRaisesRegex(ValueError,'source drift'):m.reflect(a)
  self.assertEqual(a.calls,1);self.assertFalse(a.applies)
 def test_stop_and_unknown_response_never_dispatch_twice(self):
  a=Fake();a.stopped=True
  with self.assertRaisesRegex(ValueError,'Stopped'):m.reflect(a)
  self.assertEqual(a.calls,0)
  a.stopped=False;a.unknown=True
  with self.assertRaisesRegex(ValueError,'Unknown'):m.reflect(a)
  with self.assertRaisesRegex(ValueError,'no redispatch'):m.reflect(a)
  self.assertEqual(a.calls,1);self.assertFalse(a.applies)
if __name__=='__main__':unittest.main()
