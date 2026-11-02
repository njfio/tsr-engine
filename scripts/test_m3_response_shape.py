"""Synthetic decoder regression fixtures, never captured provider response bodies.

These reproduce the retained error class, not the unknown sixth response.
No credential, network, subprocess, or live adapter entry point is used.
"""
import importlib.util
import json
from pathlib import Path
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location(
    'frozen_validation_broker', ROOT / 'scripts/m3-progress-validation-broker.py')
broker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(broker)


def response_fixture():
    """Generated inert valid fixture; all malformed cases derive from it."""
    proposal = dict(action='stop', path='', id='', contents='', forecast=None)
    return dict(choices=[dict(finish_reason='tool_calls', message=dict(
        role='assistant', tool_calls=[dict(id='fixture', type='function',
        function=dict(name='propose', arguments=json.dumps(proposal)))]))],
        usage=dict(total_tokens=1))


class ResponseShapeRegression(unittest.TestCase):
    def setUp(self):
        self.request = dict(id='a' * 64)
        # Any accidental expansion into transport/credential work fails offline.
        for name in ('credential', 'request_json', 'run'):
            guard = patch.object(broker, name, side_effect=AssertionError('offline only'))
            guard.start()
            self.addCleanup(guard.stop)

    def assert_code(self, response, code):
        with self.assertRaises(broker.BrokerFailure) as caught:
            broker.decode(response, self.request)
        self.assertEqual(caught.exception.code, code)

    def test_valid_single_propose_envelope(self):
        result = broker.decode(response_fixture(), self.request)
        self.assertEqual(result, dict(reply=dict(request='a' * 64,
            proposal=dict(action='stop'), forecast=None), billed_tokens=1))

    def test_tool_shape_equivalence_class(self):
        for variant in ('absent', 'empty', 'multiple', 'wrong_name', 'missing_function'):
            with self.subTest(variant=variant):
                response = response_fixture()
                message = response['choices'][0]['message']
                if variant == 'absent':
                    del message['tool_calls']
                elif variant == 'empty':
                    message['tool_calls'] = []
                elif variant == 'multiple':
                    message['tool_calls'] *= 2
                elif variant == 'wrong_name':
                    message['tool_calls'][0]['function']['name'] = 'other'
                else:
                    del message['tool_calls'][0]['function']
                self.assert_code(response, 'tool_shape')

    def test_alternate_envelopes_are_not_silently_coerced(self):
        for variant in ('legacy_function_call', 'content_json', 'top_level_output'):
            with self.subTest(variant=variant):
                response = response_fixture()
                message = response['choices'][0]['message']
                function = message.pop('tool_calls')[0]['function']
                if variant == 'legacy_function_call':
                    message['function_call'] = function
                elif variant == 'content_json':
                    message['content'] = function['arguments']
                else:
                    response['output'] = [dict(type='function_call', **function)]
                self.assert_code(response, 'tool_shape')

    def test_truncation_and_refusal_precede_tool_shape(self):
        for reason, refusal, expected in (
            ('length', None, 'output_truncated'),
            ('content_filter', None, 'provider_refusal'),
            ('stop', 'inert fixture refusal', 'provider_refusal'),
        ):
            with self.subTest(reason=reason):
                response = response_fixture()
                response['choices'][0].update(finish_reason=reason,
                    message=dict(refusal=refusal))
                self.assert_code(response, expected)

    def test_absent_finish_reason_does_not_prove_no_truncation(self):
        response = response_fixture()
        del response['choices'][0]['finish_reason']
        response['choices'][0]['message'] = {}
        self.assert_code(response, 'tool_shape')

    def test_choices_and_arguments_fail_distinctly(self):
        response = response_fixture()
        response['choices'] = []
        self.assert_code(response, 'choice_count')
        response = response_fixture()
        response['choices'][0]['message']['tool_calls'][0]['function']['arguments'] = '{'
        with self.assertRaises(json.JSONDecodeError):
            broker.decode(response, self.request)

    def test_action_validation_is_preserved(self):
        cases = [
            (dict(action='host_exec', path='', id='', contents='', forecast=None), 'proposal_type'),
            (dict(action='stop', path='', id='', contents='', forecast=True), 'forecast_type'),
            (dict(action='test', path='', id='unapproved', contents='', forecast=.5), 'smoke_scope'),
            (dict(action='stop', path='', id='', contents='', forecast=None, extra=True), 'proposal_fields'),
        ]
        for args, expected in cases:
            with self.subTest(expected=expected):
                response = response_fixture()
                response['choices'][0]['message']['tool_calls'][0]['function']['arguments'] = json.dumps(args)
                self.assert_code(response, expected)

    def test_retained_sixth_diagnostic_has_no_envelope(self):
        path = ROOT / 'docs/m3-progress-validation-execution/m3-provider-cost-diagnostics.jsonl'
        rows = [json.loads(line) for line in path.read_text().splitlines()]
        rows = [r for r in rows if r.get('reservation_id') == 'd9d3943d-b1d8-45ba-9175-373f1e25351d']
        self.assertEqual(len(rows), 1)
        row = rows[0]
        self.assertEqual((row['code'], row['stage'], row['settled']), ('tool_shape', 'decode', True))
        self.assertEqual(set(row), {'at', 'code', 'elapsed_ms', 'input_bytes',
            'reservation_id', 'settled', 'stage', 'status', 'wire_bytes'})


if __name__ == '__main__':
    unittest.main()
