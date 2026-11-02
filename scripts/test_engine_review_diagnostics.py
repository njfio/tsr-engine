"""Bounded static quarantine reasons; never persist rejected prose or exception text."""
import copy
import importlib.util
import json
from pathlib import Path
from types import SimpleNamespace
import unittest
import test_engine_synthetic_review as fixture
spec=importlib.util.spec_from_file_location('review_cycle_diagnostics',Path(__file__).with_name('engine-synthetic-cycle.py'))
c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)
class ReviewDiagnostics(unittest.TestCase):
    def setUp(self):
        f=fixture.ReviewContract();f.setUp();self.f=f;self.receipt={'body_sha256':'d'*64}
    def check(self,value):return c.validate_review_output(value,self.f.schema,self.f.binding,fixture.transport,fixture.r,self.receipt)
    def test_valid_reports_retained_and_invalid_reasons_exact(self):
        valid=self.f.report();kept,diag=self.check(valid)
        self.assertEqual(kept,valid);self.assertEqual((diag['stage'],diag['code']),('validated','valid'))
        cases=[(lambda v:v['findings']['scope_integrity'].update(explanation='é'*300),'schema','schema_text'),
               (lambda v:v['findings'].pop('parent_fidelity'),'schema','schema_object'),
               (lambda v:v['findings']['scope_integrity'].update(finding='unknown'),'semantic_contract','accept_requires_met'),
               (lambda v:v['findings']['scope_integrity'].update(basis='unknown'),'semantic_contract','unknown_basis_cannot_establish'),
               (lambda v:v['findings']['scope_integrity'].update(basis='observed_simulator'),'semantic_contract','observed_receipt_required'),
               (lambda v:v['findings']['scope_integrity'].update(evidence=['private-sentinel']),'schema','schema_enum')]
        for change,stage,code in cases:
            value=copy.deepcopy(valid);change(value);kept,diag=self.check(value)
            self.assertIsNone(kept);self.assertEqual((diag['stage'],diag['code']),(stage,code))
            self.assertEqual(diag['candidate_sha256'],c.digest(value));self.assertEqual(diag['response_body_sha256'],'d'*64)
            self.assertNotIn('private-sentinel',json.dumps(diag));self.assertLess(len(json.dumps(diag)),4096)
    def test_unrecognized_exception_content_not_retained(self):
        def fail(*_):raise ValueError('SECRET arbitrary provider/source wording')
        kept,diag=c.validate_review_output(self.f.report(),self.f.schema,self.f.binding,SimpleNamespace(validate=fail),fixture.r,self.receipt)
        self.assertIsNone(kept);self.assertEqual(diag['code'],'schema_validation_error');self.assertNotIn('SECRET',json.dumps(diag))
        kept,diag=c.validate_review_output(self.f.report(),self.f.schema,self.f.binding,fixture.transport,SimpleNamespace(validate=fail,CRITERIA=fixture.r.CRITERIA),self.receipt)
        self.assertIsNone(kept);self.assertEqual(diag['code'],'contract_validation_error');self.assertNotIn('SECRET',json.dumps(diag))
if __name__=='__main__':unittest.main()
