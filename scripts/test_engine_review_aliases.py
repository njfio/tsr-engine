import copy,json,unittest
from test_engine_browser_adapter import m
b=m.base.bound_output
class Aliases(unittest.TestCase):
 def payload(self,tag='a'):
  ids=['context:'+tag*64,'candidate:'+'b'*64,'receipt:'+'c'*64];binding={'review_id':'d'*64,'evidence_ids':ids,'observed_receipts':[ids[2]]}
  return {'binding':binding,'evidence':{i:{'fact':i} for i in ids},'response_schema':m.base.load('alias_schema','engine-project-output.py').review_schema(binding,m.base.review_contract.CRITERIA)}
 def wire(self,p):return {'messages':[{'role':'system','content':'Authored alias test'},{'role':'user','content':json.dumps(p)}]}
 def content(self,ref='e0'):
  return {'verdict':'accept','findings':{k:{'finding':'met','basis':'static_source','evidence':[ref],'explanation':'Authored','limitations':'No live evidence'} for k in m.base.review_contract.CRITERIA}}
 def test_exact_mapping_and_request_local_meaning(self):
  p=b.alias_review_payload(self.payload());q=b.alias_review_payload(self.payload('f'))
  self.assertNotEqual(b.expand_review(self.content(),self.wire(p)),b.expand_review(self.content(),self.wire(q)))
  self.assertEqual(b.expand_review(self.content(),self.wire(p))['findings']['scope_integrity']['evidence'],['context:'+'a'*64])
  for ref in ['e99','context:'+'a'*64]:
   with self.assertRaises(ValueError):b.expand_review(self.content(ref),self.wire(p))
 def test_bad_map_and_duplicate_rejected(self):
  p=b.alias_review_payload(self.payload());bad=copy.deepcopy(p);bad['reference_catalog']['aliases']['e0']='context:'+'f'*64
  with self.assertRaises(ValueError):b.expand_review(self.content(),self.wire(bad))
  c=self.content();c['findings']['scope_integrity']['evidence']=['e0','e0']
  with self.assertRaises(ValueError):b.expand_review(c,self.wire(p))
 def test_native_observed_basis_still_requires_receipt(self):
  original=self.payload();p=b.alias_review_payload(original);c=self.content();c['review_id']='d'*64
  c['findings']['epistemic_grounding']['basis']='observed_test_receipt'
  with self.assertRaises(ValueError):m.base.review_contract.validate(b.expand_review(c,self.wire(p)),original['binding'])
  c['findings']['epistemic_grounding']['evidence']=['e2']
  self.assertEqual(m.base.review_contract.validate(b.expand_review(c,self.wire(p)),original['binding']),'accept')
 def test_reject_and_unknown_are_unchanged(self):
  p=b.alias_review_payload(self.payload())
  for v,f in [('reject','not_met'),('unknown','unknown')]:
   c=self.content();c['verdict']=v;c['findings']['stage_adequacy']['finding']=f
   self.assertEqual(b.expand_review(c,self.wire(p))['verdict'],v)
 def test_replay_reconstructs_map_from_exact_request(self):
  wire=self.wire(b.alias_review_payload(self.payload()));raw=self.content();meta={'review_id':'d'*64};ad={'fixture':True};source={'state_root':'r','source_snapshot':'s','scope':'p'}
  ident=b.digest({'admission':b.digest(ad),'kind':'review','index':0,'body':b.digest(wire)})
  dispatch={'candidate_sha256':b.digest(raw),'receipt':{'usage_status':'reported'}};settled={'stopped':False}
  assoc=b.associate(admission=ad,kind='review',index=0,wire=wire,ident=ident,content=raw,dispatch=dispatch,settled=settled,source=source,current_source=source)
  assoc['reference_catalog_sha256']=b.digest(json.loads(wire['messages'][1]['content'])['reference_catalog'])
  rec={'association':assoc,'model_content':raw,'host_metadata':meta,'reference_request':wire,'source':source,'native_content_sha256':b.digest(b.expand_review(b.attach_review(raw,meta),wire))}
  args=dict(admission=ad,ident=ident,dispatch=dispatch,settled=settled,source=source)
  b.verify_envelope(rec,**args);b.verify_envelope(rec,**args)
  bad=copy.deepcopy(rec);bad['reference_request']=self.wire(b.alias_review_payload(self.payload('f')))
  with self.assertRaises(ValueError):b.verify_envelope(bad,**args)
  for changed in ({'source':{**source,'state_root':'other'}},{'settled':{'unknown_usage':1}},{'ident':'other'}):
   with self.assertRaises(ValueError):b.verify_envelope(rec,**{**args,**changed})
if __name__=='__main__':unittest.main()
