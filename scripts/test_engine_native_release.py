"""Release path controls: authored responses only, no model/target execution."""
import copy,json,tempfile,unittest
from pathlib import Path
import test_engine_project_cycle_adapter as base
m=base.m
def strings(value):
 if isinstance(value,str):yield value
 elif isinstance(value,dict):
  for item in value.values():yield from strings(item)
 elif isinstance(value,list):
  for item in value:yield from strings(item)
class Release(base.NativeAccounting):
 def setUp(self):
  super().setUp()
  self.a=m.admission(self.root,self.origin,self.manifest,self.driver.admission['expires_at'],max_executions=2,release=True,correction_allowance=2)
 def test_format_correction_accounting_and_same_context(self):
  bodies=[];old=self.transport
  def transport(body,timeout):
   bodies.append(copy.deepcopy(body))
   if len(bodies)==1:return {'value':{'alternatives':'bad'},'receipt':self.receipt()}
   return old(body,timeout)
  a=m.Adapter(self.root,self.a,self.c,self.ledger,transport);value=a.proposal(a.native('request',None))
  self.assertTrue(value['response']['proposals']);self.assertEqual(len(bodies),2)
  self.assertEqual(value['receipt']['mode'],'fixture');self.assertTrue(a.read()[0][0]['event']['fixture_only'])
  self.assertEqual(bodies[0]['response_format'],{'type':'json_object'})
  self.assertEqual(bodies[0]['messages'],bodies[1]['messages'][:-1])
  feedback=json.loads(bodies[1]['messages'][-1]['content'])['protocol_correction']['native_structure']
  self.assertEqual(feedback['path'],'$.alternatives');self.assertEqual(feedback['code'],'array_required')
  self.assertEqual(self.ledger.status()['calls'],2);self.assertEqual(self.c.budget()['model_calls'],2)
  self.assertEqual(len(a.corrections.events()),1)
 def test_exhaustion_terminal_no_reuse(self):
  calls=[]
  def transport(body,timeout):calls.append(body);return {'value':{'alternatives':False},'receipt':self.receipt()}
  a=m.Adapter(self.root,self.a,self.c,self.ledger,transport)
  with self.assertRaisesRegex(m.d.Blocked,'exhausted'):a.proposal(a.native('request',None))
  self.assertEqual(len(calls),3)
  with self.assertRaisesRegex(m.d.Blocked,'terminal'):a.proposal(a.native('request',None))
  self.assertEqual(len(calls),3)
 def test_unknown_is_not_correctable(self):
  def transport(*_):return {'value':None,'receipt':{**self.receipt(),'usd':None}}
  a=m.Adapter(self.root,self.a,self.c,self.ledger,transport)
  with self.assertRaises(m.d.Blocked):a.proposal(a.native('request',None))
  self.assertEqual(len(a.corrections.events()),0);self.assertEqual(self.ledger.status()['calls'],1)
# Existing tests run separately; don't accidentally duplicate or change historical expectations.
for name in base.NativeAccounting.__dict__:
 if name.startswith('test_'):setattr(Release,name,None)

class PacketFlow(unittest.TestCase):
 def test_full_native_packet_return_reconsider_and_restart(self):
  import os,subprocess,time
  if not os.environ.get('TSR_B91_TEST_WORKER_BIN'):self.skipTest('explicit cfg(test) worker required')
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp).resolve();(root/'fixture-only.marker').write_text('b91-native-fixture-only@v1\n')
   scenario=m.e.scenario_prepare('pursuit-v1');frozen=root/'frozen';scenario.prepare(frozen)
   # Compile the public program/tests without executing them. Stub outcomes below
   # remain authored, irrespective of compilation; no campaign target operation.
   built=subprocess.run(['cargo','test','--offline','--locked','--no-run'],cwd=frozen/'source',capture_output=True,text=True,timeout=60)
   self.assertEqual(built.returncode,0,built.stderr)
   import shutil
   shutil.rmtree(frozen/'source/target')
   class FixtureNative(m.d.Native):
    def cli(self,*args):
     r=subprocess.run([str(m.e.ROOT/'target/debug/tsr'),'engine','--directory',str(self.directory),*args],capture_output=True,text=True,timeout=30)
     if r.returncode:raise ValueError('local authored fixture native refusal: '+r.stderr)
     return m.e.strict(r.stdout)
   (root/'state').mkdir();c=FixtureNative(root/'state')
   c.cli('project','init','--spec',str(frozen/'scope.json'),'--source',str(frozen/'source'),'--requirements',str(frozen/'requirements.json'))
   expiry=time.time()+3600
   import datetime
   expiry=datetime.datetime.fromtimestamp(expiry,datetime.timezone.utc).isoformat().replace('+00:00','Z')
   c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':120000,'reason':'Authored local packet fixture only','non_tool':{'model_calls':10,'model_tokens':409600,'wall_ms':3600000,'patch_bytes':4096,'external_cost_microusd':0}},'--admission')
   c.json_cli(('project','acceptance','feedback'),{'requirements_revision':c.cli('project','acceptance','status')['requirements_revision'],'decision_record':None,'guidance':scenario.REQUIREMENTS})
   m.e.freeze_origin(frozen,c,root/'origin.json',scenario_id='pursuit-v1');origin=json.loads((root/'origin.json').read_bytes())
   c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root'])
   camp=base.campaign;cm=camp.manifest('a'*64,root/'campaign.jsonl',fixture=True);ledger=camp.Ledger(root/'campaign.jsonl',cm,camp.digest(cm),create=True)
   def head():
    with ledger.locked() as (_,rows):return rows[-1]['hash']
   grant={'version':'b91-call-extension@v1','manifest_sha256':camp.digest(cm),'ledger_prefix_head':head(),'previous_max_calls':100,'max_calls':140,'max_execution_groups':40,'cap_microusd':camp.CAP,'reservation_microusd':camp.RESERVE,'authority_sha256':'a'*64,'reviewer':'authored local grant'}
   ledger.grant_calls(grant,camp.digest(grant))
   grant={'version':'b91-evaluation-extension@v1','manifest_sha256':camp.digest(cm),'ledger_prefix_head':head(),'previous_max_calls':140,'max_calls':200,'previous_max_execution_groups':40,'max_execution_groups':60,'cap_microusd':camp.CAP,'reservation_microusd':camp.RESERVE,'authority_sha256':'a'*64,'reviewer':'authored local grant','suite_manifest_sha256':m.e.sha(scenario.manifest()),'source_class':'newly-authored-nonprivate-rust-repair-suite'}
   ledger.grant_calls(grant,camp.digest(grant))
   ar=root/'adapter';ad=m.admission(ar,origin,cm,expiry,release=True,correction_allowance=2,max_executions=2)
   outputs=[];bodies=[];workers=[];review_bad=False;projected=[]
   def receipt():return {'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':.01}
   def transport(body,timeout):
    nonlocal review_bad
    bodies.append(copy.deepcopy(body));data=json.loads(body['messages'][1]['content'])
    self.assertEqual(body['response_format'],{'type':'json_object'})
    contexts=[v for k,v in data.get('evidence',{}).items() if k.startswith('context:')] if 'criteria' in data else [data]
    self.assertEqual(len(contexts),1)
    context=contexts[0];projected.append(copy.deepcopy(context))
    self.assertEqual(context['public_task'],scenario.public_task_context())
    self.assertEqual(context['public_task']['task'],scenario.public)
    self.assertEqual(context['parents'],origin['parents'])
    self.assertEqual(context['binding']['source_snapshot'],c.request()['coding']['snapshot'])
    self.assertTrue(all(scenario.spec['protected']['tests/contract.rs'] not in text for text in strings(context)))
    if 'criteria' in data:
     self.assertEqual(context['public_task'],projected[-2]['public_task'])
    if len(bodies)==1:return {'value':{'alternatives':'invalid authored array'},'receipt':receipt()}
    if 'criteria' in data:
     if not review_bad:
      review_bad=True;return {'value':{'findings':[]},'receipt':receipt()}
     b=data['binding'];value={'verdict':'accept','findings':{k:{'finding':'met','basis':'static_source','evidence':[b['evidence_ids'][0]],'explanation':'Authored wiring review, not semantic evidence','limitations':'Stub only'} for k in data['criteria']}}
    elif not outputs:
     value={'alternatives':[{'relational':{'parents':[p['id'] for p in data['parents']],'relationship':'Authored fixture: bounded representation can preserve distinct categories','aspiration':'Preserve both original requirements','discriminator':'Finite case outcomes may challenge the chosen means','revises':None,'catalysts':[]}}]};outputs.append(value)
    elif 'advisory_reconsideration' in data:
     prior=data['relational_history'][-1];p=prior['proposal'];lineage=data['relational_contract']['allowed_lineage'][0]
     value={'alternatives':[{'relational':{**p,'revises':prior['admission'],'development':{'original_revision':lineage['original_revision'],'prior_revision':lineage['prior_revision'],'retained_meaning':p['aspiration'],'return_reveals':'The authored fixture return reports an unmet case; not a real program execution.','unresolved':'Real semantic performance is untested.','proposed_development':None}},'return_basis':{'binding':data['return_evidence']['binding'],'observed':[x['id'] for x in data['return_evidence']['cases']][:8],'inferred':[],'unknown':['Actual program behavior is not established by fixture receipts.']}}]};outputs.append(value)
    else:
     if len(outputs)>2:self.assertEqual(len(data['relational_history']),2)
     value={'alternatives':[{'edits':[{'path':'src/lib.rs','replacement':scenario.SOURCE+'\n// authored fixture choice '+str(len(outputs))+'\n'}],'rationale':'Authored alternative only, no semantic repair claim'}]};outputs.append(value)
    return {'value':value,'receipt':receipt()}
   def worker(view):
    out=root/('worker-'+str(len(workers))+'.json');stage='failed' if not workers else 'patch'
    env={**os.environ,'TSR_B91_FIXTURE_DIRECTORY':str(c.directory),'TSR_B91_FIXTURE_OUTPUT':str(out),'TSR_B91_FIXTURE_STAGE':stage}
    try:r=subprocess.run([os.environ['TSR_B91_TEST_WORKER_BIN'],'--exact','engine::adaptive::coding::b91_fixture::fixture_worker','--nocapture'],env=env,capture_output=True,text=True,timeout=60)
    except subprocess.TimeoutExpired as error:self.fail('fixture worker timed out; stdout='+repr(error.stdout)+' stderr='+repr(error.stderr))
    self.assertEqual(r.returncode,0,r.stdout+r.stderr);value=json.loads(out.read_bytes());self.assertEqual(value['target_commands_executed'],0);workers.append(value);return value
   a=m.Adapter(ar,ad,c,ledger,transport,worker=worker)
   cr=root/'cycle';ca={'version':'local-project-cycle@v3','journal':str(cr),'scope':origin['scope'],'max_decisions':4,'max_executions':2,'expires_at':expiry,'implementation_sha256':m.d.file_sha(Path(m.loop.__file__))}
   def interrupt(point,name):
    if point=='after_result' and name=='reconsider_2_advance':raise RuntimeError('durable admission checkpoint')
   with self.assertRaisesRegex(RuntimeError,'checkpoint'):m.loop.Cycle(cr,ca,a.native,a.proposal,a.review,hook=interrupt).run()
   self.assertEqual(len(c.request()['coding']['relational']['history']),2)
   result=m.loop.Cycle(cr,ca,a.native,a.proposal,a.review).run()
   self.assertEqual(result['status'],'blocked');self.assertEqual(result['native_reason'],'proposal_resource_budget_exhausted')
   self.assertEqual(len(outputs),4);self.assertEqual(len(bodies),10);self.assertEqual(len(workers),2)
   self.assertTrue(any('advisory_reconsideration' in x for x in projected))
   self.assertTrue(any(len(x['relational_history'])==2 and 'advisory_reconsideration' not in x for x in projected))
   self.assertEqual(len({m.e.sha(x['public_task']) for x in projected}),1)
   with self.assertRaisesRegex(ValueError,'source/criteria'):
    a.project({**a.native('request',None),'source_snapshot':'f'*64})
   self.assertEqual(len(a.corrections.events()),2);self.assertEqual(ledger.status()['calls'],10);self.assertEqual(ledger.status()['execution_groups'],2)
   self.assertEqual(c.cli('project','config'),m.e.expected_configuration(scenario));self.assertEqual(c.request()['coding']['relational']['inputs']['parents'],origin['parents'])
   before=(root/'campaign.jsonl').read_bytes()
   self.assertEqual(m.loop.Cycle(cr,ca,a.native,a.proposal,a.review).run(),result)
   self.assertEqual((root/'campaign.jsonl').read_bytes(),before)
   with self.assertRaises(m.d.Blocked):a.proposal(a.native('request',None))
   self.assertEqual(ledger.status()['calls'],10)
   self.assertNotIn(scenario.spec['protected']['tests/contract.rs'],json.dumps(bodies))

class Contracts(unittest.TestCase):
 def test_full_public_contract_allowlist_and_drift(self):
  packet=m.e.scenario_prepare('pursuit-v1');context=packet.public_task_context()
  self.assertEqual(context['task'],json.loads((m.e.ROOT/'experiments/pursuit-evaluation-v1/public.json').read_bytes()))
  self.assertEqual(context['goal'],packet.spec['goal'])
  self.assertIn('Distinct u32 note IDs',context['task']['input_contract'])
  self.assertEqual(set(context),{'version','provenance_status','packet_manifest_sha256','public_file_sha256','goal','task'})
  for body in packet.spec['protected'].values():self.assertTrue(all(body not in text for text in strings(context)))
  self.assertIn('Historical frozen',context['provenance_status'])
  self.assertEqual(list(strings({'nested':[{'body':'line1\nline2'}]})),['line1\nline2'])
  packet.public['input_contract']='unbounded arbitrary input'
  with self.assertRaisesRegex(ValueError,'contract drift'):packet.public_task_context()
  packet=m.e.scenario_prepare('pursuit-v1');packet.public['private_history']='not exportable'
  with self.assertRaisesRegex(ValueError,'contract drift'):packet.public_task_context()
 def test_precise_shape_and_identity_rejection(self):
  h=m.load('test_project_shape','engine-project-output.py')
  with self.assertRaises(h.FormatFailure) as got:h.proposal_shape({'alternatives':'bad'})
  self.assertEqual(m.correction.native_feedback(got.exception.diagnostic)['path'],'$.alternatives')
  b={'review_id':'a'*64,'evidence_ids':['case:a']};schema=h.review_schema(b,{'one':'criterion'})
  good={'review_id':b['review_id'],'verdict':'unknown','findings':{'one':{'finding':'unknown','basis':'unknown','evidence':['case:a'],'explanation':'Unknown','limitations':'Not observed'}}}
  h.check(good,schema)
  wrong=copy.deepcopy(good);wrong['review_id']='b'*64
  with self.assertRaises(ValueError) as got:h.check(wrong,schema)
  self.assertNotIsInstance(got.exception,h.FormatFailure)
  for value in (None,[],False):
   bad=copy.deepcopy(good);bad['findings']['one']['explanation']=value
   with self.assertRaises(h.FormatFailure):h.check(bad,schema)
 def test_prospective_cap_validator_does_not_apply_grant(self):
  c=base.campaign
  g={'version':'native-pursuit-extension@v1','manifest_sha256':'a'*64,'ledger_prefix_head':'b'*64,'previous_max_calls':220,'max_calls':228,'previous_max_execution_groups':60,'max_execution_groups':62,'cap_microusd':c.CAP,'reservation_microusd':c.RESERVE,'authority_sha256':'c'*64,'reviewer':'authored fixture','suite_manifest_sha256':'d'*64,'source_class':'newly-authored-nonprivate-native-pursuit'}
  c.validate_evaluation_grant(g,'a'*64,'b'*64,220,60)
  for key,value in [('max_calls',229),('max_execution_groups',63),('cap_microusd',6000000),('source_class','private')]:
   with self.assertRaises(ValueError):c.validate_evaluation_grant({**g,key:value},'a'*64,'b'*64,220,60)

import test_engine_b91_campaign_extension as extension
class PursuitGrant(unittest.TestCase):
 def test_new_grant_is_append_only_and_exact(self):
  h=extension.EvaluationExtension(methodName='test_second_inquiry_extension_exact_and_append_only');h.setUp();self.addCleanup(h.doCleanups)
  h.test_second_inquiry_extension_exact_and_append_only()
  before=h.path.read_bytes()
  with h.l.locked() as (_,rows):head=rows[-1]['hash']
  c=base.campaign
  g={'version':'native-pursuit-extension@v1','manifest_sha256':c.digest(h.manifest),'ledger_prefix_head':head,'previous_max_calls':220,'max_calls':228,'previous_max_execution_groups':60,'max_execution_groups':62,'cap_microusd':c.CAP,'reservation_microusd':c.RESERVE,'authority_sha256':'c'*64,'reviewer':'authored fixture only','suite_manifest_sha256':'d'*64,'source_class':'newly-authored-nonprivate-native-pursuit'}
  h.l.grant_calls(g,c.digest(g));self.assertTrue(h.path.read_bytes().startswith(before))
  self.assertEqual(h.l.status()['effective_max_calls'],228);self.assertEqual(h.l.status()['effective_max_execution_groups'],62)
  with self.assertRaises(ValueError):h.l.grant_calls(g,c.digest(g))
  before=h.path.read_bytes()
  with h.l.locked() as (_,rows):head=rows[-1]['hash']
  revised={**g,'version':'native-pursuit-extension@v2','ledger_prefix_head':head,'previous_max_calls':228,'max_calls':230,'previous_max_execution_groups':62,'max_execution_groups':62}
  for key,value in [('max_calls',231),('max_execution_groups',63),('cap_microusd',6000000)]:
   wrong={**revised,key:value}
   with self.assertRaises(ValueError):h.l.grant_calls(wrong,c.digest(wrong))
  h.l.grant_calls(revised,c.digest(revised));self.assertTrue(h.path.read_bytes().startswith(before))
  self.assertEqual(h.l.status()['effective_max_calls'],230);self.assertEqual(h.l.status()['effective_max_execution_groups'],62)
  with self.assertRaises(ValueError):h.l.grant_calls(revised,c.digest(revised))

import test_engine_project_reconsideration as advisory_tests
class ContinuousAdvisory(advisory_tests.Advisory):
 def setUp(self):
  super().setUp();self.a.pop('advisory_profile');self.a['version']='local-project-cycle@v3';self.a['max_decisions']=1;self.blocked=False
 def native(self,kind,value):
  if kind=='response_context':return self.request()
  if kind=='request' and self.blocked:return {**self.request(),'branch':'blocked'}
  if kind=='advance':
   self.acks.append(('revision',value));self.branch='propose'
   return {'qualitative_admitted':True,'review_required':False,'next':self.request()}
  return super().native(kind,value)
 def test_completed_admission_restart_does_not_repeat(self):
  def pause(point,name):
   if point=='after_result' and name=='reconsider_0_advance':raise RuntimeError('checkpoint')
  with self.assertRaisesRegex(RuntimeError,'checkpoint'):self.runner(hook=pause).run()
  self.assertEqual(self.runner().run()['status'],'decision_budget')
  self.assertEqual(len(self.acks),1);self.assertEqual(len(self.calls),1);self.assertEqual(len(self.reviews),1)
 def test_uncertain_admission_is_not_repeated(self):
  def pause(point,name):
   if point=='after_effect' and name=='reconsider_0_advance':raise RuntimeError('checkpoint')
  with self.assertRaisesRegex(RuntimeError,'checkpoint'):self.runner(hook=pause).run()
  with self.assertRaises(advisory_tests.m.d.Blocked):self.runner().run()
  self.assertEqual(len(self.acks),1);self.assertEqual(self.actions,[])
 def test_final_unknown_review_can_acknowledge_without_new_inference(self):
  old=self.review
  def review(state,response):
   r=old(state,response);r['judgment']['decision']='unknown';self.blocked=True;return r
  self.review=review
  self.assertEqual(self.runner().run()['status'],'advisory_unknown')
  self.assertEqual(self.acks[0][0],'unknown');self.assertEqual(self.actions,[])
for name in advisory_tests.Advisory.__dict__:
 if name.startswith('test_'):setattr(ContinuousAdvisory,name,None)
