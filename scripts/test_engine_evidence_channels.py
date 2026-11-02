"""Authored contract controls, not evidence of model judgment quality."""
import copy,importlib.util,unittest
from pathlib import Path
s=importlib.util.spec_from_file_location('r',Path(__file__).with_name('engine-project-runtime-review.py'));r=importlib.util.module_from_spec(s);s.loader.exec_module(r)
class Channels(unittest.TestCase):
 def setUp(self):
  self.view={'source_snapshot':'s','scope':'scope'}
  self.item={'source_snapshot':'s','scope':'scope','assessment':'The assessor reports two checks passed; enjoyment unknown.','attribution':'authored independent assessor','failed_response_sha256':'a'*64}
 def test_attributed_report_is_not_native_receipt(self):
  result=r.attributed_assessments({'review_criticism':{'items':[self.item]}},self.view)
  self.assertEqual(result[0]['assessment'],self.item['assessment']);self.assertFalse(result[0]['observed_receipt']);self.assertFalse(result[0]['grants_authority']);self.assertEqual(result[0]['channel'],'attributed_assessment')
 def test_stale_scope_and_source_rejected(self):
  for k in self.view:
   bad={**self.item,k:'other'}
   with self.assertRaises(ValueError):r.attributed_assessments({'review_criticism':{'items':[bad]}},self.view)
 def test_basis_mapping_present_absent_and_wrong_index(self):
  response={'proposals':[{'kind':'relational'},{'kind':'experiment'}]};basis={'0':{'observed':['smoke'],'unknown':['enjoyment']}}
  x=r.indexed_basis(response,basis);self.assertEqual(x['proposals'][0]['return_basis'],basis['0']);self.assertTrue(x['proposals'][0]['present']);self.assertFalse(x['proposals'][1]['present'])
  with self.assertRaises(ValueError):r.indexed_basis(response,{'2':{}})
 def test_static_attribution_allowed_but_cannot_promote_to_observation(self):
  b={'review_id':'id','evidence_ids':['assessment:a','receipt:smoke'],'observed_receipts':['receipt:smoke']}
  value={'review_id':'id','verdict':'accept','findings':{k:{'finding':'met','basis':'static_source','evidence':['assessment:a'],'explanation':'Faithful attribution only.','limitations':'Report truth and fulfillment remain unknown.'} for k in r.CRITERIA}}
  self.assertEqual(r.validate(value,b),'accept')
  promoted=copy.deepcopy(value);promoted['findings']['epistemic_grounding']['basis']='observed_test_receipt'
  with self.assertRaisesRegex(ValueError,'observed receipt required'):r.validate(promoted,b)
  promoted['findings']['epistemic_grounding']['evidence']=['receipt:invented']
  with self.assertRaisesRegex(ValueError,'unsupported evidence'):r.validate(promoted,b)
  promoted['findings']['epistemic_grounding']['evidence']=['receipt:smoke'];self.assertEqual(r.validate(promoted,b),'accept') # semantics still require independent review
 def test_semantic_rejection_and_unknown_remain_available(self):
  b={'review_id':'id','evidence_ids':['assessment:a'],'observed_receipts':[]}
  for verdict,finding in [('reject','not_met'),('unknown','unknown')]:
   v={'review_id':'id','verdict':verdict,'findings':{k:{'finding':finding,'basis':'static_source','evidence':['assessment:a'],'explanation':'Authored rejection or insufficient evidence.','limitations':'Not a model competence test.'} for k in r.CRITERIA}}
   self.assertEqual(r.validate(v,b),verdict)

from test_engine_browser_adapter import BrowserAdapter
class NativeChannels(BrowserAdapter):
 def test_serialized_shared_contract_and_separate_evidence_classes(self):
  a=self.adapter();view=a.native('request',None);original=a.project
  def projected(v):
   request,context=original(v)
   context['review_criticism']={'items':[{'source_snapshot':v['source_snapshot'],'scope':v['scope'],'assessment':'Authored assessor reports a check; not native observation.','attribution':'fixture assessor','failed_response_sha256':'a'*64}]}
   return request,context
  a.project=projected
  p=a.proposal(view);preview=a.native('preview',{'response':p['response']});a.review({**a.native('request',None),'response_digest':preview['response_digest']},p['response'])
  bodies=[__import__('json').loads(x['messages'][1]['content']) for x in self.bodies]
  self.assertEqual(bodies[0]['evidence_contract']['version'],'project-evidence-channels@v2')
  review=bodies[-1];self.assertEqual(review['evidence_contract'],bodies[0]['evidence_contract'])
  assessments=[(k,v) for k,v in review['evidence'].items() if isinstance(v,dict) and v.get('channel')=='attributed_assessment'];self.assertEqual(len(assessments),1)
  self.assertNotIn(assessments[0][0],review['binding']['observed_receipts'])
  basis=[v for v in review['evidence'].values() if isinstance(v,dict) and v.get('version')=='proposal-return-basis@v1'];self.assertEqual(basis[0]['proposals'][0]['proposal_index'],0)
for name in dir(BrowserAdapter):
 if name.startswith('test_') and name not in NativeChannels.__dict__:setattr(NativeChannels,name,None)

if __name__=='__main__':unittest.main()
