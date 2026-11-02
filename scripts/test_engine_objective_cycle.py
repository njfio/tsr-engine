"""Authored callback state, not native service/model evidence; no external effects.

Exercises the shared durable Cycle v5 control flow. Rust objective tests separately
exercise the real native store/receipt boundary.
"""
import copy
import datetime
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('objective_cycle',Path(__file__).with_name('engine-project-cycle.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class ObjectiveCycle(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
  self.base=Path(self.tmp.name).resolve();self.root=self.base/'journal';self.state_file=self.base/'authored-state.json'
  self.state={'round':0,'phase':'propose','version':0,'previous':'','receipts':[],'round_reviews':[],'service_count':0}
  self.calls=[];self.effects=[];self.save()
  self.a={'version':'local-project-cycle@v5','journal':str(self.root),'scope':{'id':'authored-service'},'max_decisions':3,'max_executions':3,'max_service_rounds':3,'expires_at':(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(hours=1)).isoformat(),'implementation_sha256':m.d.file_sha(Path(m.__file__))}
 def save(self):self.state_file.write_text(json.dumps(self.state,sort_keys=True))
 def transition(self,phase):self.state['phase']=phase;self.state['version']+=1;self.save()
 def request(self):
  s=self.state;phase=s['phase'];branch='objective_review_required' if phase in ('round_review','objective_review','unresolved') else phase
  body={'next':phase,'round':s['round'],'previous_output':s['previous'],'receipts':copy.deepcopy(s['receipts'])}
  return {'scope':self.a['scope'],'state_root':str(s['version']),'source_snapshot':'unchanged-authored-source','binding':'cycle-'+str(s['version']),'branch':branch,'reason':'authored-control','dispatch_authorized':False,'objective':{'binding':m.e.sha(body),'context':body},'request':{'fixture_only':True}}
 def native(self,command,value):
  if command in ('request','response_context'):return self.request()
  if command=='preview':return {'cycle':self.request(),'response_digest':m.e.sha(value['response'])}
  if command=='advance':
   self.assertEqual(value['binding'],self.request()['binding']);self.assertEqual(value['review']['response_digest'],m.e.sha(value['response']))
   self.transition('service_resume');return {'service_prepared':'authored-'+str(self.state['round']),'next':self.request(),'executed':False,'review_required':False}
  if command=='service_resume':
   self.assertEqual(value['binding'],self.request()['binding']);self.assertEqual(self.state['phase'],'service_resume')
   index=self.state['round'];output='authored-output-'+str(index);self.effects.append((index,self.state['previous']))
   self.state['service_count']+=1;self.state['receipts'].append({'receipt':'authored-receipt-'+str(index),'output':output,'previous':self.state['previous']});self.state['previous']=output;self.transition('round_review')
   return {'receipt':self.state['receipts'][-1],'fixture_only':True}
  if command=='objective_review':
   r=value['review'];self.assertEqual(r['binding'],self.request()['objective']['binding']);self.assertEqual(r['receipt'],self.state['receipts'][-1]['receipt']);self.state['round_reviews'].append(copy.deepcopy(r));self.state['round']+=1
   self.transition('unresolved' if r['decision']=='unknown' else 'objective_review' if self.state['round']==3 else 'propose');return self.request()
  self.fail('unexpected authored callback '+command)
 def meter(self,key,value):
  return {key:value,'receipt':{'mode':'fixture','status':'settled','usage':{'prompt_tokens':10,'completion_tokens':10,'total_tokens':20},'usd':'0.001','elapsed_seconds':0.01}}
 def proposal(self,view):
  self.calls.append('proposal');return self.meter('response',{'state_root':view['state_root'],'proposals':[{'kind':'service','fixture_only':True}]})
 def review(self,view,response):
  self.calls.append(response.get('objective_stage','proposal_review'))
  if 'objective_stage' in response:
   stage=response['objective_stage'];v={'binding':view['objective']['binding'],'decision':'met' if stage=='round_review' else 'accept','reviewer':'authored-independent-review','generator':'authored-generator','evidence':'Authored orchestration fixture only'}
   if stage=='round_review':v['receipt']=view['objective']['context']['receipts'][-1]['receipt']
  else:v={'decision':'approve','reviewer':'authored-independent-review','generator':'authored-generator','response_digest':view['response_digest']}
  return self.meter('judgment',v)
 def runner(self,**kwargs):return m.Cycle(self.root,self.a,self.native,self.proposal,self.review,**kwargs)
 def test_three_rounds_checkpoint_carry_forward_final_assessment_no_acceptance_effect(self):
  def pause(event,name):
   if event=='after_result' and name=='service_resume_0':raise RuntimeError('authored restart checkpoint')
  with self.assertRaisesRegex(RuntimeError,'checkpoint'):self.runner(hook=pause).run()
  self.state=json.loads(self.state_file.read_text());result=self.runner().run()
  self.assertEqual(result['status'],'objective_final_assessment');self.assertFalse(result['acceptance_recorded'])
  self.assertEqual(self.effects,[(0,''),(1,'authored-output-0'),(2,'authored-output-1')]);self.assertEqual(self.state['service_count'],3)
  self.assertEqual(len(self.calls),10);self.assertEqual(len(self.state['round_reviews']),3)
  frozen=(self.root/'cycle.jsonl').read_bytes();state=self.state_file.read_bytes();calls=list(self.calls)
  self.assertEqual(self.runner().run(),result);self.assertEqual((self.root/'cycle.jsonl').read_bytes(),frozen);self.assertEqual(self.state_file.read_bytes(),state);self.assertEqual(self.calls,calls)
 def test_unknown_effect_restart_never_repeats(self):
  def crash(event,name):
   if event=='after_effect' and name=='service_resume_0':raise RuntimeError('uncertain authored boundary')
  with self.assertRaises(RuntimeError):self.runner(hook=crash).run()
  self.state=json.loads(self.state_file.read_text())
  with self.assertRaisesRegex(m.d.Blocked,'uncertain'):self.runner().run()
  self.assertEqual(self.state['service_count'],1);self.assertEqual(len(self.calls),2)
 def test_unknown_round_review_stops_without_next_round(self):
  original=self.review
  def unknown(view,response):
   result=original(view,response)
   if response.get('objective_stage')=='round_review':result['judgment']['decision']='unknown'
   return result
  self.review=unknown;result=self.runner().run();self.assertEqual(result['status'],'objective_unknown');self.assertEqual(len(self.effects),1);self.assertEqual(len(self.calls),3)
 def test_abstention_permits_no_service(self):
  self.proposal=lambda _:self.meter('response',{'abstain':'No justified action in this authored control'})
  result=self.runner().run();self.assertEqual(result['status'],'proposal_abstention');self.assertEqual(self.effects,[])
 def test_service_cap_is_not_reset_at_new_round(self):
  self.a['max_service_rounds']=1;result=self.runner().run();self.assertEqual(result['status'],'execution_budget');self.assertEqual(len(self.effects),1)
 def test_review_context_drift_rejects_before_admission(self):
  original=self.review
  def drift(view,response):
   result=original(view,response)
   if response.get('objective_stage')=='round_review':self.state['version']+=1;self.save()
   return result
  self.review=drift
  with self.assertRaisesRegex(m.d.Blocked,'changed'):self.runner().run()
  self.assertEqual(self.state['round_reviews'],[]);self.assertEqual(len(self.effects),1)
 def test_unreviewed_service_is_not_dispatched(self):
  self.transition('service_resume');result=self.runner().run();self.assertEqual(result['status'],'unreviewed_service');self.assertEqual(self.effects,[])

 def test_relationship_only_update_reaches_ordinary_request_without_goal_or_budget_reset(self):
  # The native transition and generated revision below are explicitly authored.
  # This tests transport/orchestration of existing meaning, not native admission
  # or generative discovery of a relationship.
  self.a['max_decisions']=4
  self.state['want']='Keep continuity and distinct perspectives together'
  self.state['parents']=['authored-continuity','authored-discovery']
  self.state['relationship']='Neither parent substitutes for the other'
  self.save();captured=[];original_request=self.request;original_native=self.native
  original_proposal=self.proposal
  def request():
   v=original_request();v['request']['meaning']={k:copy.deepcopy(self.state[k]) for k in ('want','parents','relationship')}
   if v['branch']=='reconsider':v['reconsideration']={'lineage':'authored-existing-relation','evidence_key':'authored-first-return'}
   return v
  self.request=request
  def native(command,value):
   if command=='advance' and value['response'].get('proposals',[{}])[0].get('kind')=='relational':
    proposal=value['response']['proposals'][0]['proposal'];self.assertEqual(proposal['revises'],'authored-existing-relation')
    self.state['relationship']=proposal['relationship'];self.transition('propose')
    return {'qualitative_admitted':True,'review_required':False,'next':self.request(),'fixture_only':True}
   result=original_native(command,value)
   if command=='objective_review' and self.state['round']==1:
    self.transition('reconsider');return self.request()
   return result
  self.native=native
  def proposal(view):
   captured.append(copy.deepcopy(view))
   if view['branch']=='reconsider':
    self.calls.append('authored-relational-proposal')
    return self.meter('response',{'state_root':view['state_root'],'proposals':[{'kind':'relational','proposal':{'revises':'authored-existing-relation','relationship':'Continuity can preserve room for discovery; retain the original wanted condition'}}]})
   return original_proposal(view)
  self.proposal=proposal;result=self.runner().run()
  ordinary=[v for v in captured if v['branch']=='propose'];self.assertEqual(len(ordinary),3)
  self.assertNotEqual(ordinary[0]['request']['meaning']['relationship'],ordinary[1]['request']['meaning']['relationship'])
  for v in ordinary:
   self.assertEqual(v['request']['meaning']['want'],self.state['want']);self.assertEqual(v['request']['meaning']['parents'],self.state['parents'])
  self.assertEqual(ordinary[1]['objective']['context']['round'],1);self.assertEqual(ordinary[1]['objective']['context']['previous_output'],'authored-output-0')
  self.assertEqual(len(self.calls),12);self.assertEqual(self.state['service_count'],3)
  self.assertEqual(result['status'],'objective_final_assessment');self.assertFalse(result['acceptance_recorded'])
  frozen=(self.root/'cycle.jsonl').read_bytes();self.state=json.loads(self.state_file.read_text())
  self.assertEqual(self.runner().run(),result);self.assertEqual((self.root/'cycle.jsonl').read_bytes(),frozen);self.assertEqual(len(self.calls),12)
 def test_unchanged_context_does_not_force_relationship_or_new_goal(self):
  before=self.request();self.assertEqual(before,self.request());seen=[];original=self.proposal
  def proposal(view):seen.append(copy.deepcopy(view));return original(view)
  self.proposal=proposal;result=self.runner().run()
  self.assertEqual(result['status'],'objective_final_assessment')
  self.assertEqual([v['branch'] for v in seen],['propose']*3)
  self.assertTrue(all(v['source_snapshot']==before['source_snapshot'] and v['scope']==before['scope'] and v['request']==before['request'] for v in seen))
  self.assertEqual(len(self.calls),10);self.assertEqual(self.state['service_count'],3)

if __name__=='__main__':unittest.main()
