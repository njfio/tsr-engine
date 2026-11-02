"""Authored local head/transition fixtures: no model, worker or gameplay execution."""
import copy,json,sqlite3,tempfile,unittest
from pathlib import Path
from recovery import advance_capture,native_head
class AdvanceRecovery(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.p=Path(self.tmp.name);(self.p/'native').mkdir()
  self.db=sqlite3.connect(self.p/'native/events.sqlite');self.addCleanup(self.db.close)
  self.db.execute('CREATE TABLE events(seq INTEGER PRIMARY KEY,id TEXT,payload TEXT)')
  self.db.execute('INSERT INTO events VALUES(1,?,?)',('authored-continuation',json.dumps({'id':'authored-continuation'})));self.db.commit()
  self.proof={'native_head':native_head(self.p)};self.intent={'name':'advance_1'}
  response={'authored':'unexecuted candidate'}
  self.cd={'proposal_1':{'response':response},'callback_input_review_1':{'kind':'review','response':response,'context':{'source':'authored'}},'review_1':{'judgment':{'decision':'approve'}}}
  self.ad={'session_continue_0':{'result':{'continuation':'authored-continuation'}}}
 def verify(self):return advance_capture(self.p,self.proof,self.ad,self.cd,self.intent)
 def test_unchanged_pre_advance_head_survives_restart(self):
  self.assertEqual(self.verify(),self.cd['callback_input_review_1'])
  self.assertEqual(native_head(self.p),self.proof['native_head'])
  self.assertEqual(self.verify(),self.cd['callback_input_review_1'])
 def test_committed_transition_or_external_append_refused(self):
  self.db.execute('INSERT INTO events VALUES(2,?,?)',('authored-committed',json.dumps({'id':'authored-committed'})));self.db.commit()
  with self.assertRaisesRegex(ValueError,'may have committed'):self.verify()
 def test_payload_index_substitution_refused(self):
  self.db.execute('UPDATE events SET payload=?',(json.dumps({'id':'substitute'}),));self.db.commit()
  with self.assertRaisesRegex(ValueError,'identity'):self.verify()
 def test_missing_or_stale_historical_witness_refused(self):
  self.ad={}
  with self.assertRaisesRegex(ValueError,'authenticated'):self.verify()
 def test_missing_review_or_changed_proposal_refused(self):
  self.cd['review_1']['judgment']['decision']='reject'
  with self.assertRaisesRegex(ValueError,'proposal/review'):self.verify()
  self.cd['review_1']['judgment']['decision']='approve';self.cd['proposal_1']={'response':{'different':'candidate'}}
  with self.assertRaisesRegex(ValueError,'proposal/review'):self.verify()
 def test_wrong_stage_cannot_be_reclassified(self):
  self.intent={'name':'resume_1'}
  with self.assertRaisesRegex(ValueError,'ordinary interrupted advance'):self.verify()
class RecoveryStopBoundary(unittest.TestCase):
 def test_stop_during_verification_blocks_new_attempt_before_preparation(self):
  from unittest.mock import patch
  from fixture_server import FixtureStore
  with tempfile.TemporaryDirectory() as tmp:
   s=FixtureStore(Path(tmp));s.approve_fixture();packet=Path(tmp)/'prior';(packet/'cycle').mkdir(parents=True)
   s.append('fixture',{'id':'old-launch','type':'run_started','packet':str(packet),'request_id':'old-request','pid':999999})
   s.active_attempts['fixture']='old-request'
   def checking(*args):
    # Authored concurrent Stop's durable control phase. The real handler then
    # waits for the project lock; it does not re-enter _start on the same thread.
    import server as ui
    with s.control('fixture'):
     event={'kind':'stop','request_id':'old-request','at':ui.now()}
     row={'previous':None,'event':event};row['hash']=ui.sha(row)
     (s.path('fixture')/'controls.jsonl').write_text(json.dumps(row)+'\n')
   with patch.object(s,'require_prior_terminal',side_effect=checking),patch.object(s,'runtime_preflight',return_value={}),patch('prepare.prepare') as prepare:
    result=s._start('fixture',{'id':'new-recovery','confirm':'Run this reviewed session with up to $10'},str(packet))
   self.assertEqual(result['state'],'stopped');prepare.assert_not_called()
   self.assertFalse(any(e['type']=='run_request' and e['id']=='new-recovery' for e in s.events('fixture')))
   self.assertEqual(s.launch_count,0)


class FullRetainedPrefix(unittest.TestCase):
 """Preserved authenticated journals; authored native/ledger faults, no effects."""
 @classmethod
 def setUpClass(cls):
  import server
  cls.root=server.ROOT
  marker=cls.root/'target/studio-context-result/recovery-packet.txt'
  if not marker.exists():raise unittest.SkipTest('optional preserved interrupted advance unavailable')
  cls.packet=Path(marker.read_text());cls.current=json.loads((cls.root/'target/native-latency-repair/before.stdout').read_text())
 def setUp(self):
  import types,server
  a=server.load('advance_prefix_test','engine-browser-adapter.py');q=self.current['request'];packet=self.packet
  self.proof={'version':'native-advance-not-committed@v1','packet':str(packet),'pid':70279,'native_head':native_head(packet),'resource_root':self.current['resource_root'],'source_root':q['state_root'],'snapshot':q['coding']['snapshot'],'files':{x:a.d.file_sha(packet/x) for x in ('adapter/adapter.jsonl','cycle/cycle.jsonl','adapter-admission.json','cycle-admission.json','prepared-manifest.json','scope-grant.json')}}
  self.budget={k:0 for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests')}
  self.status={'blocking_unknown_calls':[],'blocking_unknown_executions':[]}
  self.native=types.SimpleNamespace(budget=lambda:self.budget,request=lambda:q,cli=lambda *args:self.current)
  self.store=types.SimpleNamespace(meta=lambda p:{'registered_packet':str(packet)},events=lambda p:[{'type':'run_started','packet':str(packet),'pid':70279,'request_id':'authored-recovery-test'}],controls=lambda p:[],ledger=lambda:(None,None,types.SimpleNamespace(status=lambda:self.status),None),native=lambda p:self.native)
 def verify(self):
  from recovery import verify
  from unittest.mock import patch
  with patch('recovery.os.kill',side_effect=ProcessLookupError):return verify(self.store,'pond-courier',self.packet,self.proof)
 def test_verified_prefix_without_mutation_and_restart(self):
  before={p:(self.packet/p).read_bytes() for p in self.proof['files']}
  self.assertEqual(self.verify()['state'],'verified_native_advance_not_committed');self.assertFalse(self.verify()['redispatched'])
  self.assertEqual(before,{p:(self.packet/p).read_bytes() for p in before})
 def test_wrong_resource_history_refused(self):
  self.proof['resource_root']='0'*64
  with self.assertRaisesRegex(ValueError,'Resource history'):self.verify()
 def test_head_change_during_authoritative_verification_refused(self):
  from unittest.mock import patch
  head=self.proof['native_head']
  with patch('recovery.native_head',side_effect=[head,{'sequence':head['sequence']+1,'event':'authored-new-head'}]):
   with self.assertRaisesRegex(ValueError,'changed during verification'):self.verify()
 def test_external_and_native_unknown_refused(self):
  self.status['blocking_unknown_executions']=['authored-unknown']
  with self.assertRaisesRegex(ValueError,'Unknown external'):self.verify()
  self.status['blocking_unknown_executions']=[];self.budget['unknown_usage']=1
  with self.assertRaisesRegex(ValueError,'Unknown native'):self.verify()
 def test_prefix_tamper_and_live_worker_refused(self):
  from unittest.mock import patch
  from recovery import verify
  with patch('recovery.os.kill',return_value=None):
   with self.assertRaisesRegex(ValueError,'still exists'):verify(self.store,'pond-courier',self.packet,self.proof)
  self.proof['files']['cycle/cycle.jsonl']='0'*64
  with self.assertRaisesRegex(ValueError,'prefix changed'):self.verify()

if __name__=='__main__':unittest.main()
