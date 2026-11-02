"""AUTHORED end-to-end interaction verification. No paid calls, Docker or game edits."""
import http.client,http.server,json,tempfile,threading,time,unittest
from pathlib import Path
from unittest.mock import patch
import server as ui
from fixture_server import FixtureStore

class Interaction(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory(dir=ui.ROOT/'target');self.addCleanup(self.tmp.cleanup);self.s=FixtureStore(Path(self.tmp.name))
  self.http=http.server.ThreadingHTTPServer(('127.0.0.1',0),ui.Handler);self.http.store=self.s;self.http.expected_host='127.0.0.1:'+str(self.http.server_port);self.http.origin='http://'+self.http.expected_host
  threading.Thread(target=self.http.serve_forever,daemon=True).start();self.addCleanup(self.http.server_close);self.addCleanup(self.http.shutdown)
 def request(self,path,value=None,headers=True):
  c=http.client.HTTPConnection('127.0.0.1',self.http.server_port,timeout=120)
  h={'Content-Type':'application/json','Origin':self.http.origin,'X-TSR-CSRF':self.s.csrf} if headers else {'Content-Type':'application/json'}
  c.request('GET' if value is None else 'POST',path,None if value is None else json.dumps(value),h);r=c.getresponse();raw=r.read();status=r.status;c.close();return status,json.loads(raw)
 def wait(self,p):
  t=self.s.preparing.get(p)
  if t:t.join(90);self.assertFalse(t.is_alive())
 def test_delivered_preview_is_allowlisted_sandboxed_and_read_only(self):
  self.s.catalog[0]['artifact']=str(self.s.authored_source/'preview')
  before=self.s.events('fixture')
  for name in ('index.html','app.js','style.css'):
   c=http.client.HTTPConnection('127.0.0.1',self.http.server_port)
   c.request('GET','/preview/fixture/'+name);r=c.getresponse();body=r.read();self.assertEqual(r.status,200)
   self.assertEqual(body,(self.s.authored_source/'preview'/name).read_bytes())
   csp=r.getheader('Content-Security-Policy');self.assertIn('sandbox allow-scripts;',csp);self.assertNotIn('allow-same-origin',csp);self.assertIn("connect-src 'none'",csp);self.assertIn(self.http.origin+'/preview/fixture/',csp);self.assertNotIn("script-src 'self'",csp);c.close()
  for path in ('/preview/fixture/../../server.py','/preview/fixture/SOURCE.json','/preview/missing/index.html'):
   self.assertEqual(self.request(path)[0],400)
  self.assertEqual(self.s.events('fixture'),before);self.assertEqual(self.s.calls,[]);self.assertFalse(self.s.approved)
 def test_http_saved_refusal_authored_approval_dispatch_reply_reload_and_no_repeat(self):
  status,result=self.request('/api/projects',{'id':'authored','title':'AUTHORED new project','text':'AUTHORED fixture: explore an original browser project.'});self.assertEqual(status,200);p=result['id']
  body={'id':'question','text':'AUTHORED question: what should happen next?','confirm':'Send to engine within $10'}
  status,result=self.request('/api/project/'+p+'/send',body,False);self.assertEqual(status,403)
  status,result=self.request('/api/project/'+p+'/send',body);self.assertEqual(result['state'],'approval_required');self.assertEqual(self.s.launch_count,0)
  self.request('/api/project/'+p+'/send',body);self.assertEqual(sum(e['id']=='question' and e['type']=='input' for e in self.s.events(p)),1)
  disclosure=result['recovery'];status,_=self.request('/api/project/'+p+'/recovery-accept',{'disclosure':disclosure['digest']});self.assertEqual(status,409);self.assertFalse(self.s.approved)
  status,refused=self.request('/api/project/'+p+'/recovery-refuse',{'id':'refusal','disclosure':disclosure['digest']});self.assertEqual(refused['state'],'not_approved')
  self.s.approve_fixture() # Authored trusted-approval state; not a product auth mechanism.
  status,result=self.request('/api/project/'+p+'/send',body);self.assertEqual(result['state'],'preparing');self.wait(p)
  view=self.request('/api/project/'+p)[1]
  self.assertEqual(self.s.launch_count,1);self.assertEqual(len(self.s.calls),1)
  replies=[e for e in view['timeline'] if e['kind']=='proposal'];self.assertEqual(len(replies),1);self.assertIn('AUTHORED TEST RESPONSE',replies[0]['text']);self.assertIn('question',replies[0]['input_ids'])
  self.assertTrue(replies[0]['admission_sha256']);self.assertIn('question',view['delivery'])
  self.request('/api/project/'+p+'/send',body);self.wait(p);self.assertEqual(self.s.launch_count,1)
  self.s.input(p,{'id':'later','text':'AUTHORED later message, not answered yet.'});view=self.request('/api/project/'+p)[1];self.assertNotIn('later',replies[0]['input_ids']);self.assertNotIn('later',view['delivery'])
  reboot=ui.Store(self.s.root,self.s.catalog);reboot.ledger=self.s.ledger;self.assertTrue(reboot.view(p)['timeline']);self.assertFalse(reboot.jobs)
  result=reboot.start(p,{'id':'send-question','confirm':'Run this reviewed session with up to $10'});self.assertTrue(result['replayed']);self.assertEqual(len(self.s.calls),1)
 def test_authored_in_app_decision_to_native_response_and_restart(self):
  from decisions import Decisions
  class AuthoredDecisions(Decisions):
   def disclosure(this,p):
    events=this.store.events(p)
    return {'fixture_only':True,'project':p,'source':this.store.source_snapshot(p),'events_sha256':ui.sha(events),'resume_input_id':'human-message','input_ids':['human-message'],'text':'AUTHORED uncertain charge disclosure — no real money'}
   def apply(this,d):this.store.approve_fixture()
  self.s.decisions=AuthoredDecisions(self.s)
  p='fixture';base='/api/project/'+p
  self.request(base+'/send',{'id':'human-message','text':'AUTHORED feedback','confirm':'Send to engine within $10'})
  q=self.request(base+'/decision-present',{})[1]
  body={'choice':'approve','challenge':q['challenge'],'digest':q['digest']}
  self.assertEqual(self.request(base+'/decision',body,False)[0],403)
  status,result=self.request(base+'/decision',body);self.assertEqual(status,200,result);self.wait(p)
  self.assertEqual(self.s.launch_count,1);self.assertEqual(self.request(base+'/decision',body)[1]['state'],'recorded')
  view=self.request(base)[1];self.assertTrue(any(e['kind']=='proposal' for e in view['timeline']))
  self.s.input(p,{'id':'later-saved','text':'AUTHORED later input, not submitted'})
  self.s.decisions.resume_recorded();self.wait(p);self.assertEqual(self.s.launch_count,1)
 def test_stop_during_prepare_is_durable_and_never_launches(self):
  self.s.approve_fixture();entered=threading.Event();release=threading.Event()
  from prepare import prepare as actual
  def paused(*args,**kwargs):
   packet=actual(*args,**kwargs);entered.set();self.assertTrue(release.wait(30));return packet
  with patch('prepare.prepare',side_effect=paused):
   self.s.send('fixture',{'id':'stop','text':'AUTHORED stop test','confirm':'Send to engine within $10'});
   for _ in range(180):
    if entered.wait(.5):break
    if not self.s.preparing['fixture'].is_alive():break
   self.assertTrue(entered.is_set(),self.s.events('fixture'))
   queued=self.s.input('fixture',{'id':'during','text':'AUTHORED concurrent input'});self.assertEqual(queued['state'],'queued')
   self.s.stop('fixture');release.set();self.wait('fixture')
  self.assertEqual(self.s.launch_count,0);self.assertEqual(self.s.calls,[]);self.assertTrue(self.s.controls('fixture'))
  self.assertTrue(any('Stopped before' in e.get('text','') for e in self.s.events('fixture')))
 def test_stop_before_worker_begins_cannot_launch(self):
  self.s.approve_fixture();deferred=[]
  class Deferred:
   def __init__(self,target,daemon):self.target=target;deferred.append(self)
   def start(self):pass
   def is_alive(self):return False
  with patch('server.threading.Thread',Deferred):self.s.send('fixture',{'id':'scheduled','text':'AUTHORED scheduled stop','confirm':'Send to engine within $10'})
  self.s.stop('fixture');deferred[0].target()
  self.assertEqual(self.s.launch_count,0);self.assertEqual(self.s.calls,[])
  self.assertEqual(self.s.controls('fixture')[-1]['request_id'],'send-scheduled')
 def test_concurrent_input_at_preparation_commit_preserves_chain_and_frozen_context(self):
  self.s.approve_fixture();entered=threading.Event();release=threading.Event();late_done=threading.Event();original=self.s.append
  def append(p,event):
   if event['type']=='native_input' and event['id']=='first':
    self.assertEqual(event['source'],'source-bound UI session preparation');entered.set();self.assertTrue(release.wait(30))
   return original(p,event)
  with patch.object(self.s,'append',side_effect=append):
   # Author the queued/inactive intake condition explicitly, instead of relying
   # on a historical Pond episode's eligibility. Preparation still admits real
   # native feedback and emits the actual source-bound commit event above.
   with patch.object(self.s,'admit_input',return_value={'state':'queued','reason':'AUTHORED intake waits for session preparation'}):
    self.s.send('fixture',{'id':'first','text':'AUTHORED frozen first input','confirm':'Send to engine within $10'})
   self.assertTrue(entered.wait(90))
   late=threading.Thread(target=lambda:(self.s.input('fixture',{'id':'late','text':'AUTHORED concurrent input'}),late_done.set()));late.start()
   self.assertFalse(late_done.wait(.2));release.set();late.join(30);self.assertTrue(late_done.is_set());self.wait('fixture')
  events=self.s.events('fixture');self.assertEqual(sum(e['type']=='input' and e['id']=='late' for e in events),1)
  packet=Path(self.s.meta('fixture')['registered_packet']);ids=json.loads((packet/'scope-authority.json').read_text())['input_ids']
  self.assertIn('first',ids);self.assertNotIn('late',ids)
 def test_nonfinancial_blockers_do_not_offer_risk_approval(self):
  for value,kind in [({'available':False,'stopped':True},'accounting_unavailable'),({'available':True,'stopped':True,'unknown_executions':['AUTHORED']},'unknown_execution'),({'available':True,'stopped':True,'overrun':True},'budget_exceeded')]:
   with patch.object(self.s,'budget',return_value=value):
    d=self.s.recovery_disclosure();self.assertEqual(d['blocker'],kind);self.assertFalse(d['can_request'])
if __name__=='__main__':unittest.main()
