"""AUTHORED unlimited UI/native integration; no paid provider or Docker calls."""
import copy,json,tempfile,unittest
from pathlib import Path
import server as ui
from fixture_server import FixtureStore
class UnlimitedStore(FixtureStore):
 def __init__(self,root):
  super().__init__(root,approved=True);m=self.campaign
  with self.account.locked() as (_,rows):head=rows[-1]['hash']
  self.account.enable_unlimited({'version':'browser-unlimited-policy@v1','manifest_sha256':self.account.hash,'ledger_prefix_head':head,'provider':'CheaperInference','model':m.MODEL,'source_class':'newly-authored-nonprivate-browser-project','authority':copy.deepcopy(m.UNLIMITED_AUTHORITY),'quarantined_unknown_keys':[]})
 def launch(self,cmd,log,packet):
  grant=json.loads((packet/'scope-grant.json').read_text());self.account.grant_calls(grant,self.campaign.digest(grant))
  return super().launch(cmd,log,packet)
class UnlimitedUI(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory(dir=ui.ROOT/'target');self.addCleanup(self.tmp.cleanup);self.s=UnlimitedStore(Path(self.tmp.name))
 def wait(self):
  t=self.s.preparing.get('fixture')
  if t:t.join(100);self.assertFalse(t.is_alive())
 def test_send_native_response_restart_and_single_dispatch(self):
  body={'id':'unlimited-input','text':'AUTHORED fixture: consider this input and retain if appropriate.','confirm':'Send to engine under unlimited policy'}
  self.assertFalse(self.s.budget()['stopped']);self.s.send('fixture',body);self.wait()
  errors=[e for e in self.s.events('fixture') if e['type']=='interaction_error'];self.assertEqual(errors,[])
  self.assertEqual(len(self.s.calls),1,(self.s.path('fixture')/'run.log').read_text());self.assertEqual(self.s.launch_count,1)
  packet=Path(self.s.meta('fixture')['registered_packet']);ad=json.loads((packet/'adapter-admission.json').read_text());self.assertIsNone(ad['max_calls']);self.assertIsNone(ad['expires_at'])
  budget=self.s.native(self.s.project_native('fixture')).budget();self.assertTrue(budget['unlimited_mode']);self.assertEqual(budget['unlimited_policy_sha256'],ad['unlimited_policy_sha256'])
  self.assertTrue(self.s.view('fixture')['delivery'].get('unlimited-input'));self.s.send('fixture',body);self.wait();self.assertEqual(len(self.s.calls),1)
  reboot=ui.Store(self.s.root,self.s.catalog);reboot.ledger=self.s.ledger
  self.assertTrue(reboot.start('fixture',{'id':'send-unlimited-input','confirm':'Run under the active unlimited policy'})['replayed'])
  self.assertEqual(self.s.stop('fixture')['state'],'not_running');self.assertEqual(len(self.s.calls),1)
 def test_new_message_after_retention_has_its_own_response_and_deduplication(self):
  first={'id':'first-retention','text':'AUTHORED first input','confirm':'Send to engine under unlimited policy'}
  second={'id':'after-retention','text':'AUTHORED genuine later input','confirm':'Send to engine under unlimited policy'}
  self.s.send('fixture',first);self.wait()
  first_view=self.s.view('fixture');self.assertEqual(first_view['timeline'][-1]['outcome'],'proposal_abstention')
  self.s.send('fixture',second);self.wait();view=self.s.view('fixture')
  self.assertEqual(len(self.s.calls),2);self.assertEqual(self.s.launch_count,2)
  self.assertIn(second['id'],view['timeline'][-1]['input_ids']);self.assertIn(second['id'],view['delivery'])
  self.assertNotIn(second['id'],first_view['timeline'][-1]['input_ids'])
  self.s.send('fixture',second);self.wait();self.assertEqual(len(self.s.calls),2)
  self.assertEqual(sum(e['type']=='input' and e['id']==second['id'] for e in self.s.events('fixture')),1)
 def test_failed_worker_is_visible_without_a_fabricated_response(self):
  import types
  self.s.jobs['fixture']=types.SimpleNamespace(poll=lambda:17)
  view=self.s.view('fixture')
  self.assertIn('exit 17',view['worker_error']);self.assertFalse(view['running'])
  self.assertFalse(any(e['kind']=='decision' and e.get('current_session') for e in view['timeline']));self.assertEqual(self.s.calls,[])
 def test_preparation_preserves_existing_unlimited_policy(self):
  from prepare import prepare
  first=prepare(self.s,'fixture');before=self.s.native(self.s.project_native('fixture')).budget()
  second=prepare(self.s,'fixture');after=self.s.native(self.s.project_native('fixture')).budget()
  self.assertNotEqual(first,second);self.assertEqual(before['unlimited_policy_sha256'],after['unlimited_policy_sha256']);self.assertTrue(after['unlimited_mode']);self.assertEqual(self.s.calls,[])
 def test_independent_visual_evidence_is_not_new_user_desire(self):
  import hashlib
  from prepare import prepare
  q=self.s.native(self.s.project_native('fixture',create=True)).request();text='AUTHORED independent visual observation, not player feedback.'
  event={'id':'authored-visual','type':'assessment','role':'reviewer','text':text,'attribution':'independent_visual_review','report_sha256':hashlib.sha256(text.encode()).hexdigest(),'source_snapshot':{'native_snapshot':q['coding']['snapshot']}}
  out=prepare(self.s,'fixture',input_events=[event]);ad=json.loads((out/'adapter-admission.json').read_text());self.assertIn(text,ad['origin']['public']['public_brief']);self.assertIn('independent_assessment',ad['origin']['public']['public_brief'])
  self.assertFalse(any(text in x['meaning'] for x in ad['origin']['public']['parents']));self.assertEqual(self.s.calls,[])
  self.assertFalse(any(e['type']=='native_input' and e['id']==event['id'] for e in self.s.events('fixture')))
  event['report_sha256']='0'*64
  with self.assertRaisesRegex(ValueError,'provenance'):prepare(self.s,'fixture',input_events=[event])
  event['report_sha256']=hashlib.sha256(text.encode()).hexdigest();event['id']='authored-visual-other-source';event['source_snapshot']['native_snapshot']='0'*64
  stale=prepare(self.s,'fixture',input_events=[event]);self.assertIn('historical_source_not_current_validation',json.loads((stale/'adapter-admission.json').read_text())['origin']['public']['public_brief'])
 def test_gameplay_assessment_preserves_exact_content_and_desires(self):
  import hashlib
  from prepare import prepare
  q=self.s.native(self.s.project_native('fixture',create=True)).request()
  text='{"attribution":"AUTHORED fixture, not player preference","checks":[{"name":"control test","pass":true}]}'
  event={'id':'authored-gameplay','type':'assessment','role':'reviewer','text':text,'attribution':'independent_gameplay_assessment','report_sha256':hashlib.sha256(text.encode()).hexdigest(),'source_snapshot':{'native_snapshot':q['coding']['snapshot']}}
  out=prepare(self.s,'fixture',input_events=[event]);ad=json.loads((out/'adapter-admission.json').read_text())
  projected=json.loads(ad['origin']['public']['public_brief'].rsplit('Recorded input:\n',1)[1])[0]
  self.assertEqual(projected['text'],text);self.assertEqual(projected['attribution'],'independent_gameplay_assessment')
  self.assertFalse(any(text in x['meaning'] for x in ad['origin']['public']['parents']));self.assertEqual(self.s.calls,[])
 def test_limited_confirmation_cannot_silently_start_unlimited(self):
  with self.assertRaisesRegex(ValueError,'confirmation'):self.s.send('fixture',{'id':'x','text':'AUTHORED','confirm':'Send to engine within $10'})
  self.assertEqual(self.s.calls,[])
if __name__=='__main__':unittest.main()
