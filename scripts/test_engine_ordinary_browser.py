"""Authored comparator wiring tests. Not a model comparison or player evidence."""
import copy,json,os,unittest
from pathlib import Path
from test_engine_browser_adapter import BrowserAdapter,m
ordinary=m.load('ordinary_test','engine-browser-ordinary.py')
class Ordinary(BrowserAdapter):
 def configure(self,plain,assessed=False):
  policy={'version':'source-bound-browser-assessment@v1','script':str(Path(__file__).resolve().with_name('testdata')/'browser-project/independent-control-check.cjs'),'sha256':m.assessment.sha(Path(__file__).with_name('testdata')/'browser-project/independent-control-check.cjs'),'limitations':'Authored control fixture only'} if assessed else None
  self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=5,max_executions=2,correction_allowance=2,feedback_cycle={'version':'authorized-feedback-cycle@v1','authorization_id':'authored-pair','max_continuations':4},process_profile='ordinary-memory@v1' if plain else None,independent_assessment=policy)
  adapter=(ordinary.Adapter if plain else m.Adapter)(self.root/'adapter',self.a,self.c,self.ledger,self.transport,worker=lambda view:self.adapter_instance.resume_process(view));self.adapter_instance=adapter
  return adapter
 def test_memory_preserves_information_without_relational_interface(self):
  a=self.configure(True);view=a.native('request',None);request,ctx=a.project(view)
  raw,base=m.Adapter.project(a,view);native=m.Adapter.wire_context(a,raw,base)
  self.assertEqual(ctx['task_memory']['own_decisions'],native['own_decisions'])
  self.assertEqual(ctx['task_memory']['goals'],native['parents']);self.assertEqual(ctx['task_memory']['historical_notes'],native['relational_history'])
  self.assertNotIn('relational_contract',ctx);self.assertNotIn('relational',ctx['response_contracts'])
  self.assertEqual(ctx['return_evidence'],native['return_evidence']);self.assertEqual(ctx['proposal_references'],native['proposal_references'])
  self.assertEqual(ctx['response_envelope']['max_alternatives'],native['response_envelope']['max_alternatives'])
  self.assertEqual(a.proposal_body(view,ctx)['messages'][0],a.proposal_body(view,native)['messages'][0])
  for key in ['source_files','independent_assessments','proposal_responsibilities']:
   self.assertEqual(ctx.get(key),native.get(key))
 def test_request_retention_rejects_headers_without_storing_secret(self):
  a=self.configure(True);body=a.body('Authored test',{'test':'public fixture'});body['headers']={'Authorization':'authored-secret-marker'}
  with self.assertRaisesRegex(ValueError,'unapproved request evidence fields'):a.call('proposal',body)
  self.assertNotIn('authored-secret-marker',a.file.read_text());self.assertEqual(self.ledger.status()['calls'],0)
 def test_ordinary_two_candidates_reach_shared_preview(self):
  a=self.configure(True);view=a.native('request',None);request,ctx=a.project(view);source=next(f['source'] for f in request['coding']['files'] if f['path']=='app.js')
  value={'alternatives':[{'edits':[{'path':'app.js','old_text':source,'new_text':source+'\n// Authored candidate '+str(i)+'\n'}],'rationale':'Authored alternative '+str(i)} for i in range(2)]}
  response,basis=a.decode_proposal(value,request,ctx,view);self.assertEqual(len(response['proposals']),2)
  preview=a.native('preview',{'response':response});self.assertEqual(preview['cycle']['state_root'],view['state_root']);self.assertEqual(self.ledger.status()['execution_groups'],0)
 def exercise(self,plain,rejection=False,event=False,assessed=False,interrupt=False):
  if os.getenv('TSR_BROWSER_SEALED_AUTHORED')!='1':self.skipTest('authored sealed worker opt-in')
  a=self.configure(plain,assessed);self.playable=True;original=self.transport
  if event:
   # Fresh adapter includes an attributed event before its first operation.
   policy=[{'id':'authored-feedback','attribution':'verified_user_input','guidance':{'controls':'Authored user quotation: keep the good controls; no mandatory change'}}]
   self.a=m.admission(self.root/'event-adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=5,max_executions=2,correction_allowance=2,feedback_events=policy,feedback_cycle=self.a['feedback_cycle'],process_profile='ordinary-memory@v1')
   a=ordinary.Adapter(self.root/'event-adapter',self.a,self.c,self.ledger,self.transport,worker=lambda view:self.adapter_instance.resume_process(view));self.adapter_instance=a
  def transport(body,timeout):
   payload=json.loads(body['messages'][1]['content'])
   if 'criteria' not in payload and self.ledger.status()['execution_groups']:
    self.bodies.append(copy.deepcopy(body));return {'value':{'abstain':'Authored observed fixture complete; no further change justified'},'receipt':{'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':.1}}
   value=original(body,timeout)
   if rejection and 'criteria' in payload and not hasattr(self,'review_rejected'):
    self.review_rejected=True;value['value']['verdict']='reject';f=value['value']['findings']['stage_adequacy'];f.update(finding='not_met',explanation='Authored critique: retain an explicit fixture provenance comment')
   elif rejection and 'criteria' not in payload and hasattr(self,'review_rejected'):
    self.assertIn('Authored critique',json.dumps(payload));value['value']['alternatives'][0]['edits'][-1]['new_text']+='\n// Authored fixture provenance retained\n'
   return value
  a.injected_transport=transport;view=a.native('request',None);root=self.root/'cycle'
  ca={'version':'local-project-cycle@v3','journal':str(root),'scope':view['scope'],'max_decisions':5,'max_executions':2,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','feedback_cycle':self.a['feedback_cycle']}
  cls=ordinary.Cycle if plain else m.loop.Cycle
  def run():return m.load('ordinary_feedback_session','engine-browser-session.py').run(a,ca,cycle_type=cls) if event else cls(root,ca,a.native,a.proposal,a.review).run()
  if interrupt:
   def hook(phase,name):
    if phase=='after_effect' and name.startswith('independent_assessment_'):raise RuntimeError('authored post-assessment crash')
   a.hook=hook
   with self.assertRaisesRegex(RuntimeError,'post-assessment'):run()
   a.hook=lambda *x:None;a.recover_session_transition()
  outer=run();result=outer['checkpoints'][-1] if event else outer
  self.assertEqual(result['status'],'ordinary_retention_or_abstention' if plain else 'proposal_abstention');self.assertEqual(self.ledger.status()['calls'],5 if rejection else 3);self.assertEqual(self.ledger.status()['execution_groups'],2 if assessed else 1)
  last=json.loads(self.bodies[-1]['messages'][1]['content']);
  if not last['return_evidence']['cases']:
   import shutil
   diagnostic=m.e.ROOT/'target'/'ordinary-failed-return-diagnostic'
   if not diagnostic.exists():shutil.copytree(self.root,diagnostic)
  self.assertTrue(last['return_evidence']['cases'],json.dumps(last['return_evidence']));self.assertFalse(self.c.request()['coding']['relational']['history'])
  if assessed:
   self.assertEqual(last['independent_assessments'][0]['applicability'],'current_source');self.assertTrue(last['independent_assessments'][0]['observations']['checks'][0]['pass'])
   with self.assertRaisesRegex(ValueError,'binding mismatch'):a.native('execution_stage_receipt',{'index':0,'binding':'0'*64})
  done=a.read()[2]
  for key,value in done.items():
   if key.startswith('dispatch_'):
    saved=done['request_body_'+key[len('dispatch_'):]]
    self.assertEqual(saved['body_sha256'],m.e.sha(saved['body']))
    self.assertNotIn('headers',saved['body']);self.assertEqual(saved['body']['model'],m.base.MODEL)
  before=self.ledger.path.read_bytes();self.assertEqual(run(),outer);self.assertEqual(self.ledger.path.read_bytes(),before)
 def test_plain_real_assessment_interrupt_resume(self):self.exercise(True,assessed=True,interrupt=True)
 def test_tsr_real_assessment_return(self):self.exercise(False,assessed=True)
 def test_plain_rejection_exact_feedback_then_execution(self):self.exercise(True,rejection=True)
 def test_plain_attributed_feedback_startup_uses_ordinary_loop(self):self.exercise(True,event=True)
 def test_plain_action_return_next_choice_and_inert_restart(self):self.exercise(True)
 def test_native_without_relationship_has_free_post_action_choice(self):self.exercise(False)
for name in dir(BrowserAdapter):
 if name.startswith('test_') and name not in Ordinary.__dict__:setattr(Ordinary,name,None)
if __name__=='__main__':unittest.main()
