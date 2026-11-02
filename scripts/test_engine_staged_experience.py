"""Authored transport only: frozen conditions are not real user preferences or live TSR evidence."""
import copy,json,os
from pathlib import Path
from scripts.test_engine_browser_adapter import BrowserAdapter,m

class StagedExperience(BrowserAdapter):
 def configure(self,events,policy=True):
  self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=9,max_executions=2,correction_allowance=1,feedback_events=events,feedback_cycle={'version':'authorized-feedback-cycle@v2','authorization_id':'authored-staged-experience','max_continuations':6} if policy else None)
  a=self.adapter();self.adapter_instance=a
  ca={'version':'local-project-cycle@v3','journal':str(self.root/'staged'),'scope':a.native('request',None)['scope'],'max_decisions':9,'max_executions':2,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1'}
  if policy:ca['feedback_cycle']=self.a['feedback_cycle']
  return a,ca,m.load('staged_session','engine-browser-session.py')
 def events(self):
  want=self.a['origin']['public']['parents'][0]['meaning']
  return [{'id':'authored-condition-'+str(i),'attribution':'synthetic_harness_verification','guidance':{'controls':want+' Synthetic authored condition stage '+str(i)+'. Not player testimony; original want retained.'}} for i in range(3)]
 def test_retention_accepts_next_frozen_condition_and_abstention_stops(self):
  a,ca,session=self.configure(self.events());original=self.transport
  def transport(body,timeout):
   result=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   self.assertTrue(all(x['attribution']=='synthetic_harness_verification' for x in data['feedback_provenance']))
   result['value']={'retain':'Authored retain current condition; no fulfillment claim'} if len(data['feedback_provenance'])==1 else {'abstain':'Authored stop; do not continue to third condition'}
   return result
  a.injected_transport=transport
  result=session.run(a,ca)
  self.assertEqual(result['status'],'stopped');self.assertEqual(result['reason'],'proposal_abstention');self.assertEqual(result['feedback_applied'],2)
  self.assertEqual([x['status'] for x in result['checkpoints']],['proposal_retention','proposal_abstention'])
  self.assertEqual((self.ledger.status()['calls'],self.ledger.status()['execution_groups']),(2,0))
  self.assertNotIn('feedback_2',a.read()[2]);self.assertEqual(a.read()[2]['feedback_0']['attribution']['source'],'authored_synthetic_scenario')
  before=(self.root/'campaign.jsonl').read_bytes();self.assertEqual(session.run(self.adapter(),ca),result);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
 def test_three_conditions_two_native_actions_and_distinct_continuations(self):
  if os.getenv('TSR_BROWSER_SEALED_AUTHORED')!='1':self.skipTest('actual sealed authored browser opt-in')
  a,ca,session=self.configure(self.events());original=self.transport;self.playable=True
  def transport(body,timeout):
   result=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' in data:return result
   phase=len(data['feedback_provenance'])-1;current=(self.c.directory/'coding-work/app.js').read_text()
   if phase==0 or ('staged-'+str(phase)) in current:
    result['value']={'retain':'Authored no further change in this condition; future events remain separate'}
   elif phase==1:
    result['value']['alternatives'][0]['edits'][1]['new_text']+='\n// staged-1\n'
   else:
    result['value']={'alternatives':[{'edits':[{'path':'app.js','old_text':current,'new_text':current+'\n// staged-2\n'}],'rationale':'Authored subsequent condition fixture; not live behavior'}]}
   return result
  a.injected_transport=transport;result=session.run(a,ca)
  self.assertEqual(result['status'],'completed_checkpoints');self.assertEqual([r['status'] for r in result['checkpoints']],['proposal_retention']*3)
  self.assertEqual((self.ledger.status()['calls'],self.ledger.status()['execution_groups']),(7,2))
  continued=[(k,v) for k,v in a.read()[2].items() if k.startswith('session_continue_')]
  self.assertEqual(len(continued),2);self.assertNotEqual(continued[0][0],continued[1][0]);self.assertNotEqual(continued[0][1]['instruction']['message_id'],continued[1][1]['instruction']['message_id'])
  # Reusing an old ordinal with a new binding cannot silently return the old transition.
  key,record=continued[0];checkpoint=key[len('session_continue_'):].rsplit('_',1)[0]
  with self.assertRaisesRegex(ValueError,'replay request mismatch'):a.continue_session({'index':0,'checkpoint_id':checkpoint,'binding':a.native('request',None)['binding'],'reason':record['purpose']})
  self.assertEqual(self.c.cli('project','status')['execution_status'],'passed');self.assertFalse(self.c.cli('project','acceptance','status')['accepted'])
  before=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter();self.adapter_instance=fresh
  self.assertEqual(session.run(fresh,ca),result);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
 def test_stop_prevents_first_condition_and_preserves_state(self):
  a,ca,session=self.configure(self.events());a.root.joinpath('STOP').write_text('Authored stop')
  before=self.c.request()['state_root']
  with self.assertRaisesRegex(ValueError,'session stopped'):session.run(a,ca)
  self.assertEqual(before,self.c.request()['state_root']);self.assertEqual(self.ledger.status()['calls'],0)
 def test_retention_not_available_during_advisory_revision(self):
  a=self.adapter();v=a.native('request',None);r,c=a.project(v)
  with self.assertRaises(ValueError):a.decode_proposal({'retain':'Authored'},r,c,{**v,'branch':'reconsider'})
  with self.assertRaises(ValueError):a.check_proposal_shape({'retain':''})
 def test_refresh_arriving_with_retention_preserves_it_and_stops_schedule(self):
  a,ca,session=self.configure(self.events());original=self.transport
  def transport(body,timeout):
   result=original(body,timeout);result['value']={'retain':'Authored retention with newer input pending'}
   child={**ca,'journal':str(Path(ca['journal'])/'checkpoint-0')}
   import hashlib
   child['checkpoint_id']=hashlib.sha256(json.dumps({'adapter':a.admission,'index':0},sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
   Path(child['journal'],'CONTEXT_REFRESH').write_text(json.dumps({'version':'context-refresh@v1','cycle_admission_sha256':m.e.sha(child),'request_id':'authored-new-input','input_ids':['authored-observation'],'inputs_sha256':'a'*64}))
   return result
  a.injected_transport=transport;result=session.run(a,ca)
  self.assertEqual(result['reason'],'context_refresh_required');self.assertEqual(result['checkpoints'][0]['prior_outcome'],'proposal_retention');self.assertNotIn('feedback_1',a.read()[2])
 def test_feedback_interruption_keeps_unknown_blocked_without_republication(self):
  a,ca,session=self.configure(self.events())
  def crash(phase,name):
   if phase=='after_effect' and name=='feedback_0':raise RuntimeError('authored interrupted publication')
  a.hook=crash
  with self.assertRaisesRegex(RuntimeError,'interrupted'):session.run(a,ca)
  before=self.c.request()['state_root'];self.assertTrue(a.read()[1])
  with self.assertRaisesRegex(ValueError,'unresolved'):session.run(self.adapter(),ca)
  self.assertEqual(before,self.c.request()['state_root']);self.assertEqual(self.ledger.status()['calls'],0)
 def test_stop_between_conditions_does_not_publish_successor(self):
  a,ca,session=self.configure(self.events());original=self.transport
  def transport(body,timeout):
   result=original(body,timeout);result['value']={'retain':'Authored no change'};Path(ca['journal'],'STOP').write_text('authored human stop');return result
  a.injected_transport=transport
  with self.assertRaisesRegex(ValueError,'session stopped'):session.run(a,ca)
  self.assertNotIn('feedback_1',a.read()[2]);self.assertEqual(self.ledger.status()['calls'],1)
 def test_ordinary_arm_supports_same_retention_envelope(self):
  a,ca,_=self.configure(self.events());ordinary=m.load('staged_ordinary','engine-browser-ordinary.py');original=self.transport
  def transport(body,timeout):
   result=original(body,timeout);result['value']={'retain':'Authored ordinary no change'};return result
  a.injected_transport=transport
  result=ordinary.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention');self.assertEqual(self.ledger.status()['calls'],1)
