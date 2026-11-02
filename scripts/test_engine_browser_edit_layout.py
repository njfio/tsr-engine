"""Authored known-settled edit-layout correction; no provider or browser effects."""
import copy,json,unittest
from test_engine_browser_adapter import BrowserAdapter,m
class EditLayout(unittest.TestCase):
 def setUp(self):
  self.f=BrowserAdapter();self.f.setUp();self.addCleanup(self.f.doCleanups)
 def test_duplicate_file_regenerates_without_rewriting_or_execution(self):
  f=self.f;a=f.adapter();original=a.injected_transport;count=[0]
  def transport(body,timeout):
   result=original(body,timeout)
   if not count[0]:
    result['value']={'alternatives':[{'edits':[{'path':'index.html','old_text':'Start','new_text':'Play'},{'path':'index.html','old_text':'<canvas','new_text':'<canvas aria-label="fixture"'}],'rationale':'Authored duplicate layout'}]}
   count[0]+=1;return result
  a.injected_transport=transport;view=a.native('request',None);p=a.proposal(view)
  self.assertEqual(count[0],2);self.assertEqual(len(a.corrections.events()),1)
  self.assertIn('unique_file_edits',a.corrections.events()[0]['failure'])
  self.assertEqual((f.c.directory/'coding-work/index.html').read_text(),f.config['editable']['index.html'])
  a.native('preview',{'response':p['response']})
  self.assertEqual(f.ledger.status()['execution_groups'],0)
 def test_stale_span_does_not_receive_layout_correction(self):
  r=self.f.c.request();v={'edits':[{'path':'index.html','old_text':'Start','new_text':'Play'},{'path':'index.html','old_text':'NONEXISTENT','new_text':'x'}],'rationale':'Authored bad source'}
  with self.assertRaisesRegex(ValueError,'exact unique changed span'):m.web.bind_edit(r,v,lambda _:None)
 def test_overlapping_span_is_terminal_before_layout(self):
  r=self.f.c.request();f=next(x for x in r['coding']['files'] if x['path']=='app.js');f['source']='aaa'
  v={'edits':[{'path':'app.js','old_text':'aa','new_text':'x'},{'path':'app.js','old_text':'aaa','new_text':'y'}],'rationale':'Authored overlap'}
  with self.assertRaisesRegex(ValueError,'exact unique changed span'):m.web.bind_edit(r,v,lambda _:None)
if __name__=='__main__':unittest.main()
