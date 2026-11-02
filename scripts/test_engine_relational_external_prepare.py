import importlib.util,json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
spec=importlib.util.spec_from_file_location('prep',Path(__file__).with_name('engine-relational-external-prepare.py'))
p=importlib.util.module_from_spec(spec);spec.loader.exec_module(p)
class PreparationTests(unittest.TestCase):
    def test_only_frozen_synthetic_fields_and_never_dispatch(self):
        for pair in ['A','B']:
            r=p.readiness(pair);self.assertFalse(r['dispatch_enabled']);self.assertFalse(r['cap_guaranteed'])
            self.assertEqual(r['approval'],'pending');self.assertIsNone(r['conditional_estimate_usd'])
            c=json.loads(r['payload']['messages'][1]['content'])
            self.assertEqual(set(c),{'synthetic_only','parents','synthetic_premise'})
            self.assertTrue(all(x['text'].startswith('SYNTHETIC BENCHMARK:') for x in c['parents']))
            self.assertNotIn('tools',r['payload']);self.assertNotIn('state_root',json.dumps(c))
        with self.assertRaises(ValueError):p.payload('private-run')
    def test_tampered_allowlist_fails_without_export(self):
        with tempfile.TemporaryDirectory() as d:
            path=Path(d)/'input.json';path.write_text('{"private":"secret"}')
            with patch.object(p,'INPUT',path),self.assertRaises(ValueError):p.payload('A')
    def test_price_checks_are_conditional_not_cap_guarantee(self):
        r=p.readiness('A',{'list_input_per_million':'.30','list_output_per_million':'1.20'})
        self.assertTrue(r['pricing_within_assumed_bound']);self.assertFalse(r['cap_guaranteed']);self.assertFalse(r['dispatch_enabled'])
        for rates in [{},{'list_input_per_million':'NaN','list_output_per_million':'1'},
                      {'list_input_per_million':'.31','list_output_per_million':'1'},
                      {'list_input_per_million':True,'list_output_per_million':'1'}]:
            with self.assertRaises((ValueError,ArithmeticError)):p.readiness('A',rates)
