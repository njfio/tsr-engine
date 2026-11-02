"""Explicit AUTHORED UI verification server. Never used by server.py or paid dispatch.
All model responses and source are authored fixtures; native journals are initialized
in temporary directories, never copied from historical games or provider runs.
"""
import argparse,http.server,json,tempfile,threading
from pathlib import Path
import server as ui

from decisions import Decisions

def initialize_authored_source(root):
 """Real native state from an authored minimal browser fixture; no external effects."""
 source=Path(root)/'authored-source';native=source/'native';native.mkdir(parents=True)
 adapter=ui.load('ui_fixture_source_adapter','engine-browser-adapter.py')
 files={'index.html':'<!doctype html><title>AUTHORED UI TEST</title><button id="start">Start</button><p id="status">AUTHORED fixture only</p><canvas width="288" height="192"></canvas><script src="app.js"></script>',
        'app.js':'// AUTHORED UI test fixture; no game or model output.\n',
        'style.css':'/* AUTHORED UI fixture. */\n'}
 probe={'version':'browser-probe@v1','assets':list(files),'entry':'index.html','actions':[],
        'checks':[{'selector':'#status','text':'AUTHORED fixture only'}]}
 config={'goal':'AUTHORED UI infrastructure verification, not player preference','image':adapter.IMAGE,'user':'501:20',
         'browser':True,'browser_container_only':True,'editable':files,
         'protected':{'browser-harness.cjs':(ui.ROOT/'scripts/engine-browser-harness.cjs').read_text(),'probe.json':json.dumps(probe)},
         'probe_entry':'browser-harness.cjs','cases':[{'id':'browser_smoke','args':['probe.json'],'expected_stdout':'browser-observation@v1'}],'timeout_ms':30000}
 requirements={'controls':'AUTHORED requirement: a usable Start control; no player satisfaction claim.'}
 ui.save(source/'scope.json',config);ui.save(source/'requirements.json',requirements)
 c=adapter.d.Native(native);c.cli('project','init','--spec',str(source/'scope.json'),'--requirements',str(source/'requirements.json'))
 c.json_cli(('project-lifecycle-open',),{'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':120000,
  'reason':'AUTHORED native UI fixture initialization','non_tool':{'model_calls':10,'model_tokens':409600,'wall_ms':600000,'patch_bytes':49152,'external_cost_microusd':0}},'--admission')
 status=c.cli('project','acceptance','status')
 c.json_cli(('project','acceptance','feedback'),{'requirements_revision':status['requirements_revision'],'decision_record':status['decision_record'],'guidance':requirements})
 c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root'])
 preview=source/'preview';preview.mkdir()
 for name,text in files.items():(preview/name).write_text(text)
 return source
class AuthoredDecisions(Decisions):
 def disclosure(self,p):
  events=self.store.events(p)
  return {'fixture_only':True,'project':p,'source':self.store.source_snapshot(p),'events_sha256':ui.sha(events),'resume_input_id':next((e['id'] for e in reversed(events) if e['type']=='input'),None),'input_ids':[e['id'] for e in events if e['type']=='input'],'remaining_microusd':10000000,'remaining_calls':200,'remaining_operations':100,'minutes':120,'text':'AUTHORED UI TEST — no real financial decision or paid calls.'}
 def apply(self,decision):self.store.approve_fixture()

class FixtureStore(ui.Store):
 def __init__(self,root,approved=False):
  self.authored_source=initialize_authored_source(root)
  super().__init__(root/'state',[{'id':'fixture','title':'AUTHORED UI TEST','subtitle':'No live model calls','source':str(self.authored_source),'artifact':None}])
  self.decisions=AuthoredDecisions(self);self.fixture_only=True;self.approved=approved;self.calls=[];self.launch_count=0
  self.adapter=ui.load('ui_fixture_adapter','engine-browser-adapter.py');self.campaign=self.adapter.load('ui_fixture_campaign','engine-b91-campaign.py')
  self.manifest=self.campaign.manifest('a'*64,root/'fixture-ledger.jsonl',fixture=True)
  self.account=self.campaign.Ledger(root/'fixture-ledger.jsonl',self.manifest,self.campaign.digest(self.manifest),create=True)
 def ledger(self):return self.manifest,self.campaign,self.account,{}
 def budget(self):
  b={**self.account.status(),'available':True}
  if not self.approved:b.update(stopped=True,unknown_calls=['AUTHORED-UNKNOWN-NOT-A-REAL-CHARGE'],blocking_unknown_calls=['AUTHORED-UNKNOWN-NOT-A-REAL-CHARGE'],committed_microusd=50000)
  return b
 def approve_fixture(self):self.approved=True
 def launch(self,cmd,log,packet):
  self.launch_count+=1;a=self.adapter;ad=json.loads((packet/'adapter-admission.json').read_text());ca=json.loads((packet/'cycle-admission.json').read_text())
  def authored(body,timeout):
   self.calls.append(body)
   return {'value':{'abstain':'AUTHORED TEST RESPONSE: I have considered the supplied message and will retain this fixture without code changes. This is not live model behavior.'},'receipt':{'http_status':200,'response_status':'received','status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':.1}}
  adapter=a.Adapter(ad['directory'],ad,self.native(ad['origin']['directory']),self.account,authored)
  result=a.loop.Cycle(ca['journal'],ca,adapter.native,adapter.proposal,adapter.review).run()
  log.write((json.dumps(result)+'\n').encode());log.flush()
  class Finished:
   pid=0
   def poll(self):return 0
   def terminate(self):raise AssertionError('Authored completed process cannot be terminated')
  return Finished()

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--port',type=int,default=8767);p.add_argument('--approved',action='store_true');args=p.parse_args()
 tmp=tempfile.TemporaryDirectory(prefix='ui-authored-',dir=ui.ROOT/'target');store=FixtureStore(Path(tmp.name),args.approved)
 server=http.server.ThreadingHTTPServer(('127.0.0.1',args.port),ui.Handler);server.store=store;server.expected_host=f'127.0.0.1:{server.server_port}';server.origin='http://'+server.expected_host
 print(json.dumps({'fixture_only':True,'url':server.origin,'root':tmp.name}),flush=True)
 try:server.serve_forever()
 finally:server.server_close();tmp.cleanup()
