"""Authored native feedback-cycle verification, not model evidence."""
import json,os
from pathlib import Path
from test_engine_browser_adapter import BrowserAdapter,m
class FeedbackCycle(BrowserAdapter):
 def test_return_review_rejection_choice_execution_reflection_without_bridge(self):
  self.exercise(False)
 def test_initial_rejection_then_corrected_reflection_then_action(self):
  self.exercise(True)
 def exercise(self,initial):
  if os.getenv('TSR_BROWSER_SEALED_AUTHORED')!='1':self.skipTest('actual sealed authored browser opt-in')
  directory=self.root/'relational-native';directory.mkdir();self.c=m.d.Native(directory)
  requirements=self.root/'two-parents.json';requirements.write_text(json.dumps({'controls':'Usable controls','clarity':'Understandable authored status'}))
  self.c.cli('project','init','--spec',str(self.root/'scope.json'),'--requirements',str(requirements))
  self.c.json_cli(('project-lifecycle-open',),{'state_root':self.c.request()['state_root'],'training':None,'tool_ms_ceiling':120000,'reason':'Authored relational harness flow','non_tool':{'model_calls':12,'model_tokens':2000000,'wall_ms':1200000,'patch_bytes':49152,'external_cost_microusd':0}},'--admission')
  completion=self.c.cli('project','acceptance','status');self.c.json_cli(('project','acceptance','feedback'),{'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],'guidance':json.loads(requirements.read_text())})
  self.c.cli('project','resource-unpriced-admit','--state-root',self.c.request()['state_root'])
  request=self.c.request();coding=request['coding'];public=self.a['origin']['public'];public.update(scope=request['next_attention']['scope'],requirements_revision=coding['relational']['inputs']['requirements_revision'],parents=coding['relational']['inputs']['parents'],source_snapshots=[coding['snapshot']])
  self.a['origin']['directory']=str(directory)
  self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=8,max_executions=2 if initial else 1,correction_allowance=1,feedback_cycle={'version':'authorized-feedback-cycle@v1','authorization_id':'authored-session','max_continuations':3})
  a=self.adapter();self.adapter_instance=a;self.playable=True;original=self.transport
  def transport(body,timeout):
   data=json.loads(body['messages'][1]['content']);result=original(body,timeout)
   if 'criteria' in data:
    if not hasattr(self,'rejected_once') and any(isinstance(v,dict) and v.get('advisory_reconsideration') for v in data.get('evidence',{}).values()):
     self.rejected_once=True;result['value']['verdict']='reject'
     first=next(iter(result['value']['findings'].values()));first['finding']='not_met';first['explanation']='Authored exact rejection: state the limited scope of this browser observation'
    return result
   if not data['relational_history']:
    result['value']={'alternatives':[{'relational':{'parents':data['relational_contract']['parent_ids'],'relationship':'Authored controls and clear status may support one another','aspiration':'An understandable interactive fixture','discriminator':'Actual click and key observations can expose ambiguity','revises':None,'catalysts':[]}}]}
   elif 'advisory_reconsideration' in data or data.get('review_criticism'):
    prior=data['relational_history'][-1];proposal=prior['proposal'];lineage=data['relational_contract']['allowed_lineage'][-1]
    result['value']={'alternatives':[{'relational':{**proposal,'revises':prior['admission'],'development':{'original_revision':lineage['original_revision'],'prior_revision':lineage['prior_revision'],'retained_meaning':proposal['aspiration'],'return_reveals':('Authored corrected interpretation: the exact bounded browser return does not establish enjoyment' if data.get('review_criticism') else 'Authored interpretation of the actual bounded browser return'),'unresolved':'Player satisfaction is not observed','proposed_development':None}},'return_basis':{'observed':[x['id'] for x in data['return_evidence']['cases']][:8],'inferred':[],'unknown':['No player satisfaction evidence']}}]}
   elif getattr(self,'prepared_initial',False):
    current=(self.c.directory/'coding-work/app.js').read_text()
    result['value']={'alternatives':[{'edits':[{'path':'app.js','old_text':current,'new_text':current+'\n// Authored harmless second pursuit for lifecycle coverage\n'}],'rationale':'Authored fixture action, not live model choice'}]}
   return result
  a.injected_transport=transport;root=self.root/'relational-cycle';view=a.native('request',None)
  ca={'version':'local-project-cycle@v3','journal':str(root),'scope':view['scope'],'max_decisions':8,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','feedback_cycle':self.a['feedback_cycle']}
  if initial:
   setup={**ca,'journal':str(self.root/'setup-cycle'),'max_decisions':2}
   prepared=m.loop.Cycle(setup['journal'],setup,a.native,a.proposal,a.review).run()
   self.assertEqual(prepared['status'],'decision_budget');self.assertEqual(a.native('request',None)['branch'],'reconsider')
   self.prepared_initial=True
  result=m.loop.Cycle(root,ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'feedback_cycle_complete',json.dumps({'result':result,'basis':[v['return_basis'] for k,v in a.read()[2].items() if k.startswith('generated_')]}));self.assertEqual(self.ledger.status()['calls'],12 if initial else 8)
  history=self.c.request()['coding']['relational']['history'];self.assertEqual(len(history),3 if initial else 2);self.assertTrue(self.rejected_once);self.assertTrue(any('Authored exact rejection' in json.dumps(b) for b in self.bodies if 'criteria' not in json.loads(b['messages'][1]['content'])))
  if initial:
   done=m.loop.Cycle(root,ca,a.native,a.proposal,a.review).read()[2]
   purposes=[v['reason'] for k,v in done.items() if k.startswith('session_continue_')]
   self.assertEqual(purposes,['review_feedback','after_reflection']);self.assertEqual(sum(k.startswith('resume_') for k in done),1)
  before=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter();self.assertEqual(m.loop.Cycle(root,ca,fresh.native,fresh.proposal,fresh.review).run(),result);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())

for name in dir(BrowserAdapter):
 if name.startswith('test_') and name not in FeedbackCycle.__dict__:setattr(FeedbackCycle,name,None)
if __name__=='__main__':
 import unittest;unittest.main()
