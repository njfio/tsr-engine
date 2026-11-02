"""Authored local timeout injection; no provider or worker effects."""
import hashlib, json, subprocess, unittest
from unittest.mock import patch
from types import SimpleNamespace
from test_engine_browser_adapter import BrowserAdapter, m
class TimeoutBoundary(BrowserAdapter):
 def test_timeout_retains_settled_content_without_regeneration(self):
  adapter=self.adapter();view=adapter.native('request',None);original=self.transport
  def transport(body,timeout):
   result=original(body,timeout)
   self.captured=json.dumps(result['value']).encode()
   adapter.evidence.capture(None,self.captured,None,False)
   return result
  adapter.injected_transport=transport
  native=adapter.native
  def fault(kind,value):
   if kind=='preview':raise subprocess.TimeoutExpired(['native','preview'],120)
   return native(kind,value)
  adapter.native=fault
  with self.assertRaises(subprocess.TimeoutExpired):adapter.proposal(view)
  self.assertEqual(self.ledger.status()['calls'],1)
  self.assertFalse(self.ledger.status()['unknown_calls'])
  self.assertEqual(len(adapter.corrections.events()),0)
  path=adapter.root/'output-0/failed-response-1'
  self.assertEqual(json.loads((path/'metadata.json').read_text())['stage'],'local_controller_timeout')
  self.assertEqual((path/'content.bin').read_bytes(),self.captured)
  info=json.loads((path/'metadata.json').read_text())['parts']['content']
  self.assertEqual(info['sha256'],hashlib.sha256(self.captured).hexdigest())
  self.assertFalse(info['truncated']);self.assertFalse(info['withheld'])
  self.assertFalse(any(k.startswith('generated_') for k in adapter.read()[2]))
 def test_local_command_budget_is_finite_without_retry(self):
  with patch.object(m.e.subprocess,'run',return_value=SimpleNamespace(returncode=0,stdout=b'{}')) as run:
   self.c.cli('project','cycle','request')
  self.assertEqual(run.call_count,1);self.assertEqual(run.call_args.kwargs['timeout'],120)
for name in dir(BrowserAdapter):
 if name.startswith('test_') and name not in TimeoutBoundary.__dict__:setattr(TimeoutBoundary,name,None)
if __name__=='__main__':unittest.main()
