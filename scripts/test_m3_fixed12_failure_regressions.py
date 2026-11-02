"""Offline envelope regressions; synthetic bodies, not recovered provider responses."""
import copy
import json
from pathlib import Path
import unittest
from test_m3_edit_v3 import new, fixtures, response

ROOT = Path(__file__).resolve().parents[1]


class RetainedFailures(unittest.TestCase):
    def test_retained_envelope_categories_still_fail_closed(self):
        rows = json.loads((ROOT/'docs/m3-fixed12-result.json').read_text())['arms']
        observed = []
        for row in rows:
            if not row.get('schema_failures'):
                continue
            diagnostic = json.loads((ROOT/f"docs/m3-fixed12-execution/{row['index']}/evidence/diagnostics.json").read_text())[-1]
            shape = diagnostic['response_shape']
            # Bodies are intentionally invented harmless fixtures; only recorded shape is replayed.
            value = response({'action': 'stop'})
            if diagnostic['code'] == 'output_truncated':
                self.assertEqual(shape['finish_reason'], 'length')
                self.assertEqual(shape['tool_calls'], 'missing')
                value['choices'][0] = {'finish_reason': 'length', 'message': {'content': 'fixture'}}
            else:
                self.assertEqual(diagnostic['code'], 'tool_shape')
                self.assertEqual(shape['tool_calls'], 'multiple')
                calls = value['choices'][0]['message']['tool_calls']
                calls.append(copy.deepcopy(calls[0]))
            with self.assertRaises(new.ValidationFailure) as error:
                new.decode(value, fixtures.request())
            self.assertEqual(error.exception.code, diagnostic['code'])
            observed.append(diagnostic['code'])
        self.assertEqual(observed.count('output_truncated'), 9)
        self.assertEqual(observed.count('tool_shape'), 1)

    def test_stop_requires_no_regenerated_source_or_digest(self):
        request = fixtures.request()
        result = new.decode(response({'action': 'stop'}), request)
        self.assertEqual(result['reply']['proposal'], {'action': 'stop'})
        for extra in ('expected_digest', 'old_text', 'new_text'):
            with self.assertRaises(new.ValidationFailure):
                new.decode(response({'action': 'stop', extra: ''}), request)

    def test_decoder_never_silently_replaces_supplied_digest(self):
        request = fixtures.request()
        for digest in ('0'*64, 'f'*64):
            edit = {'action': 'edit', 'path': new.transport.allowed_path(request),
                    'expected_digest': digest, 'old_text': 'old', 'new_text': 'new', 'forecast': .5}
            result = new.decode(response(edit), request)
            self.assertEqual(result['reply']['proposal']['expected_digest'], digest)
        # Semantic current-file comparison belongs to the native controller, not this decoder.


if __name__ == '__main__':
    unittest.main()
