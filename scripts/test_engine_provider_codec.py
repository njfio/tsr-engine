import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('codec', Path(__file__).with_name('engine-provider-envelope.py'))
codec = importlib.util.module_from_spec(spec)
spec.loader.exec_module(codec)


class CodecTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.request = Path(self.temp.name) / 'request.json'
        self.completion = Path(self.temp.name) / 'completion.json'
        self.reply = {'protocol': 'tsr-proposal@v1', 'state_root': 'a' * 64,
                      'proposals': [{'kind': 'exact_edit', 'patch': {'path': 'implementation.rs',
                         'inspection_handle': 'b' * 64, 'source_digest': 'c' * 64,
                         'old_text': 'a-b', 'new_text': 'a+b'}, 'rationale': 'Try addition'}]}
        self.request.write_text(json.dumps({'protocol': self.reply['protocol'],
                                           'state_root': self.reply['state_root'], 'instructions': 'Propose'}))

    def wire(self, **overrides):
        choice = {'finish_reason': 'stop', 'message': {'role': 'assistant', 'content': json.dumps(self.reply)}}
        choice.update(overrides)
        self.completion.write_text(json.dumps({'choices': [choice]}))

    def test_attention_requires_exact_request_binding(self):
        request = json.loads(self.request.read_text())
        request['next_attention'] = {'binding': 'bound'}
        self.request.write_text(json.dumps(request))
        self.wire()
        with self.assertRaises(ValueError):
            codec.decode(self.request, self.completion)
        self.reply['attention_binding'] = 'wrong'
        self.wire()
        with self.assertRaises(ValueError):
            codec.decode(self.request, self.completion)
        self.reply['attention_binding'] = 'bound'
        self.wire()
        self.assertEqual(codec.decode(self.request, self.completion), self.reply)

    def test_round_trip_and_offline_envelope(self):
        self.wire()
        self.assertEqual(codec.decode(self.request, self.completion), self.reply)
        self.assertFalse(codec.envelope(self.request)['dispatch_enabled'])

    def test_unicode_protocol_bound_is_distinct_from_wire_escaping(self):
        request = {'protocol': 'tsr-proposal@v1', 'instructions': 'Propose', 'source': '界' * 7000}
        self.assertLess(len(json.dumps(request, ensure_ascii=False).encode()), 32768)
        self.assertGreater(len(json.dumps(request).encode()), 32768)
        self.assertFalse(codec.envelope_request(request)['dispatch_enabled'])

    def test_truncation_refusal_tools_and_foreign_state(self):
        for reason in ['length', 'tool_calls', None]:
            self.wire(finish_reason=reason)
            with self.assertRaises(ValueError):
                codec.decode(self.request, self.completion)
        for extra in [{'refusal': 'declined'}, {'tool_calls': [{}]}, {'function_call': {'name': 'x'}}]:
            self.wire(message={'role': 'assistant', 'content': json.dumps(self.reply), **extra})
            with self.assertRaises(ValueError):
                codec.decode(self.request, self.completion)
        self.reply['state_root'] = 'd' * 64
        self.wire()
        with self.assertRaises(ValueError):
            codec.decode(self.request, self.completion)

    def test_duplicate_fields_and_oversized_content(self):
        self.wire(message={'role': 'assistant', 'content': '{"protocol":"x","protocol":"y"}'})
        with self.assertRaises(ValueError):
            codec.decode(self.request, self.completion)
        self.wire(message={'role': 'assistant', 'content': 'x' * 8193})
        with self.assertRaises(ValueError):
            codec.decode(self.request, self.completion)


if __name__ == '__main__':
    unittest.main()
