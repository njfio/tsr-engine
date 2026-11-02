"""Authored paired wall-clock controls, no provider or execution."""
import importlib.util,tempfile,time,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('pairclock',Path(__file__).with_name('engine-pair-clock.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Clock(unittest.TestCase):
 def test_same_active_duration_no_restart_reset(self):
  rows={}
  def op(k,f):
   if k not in rows:rows[k]=f()
   return rows[k]
  a={'active_window_seconds':3300,'expires_at':'2099-01-01T00:00:00+00:00'}
  first=m.start(a,op,100,lambda x:10000);self.assertEqual(first,m.start(a,op,200,lambda x:10000))
  self.assertEqual(m.deadline(a,rows,lambda x:10000),3400)
  m.require(a,rows,lambda x:10000,240,now=3000)
  with self.assertRaises(ValueError):m.require(a,rows,lambda x:10000,240,now=3200)
 def test_dispatch_after_backoff_and_setup_uses_remaining_time(self):
  a={'active_window_seconds':3300,'expires_at':'x'};rows={'session_clock':{'started_at':100,'deadline':3400,'active_window_seconds':3300}};sent=[]
  result=m.wire(a,rows,lambda x:10000,{},240,lambda body,timeout:sent.append(timeout),now=3300)
  self.assertEqual(sent,[100])
  result=m.wire(a,rows,lambda x:10000,{},240,lambda *x:self.fail('must not dispatch'),now=3400)
  self.assertEqual(result['receipt']['usage_status'],'local_not_dispatched');self.assertEqual(result['receipt']['usd'],'0.000000')
 def test_adapter_retains_host_marker_despite_receipt_sanitization(self):
  spec=importlib.util.spec_from_file_location('clock_adapter',Path(__file__).with_name('engine-browser-adapter.py'));a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)
  adapter=object.__new__(a.Adapter);adapter.admission={'active_window_seconds':3300,'expires_at':'2099-01-01T00:00:00+00:00'}
  done={'session_clock':{'started_at':1,'deadline':3301,'active_window_seconds':3300}}
  adapter.read=lambda:([],{},done)
  def op(k,fn):
   if k not in done:done[k]=fn()
   return done[k]
  adapter.op=op
  result=adapter._wire({},240);stored=a.e.measured_receipt(result['receipt'])
  markers=[v for k,v in done.items() if k.startswith('local_no_dispatch_')]
  self.assertEqual(len(markers),1);self.assertFalse(markers[0]['provider_response']);self.assertEqual(markers[0]['charge_microusd'],0)
  self.assertEqual(stored['response_request_ids'],[])
  adapter._wire({},240);self.assertEqual(len(done),2)
 def test_outer_deadline_missing_clock_and_headroom(self):
  a={'active_window_seconds':3300,'expires_at':'x'}
  with self.assertRaises(ValueError):m.start(a,lambda k,f:f(),100,lambda x:3000)
  with self.assertRaises(ValueError):m.require(a,{},lambda x:10000,30,now=100)
  rows={'session_clock':{'started_at':100,'deadline':3400,'active_window_seconds':3300}}
  with self.assertRaises(ValueError):m.require(a,rows,lambda x:3000,30,now=2980)
  with self.assertRaises(ValueError):m.require(a,rows,lambda x:10000,360,now=3100)
if __name__=='__main__':unittest.main()
