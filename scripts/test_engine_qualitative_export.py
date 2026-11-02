"""Authored protocol controls only; no providers, workers or live-output adoption."""
import copy,importlib.util,json,shutil,tempfile,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('export_test',Path(__file__).with_name('engine-qualitative-export.py'));x=importlib.util.module_from_spec(s);s.loader.exec_module(x)
class Export(unittest.TestCase):
 @classmethod
 def setUpClass(cls):
  cls.tmp=tempfile.TemporaryDirectory();cls.root=Path(cls.tmp.name);cls.prepared=x.prepare(cls.root/'prepared');cls.packet_dir=cls.root/'prepared';cls.q=json.loads((cls.packet_dir/'request-local-only.json').read_text());cls.o=json.loads((cls.packet_dir/'origin.json').read_text())
 @classmethod
 def tearDownClass(cls):cls.tmp.cleanup()
 def project(self,q=None,**kw):
  q=q or self.q;return x.project(q,self.o,expected_request=x.sha(q),**kw)
 def write(self,path,v):path.write_text(json.dumps(v));return path
 def test_frozen_scope_private_exclusions_unchanged_and_stale(self):
  a=self.project();self.assertEqual(a,self.project());self.assertEqual(a['observed_returns'],[])
  q=copy.deepcopy(self.q);q['private_notes']='PRIVATE_SENTINEL';q['instructions']='PRIVATE_SENTINEL';q['response_contract']={'private':'PRIVATE_SENTINEL'};q['history']=[{'private':'PRIVATE_SENTINEL'}]
  self.assertNotIn('PRIVATE_SENTINEL',json.dumps(self.project(q)))
  with self.assertRaisesRegex(ValueError,'stale'):x.project(q,self.o,expected_request=x.sha(self.q))
  self.assertLess(len(json.dumps(x.body(a)).encode()),32768)
 def test_scope_authority_and_evidence_reject(self):
  for mutate in (lambda q:q['visible_records'][0].update(content='private replacement'),lambda q:q['authority'].update(executions=1),lambda q:q['observed_returns'].append({'fake':True}),lambda q:q['parents'][0].update(meaning='unreviewed'),lambda q:q['basis'].update(kind='observed_return'),lambda q:q['parents'][0].update(active=False)):
   q=copy.deepcopy(self.q);mutate(q)
   with self.assertRaises(ValueError):self.project(q)
 def test_stops_no_forced_calls(self):
  for proposals,outcome in (([],'stop_abstention'),([{'kind':'public_desire','proposal':{'change':'clarification'}}],'stop_clarification'),([{'kind':'new_candidate'}],'preview_then_stop')):
   self.assertEqual(x.next_step({'basis':{},'generator':'fixture','response':{'proposals':proposals}},'propose'),outcome)
  self.assertEqual(x.next_step({'state_root':'x','proposal':'y','reviewer':'fixture','decision':'unknown','reason':'Unknown is not negative'},'review'),'stop_review')
  q=self.project();q['phase']='stop'
  with self.assertRaises(ValueError):x.body(q)
 def test_real_native_quarantine_review_restart_and_next_projection(self):
  with tempfile.TemporaryDirectory() as tmp:
   r=Path(tmp);state=r/'state';shutil.copytree(self.packet_dir/'state',state);q=x.native(state,'request');ids=[p['id'] for p in q['parents']]
   candidate={'basis':q['basis'],'generator':'authored-fixture','response':{'protocol':q['protocol'],'state_root':q['state_root'],'attention_binding':q['attention_binding'],'proposals':[{'kind':'relational','binding':q['attention_binding'],'proposal':{'parents':ids,'relationship':'Authored fixture relationship, not model evidence.','aspiration':'Authored fixture wanted condition, not a winning answer.','discriminator':'Unresolved; no observed return.','revises':None,'catalysts':[]}}]}}
   path=self.write(r/'candidate.json',candidate);x.native(state,'preview',path);pid=x.native(state,'submit',path);pending_q=x.native(state,'request');self.assertEqual(pending_q['phase'],'review')
   with self.assertRaises(ValueError):self.project(pending_q)
   pending=pending_q['pending_reviews'][0];ctx=self.project(pending_q,pending={pid:x.sha(pending)});review_body=x.body(ctx);self.assertNotIn('fixture wanted condition',review_body['messages'][0]['content'])
   review={'state_root':pending_q['state_root'],'proposal':pid,'reviewer':'independent-authored-fixture','decision':'admit','reason':'PRIVATE_SENTINEL authored wiring check only'}
   after=x.native(state,'review',self.write(r/'review.json',review));g=after['guidance'][0]
   with self.assertRaises(ValueError):self.project(after)
   ctx=self.project(after,reviewed={g['revision']:x.sha(g)});self.assertNotIn('PRIVATE_SENTINEL',json.dumps(ctx));self.assertEqual(ctx,self.project(x.native(state,'request'),reviewed={g['revision']:x.sha(g)}))
   next_body=x.body(ctx);self.assertIn(g['proposal']['aspiration'],next_body['messages'][1]['content']);self.assertEqual(after['source_parents'],q['source_parents']);self.assertEqual(after['authority'],q['authority']);self.assertEqual(after['acceptance_references'],[])
   changed=copy.deepcopy(after);changed['guidance'][0]['proposal']['aspiration']='tampered'
   with self.assertRaises(ValueError):self.project(changed,reviewed={g['revision']:x.sha(g)})
   malformed=copy.deepcopy(candidate);malformed['response']['state_root']='0'*64
   with self.assertRaises(Exception):x.native(state,'preview',self.write(r/'stale.json',malformed))
   self.assertEqual(after,x.native(state,'request'))
if __name__=='__main__':unittest.main()
