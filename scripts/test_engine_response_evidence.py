"""Offline wire/failure injection, no real credential lookup or network."""
import copy,hashlib,importlib.util,io,json,os,subprocess,tempfile,unittest,urllib.error
from pathlib import Path
from unittest.mock import patch
import test_engine_synthetic_transport as base
import test_engine_exploration_validation as flow
t=base.t
class Evidence(unittest.TestCase):
 def exercise(self,parent,*,content='{"answer":1}',wire=None,finish='stop',error=None,secret='TEST_CREDENTIAL_DO_NOT_RETAIN',extra=None):
  envelope=base.Transport().envelope();envelope['model']='gpt-6-luna';envelope['choices'][0]['message']={'role':'assistant','content':content};envelope['choices'][0]['finish_reason']=finish
  if extra:envelope['choices'][0]['message'].update(extra)
  raw=wire if wire is not None else json.dumps(envelope).encode()
  sink=t.FailureEvidence(parent,'failed-response-1')
  body={'model':'gpt-6-luna','stream':False,'max_completion_tokens':8192,'response_format':{'type':'json_object'},'messages':[{'role':'user','content':'JSON synthetic control'}]}
  with patch.object(t.broker,'credential',return_value=secret),patch.object(t.urllib.request,'build_opener') as op:
   if error:op.return_value.open.side_effect=error
   else:op.return_value.open.return_value=base.Response(raw)
   result=t.invoke(body,1,expected_model='gpt-6-luna',payload_limit=32768,failure_evidence=sink)
   request=op.return_value.open.call_args.args[0]
   self.assertEqual(json.loads(request.data),body);self.assertEqual(op.return_value.open.call_count,1)
   self.assertEqual(result['receipt']['request_body_sha256'],hashlib.sha256(request.data).hexdigest())
   self.assertNotIn(secret,json.dumps(result));self.assertFalse(result['receipt']['recovery']['automatic_retry'])
  return result,sink,raw
 def test_content_failures_exact_bytes_accounting_and_excluded_reasoning(self):
  for content,code in [('{"x":','malformed_json'),('{"x":1,"x":2}','duplicate_key'),('','malformed_json')]:
   with self.subTest(code=code),tempfile.TemporaryDirectory() as tmp:
    r,s,raw=self.exercise(tmp,content=content,extra={'reasoning_content':'PRIVATE_REASONING'})
    self.assertEqual(r['receipt']['error_code'],code);self.assertEqual(r['receipt']['parse_location'],'content')
    self.assertEqual(r['receipt']['usd'],'0.001');self.assertEqual(r['receipt']['recovery']['state'],'known_terminal_invalid_response')
    folder=Path(tmp)/'failed-response-1';self.assertEqual((folder/'content.bin').read_bytes(),content.encode())
    self.assertFalse((folder/'wire.bin').exists());self.assertTrue(r['receipt']['failure_evidence']['parts']['wire']['withheld'])
    self.assertNotIn(b'PRIVATE_REASONING',b''.join(p.read_bytes() for p in folder.iterdir()))
    self.assertEqual((folder/'content.bin').stat().st_mode & 0o777,0o600)
 def test_wire_wrapper_errors_withhold_unseparable_private_bytes_and_cost_unknown(self):
  for raw,code in [(b'{"choices":','malformed_json'),(b'{"x":1,"x":2}','duplicate_key'),(b'[]','response_shape')]:
   with tempfile.TemporaryDirectory() as tmp:
    r,_,_=self.exercise(tmp,wire=raw);self.assertEqual(r['receipt']['error_code'],code)
    self.assertEqual(r['receipt']['status'],'unknown');self.assertIsNone(r['receipt']['usd'])
    self.assertEqual(r['receipt']['recovery']['state'],'reconcile_exact_request')
    self.assertFalse(list((Path(tmp)/'failed-response-1').glob('*.bin')))
 def test_truncation_exact_wire_and_bounded_prefix(self):
  with tempfile.TemporaryDirectory() as tmp:
   content='x'*70000;r,_,raw=self.exercise(tmp,content=content,finish='length')
   self.assertEqual(r['receipt']['error_code'],'incomplete_output');self.assertEqual(r['receipt']['usd'],'0.001')
   folder=Path(tmp)/'failed-response-1'
   self.assertEqual((folder/'content.bin').read_bytes(),content.encode()[:65536])
   self.assertEqual((folder/'wire.bin').read_bytes(),raw[:65536])
   self.assertTrue(r['receipt']['failure_evidence']['parts']['wire']['truncated'])
 def test_secret_suppression_not_redacted_output(self):
  for content in ['{"key":"TEST_CREDENTIAL_DO_NOT_RETAIN",','{"authorization":"Bearer arbitrary-token",','{"password":"private",',r'{"\u0070assword":"private",',r'{"key":"\u0054EST_CREDENTIAL_DO_NOT_RETAIN",']:
   with tempfile.TemporaryDirectory() as tmp:
    r,_,_=self.exercise(tmp,content=content)
    self.assertFalse(list((Path(tmp)/'failed-response-1').glob('*.bin')))
    self.assertTrue(r['receipt']['failure_evidence']['parts']['content']['withheld'])
 def test_credential_echo_in_request_id_is_not_receipt_metadata(self):
  with tempfile.TemporaryDirectory() as tmp:
   e=base.Transport().envelope();e['model']='gpt-6-luna';e['request_id']='TEST_CREDENTIAL_DO_NOT_RETAIN';e['choices'][0]['message']['content']='{'
   r,_,_=self.exercise(tmp,wire=json.dumps(e).encode())
   self.assertIsNone(r['receipt']['request_id']);self.assertEqual(r['receipt']['usd'],'0.001')
   self.assertFalse((Path(tmp)/'failed-response-1/wire.bin').exists())
 def test_valid_json_credential_echo_is_rejected_not_rewritten(self):
  with tempfile.TemporaryDirectory() as tmp:
   r,_,_=self.exercise(tmp,content=r'{"value":"\u0054EST_CREDENTIAL_DO_NOT_RETAIN"}')
   self.assertIsNone(r['value']);self.assertEqual(r['receipt']['error_code'],'credential_echo')
   self.assertEqual(r['receipt']['usd'],'0.001');self.assertFalse(list((Path(tmp)/'failed-response-1').glob('*.bin')))
 def test_network_unknown_and_provider_rejection_no_retry_or_inferred_zero(self):
  for error in [TimeoutError('private'),urllib.error.HTTPError(t.ENDPOINT,400,'bad',{},io.BytesIO(b'{"error":{"param":"response_format","code":"unsupported"}}'))]:
   with tempfile.TemporaryDirectory() as tmp:
    r,_,_=self.exercise(tmp,error=error);self.assertIsNone(r['receipt']['usd']);self.assertEqual(r['receipt']['status'],'unknown')
    self.assertEqual(r['receipt']['recovery']['state'],'reconcile_exact_request')
    if isinstance(error,urllib.error.HTTPError):self.assertEqual(r['receipt']['structured_output_status'],'rejected_by_provider')
 def test_success_not_persisted_and_symlink_overwrite_rejected(self):
  with tempfile.TemporaryDirectory() as tmp:
   r,s,_=self.exercise(tmp);self.assertEqual(r['value'],{'answer':1});self.assertFalse((Path(tmp)/'failed-response-1').exists())
   s.clear();self.assertEqual(s.parts,{})
   os.symlink(tmp,Path(tmp)/'link')
   with self.assertRaises(ValueError):t.FailureEvidence(Path(tmp)/'link','failed-response-1')
   os.symlink(tmp,Path(tmp)/'failed-response-1')
   r,s,_=self.exercise(tmp,content='{');self.assertEqual(r['receipt']['failure_evidence']['status'],'retention_failed')
   self.assertEqual(r['receipt']['usd'],'0.001');self.assertFalse((Path(tmp)/'content.bin').exists())
 def test_pinned_directory_survives_replacement_and_ancestor_symlink(self):
  with tempfile.TemporaryDirectory() as tmp:
   base=Path(tmp);parent=base/'admitted';parent.mkdir();sink=t.FailureEvidence(parent,'failed-response-1')
   sink.capture(b'{}',b'{"x":',None,True)
   parent.rename(base/'original');outside=base/'other';outside.mkdir();os.symlink(outside,parent)
   sink.retain('content_parse');sink.clear()
   self.assertTrue((base/'original/failed-response-1/content.bin').exists())
   self.assertEqual(list(outside.iterdir()),[])
 def test_supported_configuration_and_unsupported_metadata(self):
  m=flow.x.json_output
  self.assertEqual(m.response_format('gpt-6-luna',m.CAPABILITY),{'type':'json_object'})
  for change in [{'model':'other'},{'endpoint':'/v1/responses'},{'features':[]}]:
   with self.assertRaises(ValueError):m.response_format('gpt-6-luna',{**m.CAPABILITY,**change})

 def test_real_serialized_requests_strict_transport_then_native_six_stage_flow(self):
  d,x=flow.d,flow.x
  class WireLedger(flow.Ledger):
   def preflight_run(self,**kw):
    if self.mode in ('corrected','review_corrected','reflection_shape_corrected'):
     assert kw=={'calls':7,'execution_groups':1}
     return {'status':{'calls':197,'execution_groups':57,'known_cost_microusd':163331}}
    return super().preflight_run(**kw)
   def run_call(self,*args,**kw):
    authored=super().run_call(*args,**kw)['value'];body=args[2]
    if self.mode=='native_invalid':authored['inquiry']['exploration']['considered_records']='R1'
    if self.mode=='reflection_shape_corrected' and self.calls==4:
     authored['protocol']=authored['response'].pop('protocol');authored['state_root']=authored['response'].pop('state_root');authored['stage']='reflect'
    envelope=base.Transport().envelope();envelope['model']='gpt-6-luna';envelope['choices'][0]['message']['content']=json.dumps(authored)
    if self.mode in ('corrected','review_corrected') and self.calls==(2 if self.mode=='review_corrected' else 1):envelope['choices'][0]['message']['content']='{bad'
    with patch.object(d.t.broker,'credential',return_value='FIXTURE_CREDENTIAL'),patch.object(d.t.urllib.request,'build_opener') as op:
     op.return_value.open.return_value=base.Response(json.dumps(envelope).encode())
     result=args[4](body,1)
     assert json.loads(op.return_value.open.call_args.args[0].data)==body
     if not (self.mode in ('corrected','review_corrected') and self.calls==(2 if self.mode=='review_corrected' else 1)):assert result['receipt']['structural_status']=='typed_json'
     return result
  for mode in ('complete','native_invalid','corrected','review_corrected','reflection_shape_corrected'):
   with tempfile.TemporaryDirectory() as tmp:
    root=Path(tmp)/'run';grant=flow.Validation().prepare(root);ledger=WireLedger(mode)
    if mode in ('corrected','review_corrected','reflection_shape_corrected'):
     g=x.read(grant);g.update(correction_allowance=1,max_calls=7,reservation_microusd=350000);grant.write_text(json.dumps(g))
    with patch.object(d,'ledger',return_value=ledger),patch.object(d.c,'official_preflight',return_value={'fixture_only':True}):
     result=d.run(root,grant)
     if mode in ('complete','corrected','review_corrected','reflection_shape_corrected'):self.assertEqual(result['reason'],'completed_next_preview_no_further_adoption',result)
     else:self.assertIn('response contract rejected',result['reason'])
     self.assertFalse(d.run(root,grant)['dispatch'])
    if mode=='complete':
     self.assertEqual((ledger.calls,ledger.ops),(6,1));self.assertFalse(list(root.glob('failed-response-*')))
    elif mode=='reflection_shape_corrected':
     self.assertEqual((ledger.calls,ledger.ops),(7,1))
     correction=json.loads(ledger.bodies[4]['messages'][-1]['content'])['protocol_correction']['native_structure']
     self.assertEqual(correction['path'],'$.protocol');self.assertEqual(correction['valid_locations'],['$.response.protocol'])
     self.assertEqual(ledger.bodies[4]['messages'][:-1],ledger.bodies[3]['messages'])
     self.assertIn('protocol',json.loads((root/'failed-response-4/content.bin').read_bytes()))
    elif mode in ('corrected','review_corrected'):
     self.assertEqual((ledger.calls,ledger.ops),(7,1))
     self.assertEqual((root/('failed-response-2' if mode=='review_corrected' else 'failed-response-1')/'content.bin').read_bytes(),b'{bad')
     self.assertEqual(x.read(root/'result-1.json')['receipt']['status'],'settled')
    else:
     self.assertEqual((ledger.calls,ledger.ops),(1,0))
     raw=(root/'failed-response-1/content.bin').read_bytes()
     self.assertEqual(json.loads(raw)['inquiry']['exploration']['considered_records'],'R1')
     self.assertEqual(x.read(root/'contract-rejection-1.json')['failure_evidence']['directory'],'failed-response-1')
if __name__=='__main__':unittest.main()
