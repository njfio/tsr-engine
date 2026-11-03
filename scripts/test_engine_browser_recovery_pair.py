"""Authored reproductions of actual public failure shapes; no API. Raw failures remain in frozen local evidence."""
import copy,json,os,unittest
from pathlib import Path
from test_engine_browser_adapter import BrowserAdapter,m
ordinary=m.load('pair_recovery_ordinary','engine-browser-ordinary.py')
class Recovery(BrowserAdapter):
 def configure(self,plain=False):
  self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=4,max_executions=2,correction_allowance=2,feedback_cycle={'version':'authorized-feedback-cycle@v1','authorization_id':'authored-recovery','max_continuations':4},process_profile='ordinary-memory@v1' if plain else None)
  a=(ordinary.Adapter if plain else m.Adapter)(self.root/'adapter',self.a,self.c,self.ledger,self.transport,worker=lambda v:self.adapter_instance.resume_process(v));self.adapter_instance=a;return a
 def recovered(self,plain,stale=False,duplicate=False):
  a=self.configure(plain);calls=[];original=self.transport
  failure={'alternatives':[{'edits':[{'path':'app.js','old_text':'// no longer current','new_text':'// intended authored revision'}],'rationale':'Authored stale fixture'}]} if stale else json.loads(Path(__file__).with_name('testdata').joinpath('pond-pair-recovery/nested-abstention.json').read_text())
  def wire(body,timeout):
   calls.append(body);result=original(body,timeout)
   result['value']=failure if len(calls)==1 or duplicate else {'abstain':'Authored fresh decision: no change warranted by this fixture'}
   return result
  a.injected_transport=wire
  if duplicate:
   with self.assertRaisesRegex(Exception,'identical invalid candidate'):a.proposal(a.native('request',None))
   self.assertEqual(self.ledger.status()['calls'],2);return
  value=a.proposal(a.native('request',None));self.assertIn('abstain',value['response']);self.assertEqual(self.ledger.status()['calls'],2);self.assertEqual(self.ledger.status()['execution_groups'],0)
  if stale:
   self.assertIn('native_validation_feedback',json.loads(calls[1]['messages'][1]['content']));self.assertEqual(len(a.corrections.events()),0);self.assertEqual(a.rejection_count(),1)
   self.assertTrue(any(k.startswith('native_rejection_') for k in a.read()[2]))
  else:
   self.assertIn('$.alternatives[0].abstain',calls[1]['messages'][-1]['content']);self.assertEqual(len(a.corrections.events()),1)
  # A completed callback restart is a new request only if explicitly requested;
  # inspect durable result/recovery without dispatching or mutating its outputs.
  before=self.ledger.path.read_bytes();b=(ordinary.Adapter if plain else m.Adapter)(a.root,self.a,self.c,self.ledger,wire,worker=a.worker);self.assertEqual(b.read()[2],a.read()[2]);self.assertEqual(before,self.ledger.path.read_bytes())
 def test_misplaced_alternative_field_names_where_it_belongs(self):
  a=self.configure();calls=[];original=self.transport
  edit={'edits':[{'path':'app.js','old_text':'// no longer current','new_text':'// revision'}],'rationale':'Authored fixture'}
  def wire(body,timeout):
   calls.append(body);result=original(body,timeout)
   result['value']={'alternatives':[edit],'expectation':{'anticipated':'authored'}} if len(calls)==1 else {'abstain':'Authored fresh decision'}
   return result
  a.injected_transport=wire
  a.proposal(a.native('request',None))
  correction=json.loads(calls[1]['messages'][-1]['content'])['protocol_correction']['native_structure']
  self.assertEqual((correction['path'],correction['code']),('$','unsupported_field'))
  self.assertEqual(correction['valid_locations'],['$.alternatives[0].expectation'])
  self.assertEqual(correction['allowed_fields'],['alternatives'])
 def test_misplaced_fields_diagnostic_covers_known_fields_only(self):
  a=self.configure()
  def locations(value):
   with self.assertRaises(ValueError) as caught:a.check_proposal_shape(value)
   return json.loads(caught.exception.diagnostic.split('TSR_SHAPE_V1:',1)[1]).get('valid_locations')
  self.assertEqual(locations({'edits':[],'rationale':'r','expectation':{}}),['$.alternatives[0].edits','$.alternatives[0].expectation','$.alternatives[0].rationale'])
  self.assertEqual(locations({'alternatives':[{}],'rationale':'r','invented':1}),['$.alternatives[0].rationale'])
  self.assertIsNone(locations({'alternatives':[{}],'invented':1}))
  a.check_proposal_shape({'alternatives':[{'edits':[],'rationale':'r','expectation':{}}]})
 def test_interrupted_format_recovers_exact_feedback(self):
  for _ in (None,):
   a=self.configure(True);calls=[];original=self.transport
   bad=json.loads(Path(__file__).with_name('testdata').joinpath('pond-pair-recovery/nested-abstention.json').read_text())
   def wire(body,timeout):
    calls.append(body);v=original(body,timeout);v['value']=bad if len(calls)==1 else {'abstain':'Authored recovered retention'};return v
   a.injected_transport=wire
   def hook(phase,name):
    if phase=='after_result' and name.startswith('format_correction_'):raise RuntimeError('authored correction interruption')
   a.hook=hook
   with self.assertRaisesRegex(RuntimeError,'interruption'):a.proposal(a.native('request',None))
   a.hook=lambda *x:None;self.assertTrue(a.recovery_ready()['ready']);value=a.proposal(a.native('request',None));self.assertIn('abstain',value['response']);self.assertEqual(len(calls),2)
   self.assertIn('protocol_correction',calls[-1]['messages'][-1]['content']);self.assertEqual(len(a.corrections.events()),1)
 def test_interrupted_correction_reservation_recovers(self):
  a=self.configure(True);calls=[];original=self.transport
  def wire(body,timeout):
   calls.append(body);v=original(body,timeout);v['value']={'alternatives':[{'abstain':'Authored retention'}]} if len(calls)==1 else {'abstain':'Authored retention'};return v
  a.injected_transport=wire;reserve=a.corrections.feedback
  def crash(*args,**kwargs):
   reserve(*args,**kwargs);raise RuntimeError('after durable reservation')
  a.corrections.feedback=crash
  with self.assertRaisesRegex(RuntimeError,'durable reservation'):a.proposal(a.native('request',None))
  a.corrections.feedback=reserve;self.assertTrue(a.recovery_ready()['ready'])
  value=a.proposal(a.native('request',None));self.assertIn('abstain',value['response']);self.assertEqual(len(calls),2);self.assertEqual(len(a.corrections.events()),1)
  self.assertIn('$.alternatives[0].abstain',calls[-1]['messages'][-1]['content'])
 def test_frozen_actual_response_shapes(self):
  base=Path(__file__).resolve().parents[1]/'target/pond-pair-prepared-v3'
  stale=base/'A/adapter/output-2/failed-response-1/content.bin';nested=base/'B/adapter/output-0/failed-response-1/content.bin'
  if not stale.exists() or not nested.exists():self.skipTest('local frozen live evidence unavailable; authored portable reproductions still covered')
  a=self.configure();view=a.native('request',None);request,ctx=a.project(view)
  with self.assertRaises(m.e.StageValidationError) as caught:a.decode_proposal(json.loads(stale.read_bytes()),request,ctx,view)
  self.assertEqual(caught.exception.code,'stale_edit_span')
  with self.assertRaises(ValueError) as caught:a.check_proposal_shape(json.loads(nested.read_bytes()))
  self.assertIn('TSR_SHAPE_V1',str(getattr(caught.exception,'diagnostic','')))
 def test_tsr_nested_format(self):self.recovered(False)
 def test_plain_nested_format(self):self.recovered(True)
 def test_tsr_stale_fresh_choice(self):self.recovered(False,stale=True)
 def test_plain_stale_fresh_choice(self):self.recovered(True,stale=True)
 def test_repeat_stale_terminal(self):self.recovered(False,stale=True,duplicate=True)
 def test_actual_stale_fixture_and_ambiguous_forbidden_mixed(self):
  a=self.configure();view=a.native('request',None);request,ctx=a.project(view)
  failure=json.loads(Path(__file__).with_name('testdata').joinpath('pond-pair-recovery/stale-span.json').read_text())
  with self.assertRaises(m.e.StageValidationError) as caught:a.decode_proposal(failure,request,ctx,view)
  self.assertEqual(caught.exception.code,'stale_edit_span')
  for value in [
   {'alternatives':[{'edits':[{'path':'probe.json','old_text':'x','new_text':'y'}],'rationale':'forbidden'}]},
   {'alternatives':[{'edits':[{'path':'app.js','old_text':'','new_text':'x'}],'rationale':'empty'}]},
   {'alternatives':[{'edits':[{'path':'app.js','old_text':'/','new_text':'x'}],'rationale':'ambiguous'}]},
   {'alternatives':[failure['alternatives'][0],{'abstain':'mixed'}]}]:
   with self.assertRaises(ValueError) as rejected:a.decode_proposal(value,request,ctx,view)
   self.assertNotEqual(getattr(rejected.exception,'code',None),'stale_edit_span')
for name in dir(BrowserAdapter):
 if name.startswith('test_') and name not in Recovery.__dict__:setattr(Recovery,name,None)
if __name__=='__main__':unittest.main()
