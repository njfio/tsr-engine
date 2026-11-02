"""Authored transport + native contrast/identity cycle. No live provider calls."""
import copy,json
from pathlib import Path
from scripts.test_engine_browser_desire import BrowserDesire
from scripts.test_engine_browser_adapter import m
class BrowserExpansion(BrowserDesire):
 def test_generated_child_meaning_full_transport_and_inert_restart(self):
  a,ca=self.configure_expansion();original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:
    children=[p for p in data['parents'] if 'original_contributors' in p]
    if not children:out['value']={'alternatives':[{'contrast_preference':{'aspiration':'r0','contrast':'c0'}}]}
    elif not children[0].get('revision'):
     child=children[0]
     self.assertIn(child['id'],data['public_meaning_references']['eligible_contributors'])
     out['value']={'alternatives':[{'public_desire':{'identity':child['id'],'revises':None,'parents':child['parents']+[child['id']],'meaning':'Authored recoverable exploration wanting, with fulfillment unknown and both obligations preserved.','change':'development','basis':{'kind':'unknown'}}}]}
    else:out['value']={'retain':'Authored retention after independently reviewed child meaning; not empirical fulfillment.'}
   return out
  a.injected_transport=transport
  result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention')
  request,context=a.project(a.native('request',None));i=request['coding']['relational']['inputs']
  child=next(p for p in i['parents'] if 'original_contributors' in p)
  origin=i['generated_preferences'][0]
  self.assertNotEqual(child['meaning'],origin['meaning']);self.assertEqual(child['generated_origin'],origin)
  self.assertEqual(len(i['acceptance_parents']),2);self.assertEqual(child['fulfillment'],'unknown')
  helper=m.load('child_expansion_verify','engine-browser-expansion.py')
  for key,value in [('contributors',[]),('grants_authority',True)]:
   bad=copy.deepcopy(request);p=next(p for p in bad['coding']['relational']['inputs']['parents'] if 'original_contributors' in p);p[key]=value
   with self.assertRaises(ValueError):helper.verify(a,bad)
  before=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter()
  self.assertEqual(m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review).run(),result)
  self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
  self.assertEqual(self.ledger.status()['calls'],5)
 def configure_expansion(self):
  wants={'orientation':'Keep navigation dependable','discovery':'Make unfamiliar perspectives accessible'}
  self.c.json_cli(('project','acceptance','requirements'),wants)
  status=self.c.cli('project','acceptance','status')
  self.c.json_cli(('project','acceptance','review'),{'snapshot':status['snapshot'],'requirements_revision':status['requirements_revision'],'test_receipt':status['test_receipt'],'decision':'reject','evidence':{k:{'met':False,'evidence':'Authored independent fixture judgment, not live gameplay: this condition is unmet'} for k in wants}})
  status=self.c.cli('project','acceptance','status')
  self.c.json_cli(('project','acceptance','feedback'),{'requirements_revision':status['requirements_revision'],'decision_record':status['decision_record'],'guidance':wants})
  q=self.c.request();ctx=self.c.cli('project','cycle','request');r=q['coding']['relational']
  p={'parents':[x['id'] for x in r['inputs']['parents']],'relationship':'Authored orientation can make unfamiliar material approachable','aspiration':'Keep unfamiliar routes approachable with recoverable orientation','discriminator':'Observe unfamiliar choices and recoverable return routes separately','revises':None,'catalysts':[],'development':None}
  response={'protocol':q['protocol'],'state_root':q['state_root'],'attention_binding':q['next_attention']['binding'],'proposals':[{'kind':'relational','binding':r['binding'],'proposal':p}]}
  preview=self.c.json_cli(('project','cycle','preview'),response,'--response')
  advance={'binding':ctx['binding'],'response':response,'review':{'binding':ctx['binding'],'response_digest':preview['response_digest'],'state_root':q['state_root'],'source_snapshot':q['coding']['snapshot'],'reviewer':'authored initial independent reviewer','generator':'authored initial generator','decision':'approve'}}
  result=self.c.json_cli(('project','cycle','advance'),advance)
  self.c.cli('project','relational','admit','--state-root',self.c.request()['state_root'],'--proposal',result['proposal_record'],'--index','0')
  q=self.c.request();i=q['coding']['relational']['inputs'];public=self.a['origin']['public'];public.update(requirements_revision=i['requirements_revision'],parents=i['parents'],public_relationships=[m.e.sha(h) for h in q['coding']['relational']['history']])
  return self.configure()
 def test_native_generated_identity_consumed_by_next_choice_and_restart(self):
  a,ca=self.configure_expansion();original=self.transport
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:
    if data.get('generated_preferences'):
     self.assertEqual(data['generated_preferences'][0]['fulfillment'],'unknown')
     self.assertEqual(len(data['parents']),3)
     self.assertEqual(data['generated_preference_statuses'][0]['status'],'open')
     out['value']={'retain':'Authored choice retains source while keeping the derived contributor available; no fulfillment claim.'}
    else:out['value']={'alternatives':[{'contrast_preference':{'aspiration':'r0','contrast':'c0'}}]}
   return out
  a.injected_transport=transport
  result=m.loop.Cycle(ca['journal'],ca,a.native,a.proposal,a.review).run()
  self.assertEqual(result['status'],'proposal_retention');q=self.c.request();i=q['coding']['relational']['inputs'];self.assertEqual(len(i['generated_preferences']),1)
  request,context=a.project(a.native('request',None));helper=m.load('expansion_verified','engine-browser-expansion.py')
  bad=copy.deepcopy(request);bad['coding']['relational']['inputs']['generated_preferences'][0]['meaning']='forged replacement'
  with self.assertRaises(ValueError):helper.verify(a,bad)
  bad=copy.deepcopy(request);bad['coding']['relational']['inputs']['generated_preferences'][0]['contributors'][0]['revision']='f'*64
  with self.assertRaises(ValueError):helper.verify(a,bad)
  bad=copy.deepcopy(request);bad['coding']['relational']['inputs']['parents'][0]['original_contributors']=[]
  with self.assertRaises(ValueError):helper.verify(a,bad)
  refs=m.load('expansion_refs','engine-proposal-references.py');wire=refs.project(context,request)
  self.assertEqual(len(wire['public_meaning_references']['eligible_originals']),2)
  bad={'alternatives':[{'contrast_preference':{'aspiration':'r0','contrast':'stale'}}]}
  with self.assertRaises(ValueError):refs.expand(bad,wire['proposal_references'],request)
  public=copy.deepcopy(a.admission['origin']['public'])
  public['public_preferences']=[m.e.sha(x) for x in request['coding']['relational']['inputs']['generated_preferences']]
  public['public_relationships']=[m.e.sha(x) for x in request['coding']['relational']['history']]
  m.web.project(request,public) # Positive control for the exact export projection being tested.
  bad=copy.deepcopy(request);bad['coding']['relational']['inputs']['generated_preference_statuses'][0]['id']='f'*64
  with self.assertRaisesRegex(ValueError,'generated preference status projection'):m.web.project(bad,public)
  self.assertEqual(len(i['acceptance_parents']),2);self.assertEqual((self.ledger.status()['calls'],self.ledger.status()['execution_groups']),(3,0))
  before=(self.root/'campaign.jsonl').read_bytes();fresh=self.adapter();self.assertEqual(m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review).run(),result);self.assertEqual(before,(self.root/'campaign.jsonl').read_bytes())
 def test_lost_preference_result_recovers_without_second_admission(self):
  a,ca=self.configure_expansion();original=self.transport;native=a.native
  def transport(body,timeout):
   out=original(body,timeout);data=json.loads(body['messages'][1]['content'])
   if 'criteria' not in data:out['value']={'retain':'Authored retain after exact recovery'} if data.get('generated_preferences') else {'alternatives':[{'contrast_preference':{'aspiration':'r0','contrast':'c0'}}]}
   return out
  a.injected_transport=transport
  def fail(kind,value):
   result=native(kind,value)
   if kind=='advance':raise RuntimeError('authored lost preference response')
   return result
  with self.assertRaisesRegex(RuntimeError,'lost preference'):m.loop.Cycle(ca['journal'],ca,fail,a.proposal,a.review).run()
  fresh=self.adapter();fresh.injected_transport=transport
  result=m.loop.Cycle(ca['journal'],ca,fresh.native,fresh.proposal,fresh.review).run()
  self.assertEqual(result['status'],'proposal_retention');self.assertEqual(len(self.c.request()['coding']['relational']['inputs']['generated_preferences']),1)
  self.assertEqual(self.ledger.status()['calls'],3)
for name in dir(BrowserDesire):
 if name.startswith('test_') and name not in BrowserExpansion.__dict__:setattr(BrowserExpansion,name,None)
