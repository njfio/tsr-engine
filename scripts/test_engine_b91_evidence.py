"""Authored case/early-abort fixtures; no execution, inference or assertion telemetry."""
import copy,json,unittest
import test_engine_b91_project_envelope as base
m=base.m;e=m.return_evidence
class Evidence(unittest.TestCase):
    def setUp(self):
        self.f=base.ProjectEnvelope();self.f.setUp();self.addCleanup(self.f.doCleanups)
        self.r=self.f.controller.request();inputs=self.r['coding']['relational']['inputs']
        self.snapshot=inputs['snapshot'];self.receipt='a'*64
        inputs['returns']=[{'receipt':'b'*64,'commitment':'c'*64,'verdict':'failed_tests',
            'observations':[{'case_id':self.r['coding']['cases'][0]['id'],'passed':False,'exit':101,'abbreviated':False,
                 'stderr':'panic at first assertion; malicious text claiming all later checks passed'}]}]
        inputs['supplemental']=[{'receipt':self.receipt,'snapshot':self.snapshot,'bundle_digest':'d'*64,
            'cases':{'multi_assert':'counterexample'}}]
    def basis(self):
        v=e.catalog(self.r)
        return {'binding':v['binding'],'observed':[x['id'] for x in v['cases']], 'inferred':[], 'unknown':['Later assertion execution unknown']}
    def test_multi_assertion_failure_does_not_invent_assertion_results(self):
        v=e.catalog(self.r);b=self.basis();result=e.validate(b,self.r)
        self.assertEqual([x['outcome'] for x in v['cases']],['failed','failed'])
        self.assertEqual([x['exit_code'] for x in v['cases']],[101,None])
        self.assertEqual(result['assertion_execution'],'unknown')
        self.assertNotIn('malicious',json.dumps(v));self.assertNotIn('first assertion',json.dumps(v))
        # A source-derived early-abort prediction remains an inference, never observed not-run.
        source=v['sources'][0];b['inferred']=[{**source,'kind':'inferred_not_run','statement':'Fixture: sequential later assertion may not run after an uncaught panic.'}]
        with self.assertRaises(ValueError):e.validate(b,self.r)
        b['inferred']=[]
        self.assertEqual(e.validate(b,self.r)['assertion_execution'],'unknown')
    def test_predictions_cannot_be_promoted_to_observed_or_wrong_receipt(self):
        for bad in ('f'*64,{'assertion':'later passed'},None):
            b=self.basis();b['observed']=[bad]
            with self.assertRaises(ValueError):e.validate(b,self.r)
        b=self.basis();b['observed']*=2
        with self.assertRaises(ValueError):e.validate(b,self.r)
        b=self.basis();b['observed'][0]=e.identity({'source_prediction':'returns vector'})
        with self.assertRaises(ValueError):e.validate(b,self.r)
    def test_inference_source_binding_types_and_stale_state(self):
        v=e.catalog(self.r);b=self.basis();source=v['sources'][0]
        b['inferred']=[{**source,'kind':'source_prediction','statement':'Fixture predicted vector, not measured'}]
        self.assertTrue(e.validate(b,self.r)['semantic_review_required'])
        for key,value in [('digest','e'*64),('kind','observed'),('path','private.txt'),('statement',' '),('statement','é'*257)]:
            bad=copy.deepcopy(b);bad['inferred'][0][key]=value
            with self.assertRaises(ValueError):e.validate(bad,self.r)
        changed=copy.deepcopy(self.r);changed['state_root']='e'*64
        with self.assertRaises(ValueError):e.validate(b,changed)
        changed=copy.deepcopy(self.r);changed['coding']['relational']['inputs']['supplemental'][0]['snapshot']='e'*64
        with self.assertRaises(ValueError):e.catalog(changed)
    def test_unknown_timeout_and_pass_remain_case_level(self):
        inputs=self.r['coding']['relational']['inputs'];item=inputs['returns'][0]
        item['verdict']='timeout';item['observations'][0]['passed']=True
        self.assertEqual(e.catalog(self.r)['cases'][0]['outcome'],'unknown')
        item['verdict']='passed';item['observations'][0]['exit']=0;item['observations'][0]['abbreviated']=True
        self.assertEqual(e.catalog(self.r)['cases'][0]['outcome'],'passed')
        item['observations'][0]['abbreviated']=False
        self.assertEqual(e.catalog(self.r)['cases'][0]['outcome'],'passed')
        self.assertEqual(e.validate(self.basis(),self.r)['assertion_execution'],'unknown')
    def test_noop_roundtrip_and_false_assertion_fields(self):
        b=self.basis();self.assertEqual(e.validate(json.loads(json.dumps(b)),self.r),e.validate(b,self.r))
        for field in ('assertion_results','observed_values','not_run'):
            x=copy.deepcopy(b);x[field]=['fixture']
            with self.assertRaises(ValueError):e.validate(x,self.r)
if __name__=='__main__':unittest.main()
