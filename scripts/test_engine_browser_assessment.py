"""Authored assessment/provenance fault injection, no provider or Docker calls."""
import json,types,unittest,copy
from pathlib import Path
from test_engine_browser_adapter import BrowserAdapter,m
class Assessment(BrowserAdapter):
 def configure(self):
  script=self.root/'authored-verifier.cjs';script.write_text('// Authored non-executed verifier fixture\n')
  policy={'version':'source-bound-browser-assessment@v1','script':str(script),'sha256':m.assessment.sha(script),'limitations':'Authored fixture observations only, not gameplay evidence'}
  self.a=m.admission(self.root/'adapter',self.a['origin'],self.c.cli('project','config'),self.manifest,self.a['expires_at'],max_decisions=4,max_executions=3,correction_allowance=2,independent_assessment=policy)
  a=m.Adapter(self.root/'adapter',self.a,self.c,self.ledger,self.transport,worker=lambda v:None);a.enroll();self.runs=0
  def runner():
   self.runs+=1;source=m.assessment.sources(self.c.directory/'coding-work',self.a['origin']['public']['editable'])
   return types.SimpleNamespace(returncode=0,stdout=json.dumps({'version':'authored-observations','source':source,'checks':[{'name':'authored observation · 状態 🐸','pass':True}]}).encode(),stderr=b'')
  a.assessment_runner=runner;return a
 def test_collection_current_projection_no_repeat_and_stale_output(self):
  a=self.configure();request=self.c.request();record=m.assessment.collect(a,request)
  self.assertEqual(record['status'],'observed');self.assertEqual(self.runs,1);self.assertEqual(m.assessment.collect(a,request),record);self.assertEqual(self.runs,1)
  _,ctx=a.project(a.native('request',None));self.assertEqual(ctx['independent_assessments'][0]['applicability'],'current_source');self.assertEqual(ctx['independent_assessments'][0]['observations']['checks'],record['observations']['checks'])
  self.assertEqual(self.ledger.status()['execution_groups'],1)
  with self.assertRaisesRegex(ValueError,'source mismatch'):m.assessment.decode(json.dumps({'source':{},'checks':[]}),record['source'])
 def test_reflection_observer_is_authenticated_current_and_never_native(self):
  a=self.configure();request=self.c.request();record=m.assessment.collect(a,request)
  _,ctx=a.project(a.native('request',None));evidence=m.assessment.reflection_evidence(a,request,ctx)
  self.assertEqual(len(evidence),1);self.assertFalse(evidence[0]['native_observation']);self.assertEqual(evidence[0]['report'],record)
  self.assertEqual(m.assessment.reflection_evidence(a,request,ctx),evidence);self.assertEqual(self.runs,1)
  bad=copy.deepcopy(ctx);bad['independent_assessments'][0]['observations']['checks'][0]['pass']=False
  with self.assertRaisesRegex(ValueError,'unowned'):m.assessment.reflection_evidence(a,request,bad)
  original=a.read;rows,pending,done=original();forged=copy.deepcopy(done)
  key=next(k for k in forged if k.startswith('independent_assessment_'));forged[key]['observations']['checks'][0]['pass']=False
  a.read=lambda:(rows,pending,forged)
  with self.assertRaisesRegex(ValueError,'not authenticated'):m.assessment.reflection_evidence(a,request,bad)
  a.read=original
  changed=copy.deepcopy(request);changed['coding']['snapshot']='f'*64
  with self.assertRaisesRegex(ValueError,'applicability'):m.assessment.reflection_evidence(a,changed,ctx)
  historical=copy.deepcopy(ctx);historical['independent_assessments'][0]['applicability']='historical_source_only'
  self.assertEqual(m.assessment.reflection_evidence(a,changed,historical),[])
  changed=copy.deepcopy(request);changed['coding']['files'][0]['source']+=' altered'
  with self.assertRaisesRegex(ValueError,'source digest'):m.assessment.reflection_evidence(a,changed,ctx)
 def test_interrupted_settled_assessment_recovers_without_execution(self):
  a=self.configure()
  def hook(phase,name):
   if phase=='after_effect' and name.startswith('independent_assessment_'):raise RuntimeError('authored crash after settled operation')
  a.hook=hook
  with self.assertRaisesRegex(RuntimeError,'authored crash'):m.assessment.collect(a,self.c.request())
  self.assertEqual(self.runs,1);self.assertFalse(self.ledger.status()['stopped']);a.hook=lambda *x:None
  a.recover_session_transition();self.assertFalse(a.read()[1]);m.assessment.collect(a,self.c.request());self.assertEqual(self.runs,1)
 def test_tampered_recovery_receipt_stops(self):
  a=self.configure();a.hook=lambda phase,name:(_ for _ in ()).throw(RuntimeError('crash')) if phase=='after_effect' and name.startswith('independent_assessment_') else None
  with self.assertRaises(RuntimeError):m.assessment.collect(a,self.c.request())
  f=next(a.root.rglob('receipt.json'));record=json.loads(f.read_text());record['observations']['checks'][0]['pass']=False;f.write_text(json.dumps(record))
  with self.assertRaisesRegex(ValueError,'not authenticated'):m.assessment.recover(a)
  self.assertEqual(self.runs,1)
 def test_unresolved_execution_cannot_recover(self):
  a=self.configure()
  a.assessment_runner=lambda:(_ for _ in ()).throw(TimeoutError('authored unknown effect'))
  with self.assertRaises(TimeoutError):m.assessment.collect(a,self.c.request())
  self.assertTrue(self.ledger.status()['stopped'])
  with self.assertRaisesRegex(ValueError,'unresolved'):m.assessment.recover(a)
 def test_preexec_missing_executable_is_received_failure_not_observation(self):
  a=self.configure();a.assessment_runner=lambda:(_ for _ in ()).throw(FileNotFoundError(2,'No such file or directory','docker'))
  record=m.assessment.collect(a,self.c.request());self.assertEqual(record['status'],'not_started');self.assertIsNone(record['observations']);self.assertFalse(self.ledger.status()['stopped']);self.assertEqual(self.ledger.status()['execution_groups'],1)
  self.assertEqual(m.assessment.collect(a,self.c.request()),record)
 def test_missing_runtime_preflight_prevents_reservation(self):
  from unittest.mock import patch
  a=self.configure();del a.assessment_runner
  with patch.object(m.assessment.shutil,'which',return_value=None):
   with self.assertRaisesRegex(ValueError,'before assessment reservation'):m.assessment.collect(a,self.c.request())
  self.assertEqual(self.ledger.status()['execution_groups'],0);self.assertFalse(a.read()[1])
 def test_unrelated_file_not_found_remains_unknown(self):
  a=self.configure();a.assessment_runner=lambda:(_ for _ in ()).throw(FileNotFoundError(2,'after process','output-file'))
  with self.assertRaises(FileNotFoundError):m.assessment.collect(a,self.c.request())
  self.assertTrue(self.ledger.status()['stopped'])
for name in dir(BrowserAdapter):
 if name.startswith('test_') and name not in Assessment.__dict__:setattr(Assessment,name,None)
if __name__=='__main__':unittest.main()
