"""Authored native multi-checkpoint wiring; never live model/player evidence."""
import json, os
from pathlib import Path
from scripts.test_engine_browser_adapter import BrowserAdapter, m

class Sustained(BrowserAdapter):
 def test_scope_grant_uses_enrollment_canonicalization_with_unicode(self):
  self.a['origin']['public']['public_brief']+=' Unicode → test'
  a=self.adapter();g=a.scope_grant('a'*64,'Authored grant review',500000)
  self.assertEqual(g['session_admission_sha256'],m.e.sha(self.a));self.assertEqual(g['session_trial_sha256'],a.trial)
  campaign=m.load('test_campaign_identity','engine-b91-campaign.py')
  self.assertNotEqual(campaign.digest(self.a),g['session_admission_sha256'])
  self.ledger.grant_calls(g,campaign.digest(g));a.enroll()
  self.assertEqual(self.ledger.status()['calls'],0)
 def test_known_capacity_refusal_is_terminal_without_native_reservation(self):
  a=self.adapter();view=a.native('request',None);before=self.c.budget();original=self.ledger.run_call
  def refused(*args,**kwargs):
   self.assertTrue(kwargs['preflight_only']);raise ValueError('Authored session reservation limit')
  self.ledger.run_call=refused
  root=self.root/'capacity-cycle'
  ca={'version':'local-project-cycle@v3','journal':str(root),'scope':view['scope'],'max_decisions':2,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1'}
  cycle=m.loop.Cycle(root,ca,a.native,a.proposal,a.review);result=cycle.run()
  self.assertEqual(result['status'],'capacity_stop');self.assertFalse(cycle.read()[1]);self.assertFalse(a.read()[1])
  self.assertEqual(self.c.budget()['model_calls'],before['model_calls']);self.assertEqual(self.ledger.status()['calls'],0)
  self.assertEqual(m.loop.Cycle(root,ca,a.native,a.proposal,a.review).run(),result)
  self.ledger.run_call=original
 def test_relational_admission_execution_reconsideration_native_aliases(self):
  if os.getenv('TSR_BROWSER_SEALED_AUTHORED')!='1':self.skipTest('actual sealed authored browser opt-in')
  directory=self.root/'relational-native';directory.mkdir();self.c=m.d.Native(directory)
  requirements=self.root/'two-parents.json';requirements.write_text(json.dumps({'controls':'Usable controls','clarity':'Understandable authored status'}))
  self.c.cli('project','init','--spec',str(self.root/'scope.json'),'--requirements',str(requirements))
  self.c.json_cli(('project-lifecycle-open',),{'state_root':self.c.request()['state_root'],'training':None,'tool_ms_ceiling':120000,'reason':'Authored relational harness flow','non_tool':{'model_calls':12,'model_tokens':2000000,'wall_ms':600000,'patch_bytes':49152,'external_cost_microusd':0}},'--admission')
  completion=self.c.cli('project','acceptance','status');self.c.json_cli(('project','acceptance','feedback'),{'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],'guidance':json.loads(requirements.read_text())})
  self.c.cli('project','resource-unpriced-admit','--state-root',self.c.request()['state_root'])
  request=self.c.request();coding=request['coding'];public=self.a['origin']['public'];public.update(scope=request['next_attention']['scope'],requirements_revision=coding['relational']['inputs']['requirements_revision'],parents=coding['relational']['inputs']['parents'],source_snapshots=[coding['snapshot']])
  self.a['origin']['directory']=str(directory)
  self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=5,max_executions=1,correction_allowance=1)
  a=self.adapter();self.adapter_instance=a;self.playable=True;original=self.transport
  def transport(body,timeout):
   data=json.loads(body['messages'][1]['content']);result=original(body,timeout)
   if 'criteria' in data:return result
   if not data['relational_history']:
    result['value']={'alternatives':[{'relational':{'parents':data['relational_contract']['parent_ids'],'relationship':'Authored controls and clear status may support one another','aspiration':'An understandable interactive fixture','discriminator':'Actual click and key observations can expose ambiguity','revises':None,'catalysts':[]}}]}
   elif 'advisory_reconsideration' in data:
    prior=data['relational_history'][-1];proposal=prior['proposal'];lineage=data['relational_contract']['allowed_lineage'][-1]
    result['value']={'alternatives':[{'relational':{**proposal,'revises':prior['admission'],'development':{'original_revision':lineage['original_revision'],'prior_revision':lineage['prior_revision'],'retained_meaning':proposal['aspiration'],'return_reveals':'Authored interpretation of the actual bounded browser return','unresolved':'Player satisfaction is not observed','proposed_development':None}},'return_basis':{'observed':[x['id'] for x in data['return_evidence']['cases']][:8],'inferred':[],'unknown':['No player satisfaction evidence']}}]}
   return result
  a.injected_transport=transport;root=self.root/'relational-cycle';view=a.native('request',None)
  ca={'version':'local-project-cycle@v3','journal':str(root),'scope':view['scope'],'max_decisions':5,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1'}
  result=m.loop.Cycle(root,ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'review_required');self.assertEqual(self.ledger.status()['calls'],6)
  history=self.c.request()['coding']['relational']['history'];self.assertEqual(len(history),2)
  before=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter();self.assertEqual(m.loop.Cycle(root,ca,fresh.native,fresh.proposal,fresh.review).run(),result);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
 def test_two_feedback_checkpoints_one_budget_and_restart(self):
  if os.getenv('TSR_BROWSER_SEALED_AUTHORED')!='1':self.skipTest('actual sealed authored browser opt-in')
  self.playable=True
  events=[{'id':'authored-1','attribution':'synthetic_harness_verification','guidance':{'controls':'Authored harness event one: usable controls'}},{'id':'authored-2','attribution':'synthetic_harness_verification','guidance':{'controls':'Authored harness event two: retain usable controls and document implementation'}}]
  self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=4,max_executions=2,correction_allowance=1,feedback_events=events)
  a=self.adapter();self.adapter_instance=a;original=self.transport
  def transport(body,timeout):
   data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data and 'event two' in json.dumps(data['parents']):
    self.assertTrue(data['browser_returns'])
    result=original(body,timeout)
    current=(self.c.directory/'coding-work/app.js').read_text()
    result['value']={'alternatives':[{'edits':[{'path':'app.js','old_text':current,'new_text':current+'\n// Authored second checkpoint fixture\n'}],'rationale':'Authored second complete incremental edit; not model behavior'}]}
    return result
   return original(body,timeout)
  a.injected_transport=transport
  root=self.root/'session';view=a.native('request',None)
  ca={'version':'local-project-cycle@v3','journal':str(root),'scope':view['scope'],'max_decisions':4,'max_executions':2,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1'}
  session=m.load('test_session','engine-browser-session.py')
  result=session.run(a,ca)
  self.assertEqual(result['status'],'completed_checkpoints');self.assertEqual(len(result['checkpoints']),2)
  self.assertEqual((self.ledger.status()['calls'],self.ledger.status()['execution_groups']),(4,2))
  self.assertEqual(self.c.cli('project','status')['execution_status'],'passed')
  self.assertFalse(self.c.cli('project','acceptance','status')['accepted'])
  _,context=a.project(a.native('request',None));self.assertIn('event two',json.dumps(context['parents']))
  before=(self.root/'campaign.jsonl').read_bytes()
  fresh=self.adapter();self.adapter_instance=fresh
  self.assertEqual(session.run(fresh,ca),result);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
 def test_feedback_unknown_scope_and_duplicate_are_not_authority(self):
  with self.assertRaises(ValueError):m.validate_feedback_events([{'id':'x','attribution':'model','guidance':{'controls':'x'}}],self.a['origin']['public']['parents'])
  with self.assertRaises(ValueError):m.validate_feedback_events([{'id':'x','attribution':'verified_user_input','guidance':{'permission':'new access'}}],self.a['origin']['public']['parents'])
