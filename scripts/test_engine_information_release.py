"""Authored local provider/worker fixture, not live behavior. Temporary corpus only."""
import copy,json,tempfile,unittest
from pathlib import Path
import test_engine_project_cycle_adapter as base
from test_engine_native_release import strings
m=base.m
class InformationFlow(unittest.TestCase):
 def test_full_information_read_reconsider_return_and_restart(self):
  import os,subprocess,time
  if not os.environ.get('TSR_B91_TEST_WORKER_BIN'):self.skipTest('explicit cfg(test) worker required')
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp).resolve();(root/'fixture-only.marker').write_text('b91-native-fixture-only@v1\n')
   scenario=m.e.scenario_prepare('sequential-pursuit-v1');frozen=root/'frozen';scenario.prepare(frozen)
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
   c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':120000,'reason':'Authored local packet fixture only','non_tool':{'model_calls':16,'model_tokens':655360,'wall_ms':3600000,'patch_bytes':4096,'external_cost_microusd':0}},'--admission')
   c.json_cli(('project','acceptance','feedback'),{'requirements_revision':c.cli('project','acceptance','status')['requirements_revision'],'decision_record':None,'guidance':scenario.REQUIREMENTS})
   m.e.freeze_origin(frozen,c,root/'origin.json',scenario_id='sequential-pursuit-v1');origin=json.loads((root/'origin.json').read_bytes())
   c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root'])
   camp=base.campaign;cm=camp.manifest('a'*64,root/'campaign.jsonl',fixture=True);ledger=camp.Ledger(root/'campaign.jsonl',cm,camp.digest(cm),create=True)
   def head():
    with ledger.locked() as (_,rows):return rows[-1]['hash']
   grant={'version':'b91-call-extension@v1','manifest_sha256':camp.digest(cm),'ledger_prefix_head':head(),'previous_max_calls':100,'max_calls':140,'max_execution_groups':40,'cap_microusd':camp.CAP,'reservation_microusd':camp.RESERVE,'authority_sha256':'a'*64,'reviewer':'authored local grant'}
   ledger.grant_calls(grant,camp.digest(grant))
   grant={'version':'b91-evaluation-extension@v1','manifest_sha256':camp.digest(cm),'ledger_prefix_head':head(),'previous_max_calls':140,'max_calls':200,'previous_max_execution_groups':40,'max_execution_groups':60,'cap_microusd':camp.CAP,'reservation_microusd':camp.RESERVE,'authority_sha256':'a'*64,'reviewer':'authored local grant','suite_manifest_sha256':m.e.sha(scenario.manifest()),'source_class':'newly-authored-nonprivate-rust-repair-suite'}
   ledger.grant_calls(grant,camp.digest(grant))
   info=c.json_cli(('project','information-configure'),{'state_root':c.request()['state_root'],'manifest':scenario.information_manifest(frozen/'information'),'local_context_bytes':131072,'max_reads':1,'max_read_ms':1000,'exploration_allowed':True})
   ar=root/'adapter';ad=m.admission(ar,origin,cm,expiry,release=True,correction_allowance=2,max_executions=3,max_decisions=7,information_admission=info['context']['admission'])
   outputs=[];bodies=[];workers=[];review_bad=False;projected=[]
   def receipt():return {'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':.01}
   def transport(body,timeout):
    nonlocal review_bad
    self.assertLessEqual(len(json.dumps(body,ensure_ascii=False,allow_nan=False).encode()),32768)
    bodies.append(copy.deepcopy(body));data=json.loads(body['messages'][1]['content'])
    self.assertEqual(body['response_format'],{'type':'json_object'})
    contexts=[v for k,v in data.get('evidence',{}).items() if k.startswith('context:')] if 'criteria' in data else [data]
    self.assertEqual(len(contexts),1)
    context=contexts[0]
    first=(scenario.packet/'condition-a.json').read_text();second=(scenario.packet/'condition-b.json').read_text()
    self.assertTrue(all(second not in text for text in strings(context)))
    if not context['information']['context']['receipts']:self.assertTrue(all(first not in text for text in strings(context)))
    self.assertNotIn(str(frozen/'information'),json.dumps(context))
    projected.append(copy.deepcopy(context))
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
    elif len(outputs)==1:
     q=data['information'];value={'alternatives':[{'information':{'binding':q['binding'],'ids':[q['context']['index'][0]['id']],'parents':[p['id'] for p in q['context']['parents']],'question':'Could this authored illustration clarify the relation?','relevance':'It may inform preservation and perspective coverage; no preferred answer.','uncertainty':'Actual program behavior remains unknown.','access_probability':0.5}}]};outputs.append(value)
    elif 'advisory_reconsideration' in data:
     prior=data['relational_history'][-1];p=prior['proposal'];lineage=data['relational_contract']['allowed_lineage'][0]
     value={'alternatives':[{'relational':{**p,'revises':prior['admission'],'development':{'original_revision':lineage['original_revision'],'prior_revision':lineage['prior_revision'],'retained_meaning':p['aspiration'],'return_reveals':'Actual local read proves access to fictional bytes only. Program outcomes in this test are authored worker stubs.','unresolved':'Real semantic performance is untested.','proposed_development':None}},'return_basis':{'binding':data['return_evidence']['binding'],'observed':[x['id'] for x in data['return_evidence']['cases']][:8],'inferred':[],'unknown':['Actual program behavior is not established by fixture receipts.']}}]};outputs.append(value)
    else:
     if len(outputs)>2:self.assertGreaterEqual(len(data['relational_history']),2)
     value={'alternatives':[{'edits':[{'path':'src/lib.rs','replacement':scenario.SOURCE+'\n// authored fixture choice '+str(len(outputs))+'\n'}],'rationale':'Authored alternative only, no semantic repair claim'}]};outputs.append(value)
    return {'value':value,'receipt':receipt()}
   def worker(view):
    out=root/('worker-'+str(len(workers))+'.json');stage='failed' if not workers else 'patch'
    env={**os.environ,'TSR_B91_FIXTURE_DIRECTORY':str(c.directory),'TSR_B91_FIXTURE_OUTPUT':str(out),'TSR_B91_FIXTURE_STAGE':stage}
    try:r=subprocess.run([os.environ['TSR_B91_TEST_WORKER_BIN'],'--exact','engine::adaptive::coding::b91_fixture::fixture_worker','--nocapture'],env=env,capture_output=True,text=True,timeout=60)
    except subprocess.TimeoutExpired as error:self.fail('fixture worker timed out; stdout='+repr(error.stdout)+' stderr='+repr(error.stderr))
    self.assertEqual(r.returncode,0,r.stdout+r.stderr);value=json.loads(out.read_bytes());self.assertEqual(value['target_commands_executed'],0);workers.append(value);return value
   a=m.Adapter(ar,ad,c,ledger,transport,worker=worker)
   cr=root/'cycle';ca={'version':'local-project-cycle@v4','journal':str(cr),'scope':origin['scope'],'max_decisions':7,'max_executions':3,'max_information_reads':1,'expires_at':expiry,'implementation_sha256':m.d.file_sha(Path(m.loop.__file__))}
   def interrupt(point,name):
    if point=='after_result' and name=='reconsider_2_advance':raise RuntimeError('durable admission checkpoint')
   with self.assertRaisesRegex(RuntimeError,'checkpoint'):m.loop.Cycle(cr,ca,a.native,a.proposal,a.review,hook=interrupt).run()
   self.assertEqual(len(c.request()['coding']['relational']['history']),2)
   result=m.loop.Cycle(cr,ca,a.native,a.proposal,a.review).run()
   self.assertEqual(result['status'],'blocked');self.assertEqual(result['native_reason'],'proposal_resource_budget_exhausted')
   self.assertEqual(len(outputs),7);self.assertEqual(len(bodies),16);self.assertEqual(len(workers),2)
   self.assertTrue(any('advisory_reconsideration' in x for x in projected))
   self.assertTrue(any(len(x['relational_history'])==2 and 'advisory_reconsideration' not in x for x in projected))
   self.assertEqual(len({m.e.sha(x['public_task']) for x in projected}),1)
   with self.assertRaisesRegex(ValueError,'source/criteria'):
    a.project({**a.native('request',None),'source_snapshot':'f'*64})
   self.assertEqual(len(a.corrections.events()),2);self.assertEqual(ledger.status()['calls'],16);self.assertEqual(ledger.status()['execution_groups'],3)
   self.assertEqual(c.cli('project','config'),m.e.expected_configuration(scenario));self.assertEqual(c.request()['coding']['relational']['inputs']['parents'],origin['parents'])
   before=(root/'campaign.jsonl').read_bytes()
   self.assertEqual(m.loop.Cycle(cr,ca,a.native,a.proposal,a.review).run(),result)
   self.assertEqual((root/'campaign.jsonl').read_bytes(),before)
   with self.assertRaises(m.d.Blocked):a.proposal(a.native('request',None))
   self.assertEqual(ledger.status()['calls'],16)
   self.assertNotIn(scenario.spec['protected']['tests/contract.rs'],json.dumps(bodies))
   accesses=[x for p in projected for x in p['return_evidence']['cases'] if x['channel']=='information_access']
   self.assertTrue(accesses);self.assertTrue(all(x['outcome']=='content_accessed' for x in accesses))
   self.assertEqual(len(c.cli('project','information-status')['context']['receipts']),1)
   print('authored full-flow request byte maximum:',max(len(json.dumps(b,ensure_ascii=False,allow_nan=False).encode()) for b in bodies))
   # Maximum-slot size counterfactual in EXISTING model-owned slots only. Frozen public
   # contract, parent meanings, corpus bytes, metadata, IDs and observed facts stay exact.
   # Whole-proposal and lineage validation are NOT claimed for these padded values.
   # This is a size upper-bound stress, not native evidence or semantic validation.
   def text512(value):return value+('x'*(512-len(value.encode())))
   def proposal_growth(p):
    p=copy.deepcopy(p)
    for k in ('relationship','aspiration','discriminator'):
     if isinstance(p.get(k),str):p[k]=text512(p[k])
    if isinstance(p.get('development'),dict):
     for k in ('retained_meaning','return_reveals','unresolved','proposed_development'):
      if isinstance(p['development'].get(k),str):p['development'][k]=text512(p['development'][k])
    return p
   def context_growth(c):
    c=copy.deepcopy(c)
    for entry in c['relational_history']:entry['proposal']=proposal_growth(entry['proposal'])
    # A 2048-byte editable source is admitted; hashes remain fixed-width in this size check.
    if len(c['source']['source'].encode())<2048:c['source']['source']+=' '*(2048-len(c['source']['source'].encode()))
    return c
   sizes=[]
   for body in bodies:
    payload=json.loads(body['messages'][1]['content']);grown=copy.deepcopy(payload)
    if 'criteria' in payload:
     for key,value in payload['evidence'].items():
      if key.startswith('context:'):grown['evidence'][key]=context_growth(value)
      elif key.startswith('candidate:'):
       for proposal in grown['evidence'][key]['proposals']:
        if proposal['kind']=='relational':proposal['proposal']=proposal_growth(proposal['proposal'])
        if isinstance(proposal.get('rationale'),str):proposal['rationale']=text512(proposal['rationale'])
        for patch in proposal.get('patches',[]):
         for name in ('old_text','new_text'):
          if len(patch[name].encode())<2048:patch[name]+=' '*(2048-len(patch[name].encode()))
    else:grown=context_growth(payload)
    stressed=copy.deepcopy(body);stressed['messages'][1]['content']=m.e.canonical(grown).decode()
    size=len(json.dumps(stressed,ensure_ascii=False,allow_nan=False).encode());sizes.append(size)
    if size>32768:
     with self.assertRaises(m.d.Blocked):a.body(stressed['messages'][0]['content'],grown)
    else:self.assertLessEqual(len(json.dumps(a.body(stressed['messages'][0]['content'],grown),ensure_ascii=False,allow_nan=False).encode()),32768)
    # Frozen immutable fields are never padded, including record claims and evidence.
    original_context=next(v for k,v in payload['evidence'].items() if k.startswith('context:')) if 'criteria' in payload else payload
    grown_context=next(v for k,v in grown['evidence'].items() if k.startswith('context:')) if 'criteria' in grown else grown
    for key in ('public_task','parents','information','return_evidence','binding'):
     self.assertEqual(grown_context[key],original_context[key])
   maximum=max(len(json.dumps(b,ensure_ascii=False,allow_nan=False).encode()) for b in bodies)
   self.assertLess(maximum,31683) # Lossless schema dedup reduces the measured prior fixture maximum.
   self.assertTrue(any(n>32768 for n in sizes)) # Maximum-slot stress can exceed the finite export profile.
   print('model-owned maximum-slot counterfactual bytes:',max(sizes),'refused:',sum(n>32768 for n in sizes))
   if os.environ.get('TSR_INFORMATION_WIRE_FIXTURE'):Path(os.environ['TSR_INFORMATION_WIRE_FIXTURE']).write_text(json.dumps(bodies))
   self.assertTrue(any('information_receipts' in json.loads(b['messages'][1]['content']).get('binding',{}) for b in bodies))

class Bounds(unittest.TestCase):
 def test_compact_schema_is_exactly_equivalent_and_references_are_strict(self):
  helper=m.load('compact_review_contract','engine-project-output.py')
  binding={'review_id':'a'*64,'evidence_ids':['static:a','receipt:b'],'information_receipts':['receipt:b'],'observed_receipts':[]}
  expanded=helper.review_schema(binding,m.review_contract.CRITERIA)
  compact=helper.review_schema(binding,m.review_contract.CRITERIA,compact=True)
  restored=copy.deepcopy(compact);finding=restored.pop('$defs')['finding']
  restored['properties']['findings']['properties']={k:copy.deepcopy(finding) for k in m.review_contract.CRITERIA}
  self.assertEqual(restored,expanded);self.assertLess(len(json.dumps(compact)),len(json.dumps(expanded)))
  valid={'review_id':binding['review_id'],'verdict':'unknown','findings':{k:{'finding':'unknown','basis':'unknown','evidence':['static:a'],'explanation':'Unknown','limitations':'No empirical result'} for k in m.review_contract.CRITERIA}}
  for schema in (expanded,compact):
   helper.check(valid,schema)
   bad=copy.deepcopy(valid);bad['findings']['parent_fidelity']['explanation']=[]
   with self.assertRaises(ValueError):helper.check(bad,schema)
  for schema in ({'$ref':'https://example.invalid/schema'},{'$ref':'#/$defs/missing'},{'$defs':{'loop':{'$ref':'#/$defs/loop'}},'$ref':'#/$defs/loop'}):
   with self.assertRaises(ValueError):helper.check({},schema)
 def test_information_correction_reports_full_envelope_path(self):
  import types
  fake=types.SimpleNamespace(release=True,information=True,admission={})
  context={'information':{'choice_schema':{'type':'object','required':['ids'],'properties':{'ids':{'type':'array','items':{'type':'string'}}}}}}
  try:m.Adapter.decode_proposal(fake,{'alternatives':[{'information':{'ids':'record-a'}}]}, {},context,{})
  except ValueError as failure:diagnostic=failure.diagnostic
  else:self.fail('scalar ids accepted')
  self.assertEqual(m.correction.native_feedback(diagnostic)['path'],'$.alternatives[0].information.ids')
  helper=m.load('numeric_information_shape','engine-project-output.py')
  try:helper.check('0.5',{'type':'number'},'$.alternatives[0].information.access_probability')
  except ValueError as failure:self.assertEqual(m.correction.native_feedback(failure.diagnostic)['expected_type'],'number')
  else:self.fail('string probability accepted')
  with tempfile.TemporaryDirectory() as tmp:
   budget=m.correction.Budget(Path(tmp)/'corrections.jsonl',{'authored':True},1)
   feedback=budget.feedback('proposal','a'*64,{'receipt':{'status':'settled'}},'b'*64,diagnostic)
   original={'messages':[{'role':'user','content':'unchanged authored task'}]};wire=budget.body(original,feedback)
   self.assertEqual(wire['messages'][:-1],original['messages'])
   self.assertIn('$.alternatives[0].information.ids',wire['messages'][-1]['content'])
 def test_clock_price_refresh_and_pre_reservation_failure(self):
  import contextlib,types
  launcher=m.load('information_launcher_test','engine-native-cycle.py');camp=base.campaign
  q={'version':'b91-price-quote@v2','model':camp.MODEL,'rates':{k:'0' for k in camp.PRICE_CEILINGS},'ceilings':camp.PRICE_CEILINGS,'input_token_price_threshold':-1,'retrieved_at':100,'catalog_sha256':'a'*64}
  class Ledger:
   def __init__(self):self.rows=[{'event':{'kind':'price','value':q}}];self.saved=[]
   @contextlib.contextmanager
   def locked(self):yield None,self.rows
   def record_price(self,v):self.saved.append(v)
  l=Ledger();calls=[]
  def fetch():calls.append('catalog');return {**q,'retrieved_at':3500}
  launcher.refresh_price(l,camp,now=3399,fetch=fetch);self.assertFalse(calls)
  launcher.refresh_price(l,camp,now=3500,fetch=fetch);self.assertEqual(len(l.saved),1)
  bad={**q,'retrieved_at':3500,'rates':{**q['rates'],'input_per_million':'0.1'}}
  with self.assertRaisesRegex(ValueError,'price changed'):launcher.refresh_price(l,camp,now=3500,fetch=lambda:bad)
  self.assertEqual(len(l.saved),1)
  launcher.require_headroom({'max_calls':16,'expires_at':5500},{},lambda x:x,now=100)
  with self.assertRaises(ValueError):launcher.require_headroom({'max_calls':16,'expires_at':3700},{},lambda x:x,now=100)
  effects=[]
  def reject():raise ValueError('read-only price refusal')
  fake=types.SimpleNamespace(release=True,admission={'expires_at':'2099-01-01T00:00:00Z'},read=lambda:([],None,{}),before_call=reject,enroll=lambda:effects.append('enroll'))
  with self.assertRaisesRegex(ValueError,'price refusal'):m.Adapter.call(fake,'proposal',{})
  self.assertFalse(effects)
 def test_prepare_rejects_unreviewed_code_and_missing_authority_before_state(self):
  from unittest.mock import patch
  helper=m.load('information_prepare_test','engine-information-prepare.py')
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp);release=root/'release.json';authority=root/'authority.json';campaign=root/'campaign.json';target=root/'new-state'
   authority.write_text('{}');campaign.write_text('{}');release.write_text('{}')
   argv=['prepare','--directory',str(target),'--release',str(release),'--authority',str(authority),'--campaign-manifest',str(campaign)]
   with patch('sys.argv',argv),self.assertRaisesRegex(ValueError,'preparation implementation drift'):helper.main()
   release.write_text(json.dumps({'preparer_sha256':m.d.file_sha(Path(helper.__file__)),'implementation':m.pins(),'binary_sha256':m.d.file_sha(m.e.ROOT/'target/debug/tsr'),'packet_sha256':m.e.scenario_prepare('sequential-pursuit-v1').pin,'reviewer':'authored fixture'}))
   with patch('sys.argv',argv),self.assertRaisesRegex(ValueError,'explicit future scope authority'):helper.main()
   self.assertFalse(target.exists());self.assertEqual(campaign.read_text(),'{}')
 def test_typed_counts_and_grant_are_finite(self):
  import datetime,time
  with tempfile.TemporaryDirectory() as tmp:
   root=Path(tmp).resolve();deadline=datetime.datetime.fromtimestamp(time.time()+600,datetime.timezone.utc).isoformat()
   admission={'version':'local-project-cycle@v4','journal':str(root),'scope':'a'*64,'max_decisions':7,'max_executions':3,'max_information_reads':1,'expires_at':deadline,'implementation_sha256':m.d.file_sha(Path(m.loop.__file__))}
   state={'branch':'resume','scope':'a'*64,'state_root':'b'*64,'source_snapshot':'c'*64,'binding':'d'*64,'dispatch_authorized':False,'plan':{'authored':True}}
   calls=[]
   def native(kind,value):
    calls.append(kind)
    if kind=='request':return state
    raise AssertionError('no effect should be dispatched at typed cap')
   cycle=m.loop.Cycle(root,admission,native,None,None)
   for n in range(3):cycle.op('advance_'+str(n),lambda:{'next':state})
   for n in range(2):cycle.op('resume_'+str(n),lambda:{'authored':True})
   self.assertEqual(cycle.run()['status'],'sealed_execution_budget')
   self.assertEqual(calls,['request'])
   for invalid in (2,True):
    with self.assertRaises(ValueError):m.loop.Cycle(root,{**admission,'max_information_reads':invalid},native,None,None)
  campaign=base.campaign
  g={'version':'native-information-extension@v1','manifest_sha256':'a'*64,'ledger_prefix_head':'b'*64,'previous_max_calls':230,'max_calls':238,'previous_max_execution_groups':62,'max_execution_groups':64,'cap_microusd':5000000,'reservation_microusd':50000,'authority_sha256':'c'*64,'reviewer':'authored local fixture','suite_manifest_sha256':'d'*64,'source_class':'newly-authored-nonprivate-native-information-pursuit'}
  campaign.validate_evaluation_grant(g,'a'*64,'b'*64,230,62)
  for key,value in [('max_calls',239),('max_execution_groups',65),('cap_microusd',6000000)]:
   with self.assertRaises(ValueError):campaign.validate_evaluation_grant({**g,key:value},'a'*64,'b'*64,230,62)
 def test_information_schema_and_review_are_not_test_success(self):
  helper=m.load('information_bounds','engine-project-output.py')
  b={'review_id':'a'*64,'evidence_ids':['read:a'],'observed_receipts':[],'information_receipts':['read:a']}
  good={'review_id':b['review_id'],'verdict':'accept','findings':{k:{'finding':'met','basis':'observed_information_receipt','evidence':['read:a'],'explanation':'Read confirms identity only, not truth','limitations':'Authored local fixture'} for k in m.review_contract.CRITERIA}}
  helper.check(good,helper.review_schema(b,m.review_contract.CRITERIA));m.review_contract.validate(good,b)
  bad=copy.deepcopy(good)
  for f in bad['findings'].values():f['basis']='observed_test_receipt'
  with self.assertRaises(ValueError):m.review_contract.validate(bad,b)
  with self.assertRaises(ValueError):m.review_contract.validate(good,{**b,'information_receipts':[]})
  for value in [True,None,'0.5']:
   with self.assertRaises(ValueError):helper.check(value,{'type':'number','minimum':0,'maximum':1})
