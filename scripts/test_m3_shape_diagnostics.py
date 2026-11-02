"""Offline synthetic metadata tests. No provider bodies are persisted."""
import ast
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

b = load('m3-shape-diagnostics-broker')
old = load('m3-progress-validation-broker')

def fixture():
    return {'choices': [{'finish_reason': 'tool_calls', 'message': {'tool_calls': [
        {'type': 'function', 'function': {'name': 'propose', 'arguments': json.dumps(
            dict(action='stop', path='', id='', contents='', forecast=None))}}]}}],
        'usage': {'total_tokens': 1},
        'billing': {'status': 'settled', 'currency': 'USD', 'billed_cost_usd': '0.001'}}

class Diagnostics(unittest.TestCase):
    def test_strict_decoder_and_encoder_unchanged(self):
        def functions(name):
            tree = ast.parse((ROOT / 'scripts' / (name + '.py')).read_text())
            return {node.name: ast.dump(node, include_attributes=False) for node in tree.body
                    if isinstance(node, ast.FunctionDef)}
        before, after = functions('m3-progress-validation-broker'), functions('m3-shape-diagnostics-broker')
        for name in before.keys() - {'run'}:
            self.assertEqual(before[name], after[name], name)

    def test_missing_multiple_and_wrong_name(self):
        for value, expected in [(None, 'non_array'), ([], 'zero'), ([{}, {}], 'multiple')]:
            r = fixture();r['choices'][0]['message']['tool_calls'] = value
            self.assertEqual(b.response_shape(r)['tool_calls'], expected)
        r = fixture();r['choices'][0]['message'] = {}
        self.assertEqual(b.response_shape(r)['tool_calls'], 'missing')
        r = fixture();r['choices'][0]['message']['tool_calls'][0]['function']['name'] = 'DO_NOT_STORE'
        shape = b.response_shape(r)
        self.assertFalse(shape['name_is_propose'])
        self.assertNotIn('DO_NOT_STORE', json.dumps(shape))

    def test_alternate_envelopes(self):
        r = fixture();message = r['choices'][0]['message']
        message['function_call'] = message.pop('tool_calls')[0]['function']
        message['content'] = 'PRIVATE_CONTENT'
        r['output'] = [{'private': 'PRIVATE_CONTENT'}]
        shape = b.response_shape(r)
        self.assertEqual(shape['legacy_function_call_type'], 'object')
        self.assertEqual(shape['content_type'], 'string')
        self.assertEqual(shape['top_level_output_type'], 'array')
        self.assertNotIn('PRIVATE_CONTENT', json.dumps(shape))

    def test_finish_refusal_truncation(self):
        for reason in ('length', 'content_filter', 'stop', 'tool_calls', 'function_call', None, 'PRIVATE'):
            r = fixture();r['choices'][0]['finish_reason'] = reason
            r['choices'][0]['message']['refusal'] = 'PRIVATE'
            shape = b.response_shape(r)
            self.assertEqual(shape['finish_reason'], 'null' if reason is None else 'other_string' if reason == 'PRIVATE' else reason)
            self.assertTrue(shape['refusal_truthy'])
            self.assertNotIn('PRIVATE', json.dumps(shape))

    def test_argument_parse_and_types(self):
        for value, expected in [('{PRIVATE', 'invalid_json'), ('"PRIVATE"', 'valid_json'),
                                 ('{}', 'valid_json'), (None, 'non_string')]:
            r = fixture();r['choices'][0]['message']['tool_calls'][0]['function']['arguments'] = value
            shape = b.response_shape(r)
            self.assertEqual(shape['argument_parse'], expected)
            self.assertNotIn('PRIVATE', json.dumps(shape))
        r = fixture();del r['choices'][0]['message']['tool_calls'][0]['function']['arguments']
        self.assertEqual(b.response_shape(r)['argument_parse'], 'missing')

    def test_malformed_shapes_are_bounded(self):
        samples = [None, [], 'PRIVATE', {'choices': [None]}, {'choices': [{}]},
            {'choices': [{'message': {'tool_calls': [None]}}]},
            {'choices': [{'message': {'tool_calls': [{'function': None}]}}]}]
        for response in samples:
            shape = b.response_shape(response)
            self.assertLess(len(json.dumps(shape)), 1024)
            self.assertNotIn('PRIVATE', json.dumps(shape))

    def test_observer_does_not_mutate_or_change_decode(self):
        responses = [fixture()]
        malformed = fixture()
        malformed['choices'][0]['message']['tool_calls'][0]['function']['arguments'] = '{'
        responses.append(malformed)
        for response in responses:
            before = copy.deepcopy(response)
            shape = b.response_shape(response)
            self.assertEqual(response, before)
            if shape.get('argument_parse') == 'invalid_json':
                with self.assertRaises(json.JSONDecodeError):
                    b.decode(response, dict(id='a'*64))
            else:
                self.assertEqual(b.decode(response, dict(id='a'*64)), old.decode(response, dict(id='a'*64)))

    def test_settled_failure_persists_only_metadata_without_retry(self):
        response = fixture()
        response['choices'][0]['message'] = {'content': 'PRIVATE_SOURCE_AND_SECRET'}
        request = dict(id='a'*64, deadline_ms=10000, remaining_tokens=100000)
        catalog = {'data': [{'id': b.MODEL, 'pricing': {
            'list_input_per_million': '.15', 'list_output_per_million': '.6'}}]}
        with tempfile.TemporaryDirectory() as directory:
            ledger = Path(directory) / 'ledger.jsonl';ledger.write_text('')
            with patch.object(b, 'credential', return_value='INERT_TEST_KEY'), \
                 patch.object(b, 'encode', return_value={}), \
                 patch.object(b, 'smoke_budget', return_value=b.D(0)), \
                 patch.object(b, 'request_json', side_effect=[catalog, response]) as transport:
                with self.assertRaises(b.BrokerFailure) as error:
                    b.complete_with_diagnostics(request, ledger)
                self.assertEqual(error.exception.code, 'tool_shape')
                self.assertEqual(transport.call_count, 2) # one catalog, one completion
            diagnostic = json.loads(ledger.with_name('ledger-diagnostics.jsonl').read_text())
            self.assertTrue(diagnostic['settled'])
            self.assertEqual(diagnostic['response_shape']['tool_calls'], 'missing')
            self.assertEqual(diagnostic['code'], 'tool_shape')
            for path in Path(directory).iterdir():
                self.assertNotIn('PRIVATE_SOURCE_AND_SECRET', path.read_text())
                self.assertNotIn('INERT_TEST_KEY', path.read_text())
            self.assertEqual([json.loads(line)['kind'] for line in ledger.read_text().splitlines()], ['reserve', 'settled'])

if __name__ == '__main__': unittest.main()
