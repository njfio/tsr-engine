"""Envelope fixtures only; no inference or target execution."""
import json
import unittest
import test_engine_b91_project_envelope as base
m=base.m
class WireContract(unittest.TestCase):
    def test_schema_matches_existing_stage_envelope_rules(self):
        values={'relational':{'relational':{'parents':['a','b'],'relationship':'fixture','aspiration':'fixture','discriminator':'fixture'}},
                'experiment':{'experiment':{'harness':{},'cases':[],'mapping':{}}},
                'edits':{'edits':[{'path':'src/lib.rs','replacement':'fixture'}],'rationale':'fixture'}}
        for stage in m.STAGES:
            contract=m.response_contract(stage);schema=contract['envelope_schema'];branches=schema.get('oneOf',[schema])
            expected=({m.REQUIRED_ENVELOPES[stage]} if stage in m.REQUIRED_ENVELOPES else set(values))
            self.assertEqual(set(contract['payload_location']),expected)
            for kind,value in values.items():
                if kind in expected:
                    self.assertEqual(m.validate_stage_envelope(stage,value),kind)
                    branch=next(b for b in branches if kind in b['properties'])
                    self.assertTrue(set(branch['required'])<=set(value));self.assertFalse(branch['additionalProperties'])
                    self.assertEqual(contract['payload_location'][kind],'$.'+kind)
                else:
                    with self.assertRaisesRegex(m.StageValidationError,'wrong_stage_envelope'):m.validate_stage_envelope(stage,value)
            with self.assertRaisesRegex(m.StageValidationError,'missing_or_ambiguous_envelope'):m.validate_stage_envelope(stage,values['relational']['relational'])
            with self.assertRaisesRegex(m.StageValidationError,'missing_or_ambiguous_envelope'):m.validate_stage_envelope(stage,{**values['relational'],**values['experiment']})
    def test_revision_basis_condition_matches_gate_and_preserves_other_envelopes(self):
        f=base.ProjectEnvelope();f.setUp();self.addCleanup(f.doCleanups)
        for stage in m.STAGES:
            _,body=m.project(f.journal,f.fake(),stage)
            ctx=json.loads(body['messages'][1]['content']);schema=ctx['response_contract']['envelope_schema']
            for branch in schema.get('oneOf',[schema]):
                if 'relational' not in branch['properties']:
                    self.assertNotIn('then',branch);continue
                condition=branch['if']['properties']['relational']
                self.assertEqual(condition,{'required':['revises'],'properties':{'revises':{'not':{'type':'null'}}}})
                self.assertEqual(branch['then'],{'required':['return_basis']})
                self.assertEqual(branch['required'],['relational'])  # Initial null revision still permits omission.
                basis=branch['properties']['return_basis']
                self.assertEqual(set(basis['required']),{'binding','observed','inferred','unknown'})
                self.assertFalse(basis['additionalProperties'])
                ids=[x['id'] for x in ctx['return_evidence']['cases']]
                self.assertEqual(basis['properties']['observed']['items'],{'type':'string','enum':ids} if ids else False)
                with self.assertRaises(ValueError):m.return_evidence.validate(None,f.controller.request())

    def test_observed_schema_enumerates_only_current_catalog_ids(self):
        import test_engine_b91_evidence as evidence_tests
        f=evidence_tests.Evidence();f.setUp();self.addCleanup(f.doCleanups)
        view=m.return_evidence.prompt_catalog(f.r);ids=[x['id'] for x in view['cases']]
        for stage in m.STAGES:
            schema=m.response_contract(stage,ids)['envelope_schema']
            for branch in schema.get('oneOf',[schema]):
                if 'relational' not in branch['properties']:continue
                item=branch['properties']['return_basis']['properties']['observed']['items']
                self.assertEqual(item,{'type':'string','enum':ids})
                for case in view['cases']:
                    self.assertNotIn(case['case_id'],item['enum'])
                    b=f.basis();b['observed']=[case['case_id']]
                    with self.assertRaises(ValueError):m.return_evidence.validate(b,f.r)
                b=f.basis();self.assertEqual(len(m.return_evidence.validate(b,f.r)['resolved_observations']),len(ids))
        empty=m.response_contract('relation_after_experiment',[])['envelope_schema']
        self.assertIs(empty['properties']['return_basis']['properties']['observed']['items'],False)
        f.r['coding']['relational']['inputs']['returns']=[];f.r['coding']['relational']['inputs']['supplemental']=[]
        b={'binding':m.return_evidence.catalog(f.r)['binding'],'observed':[],'inferred':[],'unknown':[]}
        self.assertEqual(m.return_evidence.validate(b,f.r)['resolved_observations'],[])
        b['observed']=['0']
        with self.assertRaises(ValueError):m.return_evidence.validate(b,f.r)

    def test_all_normal_projections_include_outer_contract_without_answers(self):
        f=base.ProjectEnvelope();f.setUp();self.addCleanup(f.doCleanups);fake=f.fake()
        for stage in m.STAGES:
            _,body=m.project(f.journal,fake,stage);ctx=json.loads(body['messages'][1]['content'])
            self.assertEqual(ctx['response_contract'],m.response_contract(stage,[x['id'] for x in ctx['return_evidence']['cases']]))
            self.assertIn('outer envelope is mandatory',body['messages'][0]['content'])
            self.assertNotIn('fixture',json.dumps(ctx['response_contract']))
            self.assertNotIn('example',ctx['response_contract'])
if __name__=='__main__':unittest.main()
