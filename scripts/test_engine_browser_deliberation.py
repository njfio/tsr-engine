"""Authored transport exercises separate components; no provider calls."""
import copy,json,unittest,os
from pathlib import Path
from scripts.test_engine_browser_desire import BrowserDesire
from scripts.test_engine_browser_adapter import m
class BrowserDeliberation(BrowserDesire):
 def transport_for(self,a,dependency='independent',reject=False):
  original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:
    out['value']={'deliberation':{'artifact':{'retain':'Authored keep current artifact while considering wanting'},'development':{'provisional_preference':{'parents':['p0'],'meaning':'Authored wanting for recoverable orientation','rationale':'Uncertain generated possibility, not player testimony or success.'}},'artifact_dependency':dependency}}
   elif reject:
    out['value']['verdict']='reject';out['value']['findings']['parent_fidelity']['finding']='not_met'
    out['value']['findings']['parent_fidelity']['explanation']='Authored development rejection; no semantic approval shopping.'
   return out
  a.injected_transport=transport
 def test_retain_artifact_with_admitted_wanting_and_inert_restart(self):
  a,ca=self.configure();self.transport_for(a);before=self.c.request()['coding']['snapshot']
  result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention');self.assertTrue(result['development']['advance']['preference_admitted'])
  self.assertEqual(self.c.request()['coding']['snapshot'],before);self.assertEqual(self.ledger.status()['calls'],2)
  ledger=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter()
  self.assertEqual(m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review).run(),result);self.assertEqual((self.root/'campaign.jsonl').read_bytes(),ledger)
 def test_rejected_development_does_not_authorize_dependent_artifact(self):
  a,ca=self.configure();self.transport_for(a,'requires_development',True)
  result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'deliberation_dependency_unmet');self.assertFalse(result['artifact_executed'])
  self.assertNotIn('generated_preferences',self.c.request()['coding']['relational']['inputs']);self.assertEqual(self.ledger.status()['calls'],2)
 def test_no_development_retains_without_invented_identity(self):
  a,ca=self.configure();original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);out['value']={'deliberation':{'artifact':{'retain':'Authored no new meaning'},'development':None,'artifact_dependency':'independent'}};return out
  a.injected_transport=transport;result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention');self.assertEqual(self.ledger.status()['calls'],1)
 def test_same_identity_component_preserves_exact_admitted_delta(self):
  a,ca=self.configure();original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:out['value']={'deliberation':{'artifact':{'retain':'Authored unchanged artifact'},'development':self.proposal(data)['alternatives'][0],'artifact_dependency':'independent'}}
   return out
  a.injected_transport=transport;result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertTrue(result['development']['advance']['meaning_admitted']);self.assertEqual(self.ledger.status()['calls'],2)
 def test_independent_artifact_retention_survives_development_rejection(self):
  a,ca=self.configure();self.transport_for(a,reject=True)
  result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention');self.assertEqual(result['development']['judgment']['decision'],'reject')
  self.assertFalse(result['development']['advance']);self.assertEqual(self.ledger.status()['execution_groups'],0)
 def test_local_record_interruption_stop_and_restart_no_duplicate(self):
  from pathlib import Path
  a,ca=self.configure();self.transport_for(a)
  def hook(phase,name):
   if phase=='after_intent' and name=='compound_0_development_derive_input':raise RuntimeError('authored local interruption')
  with self.assertRaisesRegex(RuntimeError,'authored local'):m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review,hook=hook).run()
  before=self.ledger.status()['calls'];Path(ca['journal'],'STOP').write_text('authored Stop')
  with self.assertRaises(Exception):m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(self.ledger.status()['calls'],before);Path(ca['journal'],'STOP').unlink()
  result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertTrue(result['development']['advance']['preference_admitted']);self.assertEqual(self.ledger.status()['calls'],2)
 def test_lost_native_component_result_recovers_without_second_admission(self):
  a,ca=self.configure();self.transport_for(a);native=a.native
  def interrupt(kind,value):
   result=native(kind,value)
   if kind=='advance':raise RuntimeError('authored committed component response lost')
   return result
  with self.assertRaisesRegex(RuntimeError,'component response lost'):m.loop.Cycle(ca['journal'],ca,interrupt,a.proposal,a.review).run()
  result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertTrue(result['development']['advance']['preference_admitted']);self.assertEqual(self.ledger.status()['calls'],2)
 def test_artifact_review_rejection_preserves_admitted_development(self):
  a,ca=self.configure();original=self.transport;proposed=False;reviews=0
  def transport(body,timeout):
   nonlocal proposed,reviews
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:
    if proposed:out['value']={'retain':'Authored fresh choice after artifact rejection'}
    else:
     proposed=True;out['value']={'deliberation':{'artifact':out['value']['alternatives'][0],'development':{'provisional_preference':{'parents':['p0'],'meaning':'Authored future orientation','rationale':'Uncertain possibility without success claim'}},'artifact_dependency':'independent'}}
   else:
    reviews+=1
    if reviews!=2:return out
    out['value']['verdict']='reject';field=next(iter(out['value']['findings']));out['value']['findings'][field]['finding']='not_met'
   return out
  a.injected_transport=transport;result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention');self.assertEqual(self.ledger.status()['execution_groups'],0)
  self.assertEqual(len(self.c.request()['coding']['relational']['inputs']['generated_preferences']),1)
  self.assertEqual(self.ledger.status()['calls'],4)
 def test_exact_delta_guard_rejects_second_revision_and_source_drift(self):
  helper=m.load('compound_guard_test','engine-browser-deliberation.py')
  before={'snapshot':'s','requirements_revision':'r','files':[],'acceptance_parents':[{'id':'p','meaning':'original'}],'parents':[{'id':'p','meaning':'original'}],'history':[],'guidance':[]}
  after=copy.deepcopy(before);after['parents'][0]['meaning']='admitted'
  development={'proposals':[{'kind':'public_desire','proposal':{'identity':'p'}}]}
  capture={'public':{'active':copy.deepcopy(after['parents'])},'advance_result':{}}
  helper.unchanged(before,after,development,capture)
  later=copy.deepcopy(after);later['parents'][0]['meaning']='different later admitted revision'
  with self.assertRaisesRegex(ValueError,'exact admitted'):helper.unchanged(before,later,development,capture)
  for field in ('snapshot','requirements_revision','files','acceptance_parents'):
   bad=copy.deepcopy(after);bad[field]='forged'
   with self.subTest(field=field),self.assertRaises(ValueError):helper.unchanged(before,bad,development,capture)
 def two_roots(self):
  directory=self.root/'two-root-native';directory.mkdir();self.c=m.d.Native(directory)
  requirements=self.root/'two-parents.json';requirements.write_text(json.dumps({'controls':'Usable controls','clarity':'Understandable authored status'}))
  self.c.cli('project','init','--spec',str(self.root/'scope.json'),'--requirements',str(requirements))
  self.c.json_cli(('project-lifecycle-open',),{'state_root':self.c.request()['state_root'],'training':None,'tool_ms_ceiling':120000,'reason':'Authored continuous deliberation','non_tool':{'model_calls':20,'model_tokens':2000000,'wall_ms':600000,'patch_bytes':49152,'external_cost_microusd':0}},'--admission')
  completion=self.c.cli('project','acceptance','status');self.c.json_cli(('project','acceptance','feedback'),{'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],'guidance':json.loads(requirements.read_text())})
  self.c.cli('project','resource-unpriced-admit','--state-root',self.c.request()['state_root'])
  request=self.c.request();coding=request['coding'];self.a['origin']['public'].update(scope=request['next_attention']['scope'],requirements_revision=coding['relational']['inputs']['requirements_revision'],parents=coding['relational']['inputs']['parents'],source_snapshots=[coding['snapshot']]);self.a['origin']['directory']=str(directory)
 def relation(self,data):
  return {'relational':{'parents':data['relational_contract']['parent_ids'],'relationship':'Authored controls and clear status may support one another','aspiration':'An understandable interactive fixture','discriminator':'Actual click and key observations can expose ambiguity','revises':None,'catalysts':[]}}
 def test_relationship_capture_interruption_exact_recovery(self):
  self.two_roots();a,ca=self.configure();original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:out['value']={'deliberation':{'artifact':{'retain':'Authored keep artifact'},'development':self.relation(data),'artifact_dependency':'independent'}}
   return out
  a.injected_transport=transport
  def hook(phase,name):
   if phase=='after_intent' and name.startswith('qualitative_'):raise RuntimeError('authored local relationship capture interruption')
  a.hook=hook
  with self.assertRaisesRegex(RuntimeError,'capture interruption'):m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  a.hook=lambda *_:None
  result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertTrue(result['development']['advance']['qualitative_admitted']);self.assertEqual(len(self.c.request()['coding']['relational']['history']),1);self.assertEqual(self.ledger.status()['calls'],2)
 def test_artifact_reviewer_abstention_terminal_and_restart(self):
  a,ca=self.configure();original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:out['value']={'deliberation':{'artifact':out['value']['alternatives'][0],'development':None,'artifact_dependency':'independent'}}
   else:
    out['value']['verdict']='abstain'
    for finding in out['value']['findings'].values():finding['finding']='unknown'
   return out
  a.injected_transport=transport
  def hook(phase,name):
   if phase=='after_result' and name=='compound_0_artifact_outcome':raise RuntimeError('authored settled abstention interruption')
  with self.assertRaisesRegex(RuntimeError,'settled abstention'):m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review,hook=hook).run()
  calls=self.ledger.status()['calls'];result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run();self.assertEqual(self.ledger.status()['calls'],calls)
  self.assertEqual(result['status'],'review_abstain');before=(self.root/'campaign.jsonl').read_bytes()
  self.assertEqual(m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run(),result);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes());self.assertEqual(self.ledger.status()['calls'],2)
 @unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1','existing isolated browser runtime required')
 def test_edit_plus_development_actual_execution_and_return(self):
  self.playable=True;a,ca=self.configure();original=self.transport;generated=0
  def transport(body,timeout):
   nonlocal generated
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:
    generated+=1
    if generated==1:out['value']={'deliberation':{'artifact':out['value']['alternatives'][0],'development':{'provisional_preference':{'parents':['p0'],'meaning':'Authored recoverable orientation','rationale':'Uncertain generated wanting, no empirical claim'}},'artifact_dependency':'independent'}}
    else:
     self.assertTrue(data['browser_returns']);out['value']={'retain':'Authored next choice consumes the returned control observations; not player fulfillment'}
   return out
  a.injected_transport=transport;result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention');self.assertEqual(self.c.cli('project','status')['execution_status'],'passed');self.assertEqual(self.ledger.status()['execution_groups'],1);self.assertEqual(self.ledger.status()['calls'],4)
  before=(self.root/'campaign.jsonl').read_bytes();self.assertEqual(m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run(),result);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
 @unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1','existing isolated browser runtime required')
 def test_independent_edit_after_development_review_abstention(self):
  self.playable=True;a,ca=self.configure();original=self.transport;generated=0;reviews=0
  before=self.c.request()['coding']['snapshot']
  def transport(body,timeout):
   nonlocal generated,reviews
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:
    generated+=1
    if generated==1:out['value']={'deliberation':{'artifact':out['value']['alternatives'][0],'development':{'provisional_preference':{'parents':['p0'],'meaning':'Authored recoverable orientation','rationale':'Uncertain generated wanting, not observed fulfillment'}},'artifact_dependency':'independent'}}
    else:
     self.assertTrue(data['browser_returns']);out['value']={'retain':'Authored stop after independently reviewed artifact return'}
   else:
    reviews+=1
    if reviews==1:
     out['value']['verdict']='abstain'
     for finding in out['value']['findings'].values():finding['finding']='unknown'
   return out
  a.injected_transport=transport;cycle=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review);result=cycle.run()
  done=cycle.read()[2];development=done['compound_0_development_outcome'];artifact=done['compound_0_artifact_outcome']
  self.assertEqual(development['judgment']['decision'],'abstain');self.assertIsNone(development['advance'])
  self.assertEqual(artifact['judgment']['decision'],'approve');self.assertIsNotNone(artifact['advance'])
  self.assertNotEqual(development['judgment']['reviewer'],artifact['judgment']['reviewer'])
  self.assertFalse(self.c.request()['coding']['relational']['inputs'].get('generated_preferences'))
  status=self.c.cli('project','status');self.assertEqual(status['execution_status'],'passed',json.dumps(status))
  self.assertNotEqual(self.c.request()['coding']['snapshot'],before);self.assertFalse(self.c.cli('project','acceptance','status')['accepted'])
  self.assertEqual(result['status'],'proposal_retention')
  self.assertEqual((self.ledger.status()['calls'],self.ledger.status()['execution_groups']),(4,1));self.assertEqual(reviews,2)
  saved=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter()
  self.assertEqual(m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review).run(),result)
  self.assertEqual((self.root/'campaign.jsonl').read_bytes(),saved)
 @unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1','existing isolated browser runtime required')
 def test_new_wanting_in_dedicated_reconsideration_after_real_return(self):
  self.two_roots();self.playable=True;a,ca=self.configure();original=self.transport;considered=[]
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' in data:return out
   if not data['relational_history']:out['value']={'deliberation':{'artifact':out['value']['alternatives'][0],'development':self.relation(data),'artifact_dependency':'independent'}}
   elif 'advisory_reconsideration' in data:
    considered.append(data);self.assertIn('provisional_preference',data['choice_summary']['forms']);self.assertNotIn('edit',data['choice_summary']['forms'])
    out['value']={'deliberation':{'artifact':{'retain':'Authored unchanged tested artifact'},'development':{'provisional_preference':{'parents':['p0'],'meaning':'Authored recoverable orientation after bounded observations','rationale':'An uncertain possibility; observed controls do not establish enjoyment'}},'artifact_dependency':'independent'}}
   else:out['value']={'retain':'Authored no more changes'}
   return out
  a.injected_transport=transport;result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertTrue(considered);self.assertEqual(result['status'],'proposal_retention');self.assertTrue(result['development']['advance']['preference_admitted']);self.assertEqual(self.c.cli('project','status')['execution_status'],'passed');self.assertEqual(self.ledger.status()['execution_groups'],1)
 @unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1','existing isolated browser runtime required')
 def test_standalone_reconsideration_committed_result_recovers_marker(self):
  self.two_roots();self.playable=True;a,ca=self.configure();original=self.transport;considered=[]
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' in data:return out
   if not data['relational_history']:out['value']={'deliberation':{'artifact':out['value']['alternatives'][0],'development':self.relation(data),'artifact_dependency':'independent'}}
   elif 'advisory_reconsideration' in data:
    considered.append(data);self.assertIn('provisional_preference',data['choice_summary']['forms']);self.assertNotIn('edit',data['choice_summary']['forms'])
    out['value']={'alternatives':[{'provisional_preference':{'parents':['p0'],'meaning':'Authored recoverable orientation after bounded observations','rationale':'An uncertain possibility; observed controls do not establish enjoyment'}}]}
   else:out['value']={'retain':'Authored no more changes'}
   return out
  a.injected_transport=transport
  def hook(phase,name):
   if phase=='after_result' and name.startswith('reconsider_') and name.endswith('_advance'):raise RuntimeError('authored standalone marker interruption')
  with self.assertRaisesRegex(RuntimeError,'marker interruption'):m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review,hook=hook).run()
  result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertTrue(considered);self.assertEqual(result['status'],'proposal_retention');self.assertEqual(len(self.c.request()['coding']['relational']['inputs']['generated_preferences']),1);self.assertEqual(self.c.cli('project','status')['execution_status'],'passed');self.assertEqual(self.ledger.status()['execution_groups'],1)
 @unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED')=='1','existing isolated browser runtime required')
 def test_new_relationship_in_dedicated_reconsideration_after_real_return(self):
  self.two_roots();self.playable=True;a,ca=self.configure();original=self.transport;considered=[]
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' in data:return out
   if not data['relational_history']:out['value']={'deliberation':{'artifact':out['value']['alternatives'][0],'development':self.relation(data),'artifact_dependency':'independent'}}
   elif 'advisory_reconsideration' in data:
    considered.append(data);self.assertIn('provisional_preference',data['choice_summary']['forms']);self.assertNotIn('edit',data['choice_summary']['forms'])
    development=self.relation(data);development['relational']['aspiration']='Authored new possibility after bounded return, not a prior-lineage revision'
    out['value']={'deliberation':{'artifact':{'retain':'Authored unchanged tested artifact'},'development':development,'artifact_dependency':'independent'}}
   else:out['value']={'retain':'Authored no more changes'}
   return out
  a.injected_transport=transport;result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertTrue(considered);self.assertEqual(result['status'],'proposal_retention');self.assertTrue(result['development']['advance']['qualitative_admitted']);self.assertEqual(len(self.c.request()['coding']['relational']['history']),2);self.assertEqual(self.c.cli('project','status')['execution_status'],'passed');self.assertEqual(self.ledger.status()['execution_groups'],1)
 def test_compound_terminal_intent_recovers_without_model_or_effect(self):
  a,ca=self.configure();original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);out['value']={'deliberation':{'artifact':{'retain':'Authored complete'},'development':None,'artifact_dependency':'independent'}};return out
  a.injected_transport=transport
  def hook(phase,name):
   if phase=='after_intent' and name=='terminal':raise RuntimeError('authored terminal interruption')
  with self.assertRaisesRegex(RuntimeError,'terminal interruption'):m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review,hook=hook).run()
  before=(self.root/'campaign.jsonl').read_bytes();result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention');self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
for name in dir(BrowserDesire):
 if name.startswith('test_') and name not in BrowserDeliberation.__dict__:setattr(BrowserDeliberation,name,None)
