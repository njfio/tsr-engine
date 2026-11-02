"""Authored model transport, actual native meaning admission. No provider calls."""
import copy,json
from pathlib import Path
from scripts.test_engine_browser_adapter import BrowserAdapter,m
helper=m.load('desire_test','engine-browser-desire.py')
class BrowserDesire(BrowserAdapter):
 def configure(self,events=None):
  self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=6,max_executions=1,correction_allowance=1,feedback_events=events or [],feedback_cycle={'version':'authorized-feedback-cycle@v2','authorization_id':'authored-meaning','max_continuations':4})
  a=self.adapter();self.adapter_instance=a
  ca={'version':'local-project-cycle@v3','journal':str(self.root/'meaning-cycle'),'scope':a.native('request',None)['scope'],'max_decisions':6,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','feedback_cycle':self.a['feedback_cycle']}
  return a,ca
 def proposal(self,data):
  return {'alternatives':[{'public_desire':{'identity':'p0','revises':data['parents'][0].get('revision'),'parents':['p0'],'meaning':('Start control is usable, with its purpose understandable before interaction; this authored check does not measure player satisfaction.'+(' Authored capacity context; no observed fulfillment claim.'*20 if getattr(self,'long_meaning',False) else '')),'change':'development','basis':{'kind':'unknown'}}}]}
 def test_native_meaning_review_next_choice_and_restart(self):
  a,ca=self.configure();old=self.c.request();original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:
    if data['parents'][0].get('revision'):
     self.assertIn('Start control',data['acceptance_parents'][0]['meaning']);self.assertTrue(data['public_desire_history'])
     out['value']={'retain':'Authored retained current source after reviewed meaning; not fulfillment.','development_declined':'Authored fixture: development considered, none proposed at this checkpoint'}
    else:out['value']=self.proposal(data)
   return out
  a.injected_transport=transport
  result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention')
  new=self.c.request();i=new['coding']['relational']['inputs'];self.assertEqual(i['parents'][0]['id'],old['coding']['relational']['inputs']['parents'][0]['id'])
  self.assertEqual(old['coding']['snapshot'],new['coding']['snapshot']);self.assertEqual(i['acceptance_parents'],old['coding']['relational']['inputs']['parents'])
  self.assertEqual(len(i['public_desires']['history']),1);self.assertFalse(self.c.cli('project','acceptance','status')['accepted'])
  self.assertEqual((self.ledger.status()['calls'],self.ledger.status()['execution_groups']),(3,0))
  before=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter();self.assertEqual(m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review).run(),result);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
 def test_binding_and_export_negative_controls(self):
  a,ca=self.configure();r,c=a.project(a.native('request',None));refs=m.load('desire_refs','engine-proposal-references.py');wire=refs.project(c,r);raw=self.proposal(wire)
  bound=refs.expand(raw,wire['proposal_references'],r)['alternatives'][0]['public_desire'];helper.bind(r,bound)
  for key,value in [('identity','f'*64),('revises','f'*64),('binding','f'*64),('parents',[]),('basis',{'kind':'observation','receipt':'f'*64}),('generator','invented')]:
   with self.subTest(key=key),self.assertRaises(ValueError):helper.bind(r,{**bound,key:value})
  bad=copy.deepcopy(raw);bad['alternatives'][0]['public_desire']['binding']='f'*64
  with self.assertRaises(ValueError):refs.expand(bad,wire['proposal_references'],r)
  stale=copy.deepcopy(r);stale['state_root']='f'*64
  with self.assertRaises(ValueError):refs.expand(raw,wire['proposal_references'],stale)
 def test_stop_no_dispatch(self):
  a,ca=self.configure();a.root.joinpath('STOP').write_text('authored Stop')
  with self.assertRaises(Exception):m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(self.ledger.status()['calls'],0)
 def test_later_operator_feedback_remains_authoritative(self):
  events=[{'id':'authored-later-guidance','attribution':'synthetic_harness_verification','guidance':{'controls':'Keep the usable Start control and prioritize keyboard access. Authored guidance, not player testimony.'}}]
  a,ca=self.configure(events);original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:out['value']={'retain':'Authored retained','development_declined':'Authored fixture: development considered, none proposed at this checkpoint'} if data['parents'][0].get('revision') else self.proposal(data)
   return out
  a.injected_transport=transport;m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  before=self.c.request()['coding']['relational']['inputs']['public_desires']['history']
  event=a.apply_feedback(0);r,c=a.project(a.native('request',None))
  self.assertEqual(c['acceptance_parents'][0]['meaning'],self.a['feedback_events'][0]['guidance']['controls'])
  self.assertEqual(before,r['coding']['relational']['inputs']['public_desires']['history'])
  self.assertNotEqual(c['parents'][0]['meaning'],c['acceptance_parents'][0]['meaning'])
  self.assertEqual(event['acceptance_parents'],c['acceptance_parents'])
 def test_rejection_does_not_admit_or_resample(self):
  a,ca=self.configure();original=self.transport;generated=0
  def transport(body,timeout):
   nonlocal generated
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' in data:
    out['value']['verdict']='reject';out['value']['findings']['parent_fidelity']['finding']='not_met'
    out['value']['findings']['parent_fidelity']['explanation']='Authored rejection; choose freely next.'
   else:
    generated+=1;out['value']=self.proposal(data) if generated==1 else {'retain':'Authored keep original meaning after critique','development_declined':'Authored fixture: development considered, none proposed at this checkpoint'}
   return out
  a.injected_transport=transport;result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention');self.assertNotIn('public_desires',self.c.request()['coding']['relational']['inputs'])
  self.assertEqual(self.ledger.status()['calls'],3)
 def test_lost_advance_result_recovers_exact_native_receipt(self):
  a,ca=self.configure();original=self.transport;original_native=a.native
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:out['value']={'retain':'Authored retained after recovery','development_declined':'Authored fixture: development considered, none proposed at this checkpoint'} if data['parents'][0].get('revision') else self.proposal(data)
   return out
  a.injected_transport=transport
  def interrupt(kind,value):
   result=original_native(kind,value)
   if kind=='advance':raise RuntimeError('authored lost committed advance response')
   return result
  with self.assertRaisesRegex(RuntimeError,'lost committed'):m.loop.Cycle(ca['journal'],ca,interrupt,a.proposal,a.review).run()
  history=self.c.request()['coding']['relational']['inputs']['public_desires']['history'];self.assertEqual(len(history),1)
  pending_cycle=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review);saved=pending_cycle.read()[2]['meaning_input_0']
  path=self.c.directory/'coding-work/app.js';source=path.read_bytes();path.write_bytes(source+b'\n// authored drift\n')
  try:
   with self.assertRaises(Exception):self.c.json_cli(('project','cycle','meaning-advance-receipt'),saved)
  finally:path.write_bytes(source)
  fresh=self.adapter();fresh.injected_transport=transport
  result=m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review).run()
  self.assertEqual(result['status'],'proposal_retention');self.assertEqual(history,self.c.request()['coding']['relational']['inputs']['public_desires']['history'])
  self.assertEqual(self.ledger.status()['calls'],3)
  # Exact-input recovery is deliberately unavailable once later progression exists.
  cycle=m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review)
  value=cycle.read()[2]['meaning_input_0'];bad=copy.deepcopy(value);bad['binding']='f'*64
  with self.assertRaises(Exception):self.c.json_cli(('project','cycle','meaning-advance-receipt'),bad)
 def test_missing_native_receipt_never_replays(self):
  a,ca=self.configure();original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:out['value']=self.proposal(data)
   return out
  a.injected_transport=transport
  def before(kind,value):
   if kind=='advance':raise RuntimeError('authored interruption before native transaction')
   return a.native(kind,value)
  with self.assertRaises(RuntimeError):m.loop.Cycle(ca['journal'],ca,before,a.proposal,a.review).run()
  fresh=self.adapter();before_root=self.c.request()['state_root']
  with self.assertRaises(Exception):m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review).run()
  self.assertEqual(before_root,self.c.request()['state_root']);self.assertEqual(self.ledger.status()['calls'],2)
 def test_interrupted_capture_recovers_without_second_admission(self):
  a,ca=self.configure();original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:out['value']={'retain':'Authored retained','development_declined':'Authored fixture: development considered, none proposed at this checkpoint'} if data['parents'][0].get('revision') else self.proposal(data)
   return out
  a.injected_transport=transport
  def hook(phase,name):
   if phase=='after_effect' and name.startswith('meaning_admission_'):raise RuntimeError('authored interrupted read-only capture')
  a.hook=hook
  with self.assertRaisesRegex(RuntimeError,'interrupted read-only'):m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertTrue(a.read()[1]);history=self.c.request()['coding']['relational']['inputs']['public_desires']['history']
  fresh=self.adapter();fresh.injected_transport=transport
  self.assertEqual(fresh.recover_session_transition(),1)  # same preflight used by real launcher
  self.assertEqual(fresh.recover_session_transition(),0)
  result=m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review).run()
  self.assertEqual(result['status'],'proposal_retention');self.assertFalse(fresh.read()[1]);self.assertEqual(history,self.c.request()['coding']['relational']['inputs']['public_desires']['history'])
  self.assertEqual(self.ledger.status()['calls'],3)
 def test_partial_feedback_never_promotes_generated_meaning(self):
  a,ca=self.configure();r=self.c.request();i=r['coding']['relational']['inputs'];base=copy.deepcopy(i['parents'])
  second={**base[0],'id':'b'*64,'requirement':'second','meaning':'Authored original second want'}
  i['acceptance_parents']=base+[second];i['parents']=[{**base[0],'meaning':'Authored generated development, not operator text'},second]
  completion=self.c.cli('project','acceptance','status');checkpoint=m.web.checkpoint(r,completion,self.c.budget())
  draft=m.web.feedback_draft(checkpoint,r,completion,'authored-partial',{'second':'Authored new second guidance'},attribution='synthetic_harness_verification')
  self.assertEqual(draft['native_feedback']['guidance'][base[0]['requirement']],base[0]['meaning'])
  self.assertEqual(draft['native_feedback']['guidance']['second'],'Authored new second guidance')
 def test_ordinary_does_not_offer_meaning_actions(self):
  a,ca=self.configure();ordinary=m.load('meaning_ordinary','engine-browser-ordinary.py');o=ordinary.Adapter(a.root,a.admission,self.c,self.ledger,self.transport)
  r,c=o.project(o.native('request',None));self.assertNotIn('public_desire',c['response_contracts']);self.assertIn('acceptance_parents',c)
  with self.assertRaises(ValueError):o.decode_proposal({'alternatives':[{'public_desire':{}}]},r,c,o.native('request',None))
 def test_successive_development_preserves_identity_and_superseded_history(self):
  a,ca=self.configure();original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:
    history=data['public_desire_history']
    if len(history)>=2:out['value']={'retain':'Authored retain after second development','development_declined':'Authored fixture: development considered, none proposed at this checkpoint'}
    else:
     out['value']=self.proposal(data)
     if history:out['value']['alternatives'][0]['public_desire']['meaning']='Start control remains usable and understandable before interaction, with clear keyboard affordance; this authored check does not measure player satisfaction.'
   return out
  a.injected_transport=transport;result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention');inputs=self.c.request()['coding']['relational']['inputs'];history=inputs['public_desires']['history']
  self.assertEqual(len(history),2);self.assertTrue(history[0]['superseded']);self.assertFalse(history[1]['superseded'])
  self.assertEqual(history[0]['next']['id'],history[1]['next']['id']);self.assertEqual(history[0]['next']['parents'],history[1]['next']['parents'])
  self.assertEqual(inputs['parents'][0]['original_meaning'],inputs['acceptance_parents'][0]['meaning'])
  self.assertEqual(self.ledger.status()['calls'],5)
 def test_meaning_then_ordinary_browser_action_and_return(self):
  import os
  if os.getenv('TSR_BROWSER_SEALED_AUTHORED')!='1':self.skipTest('existing sealed authored browser opt-in')
  a,ca=self.configure();original=self.transport;self.playable=True
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:
    if not data['parents'][0].get('revision'):out['value']=self.proposal(data)
    elif data['browser_returns']:out['value']={'retain':'Authored observed protected browser return; no claim of player satisfaction','development_declined':'Authored fixture: development considered, none proposed at this checkpoint'}
   return out
  a.injected_transport=transport;result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention');self.assertEqual(self.ledger.status()['execution_groups'],1)
  status=self.c.cli('project','status');self.assertEqual(status['execution_status'],'passed',json.dumps(status))
  i=self.c.request()['coding']['relational']['inputs'];self.assertEqual(len(i['public_desires']['history']),1);self.assertTrue(i['returns'])
  self.assertFalse(self.c.cli('project','acceptance','status')['accepted'])
 def test_browser_meaning_over_512_survives_review_and_next_choice(self):
  self.long_meaning=True
  self.test_native_meaning_review_next_choice_and_restart()
  meaning=self.c.request()['coding']['relational']['inputs']['parents'][0]['meaning']
  self.assertGreater(len(meaning.encode()),512);self.assertTrue(meaning.endswith(' Authored capacity context; no observed fulfillment claim.'))
 def test_full_history_projection_and_aggregate_capacity(self):
  a,ca=self.configure();r,c=a.project(a.native('request',None));refs=m.load('capacity_meaning_refs','engine-proposal-references.py');wire=refs.project(c,r)
  bound=refs.expand(self.proposal(wire),wire['proposal_references'],r)['alternatives'][0]['public_desire']
  with self.assertRaises(ValueError):helper.bind(r,{**bound,'meaning':'x'*131073})
  with self.assertRaises(ValueError):m.web.bind_edit(r,{'public_desire':{**bound,'meaning':'x'*131071}},lambda _:None)
  # Authored projection inventory, not 65 real/native admissions or live evidence.
  inputs=r['coding']['relational']['inputs'];inputs['acceptance_parents']=copy.deepcopy(inputs['parents'])
  history=[{'revision':str(n),'previous':{},'next':{'meaning':'Authored historical meaning '+str(n)},'superseded':n<64,'withdrawn':False} for n in range(65)]
  inputs['public_desires']={'active':inputs['parents'],'history':history}
  origin=copy.deepcopy(self.a['origin']['public']);origin['public_meanings']=[m.e.sha(h) for h in history]
  projected=m.web.project(r,origin);self.assertEqual(projected['public_desire_history'],history)
  with self.assertRaisesRegex(ValueError,'unowned'):helper.verify_history(a,r)
 def test_fresh_admission_import_revision_and_restart(self):
  self.test_native_meaning_review_next_choice_and_restart()
  request=self.c.request();inputs=request['coding']['relational']['inputs'];origin=copy.deepcopy(self.a['origin'])
  origin['public']['parents']=inputs['acceptance_parents'];origin['public']['public_meanings']=[m.e.sha(x) for x in inputs['public_desires']['history']]
  self.a=m.admission(self.root/'successor',origin,self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=3,max_executions=1,correction_allowance=1,feedback_cycle={'version':'authorized-feedback-cycle@v2','authorization_id':'authored-import','max_continuations':2})
  a=m.Adapter(self.root/'successor',self.a,self.c,self.ledger,self.transport);original=self.transport
  r,c=a.project(a.native('request',None));self.assertEqual(c['parents'],inputs['parents'])
  bad=copy.deepcopy(r);bad['coding']['relational']['inputs']['public_desires']['history'][0]['next']['meaning']='Unowned substitute'
  with self.assertRaisesRegex(ValueError,'unowned'):helper.verify_history(a,bad)
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:
    if len(data['public_desire_history'])>1:out['value']={'retain':'Authored successor retention','development_declined':'Authored fixture: development considered, none proposed at this checkpoint'}
    else:
     out['value']=self.proposal(data);out['value']['alternatives'][0]['public_desire']['meaning']='Start control stays usable and understandable, with an explicit keyboard affordance; authored hypothesis, not player satisfaction.'
   return out
  a.injected_transport=transport
  ca={'version':'local-project-cycle@v3','journal':str(self.root/'successor-cycle'),'scope':a.native('request',None)['scope'],'max_decisions':3,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','feedback_cycle':self.a['feedback_cycle']}
  result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run();self.assertEqual(result['status'],'proposal_retention')
  self.assertEqual(len(self.c.request()['coding']['relational']['inputs']['public_desires']['history']),2)
  fresh=m.Adapter(self.root/'successor',self.a,self.c,self.ledger,self.transport)
  before=(self.root/'campaign.jsonl').read_bytes();self.assertEqual(m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review).run(),result);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
