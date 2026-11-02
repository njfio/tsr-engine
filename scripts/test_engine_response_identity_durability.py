"""Authored offline HTTP and journal faults; no provider or game execution."""
import hashlib,importlib.util,json,tempfile,unittest,urllib.error
from pathlib import Path
from unittest.mock import patch
s=importlib.util.spec_from_file_location('identity_adapter',Path(__file__).with_name('engine-project-cycle-adapter.py'));a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
t=a.transport_module
UID='77fb54b7-249a-477d-bbb4-fb2d331a86b1'
class Interrupted(BaseException):pass
class Response:
 status=200
 def __init__(self,headers,read):self.headers=headers;self.reader=read
 def __enter__(self):return self
 def __exit__(self,*args):pass
 def read(self,n):return self.reader(n)
class IdentityDurability(unittest.TestCase):
 def body(self):return {'model':'gpt-6-luna','stream':False,'max_completion_tokens':32768,'messages':[{'role':'user','content':'authored public fixture'}]}
 def adapter(self,root):
  obj=object.__new__(a.Adapter);obj.root=root;obj.file=root/'adapter.jsonl';obj.admission={'completion_tokens':32768,'payload_bytes':32768};obj.fixture=True;obj.active=set();obj.hook=lambda *_:None;obj.validate_current_admission=lambda:None;obj.injected_transport=None;obj.evidence=None
  raw=t.serialize_request(self.body(),32768);obj.active_wire_binding={'local_request':'a'*64,'canonical_body_sha256':a.e.sha(self.body()),'wire_body_sha256':hashlib.sha256(raw).hexdigest(),'wire_body_bytes':len(raw),'source':{'state_root':'b'*64,'source_snapshot':'c'*64,'scope':'d'*64}}
  a.e.private_save(obj.file,obj.row([],{'kind':'admission','value':obj.admission,'fixture_only':True}));return obj
 def test_identity_fsynced_before_interrupted_body_and_restart_stays_pending(self):
  with tempfile.TemporaryDirectory() as tmp:
   obj=self.adapter(Path(tmp))
   def read(_):
    done=obj.read()[2];record=done['response_identity_'+'a'*64];self.assertEqual(record['identity']['response_request_ids'],[UID]);self.assertEqual(record['identity']['identity_status'],'single');raise Interrupted()
   with patch.object(t.broker,'credential',return_value='SECRET'),patch.object(t.urllib.request,'build_opener') as opener:
    opener.return_value.open.return_value=Response({'x-ci-request-id':UID,'Authorization':'SECRET'},read)
    with self.assertRaises(Interrupted):obj.op('dispatch_'+'a'*64,lambda:obj._wire(self.body(),1))
   raw=obj.file.read_text();self.assertNotIn('SECRET',raw);self.assertNotIn('Authorization',raw)
   _,pending,done=obj.read();self.assertEqual(set(pending),{'dispatch_'+'a'*64});self.assertIn('response_identity_'+'a'*64,done)
   with self.assertRaises(a.d.Blocked):obj.op('new_dispatch',lambda:self.fail('must not retry'))
 def test_http_error_identity_precedes_error_body(self):
  saved=[]
  class Error(urllib.error.HTTPError):
   def read(self,*_):self_test.assertEqual(saved[0]['response_request_ids'],[UID]);return b'{}'
  self_test=self;err=Error(t.ENDPOINT,503,'ignored',{'x-request-id':UID},None)
  with patch.object(t.broker,'credential',return_value='SECRET'),patch.object(t.urllib.request,'build_opener') as opener:
   opener.return_value.open.side_effect=err
   r=t.invoke(self.body(),1,expected_model='gpt-6-luna',payload_limit=32768,completion_limit=32768,on_response_identity=saved.append)
  self.assertEqual(saved[0]['http_status'],503);self.assertEqual(r['receipt']['status'],'unknown');self.assertIsNone(r['value'])
 def test_http_error_sink_failure_closes_without_reading(self):
  closed=[]
  class Error(urllib.error.HTTPError):
   def read(self,*_):raise AssertionError('body read after failed persistence')
   def close(self):closed.append(True);super().close()
  def fail(_):raise OSError('disk failure')
  err=Error(t.ENDPOINT,503,'ignored',{'x-ci-request-id':UID},None)
  with patch.object(t.broker,'credential',return_value='SECRET'),patch.object(t.urllib.request,'build_opener') as opener:
   opener.return_value.open.side_effect=err
   with self.assertRaises(OSError):t.invoke(self.body(),1,expected_model='gpt-6-luna',payload_limit=32768,completion_limit=32768,on_response_identity=fail)
  self.assertEqual(closed,[True])
 def test_missing_conflicting_and_preheader_failure_never_invent_identity(self):
  for headers,status in [({},'missing'),({'x-ci-request-id':UID,'x-request-id':'87fb54b7-249a-477d-bbb4-fb2d331a86b1'},'conflicting')]:
   saved=[]
   with patch.object(t.broker,'credential',return_value='SECRET'),patch.object(t.urllib.request,'build_opener') as opener:
    opener.return_value.open.return_value=Response(headers,lambda _:b'{}');t.invoke(self.body(),1,expected_model='gpt-6-luna',payload_limit=32768,completion_limit=32768,on_response_identity=saved.append)
   self.assertEqual(saved[0]['identity_status'],status)
  saved=[]
  with patch.object(t.broker,'credential',return_value='SECRET'),patch.object(t.urllib.request,'build_opener') as opener:
   opener.return_value.open.side_effect=TimeoutError();t.invoke(self.body(),1,expected_model='gpt-6-luna',payload_limit=32768,completion_limit=32768,on_response_identity=saved.append)
  self.assertEqual(saved,[])
 def test_sink_failure_stops_before_body(self):
  def fail(_):raise OSError('disk failure')
  with patch.object(t.broker,'credential',return_value='SECRET'),patch.object(t.urllib.request,'build_opener') as opener:
   opener.return_value.open.return_value=Response({'x-ci-request-id':UID},lambda _:self.fail('read after persistence failure'))
   r=t.invoke(self.body(),1,expected_model='gpt-6-luna',payload_limit=32768,completion_limit=32768,on_response_identity=fail)
  self.assertIsNone(r['value']);self.assertEqual(r['receipt']['status'],'unknown');self.assertEqual(r['receipt']['diagnostic']['stage'],'identity_persistence')
 def test_wrong_body_and_substituted_identity_rejected(self):
  with tempfile.TemporaryDirectory() as tmp:
   obj=self.adapter(Path(tmp));b=obj.active_wire_binding;record={'version':'response-header-identity@v1','request_body_sha256':b['wire_body_sha256'],'request_body_bytes':b['wire_body_bytes'],'http_status':200,'response_request_ids':[UID],'identity_status':'single'}
   obj.retain_response_identity(record);obj.retain_response_identity(record)
   with self.assertRaises(ValueError):obj.retain_response_identity({**record,'request_body_sha256':'f'*64})
   with self.assertRaises(ValueError):obj.retain_response_identity({**record,'response_request_ids':['87fb54b7-249a-477d-bbb4-fb2d331a86b1']})
   obj.active_wire_binding={**b,'source':{**b['source'],'state_root':'e'*64}}
   with self.assertRaises(ValueError):obj.retain_response_identity(record)
if __name__=='__main__':unittest.main()
