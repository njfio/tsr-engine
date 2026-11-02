"""Authored provisional wanting transport; no provider or game execution."""
import copy,json
from scripts.test_engine_browser_desire import BrowserDesire
from scripts.test_engine_browser_adapter import m
class BrowserProvisional(BrowserDesire):
 def test_initial_single_contributor_recursive_identity_and_inert_restart(self):
  a,ca=self.configure();original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:
    self.assertIn('provisional_preference',data['choice_summary']['forms'])
    generated=data.get('generated_preferences',[])
    if len(generated)<2:
     parent=next((p for p in data['parents'] if 'original_contributors' in p),data['parents'][0])
     out['value']={'alternatives':[{'provisional_preference':{'parents':[parent['id']],'meaning':'Authored wanting for '+('recoverable interaction' if not generated else 'revisitable recovery choices'),'rationale':'Authored generated hypothesis from the named contributor; fulfillment remains unknown, with no user testimony or new permission.'}}]}
    else:out['value']={'retain':'Authored retention after optional generated wanting; no fulfillment claim.','development_declined':'Authored fixture: development considered, none proposed at this checkpoint'}
   return out
  a.injected_transport=transport
  result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention')
  request,context=a.project(a.native('request',None));i=request['coding']['relational']['inputs']
  self.assertEqual(len(i['generated_preferences']),2);self.assertEqual(len(i['parents']),3);self.assertEqual(len(i['acceptance_parents']),1)
  for p in i['generated_preferences']:
   self.assertIn('provisional',p);self.assertNotIn('success',p);self.assertNotIn('contrast',p);self.assertEqual(p['fulfillment'],'unknown');self.assertFalse(p['grants_authority'])
  self.assertEqual([p['status'] for p in i['generated_preference_statuses']],['open','open'])
  before=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter()
  self.assertEqual(m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review).run(),result)
  self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes());self.assertEqual(self.ledger.status()['calls'],5)
 def test_source_and_stop_negative_controls(self):
  a,ca=self.configure();request,context=a.project(a.native('request',None));refs=m.load('provisional_refs','engine-proposal-references.py');wire=refs.project(context,request)
  raw={'alternatives':[{'provisional_preference':{'parents':['p0'],'meaning':'Authored new wanting','rationale':'Unknown usefulness; current contributor only'}}]}
  expanded=refs.expand(raw,wire['proposal_references'],request)['alternatives'][0]['provisional_preference'];helper=m.load('provisional_bind','engine-browser-expansion.py');helper.bind_provisional(request,expanded)
  for key,value in [('binding','f'*64),('parents',[]),('parents',['f'*64]),('meaning',''),('rationale','')]:
   with self.subTest(key=key),self.assertRaises(ValueError):helper.bind_provisional(request,{**expanded,key:value})
  bad=copy.deepcopy(raw);bad['alternatives'][0]['provisional_preference']['success']='invented'
  with self.assertRaises(ValueError):refs.expand(bad,wire['proposal_references'],request)
  (a.root/'STOP').write_text('authored Stop')
  with self.assertRaises(Exception):m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(self.ledger.status()['calls'],0)
for name in dir(BrowserDesire):
 if name.startswith('test_') and name not in BrowserProvisional.__dict__:setattr(BrowserProvisional,name,None)

class RetainedOriginOwnership(__import__('unittest').TestCase):
 def test_owned_undeveloped_origin_survives_republication_but_forgery_does_not(self):
  helper=m.load('retained_origin_check','engine-browser-desire.py')
  a={'id':'a'*64,'meaning':'Published A','requirement':'a'}
  b={'id':'b'*64,'meaning':'Published B','requirement':'b'}
  developed={**a,'meaning':'Reviewed A with B','revision':'c'*64,'parents':[a['id'],b['id']]}
  history={'revision':'c'*64,'previous':a,'next':developed,'parent_origins':[a,b],'superseded':False,'withdrawn':False}
  public={'history':[history],'active':[developed,b],'current':[developed,b]}
  request={'coding':{'relational':{'inputs':{'public_desires':public,'parents':public['active'],'acceptance_parents':[{'id':'d'*64,'meaning':'New obligation','requirement':'d'}]}}}}
  class Owned:
   admission={'origin':{'public':{}}}
   def read(self):return None,None,{'meaning_admission_0':{'public':{'history':[copy.deepcopy(history)]}}}
  helper.verify_history(Owned(),request)
  altered=copy.deepcopy(request);altered['coding']['relational']['inputs']['parents'][1]['meaning']='Forged B'
  with self.assertRaises(ValueError):helper.verify_history(Owned(),altered)
  altered=copy.deepcopy(request);altered['coding']['relational']['inputs']['public_desires']['history'][0]['parent_origins'][1]['meaning']='Forged origin'
  with self.assertRaises(ValueError):helper.verify_history(Owned(),altered)
