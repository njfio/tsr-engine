"""Authored responses, actual ordinary native browser return; no Contrast injection.
No provider calls, player feedback, or live TSR behavioral claim.
"""
import copy,json,os,unittest,types
from pathlib import Path
from scripts.test_engine_browser_adapter import BrowserAdapter,m
h=m.load('criterion_tests','engine-criterion-return.py')

class CriterionShape(unittest.TestCase):
 def test_coverage_alias_roundtrip(self):
  b=m.load('criterion_alias_test','engine-bound-output.py');shape=m.load('criterion_shape_test','engine-project-output.py')
  binding={'review_id':'r'*64,'evidence_ids':['index.html','test_receipt'],'observed_receipts':['test_receipt']}
  payload={'binding':binding,'evidence':{'index.html':'authored','test_receipt':'authored'},'coverage':{'x':['index.html','test_receipt']},'response_schema':shape.review_schema(binding,{'x':'authored'})}
  wire=b.alias_review_payload(payload);self.assertEqual(wire['coverage'],{'x':['e0','e1']})
  value={'findings':{'x':{'evidence':['e0','e1']}}}
  self.assertEqual(b.expand_review(value,{'messages':[{}, {'content':json.dumps(wire)}]})['findings']['x']['evidence'],payload['coverage']['x'])
  bad=copy.deepcopy(wire);bad['coverage']['x']=['foreign']
  with self.assertRaises(ValueError):b.expand_review(value,{'messages':[{}, {'content':json.dumps(bad)}]})
 def test_authentic_legacy_review_reconstructs_original_prompt(self):
  b=m.load('legacy_criterion_bound','engine-bound-output.py');shape=m.load('legacy_criterion_shape','engine-project-output.py')
  admission={'authored':True};source={'state_root':'r','source_snapshot':'s','scope':'p'}
  binding={'review_id':'a'*64,'evidence_ids':['index.html'],'observed_receipts':[]}
  payload={'binding':binding,'criteria':{'x':'Authored original criterion'},'evidence':{'index.html':'Authored source'},'response_schema':shape.review_schema(binding,{'x':'Authored'})}
  body=lambda system,payload:{'messages':[{'role':'system','content':system},{'role':'user','content':json.dumps(payload)}]}
  wire,metadata=b.prepare_review_body(body(h.LEGACY_SYSTEM,payload),aliases=True)
  raw={'verdict':'unknown','findings':{'x':{'finding':'unknown','basis':'unknown','evidence':['e0'],'explanation':'Authored unknown','limitations':'No claim'}}}
  value=b.expand_review(b.attach_review(raw,metadata),wire);name='criterion_authored';kind=name+'_review'
  ident=b.digest({'admission':b.digest(admission),'kind':kind,'index':0,'body':b.digest(wire)})
  dispatch={'candidate_sha256':b.digest(raw),'receipt':{'usage_status':'reported'}};settled={'stopped':False}
  association=b.associate(admission=admission,kind=kind,index=0,wire=wire,ident=ident,content=raw,dispatch=dispatch,settled=settled,source=source,current_source=source)
  association['reference_catalog_sha256']=b.digest(json.loads(wire['messages'][1]['content'])['reference_catalog'])
  record={'association':association,'model_content':raw,'host_metadata':metadata,'source':source,'reference_request':wire,'native_content_sha256':b.digest(value)}
  done={'bound_output_'+ident:record,'request_body_'+ident:{'body':wire},'dispatch_'+ident:dispatch,'settle_'+ident:settled}
  a=types.SimpleNamespace(read=lambda:([],{},done),recovery_load=lambda *args:b,admission=admission,body=body)
  capture={'payload':payload,'request_source':source,'frame':{'snapshot':'s'}};review={'request':ident,'input_sha256':b.digest(payload),'value':value}
  self.assertEqual(h.verify_retained(a,name,capture,review),record)
  with self.assertRaisesRegex(ValueError,'differs from captured'):h.verify_retained(a,name,{**capture,'contract_version':'criterion-evidence@v2'},review)
  with self.assertRaisesRegex(ValueError,'unknown criterion'):h.capture_system({'contract_version':'invented'})
 def test_authentic_v2_review_survives_added_native_context(self):
  b=m.load('legacy_criterion_bound','engine-bound-output.py');shape=m.load('legacy_criterion_shape','engine-project-output.py')
  admission={'authored':True};source={'state_root':'r','source_snapshot':'s','scope':'p'}
  binding={'review_id':'a'*64,'evidence_ids':['index.html'],'observed_receipts':[]}
  payload={'binding':binding,'criteria':{'x':'Authored original criterion'},'evidence':{'index.html':'Authored source'},'response_schema':shape.review_schema(binding,{'x':'Authored'})}
  body=lambda system,payload:{'messages':[{'role':'system','content':system},{'role':'user','content':json.dumps(payload)}]}
  wire,metadata=b.prepare_review_body(body(h.SYSTEM,payload),aliases=True)
  raw={'verdict':'unknown','findings':{'x':{'finding':'unknown','basis':'unknown','evidence':['e0'],'explanation':'Authored unknown','limitations':'No claim'}}}
  value=b.expand_review(b.attach_review(raw,metadata),wire);name='criterion_authored';kind=name+'_review'
  ident=b.digest({'admission':b.digest(admission),'kind':kind,'index':0,'body':b.digest(wire)})
  dispatch={'candidate_sha256':b.digest(raw),'receipt':{'usage_status':'reported'}};settled={'stopped':False}
  association=b.associate(admission=admission,kind=kind,index=0,wire=wire,ident=ident,content=raw,dispatch=dispatch,settled=settled,source=source,current_source=source)
  association['reference_catalog_sha256']=b.digest(json.loads(wire['messages'][1]['content'])['reference_catalog'])
  record={'association':association,'model_content':raw,'host_metadata':metadata,'source':source,'reference_request':wire,'native_content_sha256':b.digest(value)}
  done={'bound_output_'+ident:record,'request_body_'+ident:{'body':wire},'dispatch_'+ident:dispatch,'settle_'+ident:settled}
  a=types.SimpleNamespace(read=lambda:([],{},done),recovery_load=lambda *args:b,admission=admission,body=body)
  capture={'contract_version':'criterion-evidence@v2','payload':payload,'request_source':source,'frame':{'snapshot':'s'}};review={'request':ident,'input_sha256':b.digest(payload),'value':value}
  capture['unrelated_native_context']={'guidance':'new projection is not retroactively added to wire'}
  self.assertEqual(h.verify_retained(a,name,capture,review),record)
  with self.assertRaisesRegex(ValueError,'differs from captured'):h.verify_retained(a,name,{**capture,'contract_version':'criterion-evidence@v3'},review)
  with self.assertRaisesRegex(ValueError,'unknown criterion'):h.capture_system({'contract_version':'invented'})
 def test_exact_scope_keeps_invariants_out_of_artifact_findings(self):
  profile={'version':'public-criterion-return@v2'}
  frame={'requirements':{'orientation':'Keep navigation dependable. Preserve contributor history.'},'assessment_scope':{'orientation':[{'domain':'artifact','text':'Keep navigation dependable. '},{'domain':'engine_invariant','text':'Preserve contributor history.'}]}}
  original=copy.deepcopy(frame);criteria,scope=h.assessment_contract(frame,profile)
  self.assertEqual(criteria,{'orientation':'Keep navigation dependable. '});self.assertEqual(scope['engine_invariants_not_assessed']['orientation'],['Preserve contributor history.']);self.assertEqual(frame,original)
  self.assertIn('Met is not fulfillment',scope['instruction'])
  for bad in [dict(frame,assessment_scope=None),dict(frame,assessment_scope={}),dict(frame,assessment_scope={'orientation':[{'domain':'engine_invariant','text':frame['requirements']['orientation']}]})]:
   with self.assertRaises(ValueError):h.assessment_contract(bad,profile)
  for change in ('reverse','omit','alter'):
   bad=copy.deepcopy(frame);parts=bad['assessment_scope']['orientation']
   if change=='reverse':parts.reverse()
   elif change=='omit':parts.pop()
   else:parts[0]['text']='A replacement goal. '
   with self.assertRaises(ValueError):h.assessment_contract(bad,profile)
  with self.assertRaises(ValueError):h.assessment_contract(frame,{'version':'public-criterion-return@v1'})
  legacy={'requirements':original['requirements']};self.assertEqual(h.assessment_contract(legacy,{'version':'public-criterion-return@v1'})[0],legacy['requirements'])
 def test_current_condition_provenance_and_future_exclusion(self):
  frame={'requirements':{'orientation':'Recover reliable directions'},'publication':'p','requirements_revision':'r','snapshot':'s'}
  value={'version':'criterion-published-context@v1','guidance':{'orientation':'Authored synthetic opening: No previous directions.'},'provenance':{'publication':'p','requirements_revision':'r','current_source':'s','status':'current_publication','grants_authority':False,'review_carried':False}}
  context={'published_context':value,'future_feedback_events':['storm','reopening']}
  selected=h.published_context(context,frame)
  self.assertEqual(selected,value);self.assertNotIn('storm',json.dumps(selected));self.assertNotIn('reopening',json.dumps(selected))
  selected['guidance']['orientation']='changed';self.assertNotEqual(selected,value)
  for field in ('publication','requirements_revision','current_source','status','grants_authority','review_carried'):
   bad=copy.deepcopy(value);bad['provenance'][field]='foreign'
   with self.assertRaises(ValueError):h.published_context({'published_context':bad},frame)
  bad=copy.deepcopy(value);bad['guidance']['future']='reopening'
  with self.assertRaises(ValueError):h.published_context({'published_context':bad},frame)
  for bad in (None,{},dict(value,version='foreign')):
   with self.assertRaises(ValueError):h.published_context({'published_context':bad},frame)
  carried=copy.deepcopy(value);carried['provenance']['status']='unchanged_guidance_rebound'
  self.assertEqual(h.published_context({'published_context':carried},frame),carried)
  self.assertEqual(h.capture_system({'contract_version':'criterion-evidence@v2'}),h.SYSTEM)
  self.assertEqual(h.capture_system({'contract_version':'criterion-evidence@v3'}),h.CONTEXT_SYSTEM)
 def test_context_changes_during_review_prevent_native_apply(self):
  from unittest.mock import patch
  frame={'requirements':{'x':'Authored criterion'},'publication':'p','requirements_revision':'r','snapshot':'s','evidence':{'index.html':'source'}}
  ctx={'frame':frame,'current':None,'published_context':{'version':'criterion-published-context@v1','guidance':{'x':'Authored opening; no prior directions'},'provenance':{'publication':'p','requirements_revision':'r','current_source':'s','status':'current_publication','grants_authority':False,'review_carried':False}}}
  for timing in ('before','during'):
   original=copy.deepcopy(ctx);changed=copy.deepcopy(ctx);changed['published_context']['guidance']['x']='Changed current condition'
   capture={'contract_version':'criterion-evidence@v3','frame':frame,'payload':{'published_context':original['published_context']}}
   calls=[];contexts=iter([changed if timing=='before' else original,changed])
   def cli(*args):calls.append(args);return next(contexts)
   def stage(*args):calls.append('review');return {'value':{'findings':{}},'request':'authored'}
   def op(*args):self.fail('must not append or apply stale assessment')
   controller=types.SimpleNamespace(cli=cli)
   adapter=types.SimpleNamespace(admission={'criterion_return_profile':{}},controller=controller,native=lambda *args:{},project=lambda v:({'coding':{'snapshot':'s','files':[{'path':'index.html','source':'source','editable':True}]}},{}),read=lambda:([],{}, {'criterion_hash_input':capture}),recovery_sha=lambda v:'hash',recovery_load=lambda *args:types.SimpleNamespace(_stage=stage),op=op)
   with patch.object(h,'verify',lambda *args:None),patch.object(h,'native_findings',lambda *args:{}):
    with self.assertRaisesRegex(ValueError,'published context changed '+timing):h.reflect(adapter)
   self.assertEqual('review' in calls,timing=='during')
 def test_success_projection_requires_owned_scoped_assessment(self):
  from unittest.mock import patch
  frame={'receipt':'receipt','publication':'publication','requirements_revision':'revision','snapshot':'source','assessment_scope':{'x':[{'domain':'artifact','text':'Control responds'}]}}
  row={'id':'assessment','assessment':{'frame':frame,'decision':'accept','findings':{'x':{'finding':'met'}}}}
  current={'current':row,'history':[row]}
  basis={'assessment':'assessment','receipt':'receipt','publication':'publication','requirements_revision':'revision','source_snapshot':'source','criteria':['x']}
  request={'coding':{'relational':{'inputs':{'current_success':{'id':'success','basis':basis}}}}}
  adapter=types.SimpleNamespace(admission={},controller=types.SimpleNamespace(cli=lambda *args:current))
  with self.assertRaisesRegex(ValueError,'requires admitted'):h.project(adapter,request,{})
  adapter.admission={'criterion_return_profile':{'authored':True}}
  with patch.object(h,'verify') as verified:
   out={};h.project(adapter,request,out);verified.assert_called_once_with(adapter,current);self.assertEqual(out['criterion_returns']['current'],row)
   for change in ({'assessment':'foreign'},{'source_snapshot':'old'},{'receipt':'forged'},{'criteria':['unpublished']}):
    bad=copy.deepcopy(request);bad['coding']['relational']['inputs']['current_success']['basis'].update(change)
    with self.assertRaisesRegex(ValueError,'differs from authenticated'):h.project(adapter,bad,{})
   for unrelated,verdict in [('unknown','unknown'),('not_met','reject')]:
    row['assessment']['findings']['unrelated']={'finding':unrelated}
    row['assessment']['decision']=verdict
    out={};h.project(adapter,request,out)
    self.assertEqual(out['criterion_returns']['current']['assessment']['findings']['unrelated']['finding'],unrelated)
    bad=copy.deepcopy(request);bad['coding']['relational']['inputs']['current_success']['basis']['criteria'].append('unrelated')
    with self.assertRaises(ValueError):h.project(adapter,bad,{})
   del row['assessment']['findings']['unrelated'];row['assessment']['decision']='accept'
   for finding in ['not_met','unknown']:
    row['assessment']['findings']['x']['finding']=finding
    with self.assertRaises(ValueError):h.project(adapter,request,{})
   row['assessment']['findings']['x']['finding']='met';current['history']=[]
   with self.assertRaisesRegex(ValueError,'absent from authenticated'):h.project(adapter,request,{})
 def test_profile_and_prerequisite_fail_closed(self):
  p={'version':'public-criterion-return@v1','policy':'a'*64,'policy_sha256':'b'*64,'imported_returns':[]};h.validate(p)
  with self.assertRaises(ValueError):h.validate({**p,'policy':'short'})
  frame={'prerequisites':{'x':{'kind':'static_semantic'}},'coverage':{'x':['index.html']}}
  f={'finding':'unknown','basis':'unknown','evidence':['index.html'],'explanation':'Authored','limitations':'Fixture'}
  self.assertEqual(h.native_findings({'findings':{'x':f}},frame)['x']['basis'],{'kind':'unknown'})
  with self.assertRaises(ValueError):h.native_findings({'findings':{'x':{**f,'basis':'observed_test_receipt'}}},frame)
  with self.assertRaises(ValueError):h.native_findings({'findings':{'x':{**f,'evidence':['protected']}}},frame)

@unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1','explicit existing isolated browser fixture')
class OrdinaryCriterion(BrowserAdapter):
 def configure_criteria(self,success=False):
  b=self.c.budget();life=self.c.cli('project-lifecycle-view')
  self.c.json_cli(('project-resource-extend',),{'state_root':self.c.request()['state_root'],'previous':life['admission'],'limits':{'model_calls':20,'model_tokens':2000000,'wall_ms':1800000,'patch_bytes':1048576,'external_cost_microusd':0},'tool_ms_ceiling':120000,'reason':'Authored complete anticipation and criterion-return fixture, including measured native replay latency and validator runway; no live calls'},'--request')
  wants={'orientation':'Keep navigation dependable. Preserve contributor history.','discovery':'Make unfamiliar perspectives accessible'} if not success else {'orientation':'Make keyboard movement usable. Preserve contributor history.','discovery':'Show understandable control feedback'}
  self.c.json_cli(('project','acceptance','requirements'),wants)
  status=self.c.cli('project','acceptance','status')
  self.c.json_cli(('project','acceptance','feedback'),{'requirements_revision':status['requirements_revision'],'decision_record':status['decision_record'],'guidance':wants})
  policy={'state_root':self.c.request()['state_root'],'requirements_revision':status['requirements_revision'],'requirements':wants,'coverage':{k:['app.js','index.html'] for k in wants},'prerequisites':{k:{'kind':'static_semantic'} for k in wants}}
  policy['assessment_scope']={'orientation':[{'domain':'artifact','text':wants['orientation'].split('Preserve')[0]},{'domain':'engine_invariant','text':'Preserve contributor history.'}],'discovery':[{'domain':'artifact','text':wants['discovery']}]}
  pid=self.c.json_cli(('project','criterion-policy'),policy)
  q=self.c.request();i=q['coding']['relational']['inputs'];self.a['origin']['public'].update(requirements_revision=i['requirements_revision'],parents=i['parents'])
  profile={'version':'public-criterion-return@v2','policy':pid,'policy_sha256':m.e.sha(policy),'imported_returns':[]}
  self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=8,max_executions=1,correction_allowance=1,feedback_cycle={'version':'authorized-feedback-cycle@v2','authorization_id':'authored-full-criterion-cycle','max_continuations':6},criterion_return_profile=profile)
  a=self.adapter();self.adapter_instance=a
  ca={'version':'local-project-cycle@v3','journal':str(self.root/'criterion-cycle'),'scope':a.native('request',None)['scope'],'max_decisions':8,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','feedback_cycle':self.a['feedback_cycle']}
  return a,ca
 def test_ordinary_return_to_preference_composition_and_restart(self):
  self.run_chain(False)
 def test_lost_native_apply_recovers_from_exact_commit_without_redispatch(self):
  self.run_chain(True)
 def test_success_return_to_native_preference_and_later_composition(self):
  self.run_chain(False,True)
 def test_success_lost_admission_recovers_without_duplicate(self):
  self.run_chain(False,True,True)
 def test_success_chain_with_transport_faults_and_three_later_conditions(self):
  self.run_chain(False,True,recovery=True)
 def run_chain(self,interrupt,success=False,lost_admission=False,recovery=False):
  self.playable=True;a,ca=self.configure_criteria(success);original=self.transport
  def transport(body,timeout):
   result=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'prerequisites' in data:
    self.assertEqual(data['criteria']['orientation'],'Make keyboard movement usable. ' if success else 'Keep navigation dependable. ')
    # The counted request is already pending here, so native criterion-context
    # correctly withholds a fresh Frame. Compare the wire to the exact native
    # capture retained BEFORE dispatch, not a new query during the effect.
    captures=[v for k,v in a.read()[2].items() if k.startswith('criterion_') and k.endswith('_input') and 'payload' in v]
    self.assertEqual(len(captures),1)
    self.assertEqual(data['published_context'],captures[0]['payload']['published_context'])
    self.assertEqual(data['published_context']['provenance']['publication'],captures[0]['frame']['publication'])
    self.assertEqual(data['published_context']['guidance']['orientation'],('Make keyboard movement usable. ' if success else 'Keep navigation dependable. ')+'Preserve contributor history.')
    self.assertEqual(data['assessment_scope']['engine_invariants_not_assessed']['orientation'],['Preserve contributor history.'])
    # Independent authored interpretation of actual current source, not a
    # directly inserted Contrast or acceptance rejection.
    self.assertNotIn('browser-harness.cjs',data['evidence'])
    aliases=data['reference_catalog']['aliases'];expanded={aliases[k]:v for k,v in data['evidence'].items()}
    current=self.c.request()['coding'];files={f['path']:f['source'] for f in current['files'] if f['editable']}
    self.assertEqual(expanded['app.js'],files['app.js']);self.assertEqual(expanded['index.html'],files['index.html'])
    receipt=json.loads(expanded['test_receipt'])['observation'];self.assertEqual(receipt['snapshot'],current['snapshot']);self.assertEqual(receipt['receipt'],self.c.cli('project','acceptance','status')['test_receipt'])
    result['value']={'verdict':'accept' if success else 'reject','findings':{k:{'finding':'met' if success else 'not_met','basis':'static_source','evidence':data['coverage'][k],'explanation':('Authored assessment: the current source implements keyboard movement and a visible control readout, consistent with the actual protected browser receipt.' if success else 'Authored assessment: this control-only fixture has no route navigation or unfamiliar perspectives, as its current source shows.'),'limitations':'Tests confirm controls only; no claim about actual user fulfillment.'} for k in data['criteria']}}
   elif 'criteria' in data and 'interpretation' in data.get('response_schema',{}).get('properties',{}):
    result['value']={}
    result['value']['interpretation']={'conclusion':'supported','observed':'The current authentic fixture receipt reports responding controls','revised_expectation':None,'uncertainty':'No player experience or fulfillment is established','contrast':('Control behavior is returned; its scoped usability and feedback criteria have their own independent assessment' if success else 'Control behavior is returned; the original route/discovery wants need their own assessment'),'possibilities':['Consider the independent original-criterion assessment before choosing']}
   elif 'criteria' not in data and data.get('criterion_returns',{}).get('current'):
    if not data.get('generated_preferences'):
     if not data.get('relational_history'):
      result['value']={'alternatives':[{'relational':{'parents':['p0','p1'],'relationship':('Authored hypothesis: usable movement and clear feedback together make deliberate practice possible' if success else 'Authored hypothesis: dependable orientation could support discovery beyond controls'),'aspiration':('Cultivate deliberate exploration of control combinations with understandable feedback' if success else 'Make unfamiliar routes approachable with recoverable orientation'),'discriminator':('Observe whether deliberate combinations retain understandable control feedback' if success else 'Observe both recoverable routes and unfamiliar choices'),'revises':None,'catalysts':[],'development':None}}]}
     else:result['value']={'alternatives':[{'success_preference':{'aspiration':'r0','success':'s0'}} if success else {'contrast_preference':{'aspiration':'r0','contrast':'c0'}}]}
    elif len(data.get('relational_history',[]))==1:
     refs=data['proposal_references']['parents']
     result['value']={'alternatives':[{'relational':{'parents':list(refs),'relationship':'Authored later composition includes the new native contributor with its original contributors','aspiration':('Make deliberate control exploration understandable without replacing basic movement' if success else 'Keep the return route understandable while considering unfamiliar branches'),'discriminator':'Check whether the generated contributor changes candidate route organization','revises':None,'catalysts':[],'development':None}}]}
    else:result['value']={'retain':'Authored completion of the integration exercise; no claim of fulfillment or autonomous live learning.'}
   elif 'criteria' not in data and not interrupt:
    result['value']['alternatives'][0]['expectation']={'parents':[p['id'] for p in data['parents']],'anticipated':'The fixture controls will respond to clicks and keys','grounds':'The authored source adds these event handlers','uncertainty':'This predicts controls, not route discovery or player enjoyment'}
   for item in result['value'].get('alternatives',[]):
    if 'relational' in item and data.get('criterion_returns',{}).get('current'):
     item['return_basis']={'observed':[x['id'] for x in data['return_evidence']['cases']][:8],'inferred':[],'unknown':['Authored fixture observations do not establish player fulfillment.']}
   return result
  a.injected_transport=transport
  if lost_admission:
   native=a.native
   def interrupted_advance(kind,value):
    result=native(kind,value)
    if kind=='advance' and value['response']['proposals'][0].get('kind')=='success_preference':raise RuntimeError('authored lost success admission response')
    return result
   with self.assertRaisesRegex(RuntimeError,'lost success admission'):m.loop.Cycle(ca['journal'],ca,interrupted_advance,a.proposal,a.review).run()
   a=self.adapter();self.adapter_instance=a;a.injected_transport=transport
  if interrupt:
   original_cli=self.c.json_cli
   def interrupted_cli(command,value,*args):
    result=original_cli(command,value,*args)
    if command==('project','criterion-apply'):raise RuntimeError('authored lost native criterion result')
    return result
   self.c.json_cli=interrupted_cli
   with self.assertRaisesRegex(RuntimeError,'authored lost native criterion result'):m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
   self.c.json_cli=original_cli
   a=self.adapter();self.adapter_instance=a;a.injected_transport=transport
   stop=a.root/'STOP';stop.write_text('authored Stop')
   with self.assertRaises(ValueError):a.recover_session_transition()
   stop.unlink() # explicit fixture resume, never a user Stop
   original_budget=self.c.budget
   self.c.budget=lambda:{**original_budget(),'unresolved_tool_requests':1}
   with self.assertRaisesRegex(ValueError,'externally unresolved'):a.recover_session_transition()
   self.c.budget=original_budget
   original_status=self.ledger.status
   self.ledger.status=lambda:{**original_status(),'blocking_unknown_executions':['authored unknown']}
   with self.assertRaisesRegex(ValueError,'externally unresolved'):a.recover_session_transition()
   self.ledger.status=original_status
   original_context=self.c.cli
   def missing_commit(*args):
    value=original_context(*args)
    if args==('project','criterion-context'):return {**value,'current':None}
    return value
   before_recovery=a.read();before_accounting=(self.root/'campaign.jsonl').read_bytes()
   self.c.cli=missing_commit
   with self.assertRaisesRegex(ValueError,'not proven committed'):a.recover_session_transition()
   self.c.cli=original_context
   self.assertEqual(a.read(),before_recovery);self.assertEqual((self.root/'campaign.jsonl').read_bytes(),before_accounting)
   self.assertEqual(a.recover_session_transition(),0) # committed receipt consumed; nothing left pending
  if recovery:
   import threading,time
   faults=[];observed=[];halt=threading.Event()
   def monitor():
    while not halt.wait(.05):
     try:observed.append(self.ledger.status()['calls'])
     except ValueError as error:
      if 'snapshot' not in str(error):raise
   thread=threading.Thread(target=monitor);thread.start()
   def configure_faults(adapter):
    original_op=adapter.op
    def op(name,fn,**kw):
     records=adapter.read()[2];requests=[v for k,v in records.items() if k.startswith('request_body_')]
     if name.startswith('dispatch_') and requests[-1].get('kind')=='review' and not faults:
      faults.append('review_predispatch');raise RuntimeError('authored combined fault')
     return original_op(name,fn,**kw)
    adapter.op=op
    def hook(phase,name):
     if phase=='after_result' and name.startswith('settle_') and faults==['review_predispatch','proof_input']:
      faults.append('settled_review');raise RuntimeError('authored combined fault')
    adapter.hook=hook
   try:
    configure_faults(a)
    while True:
     try:
      result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run();break
     except RuntimeError as error:
      if str(error)!='authored combined fault':raise
      a=self.adapter();self.adapter_instance=a;a.injected_transport=transport;a.launch_lock_owned=True
      if faults==['review_predispatch']:
       def proof_hook(phase,name):
        if phase=='after_intent' and name.startswith('predispatch_input_'):
         faults.append('proof_input');raise RuntimeError('authored proof input fault')
       a.hook=proof_hook
       with self.assertRaisesRegex(RuntimeError,'proof input'):a.recover_predispatch()
       a=self.adapter();self.adapter_instance=a;a.injected_transport=transport;a.launch_lock_owned=True
      a.recover_settled_response();a.recover_predispatch();configure_faults(a)
   finally:halt.set();thread.join()
   self.assertEqual(faults,['review_predispatch','proof_input','settled_review']);self.assertTrue(observed)
  else:result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  if result['status']!='proposal_retention':
   import shutil,time
   retained=m.e.ROOT/'target'/('criterion-context-failed-chain-'+str(time.time_ns()))
   shutil.copytree(self.root,retained)
   self.fail('Unexpected authored cycle terminal '+json.dumps(result)+'; retained '+str(retained))
  q=self.c.request();inputs=q['coding']['relational']['inputs'];nodes=inputs['generated_preferences']
  self.assertEqual(len(nodes),1);self.assertTrue(nodes[0]['eligible']);self.assertEqual(nodes[0]['fulfillment'],'unknown')
  self.assertEqual(len(q['coding']['relational']['history']),2)
  self.assertIn(nodes[0]['id'],q['coding']['relational']['history'][1]['proposal']['parents'])
  criteria=self.c.cli('project','criterion-context');self.assertEqual(len(criteria['history']),1)
  self.assertEqual(criteria['current']['assessment']['frame']['assessment_scope'],criteria['frame']['assessment_scope'])
  self.assertEqual(criteria['history'][0]['assessment']['frame']['assessment_scope'],criteria['frame']['assessment_scope'])
  self.assertEqual(criteria['frame']['assessment_scope']['orientation'][1],{'domain':'engine_invariant','text':'Preserve contributor history.'})
  if success:
   self.assertIsNone(inputs.get('current_contrast'));self.assertEqual(inputs['current_success']['basis']['assessment'],criteria['current']['id'])
   self.assertEqual(nodes[0]['success'],inputs['current_success']['id']);self.assertNotIn('contrast',nodes[0])
   self.assertTrue(any('success_preference' in x.get('response_contracts',{}) for x in self.calls))
  else:self.assertEqual(inputs['current_contrast']['review'],criteria['current']['id'])
  self.assertIsNone(self.c.cli('project','acceptance','status')['decision_record'])
  self.assertEqual(self.ledger.status()['calls'],10 if interrupt else 12);self.assertEqual(self.ledger.status()['execution_groups'],1)
  self.assertTrue(any('prerequisites' in x for x in self.calls));self.assertFalse(any('browser-harness.cjs' in str(x.get('evidence',{})) for x in self.calls if 'prerequisites' in x))
  before=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter();fresh.injected_transport=transport
  self.assertIsNone(h.reflect(fresh));self.assertEqual(m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review).run(),result)
  self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
  # A separately admitted session over the SAME native lineage imports exact
  # public records, not an unverified new identity or a reset accounting store.
  origin=copy.deepcopy(self.a['origin']);public=origin['public']
  public.update(parents=inputs['acceptance_parents'],source_snapshots=[q['coding']['snapshot']],public_returns=[m.e.sha(x) for x in inputs.get('returns',[])],public_relationships=[m.e.sha(x) for x in q['coding']['relational']['history']],public_preferences=[m.e.sha(x) for x in nodes])
  origin['initial_sources']={f['path']:f['source'] for f in q['coding']['files'] if f['editable']}
  public['public_pursuits']=[m.e.sha(x) for x in q['coding'].get('pursuit',{}).get('history',[])]
  profile={**self.a['criterion_return_profile'],'imported_returns':[m.e.sha(x) for x in criteria['history']]}
  folder=self.root/'fresh-criterion-session'
  next_admission=m.admission(folder,origin,self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=2,max_executions=0,criterion_return_profile=profile)
  imported=m.Adapter(folder,next_admission,self.c,self.ledger,self.transport)
  _,projected=imported.project(imported.native('request',None));self.assertEqual(projected['criterion_returns']['current'],criteria['current'])
  self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
  altered=copy.deepcopy(criteria);altered['history'][0]['assessment']['frame']['snapshot']='f'*64
  with self.assertRaises(ValueError):h.verify(imported,altered)
  bad=copy.deepcopy(criteria);bad['history'][0]['assessment']['response_digest']='f'*64
  with self.assertRaises(ValueError):h.verify(fresh,bad)
  if recovery:
   # These are authored control-workflow conditions, not live noticeboard results.
   events=[{'id':'authored-'+name,'attribution':'synthetic_harness_verification','guidance':{p['requirement']:p['meaning'] for p in inputs['acceptance_parents']}} for name in ('opening','storm','reopening')]
   feedback={'version':'authorized-feedback-cycle@v2','authorization_id':'authored-three-condition-recovery','max_continuations':3}
   next_admission=m.admission(self.root/'three-condition',origin,self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=3,max_executions=0,feedback_events=events,feedback_cycle=feedback,criterion_return_profile=profile)
   next_adapter=m.Adapter(next_admission['directory'],next_admission,self.c,self.ledger,self.transport)
   def retain_with_child(body,timeout):
    result=self.transport(body,timeout);data=json.loads(body['messages'][1]['content'])
    if 'coverage' in data and 'criteria' in data:
     result['value']={'verdict':'accept','findings':{k:{'finding':'met','basis':'static_source','evidence':data['coverage'][k],'explanation':'Authored control fixture still implements the unchanged scoped controls.','limitations':'No human fulfillment claim.'} for k in data['criteria']}}
    elif 'criteria' not in data:
     self.assertEqual(data['generated_preferences'][0]['id'],nodes[0]['id'])
     result['value']={'retain':'Authored valid retention with the existing native child available; no forced novelty.'}
    return result
   next_adapter.injected_transport=retain_with_child
   next_cycle={**ca,'journal':str(self.root/'three-condition-cycle'),'max_decisions':3,'max_executions':0,'feedback_cycle':feedback}
   session=m.load('authored_complete_session','engine-browser-session.py')
   completed=session.run(next_adapter,next_cycle);self.assertEqual(completed['status'],'completed_checkpoints');self.assertEqual(len(completed['checkpoints']),3)
   before=(self.root/'campaign.jsonl').read_bytes();self.assertEqual(session.run(next_adapter,next_cycle),completed);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())

for name in dir(BrowserAdapter):
 if name.startswith('test_') and name not in OrdinaryCriterion.__dict__:setattr(OrdinaryCriterion,name,None)
