"""Read-only regression using preserved failure; never provider calls or mutation."""
import copy,json,unittest
from pathlib import Path
from unittest.mock import patch
import server
from recovery import verify
class Recovery(unittest.TestCase):
 def setUp(self):
  self.s=server.Store();path=server.ROOT/'target/studio-context-result/live-packet.txt'
  if not path.exists():self.skipTest('optional preserved failure unavailable')
  self.packet=Path(path.read_text());a=server.load('proof_test','engine-browser-adapter.py')
  q=self.s.native(self.packet/'native').request()
  self.proof={'version':'local-projection-failure@v1','packet':str(self.packet),'pid':53404,'source_root':q['state_root'],'snapshot':q['coding']['snapshot'],'files':{x:a.d.file_sha(self.packet/x) for x in ('adapter/adapter.jsonl','cycle/cycle.jsonl','adapter-admission.json','cycle-admission.json','prepared-manifest.json','scope-grant.json')}}
 def test_actual_settled_action_is_not_repeated(self):
  before=dict(self.proof['files']);result=verify(self.s,'pond-courier',self.packet,self.proof)
  self.assertEqual(result['old_stage'],'proposal_2');self.assertFalse(result['redispatched']);self.assertTrue(result['old_journal_remains_pending'])
  a=server.load('proof_hash','engine-browser-adapter.py');self.assertEqual(before,{x:a.d.file_sha(self.packet/x) for x in before})
 def test_modified_stale_or_incomplete_disposition_refused(self):
  for change in ({'source_root':'0'*64},{'snapshot':'0'*64},{'pid':99999},{'files':{}},{'files':{**self.proof['files'],'cycle/cycle.jsonl':'0'*64}}):
   with self.subTest(change=list(change)),self.assertRaises(ValueError):verify(self.s,'pond-courier',self.packet,{**self.proof,**change})
 def test_live_worker_and_unknown_effect_refused(self):
  with patch('recovery.os.kill',return_value=None),self.assertRaisesRegex(ValueError,'still exists'):verify(self.s,'pond-courier',self.packet,self.proof)
  c,m,l,u=self.s.ledger();status=l.status();status['blocking_unknown_executions']=['authored-unknown']
  with patch.object(self.s,'ledger',return_value=(c,m,type('FakeLedger',(),{'status':lambda _:status})(),u)),self.assertRaisesRegex(ValueError,'Unknown external'):verify(self.s,'pond-courier',self.packet,self.proof)
if __name__=='__main__':unittest.main()
