"""Authored negative controls; no provider or worker calls."""
import copy,json
from scripts.test_engine_browser_adapter import BrowserAdapter,m
class RejectionBoundary(BrowserAdapter):
 def test_rejection_source_identity_replay_and_unknown_controls(self):
  a=self.adapter();view=a.native('request',None);p=a.proposal(view);preview=a.native('preview',{'response':p['response']});original=self.transport
  def reject(body,timeout):
   result=original(body,timeout);result['value']['verdict']='reject';next(iter(result['value']['findings'].values())).update(finding='not_met',explanation='Authored unsupported claim');return result
  a.injected_transport=reject
  review=a.review({**a.native('request',None),'response_digest':preview['response_digest']},p['response'])
  value={'source':a.native('request',None),'response':p['response'],'judgment':review['judgment']}
  for field in ('state_root','source_snapshot','scope'):
   bad=copy.deepcopy(value);bad['source'][field]='f'*64
   with self.assertRaises(Exception):a.native('reject_proposal',bad)
  bad=copy.deepcopy(value);bad['judgment']['response_digest']='f'*64
  with self.assertRaises(ValueError):a.native('reject_proposal',bad)
  result=a.native('reject_proposal',value);self.assertFalse(result['executed']);self.assertFalse(result['grants_authority'])
  fresh=self.adapter()
  with self.assertRaisesRegex(ValueError,'identical rejected'):fresh.native('reject_proposal',value)
  self.assertEqual(self.ledger.status()['calls'],2);self.assertEqual(self.ledger.status()['execution_groups'],0)
  self.assertEqual(len([k for k in fresh.read()[2] if k.startswith('review_rejection_')]),1)
