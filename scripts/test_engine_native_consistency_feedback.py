"""Authored semantic consistency feedback through the real native browser adapter."""
import copy,json,unittest
from test_engine_feedback_recovery import Recovery,m
class NativeConsistency(Recovery):
 source_fixture='target/v2-tsr-return-preparation'
 def exercise(self,repeat=False,interrupt=False):
  a=self.adapter();view=a.native('request',None)
  if view['branch']=='review_required':a.continue_session({'binding':view['binding'],'index':0,'reason':'initial_checkpoint'})
  view=a.native('request',None);calls=[];invalid=[]
  def transport(body,timeout):
   data=json.loads(body['messages'][1]['content']);calls.append(data)
   if len(calls)==1:
    lineage=data['relational_contract']['allowed_lineage'][-1]
    old=next(x['proposal'] for x in data['relational_history'] if x['admission']==lineage['revises'])
    proposal=copy.deepcopy(old);proposal['aspiration']+=' Authored changed wanting without declared development.'
    proposal['revises']=lineage['revises'];proposal['development']={'original_revision':lineage['original_revision'],'prior_revision':lineage['prior_revision'],'retained_meaning':old['aspiration'],'return_reveals':'Authored fixture interpretation only','unresolved':'Unknown player experience','proposed_development':None}
    invalid.append({'alternatives':[{'relational':proposal,'return_basis':{'observed':[],'inferred':[],'unknown':['Authored unknown']}}]})
   else:
    self.assertIn('native_validation_feedback',data)
    self.assertEqual(data['native_validation_feedback']['items'][-1]['code'],'no_insight_meaning_drift')
   value=invalid[0] if len(calls)==1 or repeat else {'abstain':'Authored valid choice: do not change meaning or pursue another action.'}
   return {'value':value,'receipt':{'status':'settled','usd':'0.0001','elapsed_seconds':0.1,'usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200}}}
  a.injected_transport=transport
  if interrupt:
   ca={'version':'local-project-cycle@v3','journal':str(self.root/'cycle'),'scope':view['scope'],'max_decisions':4,'max_executions':0,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(__import__('pathlib').Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1','feedback_cycle':self.policy}
   cycle=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review)
   def crash(phase,name):
    if phase=='after_result' and name.startswith('native_rejection_'):raise RuntimeError('Authored interruption after rejected candidate persisted')
   a.hook=crash
   with self.assertRaisesRegex(RuntimeError,'Authored interruption'):cycle.model_stage('proposal_0','proposal',view)
   self.assertFalse(a.read()[1]);self.assertEqual(len(calls),1)
   a=self.adapter();a.injected_transport=transport;cycle=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review)
   self.assertEqual(cycle.recover_pending(),1);self.assertEqual(cycle.recover_pending(),0)
   self.assertIn('abstain',cycle.read()[2]['proposal_0']['response'])
  elif repeat:
   with self.assertRaisesRegex(ValueError,'identical invalid candidate'):a.proposal(view)
  else:
   result=a.proposal(view);self.assertIn('abstain',result['response'])
  self.assertEqual(len(calls),2);self.assertEqual(self.ledger.status()['calls'],2);self.assertEqual(self.ledger.status()['execution_groups'],0)
  self.assertEqual(len([k for k in a.read()[2] if k.startswith('native_rejection_')]),1)
  self.assertEqual(a.corrections.events(),[])
  self.assertEqual(a.native('request',None)['source_snapshot'],view['source_snapshot'])
  fresh=self.adapter();self.assertEqual(len([k for k in fresh.read()[2] if k.startswith('native_rejection_')]),1)
  return a,view
 def test_exact_consistency_feedback_allows_abstention(self):self.exercise()
 def test_identical_invalid_candidate_does_not_loop(self):self.exercise(True)
 def test_interrupted_feedback_resumes_once(self):self.exercise(interrupt=True)
 def test_security_errors_are_not_choice_feedback(self):
  a=self.adapter();view=a.native('request',None);source={k:view[k] for k in ('state_root','source_snapshot','scope')}
  for code in ('invalid_parent_references','invalid_revision_ancestry','unknown_stale_or_withdrawn_revision','stale_state'):
   self.assertFalse(a.record_native_rejection(m.e.StageValidationError('open_choice',code),'x',{}, {},source))
  with self.assertRaisesRegex(ValueError,'stale native rejection'):a.record_native_rejection(m.e.StageValidationError('open_choice','no_insight_meaning_drift'),'x',{}, {},{**source,'state_root':'wrong'})
  self.assertEqual(self.ledger.status()['calls'],0)
for name in dir(Recovery):
 if name.startswith('test_') and name not in NativeConsistency.__dict__:setattr(NativeConsistency,name,None)
del Recovery
if __name__=='__main__':unittest.main()

class SharedBudget(unittest.TestCase):
 def test_mixed_and_inherited_rejections_share_cap(self):
  for keys,inherited in ((['native_rejection_a','review_rejection_b','review_rejection_c'],False),(['native_rejection_a','review_rejection_b'],True),(['review_rejection_a','native_rejection_b','native_rejection_c'],False)):
   a=object.__new__(m.Adapter);a.admission={'max_review_rejections':3,'origin':{'prior_validation_feedback':{'witness':True}} if inherited else {}}
   a.read=lambda:([],{},dict.fromkeys(keys,{}));a.known_stop=lambda error:error
   with self.assertRaisesRegex(ValueError,'shared semantic rejection allowance'):a.require_rejection_room()
 def test_inherited_candidate_cannot_be_resampled(self):
  a=object.__new__(m.Adapter);source={'scope':'s','state_root':'r','source_snapshot':'v'};candidate={'authored':'invalid'}
  a.admission={'feedback_cycle':{},'origin':{},'max_review_rejections':3};a.admission['feedback_cycle']={'authorized':True}
  a.admission['origin']['prior_validation_feedback']={'candidate_sha256':m.e.sha(candidate),'scope':'s','source_snapshot':'v'}
  a.native=lambda *_:source;a.read=lambda:([],{},{});a.known_stop=lambda error:error
  with self.assertRaisesRegex(ValueError,'identical inherited'):a.record_native_rejection(m.e.StageValidationError('open_choice','no_insight_meaning_drift'),'x',{},candidate,source)

class RecoveryBindings(unittest.TestCase):
 def make(self):
  a=object.__new__(m.Adapter);source={'scope':'s','state_root':'r','source_snapshot':'v'}
  a.admission={'feedback_cycle':{'authorized':True},'origin':{},'max_review_rejections':3};content={'authored':'candidate'};wire={'authored':'wire'}
  ident=m.e.sha({'admission':m.e.sha(a.admission),'kind':'proposal','index':0,'body':m.e.sha(wire)})
  dispatch={'candidate_sha256':m.e.sha(content),'receipt':{'usage_status':'reported','usd':'0.0001'}};settled={}
  association=m.base.bound_output.associate(admission=a.admission,kind='proposal',index=0,wire=wire,ident=ident,content=content,dispatch=dispatch,settled=settled,source=source,current_source=source)
  record={'association':association,'candidate':content,'candidate_sha256':m.e.sha(content),'code':'no_insight_meaning_drift','source':source}
  events={'dispatch_'+ident:dispatch,'settle_'+ident:settled,'native_rejection_'+ident:record,'review_rejection_old1':{},'review_rejection_old2':{}}
  a.read=lambda:([],{},events);a.known_stop=lambda error:error;a.validate_current_admission=lambda:None;a.native=lambda *_:source;a.ledger=type('Ledger',(),{'status':lambda _: {'stopped':False}})()
  return a,events,ident,record
 def test_third_rejection_can_resume_same_final_choice(self):
  a,_,_,_=self.make();self.assertTrue(a.recovery_ready()['ready'])
  with self.assertRaises(ValueError):a.require_rejection_room()
 def test_substituted_provenance_and_unknown_settlement_stop(self):
  for field in ('candidate','role','version','source','request','receipt','settlement','code'):
   a,events,ident,record=self.make()
   if field=='candidate':record['candidate']={'changed':True}
   if field=='role':record['association']['kind']='review'
   if field=='version':record['association']['version']='wrong'
   if field=='source':record['association']['source']={'scope':'wrong'}
   if field=='request':record['association']['request_body_sha256']='f'*64
   if field=='receipt':events['dispatch_'+ident]['receipt']['usd']='1'
   if field=='settlement':events['settle_'+ident]['unknown_usage']=1
   if field=='code':record['code']='invalid_revision_ancestry'
   with self.subTest(field=field),self.assertRaises(ValueError):a.recovery_ready()
