"""Authored real-native Adapter/Cycle recovery; no provider or execution effects."""
import copy,datetime,json,time
from pathlib import Path
from unittest.mock import patch
from test_engine_browser_adapter import BrowserAdapter,m
from test_engine_objective_continuation import failed,lookup
class NativeRecovery(BrowserAdapter):
 def test_corrected_review_failure_restart_retains_proposal(self):
  policy={'version':'authorized-feedback-cycle@v1','authorization_id':'authored-retry-test','max_continuations':1}
  self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=3,max_executions=0,correction_allowance=2,feedback_cycle=policy,transient_recovery={'version':'exact-failed-zero-recovery@v1','max_retries':1,'backoff_seconds':60})
  adapter=self.adapter();adapter.failure_lookup=lookup
  count=[0];original=self.transport
  def transport(body,timeout):
   data=json.loads(body['messages'][1]['content'])
   if 'criteria' in data or 'validation_feedback' in data or count[0]:
    count[0]+=1
    if count[0]==1:return {'value':{'verdict':[]},'receipt':{'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':.1}}
    if count[0]==2:return failed()
   return original(body,timeout)
  adapter.injected_transport=transport
  view=adapter.native('request',None);root=self.root/'cycle'
  ca={'version':'local-project-cycle@v3','journal':str(root),'scope':view['scope'],'max_decisions':3,'max_executions':0,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1','feedback_cycle':policy}
  cycle=m.loop.Cycle(root,ca,adapter.native,adapter.proposal,adapter.review)
  with patch.object(m.recovery.time,'sleep',side_effect=RuntimeError('Authored interruption during backoff')):
   with self.assertRaisesRegex(RuntimeError,'Authored interruption'):cycle.run()
  self.assertEqual(self.ledger.status()['calls'],3)
  self.assertFalse(self.ledger.status()['stopped']);self.assertFalse(adapter.read()[1])
  proposal=copy.deepcopy(cycle.read()[2]['proposal_0']);ticket=adapter.recovery_ticket()
  fresh=self.adapter();fresh.failure_lookup=lookup
  # Full fresh response to the persisted corrected request, not a repaired response.
  def valid(body,timeout):
   payload=json.loads(body['messages'][1]['content'])
   if 'original_request' in payload:
    body=copy.deepcopy(body);body['messages'][1]['content']=json.dumps(payload['original_request'])
   return original(body,timeout)
  fresh.injected_transport=valid
  resumed=m.loop.Cycle(root,ca,fresh.native,fresh.proposal,fresh.review)
  with patch.object(m.recovery.time,'time',return_value=ticket['not_before']+1):
   self.assertEqual(resumed.recover_pending(),1)
  self.assertEqual(self.ledger.status()['calls'],4)
  self.assertEqual(resumed.read()[2]['proposal_0'],proposal)
  self.assertEqual(resumed.read()[2]['review_0']['judgment']['decision'],'approve')
  before=(self.root/'campaign.jsonl').read_bytes();self.assertEqual(resumed.recover_pending(),0);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
  self.assertEqual(resumed.run()['status'],'execution_budget')
  self.assertEqual(self.ledger.status()['calls'],4)
  self.assertEqual(self.ledger.status()['execution_groups'],0)
for name in dir(BrowserAdapter):
 if name.startswith('test_') and name not in NativeRecovery.__dict__:setattr(NativeRecovery,name,None)

del BrowserAdapter
