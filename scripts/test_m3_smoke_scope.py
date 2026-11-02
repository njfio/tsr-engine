"""Provider scope checks without network, credentials or candidate execution."""
import importlib.util
import json
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('broker', Path(__file__).with_name('m3-smoke-broker.py'))
b = importlib.util.module_from_spec(spec)
spec.loader.exec_module(b)
ROOT = Path(__file__).resolve().parents[1]

class Scope(unittest.TestCase):
    def test_frozen_metadata_and_no_hidden_oracle_in_encoded_request(self):
        for job in json.loads((ROOT/'docs/m3-smoke-comparison-plan.json').read_text())['jobs']:
            req = {'protocol':'tsr-broker@v1','id':'fixture','task':job['task_id'], 'required_reads':[job['file']], 'failing_tests':[job['test']], 'allowed_test_ids':['named-failing-test'], 'failing_test_id':'named-failing-test', 'observation':'bounded fixture'}
            body = b.encode(req)
            self.assertEqual(json.loads(body['messages'][1]['content']), req)
            for path in ['../secret', job['file']+'.extra', '/tmp/other']:
                result = {'choices':[{'message':{'tool_calls':[{'function':{'name':'propose','arguments':json.dumps(dict(action='inspect',path=path,id='',contents='',forecast=None))}}]}}], 'usage':{'total_tokens':1}}
                with self.assertRaises(b.BrokerFailure): b.decode(result, req)
            bad = dict(req, required_reads=['unrelated'])
            with self.assertRaises(b.BrokerFailure): b.encode(bad)

if __name__ == '__main__': unittest.main()

class Transport(unittest.TestCase):
    def test_mocked_dispatch_preserves_prefix_and_settles_once(self):
        import tempfile
        job = json.loads((ROOT/'docs/m3-smoke-comparison-plan.json').read_text())['jobs'][0]
        req = {'protocol':'tsr-broker@v1','id':'fixture','task':job['task_id'], 'required_reads':[job['file']], 'failing_tests':[job['test']], 'allowed_test_ids':['named-failing-test'], 'failing_test_id':'named-failing-test', 'observation':'fixture', 'remaining_tokens':200000,'deadline_ms':90000}
        old = (ROOT/'target/m3-provider-cost.jsonl').read_bytes()
        with tempfile.TemporaryDirectory() as directory:
            ledger = Path(directory)/'ledger.jsonl'
            ledger.write_bytes(old)
            def transport(route, key, data=None, timeout=30):
                if route == 'models':
                    return {'data':[{'id':b.MODEL,'pricing':{'list_input_per_million':'.30','list_output_per_million':'1.20'}}]}
                self.assertGreater(timeout,80)
                self.assertLessEqual(timeout,85)
                self.assertEqual(json.loads(ledger.read_text().splitlines()[-1])['kind'],'reserve')
                return {'choices':[{'message':{'tool_calls':[{'function':{'name':'propose','arguments':json.dumps(dict(action='stop',path='',id='',contents='',forecast=None))}}]}}], 'usage':{'total_tokens':1}, 'billing':{'status':'settled','currency':'USD','billed_cost_usd':'.001'}}
            with patch.object(b,'credential',return_value='test-only'), patch.object(b,'request_json',side_effect=transport) as network:
                result = b.run(req,ledger)
            self.assertEqual(result['reply']['proposal'],{'action':'stop'})
            self.assertEqual(network.call_count,2)
            self.assertTrue(ledger.read_bytes().startswith(old))
            self.assertEqual(len(ledger.read_bytes().splitlines()), len(old.splitlines())+2)
