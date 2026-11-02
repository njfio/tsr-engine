"""Authored pre-dispatch failures; no provider or worker execution."""
import contextlib,json,sys,types,unittest
from pathlib import Path
from unittest.mock import patch
import test_server
import server as ui
class Startup(unittest.TestCase):
 def setUp(self):
  test_server.Unit.setUp(self);self.s.runtime_preflight=lambda:{'fixture_only':True};self.rid='send-authored';self.packet=self.s.path('test')/'sessions/authored';self.packet.mkdir(parents=True)
  self.a=ui.load('startup_test_adapter','engine-browser-adapter.py');self.ad={'directory':str(self.packet/'adapter')};Path(self.ad['directory']).mkdir()
  self.rows=[self.a.d.Driver.row([],{'kind':'admission','value':self.ad,'fixture_only':True})];self.write()
  ui.save(self.packet/'adapter-admission.json',self.ad);ca={'journal':str(self.packet/'cycle')};ui.save(self.packet/'cycle-admission.json',ca);ui.save(self.packet/'scope-grant.json',{});ui.save(self.packet/'prepared-manifest.json',{'adapter_sha256':self.a.e.sha(self.ad),'cycle_sha256':self.a.e.sha(ca),'grant_sha256':self.a.e.sha({})})
  q=self.s.native(self.s.project_native('test',create=True)).request();ui.save(self.packet/'scope-authority.json',{'source_root':q['state_root'],'source_snapshot':q['coding']['snapshot']})
  meta=self.s.meta('test');meta['registered_packet']=str(self.packet);ui.save(self.s.path('test')/'project.json',meta)
  self.entries=[{'hash':'head','event':{'kind':'initial'}}]
  @contextlib.contextmanager
  def locked():yield None,self.entries
  self.s.ledger=lambda:({'fixture_only':True},types.SimpleNamespace(digest=self.a.e.sha),types.SimpleNamespace(locked=locked),{})
  self.s.append('test',{'id':self.rid,'type':'run_request'})
  self.s.append('test',{'id':'launch','type':'run_started','request_id':self.rid,'pid':99999999,'packet':str(self.packet)})
 def write(self):(Path(self.ad['directory'])/'adapter.jsonl').write_text('\n'.join(json.dumps(x) for x in self.rows)+'\n')
 def test_current_interpreter(self):self.assertEqual(self.s.run_command(self.packet)[0],sys.executable)
 def test_dead_virgin_recovery_once(self):
  with patch('server.os.kill',side_effect=ProcessLookupError),patch.object(self.s,'launch',return_value=types.SimpleNamespace(pid=99999998)) as launch:
   self.assertEqual(self.s.resume_unstarted('test',self.rid)['state'],'running');launch.assert_called_once()
   with self.assertRaisesRegex(ValueError,'already used'):self.s.resume_unstarted('test',self.rid)
 def test_live_child_blocks(self):
  with patch('server.os.kill',return_value=None):
   with self.assertRaisesRegex(ValueError,'still present'):self.s.unstarted_packet('test',self.rid)
 def test_any_intent_blocks(self):
  self.rows.append(self.a.d.Driver.row(self.rows,{'kind':'intent','name':'dispatch_request'}));self.write()
  with patch('server.os.kill',side_effect=ProcessLookupError):
   with self.assertRaisesRegex(ValueError,'durable stages'):self.s.unstarted_packet('test',self.rid)
 def test_enrolled_blocks(self):
  trial=self.a.e.sha({'trial':str(Path(self.ad['directory']).resolve()),'admission':self.a.e.sha(self.ad)})
  self.entries.append({'hash':'enroll','event':{'key':trial}})
  with patch('server.os.kill',side_effect=ProcessLookupError):
   with self.assertRaisesRegex(ValueError,'enrollment'):self.s.unstarted_packet('test',self.rid)
 def test_changed_manifest_blocks(self):
  ui.save(self.packet/'scope-grant.json',{'changed':True})
  with patch('server.os.kill',side_effect=ProcessLookupError):
   with self.assertRaisesRegex(ValueError,'binding changed'):self.s.unstarted_packet('test',self.rid)
 def test_stop_blocks(self):
  self.s.controls=lambda p:[{'kind':'stop','request_id':self.rid}]
  with patch('server.os.kill',side_effect=ProcessLookupError),patch.object(self.s,'launch') as launch:
   with self.assertRaisesRegex(ValueError,'Stop'):self.s.resume_unstarted('test',self.rid)
   launch.assert_not_called()
if __name__=='__main__':unittest.main()

class Settled(Startup):
 def setUp(self):
  super().setUp();self.s.budget=lambda:{'stopped':False};ca=json.loads((self.packet/'cycle-admission.json').read_text());(self.packet/'cycle').mkdir();self.cycle_file=self.packet/'cycle/cycle.jsonl';self.cycle_rows=[self.a.d.Driver.row([],{'kind':'admission','value':ca,'fixture_only':True})];self.write_cycle()
 def write_cycle(self):self.cycle_file.write_text('\n'.join(json.dumps(x) for x in self.cycle_rows)+'\n')
 def test_settled_resume_retains_journal_and_tracks_stop(self):
  before=self.cycle_file.read_bytes()
  with patch('server.os.kill',side_effect=ProcessLookupError),patch.object(self.s,'launch',return_value=types.SimpleNamespace(pid=99999997)) as launch:
   self.assertEqual(self.s.resume_settled('test',self.rid)['state'],'running');self.assertEqual(self.cycle_file.read_bytes(),before);self.assertEqual(self.s.jobs['test'].pid,99999997);launch.assert_called_once()
 def test_pending_cycle_blocks(self):
  self.cycle_rows.append(self.a.d.Driver.row(self.cycle_rows,{'kind':'intent','name':'proposal_0'}));self.write_cycle()
  with patch('server.os.kill',side_effect=ProcessLookupError):
   with self.assertRaisesRegex(ValueError,'Unresolved'):self.s.settled_packet('test',self.rid)
 def test_terminal_blocks(self):
  intent={'kind':'intent','name':'terminal'};self.cycle_rows.append(self.a.d.Driver.row(self.cycle_rows,intent));self.cycle_rows.append(self.a.d.Driver.row(self.cycle_rows,{'kind':'result','name':'terminal','intent':self.a.e.sha(intent),'value':{'status':'abstain'}}));self.write_cycle()
  with patch('server.os.kill',side_effect=ProcessLookupError):
   with self.assertRaisesRegex(ValueError,'Completed'):self.s.settled_packet('test',self.rid)
 def test_reserved_resume_restart_cannot_launch(self):
  self.s.append('test',{'id':'reserved','type':'settled_resume','prior_launch':'launch'})
  with patch('server.os.kill',side_effect=ProcessLookupError):
   with self.assertRaisesRegex(ValueError,'already reserved'):self.s.settled_packet('test',self.rid)
 def test_settled_stop(self):
  self.s.controls=lambda p:[{'kind':'stop','request_id':self.rid}]
  with patch('server.os.kill',side_effect=ProcessLookupError),patch.object(self.s,'launch') as launch:
   with self.assertRaisesRegex(ValueError,'Stop'):self.s.resume_settled('test',self.rid)
   launch.assert_not_called()
