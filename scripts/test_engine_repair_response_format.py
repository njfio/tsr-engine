"""No inference: formatting guidance never supersedes strict/native validation."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest

ROOT=Path(__file__).parent

def load(name,file):
    spec=importlib.util.spec_from_file_location(name,ROOT/file)
    m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m

m=load('repair_format','engine-repair-response-format.py')
e=load('repair_envelope','engine-b91-project-envelope.py')

class RepairFormat(unittest.TestCase):
    def test_narrow_model_stage_contract(self):
        for stage in ('relation_after_seed','relation_after_experiment','relation_after_verification','final_proposal_only'):
            self.assertIsNone(m.response_format(stage,['requested_behavior','existing_behaviour']))
        for stage in ('experiment','boundary_patch'):
            fmt=m.response_format(stage,['requested_behavior','existing_behaviour'])
            self.assertEqual(fmt['type'],'json_schema');self.assertIs(fmt['json_schema']['strict'],False)
            self.assertLess(len(json.dumps(fmt).encode()),3000)
    def test_experiment_exact_fields_without_prescribed_solution(self):
        fmt=m.response_format('experiment',['requested_behavior','existing_behaviour'])
        root=fmt['json_schema']['schema'];experiment=root['properties']['experiment']
        self.assertEqual(root['required'],['experiment']);self.assertFalse(root['additionalProperties'])
        self.assertEqual(set(experiment['properties']),{'harness','cases','mapping'})
        self.assertFalse(experiment['additionalProperties'])
        self.assertEqual(experiment['properties']['mapping']['required'],['existing_behaviour'])
        harness=experiment['properties']['harness']
        self.assertEqual(harness['additionalProperties']['type'],'string')
        self.assertNotIn('const',harness);self.assertNotIn('enum',harness)
        self.assertNotIn('assert!',json.dumps(fmt));self.assertNotIn('assert_eq!',json.dumps(fmt))
    def test_native_extra_field_rejection_unchanged(self):
        value={'experiment':{'harness':{},'cases':[],'mapping':{},'rationale':'extra'}}
        with self.assertRaisesRegex(e.StageValidationError,'unsupported_experiment_fields'):
            e.validate_stage_envelope('experiment',value)
    def test_byte_limits_remain_native_not_unicode_character_claim(self):
        value={'edits':[{'path':'src/lib.rs','replacement':'authored fixture'}],'rationale':'é'*256}
        e.validate_stage_envelope('boundary_patch',value)
        value['rationale']+='é'
        #257 Unicode codepoints is below JSON Schema maxLength512 but fails byte512.
        self.assertLess(len(value['rationale']),512)
        with self.assertRaisesRegex(e.StageValidationError,'invalid_rationale_length'):
            e.validate_stage_envelope('boundary_patch',value)
    def test_optional_probability_is_not_forced_null(self):
        schema=m.response_format('boundary_patch',['requested_behavior','existing_behaviour'])['json_schema']['schema']
        self.assertEqual(schema['required'],['edits','rationale'])
        self.assertEqual(schema['properties']['common_event_probability']['type'],'number')
        self.assertEqual(schema['properties']['edits']['items']['properties']['path']['enum'],['src/lib.rs'])
    def test_unknown_or_duplicate_requirements_fail_before_request(self):
        for ids in ([],['requested_behavior'],['existing_behaviour','existing_behaviour'],[None,'existing_behaviour']):
            with self.assertRaises(ValueError):m.response_format('experiment',ids)
    def test_all_mapping_keys_are_published_and_no_mutation(self):
        ids=['requested_behavior','existing_behaviour'];before=copy.deepcopy(ids)
        mapping=m.response_format('experiment',ids)['json_schema']['schema']['properties']['experiment']['properties']['mapping']
        self.assertEqual(set(mapping['properties']),set(ids));self.assertFalse(mapping['additionalProperties'])
        self.assertEqual(ids,before)

if __name__=='__main__':unittest.main()
