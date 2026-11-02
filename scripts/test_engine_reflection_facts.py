"""Deterministic fact-field checks, not a natural-language truth benchmark."""
import copy,json,unittest
from test_engine_b91_evidence import Evidence,e

class ReflectionFacts(unittest.TestCase):
    def setUp(self):
        self.base=Evidence();self.base.setUp();self.addCleanup(self.base.doCleanups)
        self.r=self.base.r
    def proposal(self,text=None):
        return {'development':{'return_reveals':text if text is not None else e.case_fact_summary(self.r),
                'retained_meaning':'Preserve the original parents.',
                'proposed_development':None,'unresolved':'Universal claims remain unproven.'}}
    def test_faithful_summary_accepted_without_forcing_novelty(self):
        p=self.proposal();before=copy.deepcopy(p)
        v=e.validate_case_fact_reflection(p,self.r)
        self.assertIn('0=failed; 1=failed',v['summary']);self.assertEqual(p,before)
        self.assertEqual(v['catalog_binding'],e.catalog(self.r)['binding'])
    def test_false_pass_duplicate_and_invented_case_rejected_not_repaired(self):
        expected=e.case_fact_summary(self.r)
        for text in (expected.replace('0=failed','0=passed'),expected+' 0=passed',expected.replace('1=failed','9=failed'),
                     'passed multi_assert; failed multi_assert',expected.replace('unknown','observed')):
            p=self.proposal(text);before=copy.deepcopy(p)
            with self.assertRaises(ValueError):e.validate_case_fact_reflection(p,self.r)
            self.assertEqual(p,before)
    def test_mixed_pass_fail_unknown_and_changed_epoch(self):
        item=self.r['coding']['relational']['inputs']['returns'][0]
        item['observations'][0]['passed']=True
        self.r['coding']['relational']['inputs']['supplemental'][0]['cases']['uncertain']='unknown'
        p=self.proposal();self.assertIn('0=passed; 1=failed; 2=unknown',p['development']['return_reveals'])
        e.validate_case_fact_reflection(p,self.r)
        item['observations'][0]['passed']=False
        with self.assertRaises(ValueError):e.validate_case_fact_reflection(p,self.r)
    def test_support_subset_not_confused_with_full_catalog(self):
        basis=self.base.basis();basis['observed']=[]
        self.assertEqual(e.validate(basis,self.r)['resolved_observations'],[])
        v=e.validate_case_fact_reflection(self.proposal(),self.r)
        self.assertIn('independent of selected support refs',v['scope'])
        changed=copy.deepcopy(self.r);changed['state_root']='e'*64
        with self.assertRaises(ValueError):e.validate(basis,changed)
    def test_empty_catalog_and_eight_fact_budget(self):
        inputs=self.r['coding']['relational']['inputs'];inputs['returns']=[];inputs['supplemental']=[]
        self.assertIn('No case outcomes',e.case_fact_summary(self.r))
        inputs['supplemental']=[{'receipt':'a'*64,'snapshot':self.base.snapshot,'bundle_digest':'d'*64,
                                'cases':{f'case_{i}':'unknown' for i in range(8)}}]
        summary=e.case_fact_summary(self.r)
        self.assertLessEqual(len(summary.encode()),512)
        self.assertEqual(summary.count('=unknown'),8)
    def test_history_and_initial_are_not_rewritten(self):
        historical=self.proposal('Historical prose retained unchanged')
        before=json.dumps(historical,sort_keys=True)
        self.assertIsNone(e.validate_case_fact_reflection({'development':None},self.r))
        self.assertEqual(json.dumps(historical,sort_keys=True),before)
    def test_free_prose_still_needs_semantic_review(self):
        p=self.proposal();p['development']['proposed_development']='Unsupported passing assertion outside the typed fact field.'
        # Deliberately demonstrate the boundary, rather than falsely calling this
        # mechanical validator a semantic truth checker for arbitrary prose.
        self.assertIn('still requires semantic review',e.validate_case_fact_reflection(p,self.r)['scope'])
        self.assertTrue(e.validate(self.base.basis(),self.r)['semantic_review_required'])

if __name__=='__main__':unittest.main()
