"""Offline protocol/budget checks: no key loading and no network."""
import importlib.util
from pathlib import Path
import json
import unittest
spec=importlib.util.spec_from_file_location('broker',Path(__file__).with_name('m3-cheaper-broker.py'))
b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
class Checks(unittest.TestCase):
    def test_sized_reservation_has_tenfold_margin_and_small_request_fits_one_dollar(self):
        bound, reserve = b.sized_cost_reservation({'messages': [{'content': 'synthetic fixture'}], 'max_tokens': 2048})
        self.assertGreaterEqual(reserve, bound * 10)
        self.assertLess(reserve, b.D('1'))
        large_bound, large_reserve = b.sized_cost_reservation({'text': 'x' * 200000})
        self.assertGreater(large_reserve, reserve)
        self.assertGreaterEqual(large_reserve, large_bound * 10)
        with self.assertRaises(b.BrokerFailure): b.sized_cost_reservation({'text': 'x' * b.MAX_WIRE_BODY})

    def test_reservations_survive_failure_and_settlement_releases_unused(self):
        events=[{'kind':'reserve','id':'a','usd':'1'},{'kind':'reserve','id':'b','usd':'1'},{'kind':'settled','id':'a','usd':'0.01'}]
        self.assertEqual(b.account(map(json.dumps,events)), b.D('1.01'))
        with self.assertRaises(ValueError): b.account(map(json.dumps,events+[events[-1]]))
    def test_model_cannot_add_authority_and_refusals_are_terminal(self):
        args={'action':'stop','path':'','id':'','contents':'','forecast':None}
        result={'choices':[{'message':{'tool_calls':[{'function':{'name':'propose','arguments':json.dumps(args)}}]}}], 'usage':{'total_tokens':7}}
        self.assertEqual(b.decode(result,{'id':'fixture'})['reply']['proposal'],{'action':'stop'})
        args['argv']=['not executed'];result['choices'][0]['message']['tool_calls'][0]['function']['arguments']=json.dumps(args)
        with self.assertRaises(ValueError): b.decode(result,{'id':'fixture'})
        result['choices'][0]['message']['refusal']='refused'
        with self.assertRaises(ValueError): b.decode(result,{'id':'fixture'})
    def test_fixed_endpoint_and_bounded_request(self):
        body=b.encode({'protocol':'tsr-broker@v1','id':'fixture'})
        self.assertEqual(body['model'],'deepseek-v4.1-flash')
        self.assertEqual(body['min_discount_percent'],0)
        self.assertEqual(body['max_tokens'],16384)
        bound=(b.D(b.MAX_WIRE_BODY+8192)*b.MAX_INPUT_PRICE+b.D(b.MAX_OUTPUT)*b.MAX_OUTPUT_PRICE)/b.D(1000000)
        self.assertLess(bound,b.RESERVE/10)
        self.assertFalse(body['stream'])
        with self.assertRaises(ValueError): b.request_json('arbitrary', 'unused')

class ToolShapeChecks(unittest.TestCase):
    def test_missing_multiple_and_wrong_tool_are_terminal(self):
        valid = {'function':{'name':'propose','arguments':json.dumps({'action':'stop','path':'','id':'','contents':'','forecast':None})}}
        for calls in [[], [valid,valid], [{'function':{'name':'other','arguments':'{}'}}]]:
            with self.assertRaises(b.BrokerFailure) as error:
                b.decode({'choices':[{'message':{'tool_calls':calls}}], 'usage':{'total_tokens':1}}, {'id':'fixture'})
            self.assertEqual(error.exception.code, 'tool_shape')
        self.assertEqual(b.decode({'choices':[{'message':{'tool_calls':[valid]}}], 'usage':{'total_tokens':1}}, {'id':'fixture'})['reply']['proposal'], {'action':'stop'})

class RunChecks(unittest.TestCase):
    def setup_request(self):
        return {'protocol':'tsr-broker@v1','id':'fixture','remaining_tokens':100000,'deadline_ms':1000}
    def response(self):
        return {'choices':[{'message':{'tool_calls':[{'function':{'name':'propose','arguments':json.dumps({'action':'stop','path':'','id':'','contents':'','forecast':None})}}]}}], 'usage':{'total_tokens':7}, 'billing':{'status':'settled','currency':'USD','billed_cost_usd':'0.001'},'request_id':'fixture'}
    def attempt(self, initial, response=None, fail=False, pricing=None):
        import tempfile
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as directory:
            ledger=Path(directory)/'cost.jsonl';ledger.write_text(''.join(json.dumps(e)+'\n' for e in initial))
            def transport(route,key,data=None,timeout=30):
                if route=='models':return {'data':[{'id':b.MODEL,'pricing':pricing or {'list_input_per_million':'.30','list_output_per_million':'1.20'}}]}
                events=[json.loads(line) for line in ledger.read_text().splitlines()]
                self.assertEqual(events[-1]['kind'],'reserve')
                if fail: raise TimeoutError('synthetic')
                return response or self.response()
            with patch.object(b,'credential',return_value='unused'),patch.object(b,'request_json',side_effect=transport):
                try: value=b.run(self.setup_request(),ledger); error=None
                except Exception as e: value=None;error=e
            lines=ledger.read_text().splitlines()
            if any(json.loads(line)['kind']=='halt' for line in lines):
                with self.assertRaises(ValueError): b.account(lines)
                with patch.object(b,'credential',return_value='unused'),patch.object(b,'request_json',side_effect=transport) as mock:
                    with self.assertRaises(ValueError): b.run(self.setup_request(),ledger)
                    self.assertEqual([call.args[0] for call in mock.call_args_list], ['models'])
                return value,error,'halted'
            return value,error,b.account(lines)
    def test_cap_boundaries_and_unknown_charges_stay_reserved(self):
        def initial(n):return [{'kind':'reserve','id':str(i),'usd':'1'} for i in range(n)]
        value,error,used=self.attempt(initial(98));self.assertIsNone(error);self.assertEqual(used,b.D('98.001'))
        value,error,used=self.attempt(initial(99));self.assertIsNotNone(error);self.assertEqual(used,b.D('99'))
        value,error,used=self.attempt(initial(100));self.assertIsNotNone(error);self.assertEqual(used,b.D('100'))
        value,error,used=self.attempt([],fail=True);self.assertIsNotNone(error);self.assertEqual(used,b.D('2'))
        for billing in [{},{'status':'pending'},{'status':'settled','currency':'USD','billed_cost_usd':'9'}]:
            response=self.response();response['billing']=billing
            _,error,used=self.attempt([],response=response);self.assertIsNotNone(error);self.assertEqual(used,'halted' if billing.get('billed_cost_usd')=='9' else b.D('2'))
    def test_revised_price_bound_and_invalid_rates(self):
        self.assertEqual((b.D(b.MAX_WIRE_BODY+8192)*b.MAX_INPUT_PRICE+b.D(b.MAX_OUTPUT)*b.MAX_OUTPUT_PRICE)/b.D(1000000), b.D('.1121184'))
        for key in ['list_input_per_million', 'list_output_per_million']:
            for value in ['NaN', 'Infinity', '-.01', '.300001' if key == 'list_input_per_million' else '1.200001']:
                pricing = {'list_input_per_million':'.30', 'list_output_per_million':'1.20'}
                pricing[key] = value
                _, error, used = self.attempt([], pricing=pricing)
                self.assertIsInstance(error, b.BrokerFailure)
                self.assertEqual(error.code, 'price_bound')
                self.assertEqual(used, b.D(0))
    def test_settled_but_malformed_proposal_preserves_actual_charge(self):
        response=self.response();response['choices']=[]
        _,error,used=self.attempt([],response=response);self.assertIsNotNone(error);self.assertEqual(used,b.D('.001'))


class WireLimitChecks(unittest.TestCase):
    def test_protocol_input_and_wire_limits_are_distinct(self):
        for observation in ['"' * 28000, '\\' * 28000, '\n' * 28000, '界' * 18000, '😀' * 14000]:
            request = {'protocol':'tsr-broker@v1','id':'fixture','observation':observation}
            raw = json.dumps(request, ensure_ascii=False, separators=(',', ':')).encode()
            self.assertLess(len(raw), b.MAX_INPUT)
            encoded = json.dumps(b.encode(request)).encode()
            self.assertGreater(len(encoded), 70000)
            self.assertLessEqual(len(encoded), b.MAX_WIRE_BODY)
            self.assertEqual(json.loads(b.encode(request)['messages'][1]['content']), request)


class DiagnosticChecks(unittest.TestCase):
    def test_fixed_codes_do_not_record_exception_text_or_payload(self):
        import tempfile
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as directory:
            ledger = Path(directory)/'cost.jsonl'
            request = {'protocol':'tsr-broker@v1','observation':'PRIVATE_FIXTURE_MARKER'}
            with patch.object(b, 'run', side_effect=RuntimeError('SECRET_FIXTURE_MARKER')):
                with self.assertRaises(RuntimeError): b.complete_with_diagnostics(request, ledger)
            raw = ledger.with_name('cost-diagnostics.jsonl').read_text()
            self.assertNotIn('PRIVATE_FIXTURE_MARKER', raw)
            self.assertNotIn('SECRET_FIXTURE_MARKER', raw)
            self.assertEqual(json.loads(raw)['code'], 'transport_or_internal')
    def test_refusal_and_truncation_are_distinct_terminal_codes(self):
        for reason, code in [('content_filter','provider_refusal'),('length','output_truncated')]:
            with self.assertRaises(b.BrokerFailure) as error:
                b.decode({'choices':[{'finish_reason':reason,'message':{}}]}, {'id':'fixture'})
            self.assertEqual(error.exception.code, code)
    def test_settled_decode_failure_keeps_settlement_and_diagnostic(self):
        import tempfile
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as directory:
            ledger = Path(directory)/'cost.jsonl'
            def transport(route,key,data=None,timeout=30):
                if route=='models':return {'data':[{'id':b.MODEL,'pricing':{'list_input_per_million':'.15','list_output_per_million':'.60'}}]}
                return {'choices':[], 'billing':{'status':'settled','currency':'USD','billed_cost_usd':'.001'}}
            with patch.object(b,'credential',return_value='unused'),patch.object(b,'request_json',side_effect=transport):
                with self.assertRaises(b.BrokerFailure): b.complete_with_diagnostics({'protocol':'tsr-broker@v1','remaining_tokens':200000,'deadline_ms':1000}, ledger)
            self.assertEqual(b.account(ledger.read_text().splitlines()), b.D('.001'))
            diagnostic = json.loads(ledger.with_name('cost-diagnostics.jsonl').read_text())
            self.assertEqual(diagnostic['code'], 'choice_count')
            self.assertEqual(diagnostic['stage'], 'decode')
            self.assertTrue(diagnostic['settled'])

class CohortAccountingChecks(unittest.TestCase):
    def test_unsettled_charge_without_diagnostic_blocks_next_arm(self):
        import tempfile
        spec = importlib.util.spec_from_file_location('cohort', Path(__file__).with_name('m3-paired-cohort.py'))
        cohort = importlib.util.module_from_spec(spec); spec.loader.exec_module(cohort)
        with tempfile.TemporaryDirectory() as directory:
            ledger=Path(directory)/'cost.jsonl'
            reserve={'kind':'reserve','id':'killed-broker','usd':'1'}
            settled={'kind':'settled','id':'killed-broker','usd':'.001'}
            ledger.write_text(json.dumps(reserve)+'\n')
            with self.assertRaises(RuntimeError): cohort.require_reconciled(ledger)
            ledger.write_text(json.dumps(reserve)+'\n'+json.dumps(settled)+'\n')
            cohort.require_reconciled(ledger)
            with ledger.open('a') as file:file.write(json.dumps(settled)+'\n')
            with self.assertRaises(RuntimeError): cohort.require_reconciled(ledger)

class HandoffIdentityChecks(unittest.TestCase):
    def test_reused_pid_or_resumed_coordinator_is_rejected(self):
        from unittest.mock import patch
        spec = importlib.util.spec_from_file_location('handoff', Path(__file__).with_name('m3-start-approved-paired.py'))
        handoff = importlib.util.module_from_spec(spec); spec.loader.exec_module(handoff)
        identity = {'pid':123,'command':'expected controller','start':'original start'}
        with patch.object(handoff.subprocess, 'check_output', side_effect=['expected controller','original start','T']):
            handoff.verify_paused(identity)
        for values in [['different controller'], ['expected controller','different start'], ['expected controller','original start','S']]:
            with patch.object(handoff.subprocess, 'check_output', side_effect=values):
                with self.assertRaises(RuntimeError):handoff.verify_paused(identity)

class QueueIsolationChecks(unittest.TestCase):
    def test_queue_injects_distinct_startup_stores(self):
        import tempfile
        from unittest.mock import patch, Mock
        spec = importlib.util.spec_from_file_location('queue_test', Path(__file__).with_name('m3-detached-qualification.py'))
        queue = importlib.util.module_from_spec(spec); spec.loader.exec_module(queue)
        with tempfile.TemporaryDirectory() as directory:
            queue.STATE = Path(directory)
            child = Mock(pid=12345, returncode=0)
            child.poll.return_value = 0
            with patch.object(queue.subprocess, 'Popen', return_value=child) as spawn, patch.object(queue.subprocess, 'run', return_value=Mock(stdout='')):
                env = {'TEST_ONLY':'1'}
                queue.run('first', ['unused'], env)
                first = spawn.call_args.kwargs['env']['TSR_DB']
                queue.run('second', ['unused'], env)
                second = spawn.call_args.kwargs['env']['TSR_DB']
            self.assertNotEqual(first, second)
            self.assertTrue(Path(first).is_absolute())
            self.assertNotIn('TSR_DB', env)

class StartupIsolationChecks(unittest.TestCase):
    def test_parallel_arms_do_not_share_sqlite_startup_lock(self):
        import sqlite3
        import tempfile
        spec = importlib.util.spec_from_file_location('retry', Path(__file__).with_name('m3-retry-pilot.py'))
        retry = importlib.util.module_from_spec(spec); spec.loader.exec_module(retry)
        with tempfile.TemporaryDirectory() as directory:
            paths = [retry.startup_store(Path(directory)/source, mode)
                     for source, mode in [('agentid','record'), ('rts','record'), ('rts','rules')]]
            self.assertEqual(len(set(paths)), 3)
            connections = []
            try:
                for path in paths:
                    Path(path).parent.mkdir(exist_ok=True)
                    connection = sqlite3.connect(path, timeout=0)
                    connections.append(connection)
                    connection.execute('BEGIN EXCLUSIVE')
                    connection.execute('CREATE TABLE startup (id INTEGER)')
                with sqlite3.connect(paths[0], timeout=0) as same:
                    with self.assertRaises(sqlite3.OperationalError): same.execute('BEGIN EXCLUSIVE')
            finally:
                for connection in connections: connection.close()

if __name__=='__main__':unittest.main()
