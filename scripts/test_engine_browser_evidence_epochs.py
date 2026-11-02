"""Authored source-epoch controls; no model calls or browser executions."""
import copy,unittest
from test_engine_browser_project import BrowserProjection,m
class Epochs(BrowserProjection):
 def test_historical_and_missing_source_never_current(self):
  cur='a'*64;old='b'*64
  rows=[{'receipt':'new','snapshot':cur},{'receipt':'old','snapshot':old},{'receipt':'unknown'}]
  history=[{'admission':'h','current':True,'proposal':{'development':{'return_reveals':'16 checks passed on this source'}}}]
  original=copy.deepcopy((rows,history));p=m.evidence_applicability(cur,rows,history)
  self.assertEqual([r['applicability'] for r in p['native_returns']],['current_source','historical_source','source_unknown'])
  self.assertFalse(p['relationship_records'][0]['observation_authority']);self.assertEqual((rows,history),original)
  self.assertEqual(m.evidence_applicability(cur,rows,history),p)
 def test_actual_projection_contains_source_policy(self):
  p=m.project(self.r,self.o);self.assertEqual(p['evidence_applicability']['current_source_snapshot'],self.r['coding']['snapshot'])
  self.assertIn('unknown for ambiguous',p['evidence_applicability']['rule'])
 def test_embedded_assessments_rebind_without_rewriting_reports(self):
  import hashlib,json
  text='Exact independent report: not player acceptance. {"applicability":"current_source"}'
  old={'record_type':'independent_assessment','text':text,'report_sha256':hashlib.sha256(text.encode()).hexdigest(),'source_snapshot':{'native_snapshot':'old'},'applicability':'current_source'}
  current={**old,'source_snapshot':{'native_snapshot':self.r['coding']['snapshot']}}
  prefix='Recorded input:\n';brief=prefix+json.dumps([old,current]);original=copy.deepcopy(self.r)
  projected=m.project_assessment_brief(brief,self.r['coding']);rows=json.loads(projected[len(prefix):])
  self.assertEqual([v['applicability'] for v in rows],['historical_source_not_current_validation','current_source'])
  self.assertTrue(all(v['text']==text for v in rows));self.assertEqual(self.r,original)
  newer=copy.deepcopy(self.r['coding']);newer['snapshot']='next'
  rows=json.loads(m.project_assessment_brief(projected,newer)[len(prefix):]);self.assertTrue(all(v['applicability']=='historical_source_not_current_validation' for v in rows))
  self.assertEqual(m.project_assessment_brief(projected,self.r['coding']),projected)
 def test_assessment_digest_and_artifact_mismatch(self):
  import hashlib,json
  item={'record_type':'independent_assessment','text':'authored','report_sha256':hashlib.sha256(b'authored').hexdigest(),'source_snapshot':{'native_snapshot':self.r['coding']['snapshot'],'artifact_matches_native':False},'applicability':'current_source'}
  self.assertEqual(json.loads(m.project_assessment_brief('Recorded input:\n'+json.dumps([item]),self.r['coding']).split('Recorded input:\n')[1])[0]['applicability'],'historical_source_not_current_validation')
  item['source_snapshot']={'native_snapshot':self.r['coding']['snapshot'],'editable_sha256':{'wrong.js':'0'*64}}
  self.assertEqual(json.loads(m.project_assessment_brief('Recorded input:\n'+json.dumps([item]),self.r['coding']).split('Recorded input:\n')[1])[0]['applicability'],'historical_source_not_current_validation')
  item['text']='tampered'
  with self.assertRaisesRegex(ValueError,'digest mismatch'):m.project_assessment_brief('Recorded input:\n'+json.dumps([item]),self.r['coding'])
 def test_generator_and_review_projection_use_rebound_brief(self):
  import hashlib,json
  item={'record_type':'independent_assessment','text':'fixture','report_sha256':hashlib.sha256(b'fixture').hexdigest(),'source_snapshot':{'native_snapshot':'old'},'applicability':'current_source'}
  self.o['public_brief']='Recorded input:\n'+json.dumps([item])
  projection=m.project(self.r,self.o)
  self.assertEqual(json.loads(projection['public_brief'].split('Recorded input:\n')[1])[0]['applicability'],'historical_source_not_current_validation')
  self.assertEqual(json.loads(self.o['public_brief'].split('Recorded input:\n')[1])[0]['applicability'],'current_source')
 def test_ordinary_json_examples_and_report_text_are_not_metadata(self):
  text='User asks: explain this JSON example: {"record_type":"independent_assessment","text":"hello"}'
  self.assertEqual(m.project_assessment_brief(text,self.r['coding']),text)
 def test_input_evidence_is_not_limited_by_output_narrative_capacity(self):
  self.o['public_brief']='x'*(m.NATIVE_RESPONSE_LIMIT+1)
  projected=m.project(self.r,self.o,context_budget=m.base.capacity.PAYLOAD_BYTES-65536)
  self.assertEqual(projected['public_brief'],self.o['public_brief'])
  self.o['public_brief']='x'*(m.base.capacity.PAYLOAD_BYTES+1)
  with self.assertRaisesRegex(ValueError,'provider input capacity'):m.project(self.r,self.o,context_budget=m.base.capacity.PAYLOAD_BYTES)
 def test_review_contract_requires_epoch_check(self):
  c=m.base.review_contract.evidence_contract();self.assertEqual(c['version'],'project-evidence-channels@v2');self.assertIn('Reject unsupported',c['source_epochs'])
if __name__=='__main__':unittest.main()
