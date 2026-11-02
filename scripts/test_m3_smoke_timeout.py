"""M3-TIMEOUT-01: no network, sleeps, credentials or real charge mutation."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
spec=importlib.util.spec_from_file_location('b',Path(__file__).with_name('m3-smoke-broker.py'))
b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
ROOT=Path(__file__).resolve().parents[1]

class Timeout(unittest.TestCase):
    def request(self, milliseconds=1800000):
        job=json.loads((ROOT/'docs/m3-smoke-comparison-plan.json').read_text())['jobs'][0]
        return dict(protocol='tsr-broker@v1',id='fixture',task=job['task_id'],required_reads=[job['file']],failing_tests=[job['test']],allowed_test_ids=['named-failing-test'],failing_test_id='named-failing-test',observation='fixture',remaining_tokens=200000,deadline_ms=milliseconds)
    def response(self):
        return {'choices':[{'message':{'tool_calls':[{'function':{'name':'propose','arguments':json.dumps(dict(action='stop',path='',id='',contents='',forecast=None))}}]}}], 'usage':{'total_tokens':1},'billing':{'status':'settled','currency':'USD','billed_cost_usd':'.001'}}
    def attempt(self, remaining=1800000, elapsed=0, fail=False):
        with tempfile.TemporaryDirectory() as directory:
            ledger=Path(directory)/'ledger.jsonl'
            baseline=(ROOT/'target/m3-provider-cost.jsonl').read_bytes();ledger.write_bytes(baseline)
            clock=[0.0];calls=[]
            def transport(route,key,data=None,timeout=30):
                calls.append((route,timeout))
                if route=='models':
                    clock[0]+=elapsed
                    return {'data':[{'id':b.MODEL,'pricing':{'list_input_per_million':'.30','list_output_per_million':'1.20'}}]}
                if fail: raise TimeoutError('offline timeout')
                clock[0]+=206
                return self.response()
            with patch.object(b,'credential',return_value='inert'),patch.object(b,'request_json',side_effect=transport),patch.object(b.time,'monotonic',side_effect=lambda:clock[0]):
                try: result=b.complete_with_diagnostics(self.request(remaining),ledger);error=None
                except Exception as exc: result=None;error=exc
            raw=ledger.read_bytes()
            if fail:
                events=[json.loads(line) for line in raw[len(baseline):].splitlines()]
                self.assertEqual([e['kind'] for e in events],['reserve'])
                self.assertEqual(events[0]['usd'],'2')
                diagnostic=json.loads(ledger.with_name('ledger-diagnostics.jsonl').read_text())
                self.assertFalse(diagnostic['settled']);self.assertEqual(diagnostic['status'],'failed')
                with patch.object(b,'credential') as key,patch.object(b,'request_json') as network:
                    with self.assertRaises(ValueError): b.run(self.request(),ledger)
                    key.assert_not_called();network.assert_not_called()
            self.assertTrue(raw.startswith(baseline))
            return calls,result,error,raw[len(baseline):]
    def test_206_second_completion_fits_300_second_transport(self):
        calls,result,error,_=self.attempt()
        self.assertIsNone(error);self.assertIsNotNone(result)
        self.assertEqual(calls,[('models',30),('chat/completions',300)])
    def test_remaining_arm_time_subtracts_elapsed_and_cleanup_margin(self):
        calls,_,error,_=self.attempt(90000,elapsed=20)
        self.assertIsNone(error)
        self.assertEqual(calls[-1],('chat/completions',65))
    def test_expired_before_dispatch_makes_no_reservation(self):
        calls,_,error,tail=self.attempt(10000,elapsed=6)
        self.assertIsNotNone(error);self.assertEqual(len(calls),1);self.assertEqual(tail,b'')
    def test_timeout_is_ambiguous_blocks_next_dispatch_without_retry(self):
        calls,_,error,_=self.attempt(fail=True)
        self.assertIsInstance(error,TimeoutError)
        self.assertEqual(len(calls),2)

if __name__=='__main__': unittest.main()
