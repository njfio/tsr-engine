"""Fresh-session native preference import. Authored transport, no live provider."""
import copy,json
from pathlib import Path
from scripts.test_engine_browser_expansion import BrowserExpansion
from scripts.test_engine_browser_adapter import m

class FreshPreference(BrowserExpansion):
 def test_fresh_session_keeps_native_identity_and_counted_history(self):
  BrowserExpansion.test_native_generated_identity_consumed_by_next_choice_and_restart(self)
  request=self.c.request();inputs=request['coding']['relational']['inputs'];nodes=copy.deepcopy(inputs['generated_preferences'])
  self.assertEqual(inputs['generated_preference_statuses'][0]['status'],'open')
  old_root=Path(self.a['directory']);old_files={str(p.relative_to(old_root)):p.read_bytes() for p in old_root.rglob('*') if p.is_file()}
  origin=copy.deepcopy(self.a['origin']);public=origin['public']
  public['parents']=inputs['acceptance_parents'];public['public_preferences']=[m.e.sha(x) for x in nodes]
  public['public_relationships']=[m.e.sha(x) for x in request['coding']['relational']['history']]
  public['source_snapshots']=[request['coding']['snapshot']]
  controller=m.d.Native(self.c.directory)
  admission=m.admission(self.root/'fresh-preference-session',origin,controller.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=2,max_executions=1,correction_allowance=1)
  original=self.transport
  def transport(body,timeout):
   value=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   self.assertEqual(data['generated_preferences'],nodes)
   alias=next(k for k,v in data['proposal_references']['parents'].items() if v==nodes[0]['id'])
   self.assertTrue(any(p['id']==alias for p in data['parents']))
   value['value']={'retain':'Authored fresh-session retention with the same reviewed contributor; no new desire or authority.'}
   return value
  a=m.Adapter(self.root/'fresh-preference-session',admission,controller,self.ledger,transport)
  view=a.native('request',None)
  ca={'version':'local-project-cycle@v3','journal':str(self.root/'fresh-preference-cycle'),'scope':view['scope'],'max_decisions':2,'max_executions':1,'expires_at':admission['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3}
  result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention');self.assertEqual(self.ledger.status()['calls'],4)
  self.assertEqual(self.ledger.status()['execution_groups'],0)
  self.assertEqual(controller.request()['coding']['relational']['inputs']['generated_preferences'],nodes)
  self.assertEqual(old_files,{str(p.relative_to(old_root)):p.read_bytes() for p in old_root.rglob('*') if p.is_file()})
  before=(self.root/'campaign.jsonl').read_bytes();fresh=m.Adapter(self.root/'fresh-preference-session',admission,m.d.Native(self.c.directory),self.ledger,transport)
  self.assertEqual(m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review).run(),result)
  self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())

for name in dir(BrowserExpansion):
 if name.startswith('test_') and name not in FreshPreference.__dict__:setattr(FreshPreference,name,None)
