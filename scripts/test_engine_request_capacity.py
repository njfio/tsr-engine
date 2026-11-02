"""Local authored transport/accounting controls. No provider requests."""
import json
import tempfile
import unittest
from pathlib import Path
from test_engine_b91_campaign import m
from test_engine_campaign_price import PriceQuotes
class Capacity(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.path=Path(self.tmp.name)/'ledger'
  self.manifest=m.manifest('a'*64,self.path,fixture=True);self.ledger=m.Ledger(self.path,self.manifest,m.digest(self.manifest),create=True)
  self.trial='b'*64;self.ledger.enroll(self.trial,'c'*64);self.quote=PriceQuotes().quote();self.ledger.record_price(self.quote)
  self.body={'model':m.MODEL,'max_completion_tokens':32768,'stream':False,'messages':[{'role':'user','content':'x'*300000}]}
 def call(self,receipt):return self.ledger.run_call(self.trial,'one',self.body,240,lambda *_:{'value':{'ok':True},'receipt':receipt},payload_limit=m.capacity.PAYLOAD_BYTES)
 def test_dynamic_unknown_reservation_and_restart(self):
  self.call({'status':'settled','usd':None,'usage':{'prompt_tokens':1,'completion_tokens':1}})
  expected=max(m.RESERVE,int(m.quote_estimate(self.quote,32768,m.capacity.prompt_upper(self.body)).to_integral_value(rounding=m.decimal.ROUND_CEILING)))
  self.assertGreater(expected,m.RESERVE);self.assertEqual(self.ledger.status()['committed_microusd'],expected)
  restarted=m.Ledger(self.path,self.manifest,m.digest(self.manifest));self.assertEqual(restarted.status()['committed_microusd'],expected)
  with self.assertRaises(ValueError):self.call({})
 def test_measured_usage_over_request_upper_stays_billed_unusable(self):
  result=self.call({'status':'settled','usd':'0.001','usage':{'prompt_tokens':m.capacity.prompt_upper(self.body)+1,'completion_tokens':1}})
  self.assertIn('resource_stop',result);self.assertEqual(self.ledger.status()['known_cost_microusd'],1000)
 def test_actual_serialized_utf8_context_and_old_boundary(self):
  self.assertEqual(m.capacity.prompt_upper(self.body),len(json.dumps(self.body,allow_nan=False,ensure_ascii=False).encode())+4096)
  with self.assertRaises(ValueError):self.ledger.run_call(self.trial,'old',self.body,240,lambda *_:self.fail('no dispatch'))
  self.body['messages'][0]['content']='x'*m.capacity.PAYLOAD_BYTES
  with self.assertRaises(ValueError):self.call({})
  self.assertEqual(self.ledger.status()['calls'],0)
 def test_exact_dynamic_reservation_rejects_tampering(self):
  self.call({'status':'settled','usd':'0.001','usage':{'prompt_tokens':100,'completion_tokens':100}})
  with self.ledger.locked() as (_,rows):
   rows=__import__('copy').deepcopy(rows)
   call=next(r['event'] for r in rows if r['event']['kind']=='call');call['reserved_microusd']-=1
   with self.assertRaisesRegex(ValueError,'reservation drift'):self.ledger.fold(rows)
 def test_imported_relationship_applicability_does_not_change_provenance(self):
  web=m.load_module('history_projection','engine-browser-project.py')
  h={'current':True,'superseded':False,'withdrawn':False,'proposal':{'parents':['a','b'],'relationship':'Authored hypothesis'},'review':{'id':'original'},'root':'a'*64}
  allowed=[web.e.sha(h)];changed={**h,'current':False}
  self.assertTrue(web.owns_relationship_history(changed,allowed));self.assertFalse(changed['current'])
  self.assertTrue(web.owns_relationship_history({**changed,'superseded':True},allowed))
  self.assertTrue(web.owns_relationship_history({**changed,'withdrawn':True},allowed))
  for key,value in [('proposal',{'parents':['a','foreign'],'relationship':'Authored hypothesis'}),('review',{'id':'other'}),('root','b'*64),('current',0)]:
   forged={**changed,key:value};self.assertFalse(web.owns_relationship_history(forged,allowed))
  self.assertFalse(web.owns_relationship_history(changed,[]))
if __name__=='__main__':unittest.main()
