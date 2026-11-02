"""Authored local UI tests. No provider requests, game execution, or campaign writes."""
import copy,http.client,http.server,json,sys,tempfile,threading,unittest
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0,str(Path(__file__).parent))
import server as ui
from prepare import prepare
class Native:
 def __init__(self,path):self.directory=path;self.file=path/'fake.json';path.mkdir(parents=True,exist_ok=True)
 def request(self):
  parents=json.loads(self.file.read_text()) if self.file.exists() else [{'requirement':'play','meaning':'Original want'}]
  return {'state_root':ui.sha(parents),'coding':{'snapshot':'same-source','relational':{'inputs':{'parents':parents,'returns':[]},'history':[]}}}
 def cli(self,*args):return {'requirements_revision':'r','decision_record':'d'}
 def json_cli(self,args,value):self.file.write_text(json.dumps([{'requirement':k,'meaning':v} for k,v in value['guidance'].items()]));return {}
class Unit(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);root=Path(self.tmp.name);(root/'source/native').mkdir(parents=True)
  self.s=ui.Store(root/'state',[{'id':'test','title':'Authored fixture','source':str(root/'source'),'artifact':None}],Native)
  self.s.budget=lambda:{'available':True,'stopped':True,'unknown_calls':['authored-unknown']}
 def test_versioned_delivery_is_exact_and_read_only(self):
  import hashlib
  root=Path(self.tmp.name).resolve();base=root/'artifacts/candidate';base.mkdir(parents=True);content=b'<!doctype html><title>AUTHORED</title>';(base/'index.html').write_bytes(content)
  self.s.append('test',{'id':'v2','type':'artifact_delivered','artifact':'artifacts/candidate','source_snapshot':'authored','source_sha256':{'index.html':hashlib.sha256(content).hexdigest()}})
  before=self.s.events('test')
  with patch.object(ui,'ROOT',root):
   self.assertEqual(self.s.delivered_preview('test','v2','index.html'),content)
   for version,name in [('unknown','index.html'),('v2','../secret'),('../v2','index.html')]:
    with self.assertRaises(ValueError):self.s.delivered_preview('test',version,name)
   (base/'index.html').write_text('changed')
   with self.assertRaisesRegex(ValueError,'changed'):self.s.delivered_preview('test','v2','index.html')
  self.assertEqual(self.s.events('test'),before)
 def test_delivery_cannot_export_outside_artifacts(self):
  self.s.append('test',{'id':'bad','type':'artifact_delivered','artifact':'ui/engine','source_snapshot':'authored','source_sha256':{}})
  with self.assertRaisesRegex(ValueError,'boundary'):self.s.delivered_preview('test','bad','index.html')
 def test_settled_pause_requires_exact_dead_worker_and_no_unknowns(self):
  import types
  packet=Path(self.tmp.name)/'packet';(packet/'adapter').mkdir(parents=True);(packet/'cycle').mkdir()
  for f in ('adapter/launch.lock','cycle/run.lock'):(packet/f).touch()
  ad={'directory':str(packet/'adapter')};ca={'journal':str(packet/'cycle')}
  (packet/'adapter-admission.json').write_text(json.dumps(ad));(packet/'cycle-admission.json').write_text(json.dumps(ca))
  stop={'kind':'operator_checkpoint_pause','admission_sha256':ui.sha(ad),'worker_pid':99999999,'final_user_acceptance':False};(packet/'cycle/STOP').write_text(json.dumps(stop))
  self.s.append('test',{'id':'launch','type':'run_started','packet':str(packet),'pid':99999999})
  read=__import__('unittest.mock',fromlist=['Mock']).Mock(return_value=([],{},{}));driver=types.SimpleNamespace(read=read,row=None);a=types.SimpleNamespace(d=types.SimpleNamespace(Driver=driver),e=types.SimpleNamespace(sha=ui.sha),base=types.SimpleNamespace(capacity=types.SimpleNamespace(JOURNAL_BYTES=10000)))
  status={'blocking_unknown_calls':[],'blocking_unknown_executions':[]};ledger=types.SimpleNamespace(status=lambda:status);self.s.ledger=lambda:({'fixture_only':True},None,ledger,None)
  with patch.object(ui,'load',return_value=a),patch.object(ui.os,'kill',side_effect=ProcessLookupError):
   self.assertEqual(self.s.require_prior_terminal('test')['state'],'settled_operator_checkpoint')
   status['blocking_unknown_executions']=['authored-unknown']
   with self.assertRaisesRegex(ValueError,'external'):self.s.require_prior_terminal('test')
   status['blocking_unknown_executions']=[];read.return_value=([],{'pending':{}},{})
   with self.assertRaisesRegex(ValueError,'unresolved'):self.s.require_prior_terminal('test')
   read.return_value=([],{},{});stop['admission_sha256']='wrong';(packet/'cycle/STOP').write_text(json.dumps(stop))
   with self.assertRaisesRegex(ValueError,'Unverified'):self.s.require_prior_terminal('test')
   stop['admission_sha256']=ui.sha(ad);(packet/'cycle/STOP').write_text(json.dumps(stop))
  with patch.object(ui,'load',return_value=a),patch.object(ui.os,'kill',return_value=None):
   with self.assertRaisesRegex(ValueError,'still present'):self.s.require_prior_terminal('test')
  self.assertEqual(json.loads((packet/'cycle/STOP').read_text()),stop)
 def test_input_native_idempotent(self):
  first=self.s.input('test',{'id':'a','text':'Authored feedback'});root=first['native_root'];second=self.s.input('test',{'id':'a','text':'Authored feedback'})
  self.assertEqual(first,second);self.assertEqual(len(self.s.events('test')),2);self.assertFalse(first['model_consumed']);self.assertNotEqual(root,'same-source')
 def test_inactive_source_saves_once_without_claiming_native_admission(self):
  c=self.s.native(self.s.project_native('test',create=True));c.file.write_text('[]')
  first=self.s.input('test',{'id':'inactive','text':'AUTHORED pending feedback'})
  self.assertEqual(first['state'],'queued');self.assertIn('Saved safely',first['reason'])
  self.assertEqual(first,self.s.input('test',{'id':'inactive','text':'AUTHORED pending feedback'}))
  self.assertEqual(len(self.s.events('test')),1);self.assertEqual(c.file.read_text(),'[]')
 def test_historical_projection_is_bound_and_never_activates(self):
  c=self.s.native(self.s.project_native('test',create=True));c.file.write_text('[]');q=c.request()
  history={'version':'published-guidance-history@v1','state_root':q['state_root'],'current_source':q['coding']['snapshot'],'publications':[{'latest':True,'contributors':[{'meaning':'AUTHORED retained desire','eligible':False}]}]}
  with patch.object(Native,'cli',return_value=history):
   v=self.s.view('test');self.assertEqual(v['state']['published_history'],history);self.assertEqual(v['state']['parents'],[])
  self.assertEqual(c.file.read_text(),'[]');self.assertEqual(self.s.events('test'),[])
  self.s.cache.clear()
  with patch.object(Native,'cli',return_value={**history,'state_root':'stale'}):
   v=self.s.view('test');self.assertIn('history_error',v['state']);self.assertNotIn('published_history',v['state'])
 def test_conflict_and_traversal_rejected(self):
  self.s.input('test',{'id':'a','text':'one'})
  with self.assertRaises(ValueError):self.s.input('test',{'id':'a','text':'two'})
  with self.assertRaises(ValueError):self.s.meta('../outside')
 def test_unknown_budget_never_prepares(self):
  with patch('prepare.prepare') as p:
   with self.assertRaisesRegex(ValueError,'recovery'):self.s.start('test',{'id':'authored-run','confirm':'Run this reviewed session with up to $10'})
   p.assert_not_called()
 def test_history_tamper_rejected(self):
  self.s.input('test',{'id':'a','text':'one'});f=self.s.path('test')/'events.jsonl';f.write_text(f.read_text().replace('one','two'))
  with self.assertRaisesRegex(ValueError,'integrity'):self.s.events('test')
 def test_restart_input_recovers_without_duplicate_native_event(self):
  original=self.s.append
  def fail(p,e):
   if e['type']=='native_input':raise OSError('Authored crash after native mutation')
   return original(p,e)
  with patch.object(self.s,'append',side_effect=fail):
   with self.assertRaises(OSError):self.s.input('test',{'id':'a','text':'one'})
  before=self.s.native(self.s.project_native('test')).request()
  self.s.input('test',{'id':'a','text':'one'});self.assertEqual(before,self.s.native(self.s.project_native('test')).request());self.assertEqual(len(self.s.events('test')),2)
 def test_start_request_replay_never_dispatches(self):
  self.s.append('test',{'id':'same-start','type':'run_request','state':'preparing'})
  with patch('subprocess.Popen') as launch:
   result=self.s.start('test',{'id':'same-start','confirm':'Run this reviewed session with up to $10'})
   self.assertTrue(result['replayed']);launch.assert_not_called()
 def test_unreconciled_previous_run_queues_new_input(self):
  self.s.append('test',{'id':'prior','type':'run_started','packet':'/authored-missing-session'})
  result=self.s.input('test',{'id':'later','text':'AUTHORED later feedback'})
  self.assertEqual(result['state'],'queued');self.assertFalse(self.s.meta('test')['native'])
 def test_delivery_exact_escaped_text_and_binding(self):
  a=ui.load('delivery_test','engine-browser-adapter.py');text='Quote "hello" \\ and newline\nend'
  self.s.input('test',{'id':'quoted','text':text});packet=self.s.path('test')/'sessions/fixture';(packet/'adapter').mkdir(parents=True)
  ad={};ui.save(packet/'adapter-admission.json',ad);ui.save(packet/'scope-authority.json',{'input_ids':['quoted']});meta=self.s.meta('test');meta['registered_packet']=str(packet);ui.save(self.s.path('test')/'project.json',meta)
  body={'messages':[{'content':json.dumps({'public_brief':'Recorded input:\n'+json.dumps([{'id':'quoted','text':text}])})}]};source={'state_root':'root','source_snapshot':'snapshot','scope':'scope'};receipt={'http_status':200,'response_status':'received'}
  values={'request_body_req':{'body':body,'body_sha256':a.e.sha(body),'source':source},'dispatch_req':{'receipt':receipt},'bound_output_req':{'association':{'request':'req','request_body_sha256':a.e.sha(body),'source':source,'admission_sha256':a.e.sha(ad),'receipt_sha256':a.e.sha(receipt)}}}
  def write():
   rows=[a.d.Driver.row([],{'kind':'admission','value':ad,'fixture_only':False})]
   for name,value in values.items():
    intent={'kind':'intent','name':name};rows.append(a.d.Driver.row(rows,intent));rows.append(a.d.Driver.row(rows,{'kind':'result','name':name,'intent':a.e.sha(intent),'value':value}))
   (packet/'adapter/adapter.jsonl').write_text('\n'.join(json.dumps(r) for r in rows)+'\n')
  write();self.assertEqual(self.s.delivery('test')['quoted']['state'],'provider_delivered');values['bound_output_req']['association']['request']='other';write();self.assertEqual(self.s.delivery('test'),{})
 def test_stop_preserves_history(self):
  self.s.input('test',{'id':'a','text':'one'});before=self.s.events('test');self.assertEqual(self.s.stop('test')['state'],'not_running');self.assertEqual(before,self.s.events('test')[:len(before)]);self.assertEqual(self.s.events('test')[-1]['type'],'stop_requested')
 def test_http_origin_csrf_and_real_input_route(self):
  server=http.server.ThreadingHTTPServer(('127.0.0.1',0),ui.Handler);server.store=self.s;server.expected_host='127.0.0.1:'+str(server.server_port);server.origin='http://'+server.expected_host
  thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start();self.addCleanup(server.server_close);self.addCleanup(server.shutdown)
  c=http.client.HTTPConnection('127.0.0.1',server.server_port);body=json.dumps({'id':'http','text':'Authored HTTP feedback'})
  c.request('POST','/api/project/test/input',body,{'Content-Type':'application/json'});self.assertEqual(c.getresponse().status,403);c.close()
  c=http.client.HTTPConnection('127.0.0.1',server.server_port);c.request('POST','/api/project/test/input',body,{'Content-Type':'application/json','Origin':server.origin,'X-TSR-CSRF':self.s.csrf});r=c.getresponse();self.assertEqual(r.status,200);self.assertEqual(json.loads(r.read())['state'],'native_admitted');c.close()
  c=http.client.HTTPConnection('127.0.0.1',server.server_port);c.request('GET','/api/project/test');r=c.getresponse();v=json.loads(r.read());self.assertEqual(v['state']['snapshot'],'same-source');self.assertIn('Authored HTTP feedback',v['state']['parents'][0]['meaning']);c.close()
class RealNative(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory(dir=ui.ROOT/'target',prefix='ui-authored-');self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name)
  from fixture_server import initialize_authored_source
  self.authored_source=initialize_authored_source(self.root)
  self.s=ui.Store(self.root/'state',[{'id':'test','title':'AUTHORED UI TEST','source':str(self.authored_source),'artifact':None}])
 def test_authored_native_history_read_preserves_source(self):
  source=self.authored_source/'native'
  def authority_snapshot():
   database=source/'events.sqlite';self.assertTrue(database.is_file())
   # This fixture is checkpointed. A nonempty WAL would contain authoritative
   # events and must not be silently omitted. SHM is reader coordination only.
   wal=source/'events.sqlite-wal';self.assertTrue(not wal.exists() or wal.stat().st_size==0)
   cas=[p for p in (source/'cas').rglob('*') if p.is_file()]
   files=[p for p in (source/'coding-work').rglob('*') if p.is_file()]
   self.assertTrue(cas);self.assertTrue(files)
   self.assertTrue({'index.html','app.js','style.css','browser-harness.cjs','probe.json'}<={p.name for p in files})
   return {str(p.relative_to(source)):ui.hashlib.sha256(p.read_bytes()).hexdigest() for p in [database,*cas,*files]}
  before=authority_snapshot()
  store=ui.Store(self.root/'pond-view',[{'id':'pond','title':'AUTHORED read-only projection test','source':str(self.authored_source),'artifact':None}])
  view=store.view('pond')['state'];history=view['published_history']
  self.assertEqual(history['state_root'],view['root']);self.assertEqual(history['current_source'],view['snapshot'])
  self.assertTrue(history['publications']);self.assertFalse(history['grants_authority'])
  self.assertTrue(any(p['contributors'] for p in history['publications']))
  self.assertEqual(before,authority_snapshot())
  self.assertEqual(store.events('pond'),[]);self.assertFalse(store.meta('pond')['native'])
 def test_real_native_feedback_and_original_preserved(self):
  original=self.s.native(self.authored_source/'native').request()
  pid=self.s.create({'id':'feedback','title':'Authored input test','text':'AUTHORED fresh native input fixture, not player feedback.'})['id']
  old=self.s.native(self.s.project_native(pid)).request();result=self.s.input(pid,{'id':'authored','text':'AUTHORED LOCAL TEST, NOT PLAYER FEEDBACK. Retain the game.'});new=self.s.native(self.s.project_native(pid)).request()
  self.assertNotEqual(old['state_root'],result['native_root']);self.assertEqual(old['coding']['snapshot'],new['coding']['snapshot']);self.assertEqual(original,self.s.native(self.authored_source/'native').request());self.assertEqual(self.s.input(pid,{'id':'authored','text':'AUTHORED LOCAL TEST, NOT PLAYER FEEDBACK. Retain the game.'}),result)
 def test_approved_resume_rejects_later_native_guidance(self):
  pid=self.s.create({'id':'bound-guidance','title':'AUTHORED binding','text':'AUTHORED initial desire'})['id']
  snapshot=self.s.source_snapshot(pid)
  approval={'disclosure':{'resume_input_id':'bound-guidance','input_ids':['bound-guidance'],'source':snapshot}}
  self.s.decisions.approval=lambda p:approval
  self.s.budget=lambda:{'available':True,'stopped':False}
  self.s.input(pid,{'id':'later-saved','text':'AUTHORED saved later; must not enter earlier approval'})
  self.assertNotEqual(snapshot['native_state_root'],self.s.source_snapshot(pid)['native_state_root'])
  # Authored runtime availability isolates approval binding from host Docker readiness.
  # Wrap only the UI module's subprocess reference: native snapshot reads remain real.
  with patch.object(self.s,'runtime_preflight',return_value={'fixture_only':True}) as runtime, patch.object(ui,'subprocess',wraps=ui.subprocess) as processes:
   with self.assertRaisesRegex(ValueError,'guidance changed'):self.s.start(pid,{'id':'send-bound-guidance','confirm':'Run this reviewed session with up to $10'})
   runtime.assert_called_once_with();processes.Popen.assert_not_called()
  self.assertFalse(any(e['type']=='run_request' for e in self.s.events(pid)))
 def test_real_http_to_native_and_reload(self):
  pid=self.s.create({'id':'http','title':'Authored HTTP test','text':'AUTHORED fresh native HTTP fixture, not player feedback.'})['id']
  self.s.budget=lambda:{'available':True,'stopped':True}
  httpd=http.server.ThreadingHTTPServer(('127.0.0.1',0),ui.Handler);httpd.store=self.s;httpd.expected_host='127.0.0.1:'+str(httpd.server_port);httpd.origin='http://'+httpd.expected_host
  thread=threading.Thread(target=httpd.serve_forever,daemon=True);thread.start();self.addCleanup(httpd.server_close);self.addCleanup(httpd.shutdown)
  body=json.dumps({'id':'authored-http','text':'AUTHORED HTTP NATIVE TEST, not player feedback.'})
  c=http.client.HTTPConnection('127.0.0.1',httpd.server_port);c.request('POST','/api/project/'+pid+'/input',body,{'Content-Type':'application/json','Origin':httpd.origin,'X-TSR-CSRF':self.s.csrf});r=c.getresponse();self.assertEqual(r.status,200);receipt=json.loads(r.read());c.close()
  c=http.client.HTTPConnection('127.0.0.1',httpd.server_port);c.request('GET','/api/project/'+pid);r=c.getresponse();view=json.loads(r.read());self.assertEqual(view['state']['root'],receipt['native_root']);self.assertIn('AUTHORED HTTP NATIVE TEST',view['state']['parents'][0]['meaning']);c.close()
  reboot=ui.Store(self.root/'state',[{'id':'test','title':'AUTHORED UI TEST','source':str(self.authored_source),'artifact':None}]);self.assertEqual(reboot.view(pid)['state']['root'],receipt['native_root'])
 def test_fresh_native_project_prepare_and_authored_abstention(self):
  result=self.s.create({'id':'fresh','title':'Authored project','text':'AUTHORED HARNESS TEST: original browser interaction, not player feedback.'});p=result['id']
  self.s.input(p,{'id':'second','text':'AUTHORED second input before first Run.'})
  a=ui.load('ui_test_adapter','engine-browser-adapter.py');m=a.load('ui_test_campaign','engine-b91-campaign.py');manifest=m.manifest('a'*64,self.root/'fixture-ledger.jsonl',fixture=True);ledger=m.Ledger(self.root/'fixture-ledger.jsonl',manifest,m.digest(manifest),create=True)
  out=prepare(self.s,p,(manifest,m,ledger,{}));ad=json.loads((out/'adapter-admission.json').read_text());self.assertEqual(ad['max_calls'],200);self.assertEqual(ad['max_executions'],100);self.assertEqual(ledger.status()['calls'],0);self.assertIn('AUTHORED second input before first Run.',self.s.native(ad['origin']['directory']).request()['coding']['relational']['inputs']['parents'][0]['meaning'])
  calls=[]
  def authored(body,timeout):
   calls.append(body);return {'value':{'abstain':'AUTHORED test abstention; this is not live model behavior.'},'receipt':{'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':.1}}
  adapter=a.Adapter(ad['directory'],ad,self.s.native(ad['origin']['directory']),ledger,authored)
  ca=json.loads((out/'cycle-admission.json').read_text());cycle=a.loop.Cycle(ca['journal'],ca,adapter.native,adapter.proposal,adapter.review)
  result=cycle.run();self.assertEqual(result['status'],'proposal_abstention');self.assertEqual(len(calls),1);self.assertEqual(ledger.status()['calls'],1)
  again=a.loop.Cycle(ca['journal'],ca,adapter.native,adapter.proposal,adapter.review).run();self.assertEqual(again,result);self.assertEqual(len(calls),1)
  self.s.ledger=lambda:(manifest,m,ledger,{})
  self.s.append(p,{'id':'authored-launch','type':'run_started','packet':str(out)});self.s.require_prior_terminal(p)
if __name__=='__main__':unittest.main()
