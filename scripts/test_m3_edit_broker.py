"""Offline adapter tests and fixed decoder bridge for inert Rust integration fixtures."""
import importlib.util
import json
from pathlib import Path
import sys
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('edit_broker', Path(__file__).with_name('m3-edit-broker.py'))
broker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(broker)


def request():
    return dict(protocol='tsr-broker@v2', id='a'*64,
        task='0c5a91a6aaa862c279aed1e1602a74eda7c42c4fc0f72247de513fa948f50b70',
        state_root='b'*64, state_version=0, remaining_tokens=100000,
        remaining_action_calls=8, deadline_ms=1000, observation='inert fixture',
        allowed_test_ids=['named-failing-test'], required_reads=['crates/rts-daemon/src/error.rs'],
        failing_tests=['error::tests::protocol_error_round_trips_to_wire'], failing_test_id='named-failing-test',
        progress=dict(source_state='c'*64, inspection_current=True, test_current=True, submission='stop'))


def response(action='stop', **kwargs):
    args = {k: '' for k in broker.FIELDS}
    args.update(action=action, forecast=None)
    args.update(kwargs)
    return dict(choices=[dict(finish_reason='tool_calls', message=dict(tool_calls=[
        dict(type='function', function=dict(name='propose', arguments=json.dumps(args)))]))],
        usage=dict(total_tokens=1))


class EditAdapter(unittest.TestCase):
    def test_schema_and_valid_edit(self):
        with patch.object(broker.transport, 'request_json', side_effect=AssertionError('network forbidden')):
            body = broker.encode(request())
            self.assertEqual(body['tool_choice'], 'required')
            self.assertNotIn('parallel_tool_calls', body)
            props = body['tools'][0]['function']['parameters']['properties']
            self.assertEqual(props['action']['enum'], ['inspect', 'test', 'edit', 'stop'])
            self.assertIn('exactly ONE', body['messages'][0]['content'])
            r = response('edit', path=request()['required_reads'][0], expected_digest='d'*64,
                         old_text='broken()', new_text='fixed()', forecast=.5)
            self.assertEqual(broker.decode(r, request())['reply']['proposal']['action'], 'edit')
            self.assertEqual(broker.transport.decode, broker.decode)
            self.assertEqual(broker.transport.encode, broker.encode)

    def test_multiple_missing_and_malformed_calls(self):
        for calls in ([], [None], [{}, {}], None, 'not calls'):
            with self.subTest(calls=calls):
                r = response()
                r['choices'][0]['message']['tool_calls'] = calls
                with self.assertRaises(broker.BrokerFailure): broker.decode(r, request())
        r = response()
        r['choices'][0]['message']['tool_calls'] *= 2
        with self.assertRaises(broker.BrokerFailure): broker.decode(r, request())
        for arguments in ('{', '[]', '{}', '{"action":"stop","action":"edit"}'):
            r = response()
            r['choices'][0]['message']['tool_calls'][0]['function']['arguments'] = arguments
            with self.assertRaises((ValueError, broker.BrokerFailure)): broker.decode(r, request())

    def test_invalid_edit_and_legacy_replace_fail_closed(self):
        valid = dict(path=request()['required_reads'][0], expected_digest='d'*64,
                     old_text='broken()', new_text='fixed()', forecast=.5)
        for update in (dict(old_text=''), dict(new_text='broken()'), dict(expected_digest='bad'),
                       dict(forecast=None), dict(path='../escape'), dict(id='unexpected'),
                       dict(forecast=float('nan')), dict(forecast=True)):
            with self.subTest(update=update):
                with self.assertRaises(broker.BrokerFailure):
                    broker.decode(response('edit', **(valid | update)), request())
        with self.assertRaises(broker.BrokerFailure): broker.decode(response('replace'), request())


if __name__ == '__main__':
    if sys.argv[1:] == ['--decode-fixture']:
        # The live wrapper and actual reservation/settlement path, with only credential
        # lookup and HTTP replaced. Ledger and diagnostics exist solely in this tempdir.
        import tempfile
        spec = importlib.util.spec_from_file_location('live', Path(__file__).with_name('m3-exact-live-broker.py'))
        live = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(live)
        data = json.load(sys.stdin)
        with tempfile.TemporaryDirectory() as directory:
            ledger = Path(directory) / 'ledger.jsonl'
            ledger.write_bytes(b'')
            baseline = Path(directory) / 'baseline.json'
            baseline.write_text(json.dumps(live.budget.history.freeze(b'')))
            result = data['response']
            result['billing'] = dict(status='settled', currency='USD', billed_cost_usd='0.0001')
            result['request_id'] = 'inert-fixture'
            def fake_http(route, key, body=None, **kwargs):
                if route == 'models':
                    return {'data': [dict(id=live.edit.transport.MODEL,
                        pricing=dict(list_input_per_million='0.15', list_output_per_million='0.60'))]}
                if route == 'chat/completions':
                    assert body['tools'][0]['function']['parameters']['properties']['action']['enum'] == ['inspect', 'test', 'edit', 'stop']
                    return result
                raise AssertionError('unexpected fixture route')
            with patch.object(live.budget, 'BASELINE', baseline), \
                 patch.object(live.edit.transport, 'credential', return_value='inert-fixture'), \
                 patch.object(live.edit.transport, 'request_json', side_effect=fake_http):
                live.encode(data['request'])
                try:
                    print(json.dumps(live.edit.transport.complete_with_diagnostics(data['request'], str(ledger))))
                except (ValueError, TypeError, KeyError):
                    sys.exit(2)

    else:
        unittest.main()
