"""AUTHORED local queue/restart/Stop tests; no provider or game effects."""
import json,tempfile,types,unittest
from pathlib import Path
from unittest.mock import patch
import server as ui
import context_refresh as refresh

class Refresh(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name)
  self.s=ui.Store(self.root/'state',[{'id':'fixture','source':str(self.root/'source')}]);self.p='fixture'
  self.packet=self.root/'packet';(self.packet/'cycle').mkdir(parents=True);(self.packet/'adapter').mkdir();(self.packet/'cycle/cycle.jsonl').touch()
  for n in ('adapter/launch.lock','cycle/run.lock'):(self.packet/n).touch()
  self.meta=self.s.meta(self.p);self.meta['registered_packet']=str(self.packet);ui.save(self.s.path(self.p)/'project.json',self.meta)
  self.s.budget=lambda:{'available':True,'stopped':False,'unlimited_policy_sha256':'policy'}
  self.s.append(self.p,{'id':'original','type':'input','text':'AUTHORED old intent'})
  self.s.append(self.p,{'id':'run','type':'run_request','input_ids':['original']})
  self.launch=self.s.append(self.p,{'id':'launch','type':'run_started','request_id':'run','packet':str(self.packet),'pid':99999999})
  self.evidence=self.s.append(self.p,{'id':'evidence','type':'assessment','state':'queued','text':'AUTHORED exact-source findings'})
  self.s.append(self.p,{'id':'draft','type':'input','text':'AUTHORED saved only'})
  self.ca={'authored':'admission'};self.a=types.SimpleNamespace(e=types.SimpleNamespace(sha=ui.sha))
  self.ar=([],{},{});self.cr=([],{},{});self.starts=[]
 def inspect(self,*_):return self.a,self.packet,self.ca,self.ar,self.cr
 def boundary(self):
  with patch.object(refresh,'inspect',self.inspect):refresh.tick(self.s,self.p)
  request=json.loads((self.packet/'cycle/CONTEXT_REFRESH').read_text())
  self.cr=([],{},{'terminal':{'status':'context_refresh_required','refresh':request,'refresh_sha256':ui.sha(request),'state_root':'root','source_snapshot':'source','scope':{'id':'scope'}}})
  return request
 def start(self,p,b):
  self.starts.append(b)
  self.s.append(p,{'id':b['id'],'type':'run_request','input_ids':['original','evidence']})
 def test_queued_evidence_single_request_no_draft_and_no_duplicate_launch(self):
  request=self.boundary();self.assertEqual(request['input_ids'],['evidence'])
  self.s.start=self.start
  with patch.object(refresh,'inspect',self.inspect),patch.object(refresh.os,'kill',side_effect=ProcessLookupError):
   refresh.tick(self.s,self.p);refresh.tick(self.s,self.p)
  self.assertEqual(len(self.starts),1)
  self.assertTrue(any(e['type']=='interaction_error' for e in self.s.events(self.p))) # authored start was interrupted before launch
 def test_pending_execution_or_assessment_never_launches(self):
  self.boundary();self.s.start=self.start;self.ar=([],{'assessment':'pending'},{})
  with patch.object(refresh,'inspect',self.inspect),patch.object(refresh.os,'kill',side_effect=ProcessLookupError):refresh.tick(self.s,self.p)
  self.assertFalse(self.starts)
 def test_user_stop_prevents_even_refresh_request(self):
  self.s.stop(self.p)
  self.s.controls=lambda p:[{'kind':'stop','request_id':'run'}]
  with patch.object(refresh,'inspect',self.inspect):refresh.tick(self.s,self.p)
  self.assertFalse((self.packet/'cycle/CONTEXT_REFRESH').exists())
 def test_existing_cycle_stop_does_not_write_context_control(self):
  (self.packet/'cycle/STOP').write_text('AUTHORED cooperative pause')
  with patch.object(refresh,'inspect',self.inspect):refresh.tick(self.s,self.p)
  self.assertFalse((self.packet/'cycle/CONTEXT_REFRESH').exists())
 def test_restart_scheduled_exact_source_verified_and_new_draft_excluded(self):
  self.boundary();request=self.cr[2]['terminal']['refresh'];rid='refresh-'+ui.sha(request)
  self.s.append(self.p,{'id':rid,'type':'context_refresh_scheduled','predecessor_launch':'launch','refresh_sha256':ui.sha(request),'terminal_sha256':ui.sha(self.cr[2]['terminal'])})
  self.s.native=lambda path:types.SimpleNamespace(budget=lambda:{},cli=lambda *args:{'state_root':'root','source_snapshot':'source','scope':{'id':'scope'}})
  with patch.object(refresh,'inspect',self.inspect):result=refresh.verify_successor(self.s,self.p,rid)
  self.assertEqual(result,{'predecessor':'run','input_ids':['original','evidence']})
  self.s.append(self.p,{'id':'newer','type':'assessment','state':'queued','text':'AUTHORED later findings'})
  with patch.object(refresh,'inspect',self.inspect):self.assertEqual(refresh.verify_successor(self.s,self.p,rid),result)
  with patch.object(refresh.os,'kill',side_effect=ProcessLookupError):
   with refresh.guard(self.s,self.p,rid) as guarded:self.assertEqual(guarded,rid)
  self.s.native=lambda path:types.SimpleNamespace(budget=lambda:{},cli=lambda *args:{'state_root':'changed','source_snapshot':'source','scope':{'id':'scope'}})
  with patch.object(refresh,'inspect',self.inspect),self.assertRaisesRegex(ValueError,'source changed'):refresh.verify_successor(self.s,self.p,rid)
 def test_live_process_and_unknown_native_effect_block_successor(self):
  request=self.boundary();rid='refresh-'+ui.sha(request)
  self.s.append(self.p,{'id':rid,'type':'context_refresh_scheduled','predecessor_launch':'launch','refresh_sha256':ui.sha(request),'terminal_sha256':ui.sha(self.cr[2]['terminal'])})
  with patch.object(refresh.os,'kill',return_value=None),self.assertRaisesRegex(ValueError,'still active'):
   with refresh.guard(self.s,self.p,rid):pass
  self.s.native=lambda path:types.SimpleNamespace(budget=lambda:{'unresolved_tool_requests':['AUTHORED']})
  with patch.object(refresh,'inspect',self.inspect),self.assertRaisesRegex(ValueError,'native effects'):refresh.verify_successor(self.s,self.p,rid)
 def test_submitted_input_added_once_saved_only_excluded(self):
  self.s.append(self.p,{'id':'submitted','type':'input_submitted','input_id':'draft'})
  self.assertEqual([e['id'] for e in refresh.queued(self.s,self.p,self.launch)],['evidence','draft'])
 def test_queue_arrives_after_committed_abstention(self):
  self.cr=([],{},{'terminal':{'status':'proposal_abstention','reason':'AUTHORED retain'}})
  self.s.native=lambda path:types.SimpleNamespace(budget=lambda:{},cli=lambda *args:{'branch':'propose','state_root':'root','source_snapshot':'source','scope':{'id':'scope'}})
  self.s.start=self.start
  with patch.object(refresh,'inspect',self.inspect),patch.object(refresh.os,'kill',side_effect=ProcessLookupError):refresh.tick(self.s,self.p)
  self.assertEqual(len(self.starts),1);self.assertEqual(self.cr[2]['terminal']['status'],'proposal_abstention')
  with patch.object(refresh,'inspect',self.inspect):
   self.assertEqual(refresh.verify_successor(self.s,self.p,self.starts[0]['id']),{'predecessor':'run','input_ids':['original','evidence']})
   self.cr[2]['terminal']['reason']='AUTHORED changed terminal'
   with self.assertRaisesRegex(ValueError,'Terminal changed'):refresh.verify_successor(self.s,self.p,self.starts[0]['id'])
 def test_changed_queue_proof_rejected(self):
  request=self.boundary();rid='refresh-'+ui.sha(request)
  self.s.append(self.p,{'id':rid,'type':'context_refresh_scheduled','predecessor_launch':'launch','refresh_sha256':ui.sha(request),'terminal_sha256':ui.sha(self.cr[2]['terminal'])})
  request['inputs_sha256']='f'*64;self.cr[2]['terminal']['refresh_sha256']=ui.sha(request)
  with patch.object(refresh,'inspect',self.inspect),self.assertRaises(ValueError):refresh.verify_successor(self.s,self.p,rid)
if __name__=='__main__':unittest.main()
