"""Offline characterization of the frozen adapter and retained failure evidence.

No provider requests, credentials, candidate execution, or evidence writes.
"""
import hashlib
import importlib.util
import json
from pathlib import Path
import sqlite3
import struct
import subprocess
import unittest
from unittest.mock import patch
import test_m3_progress_broker as progress

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('frozen_broker', ROOT / 'scripts/m3-continuation-broker.py')
broker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(broker)


class FailureContracts(unittest.TestCase):
    def test_request_requires_tool_but_has_no_parallel_control(self):
        with patch.object(broker, 'request_json', side_effect=AssertionError('network forbidden')):
            body = broker.encode(progress.ProgressProtocol().request())
        self.assertEqual(body['tool_choice'], 'required')
        self.assertEqual([t['function']['name'] for t in body['tools']], ['propose'])
        self.assertNotIn('parallel_tool_calls', body)

    def test_two_individually_valid_calls_are_rejected_not_selected(self):
        request = progress.ProgressProtocol().request()
        call = dict(type='function', function=dict(name='propose', arguments=json.dumps(
            dict(action='stop', path='', id='', contents='', forecast=None))))
        response = dict(choices=[dict(finish_reason='tool_calls', message=dict(tool_calls=[call]))],
                        usage=dict(total_tokens=1))
        self.assertEqual(broker.decode(response, request)['reply']['proposal'], {'action': 'stop'})
        response['choices'][0]['message']['tool_calls'].append(call)
        self.assertEqual(broker.response_shape(response)['tool_calls'], 'multiple')
        with self.assertRaises(broker.BrokerFailure) as error:
            broker.decode(response, request)
        self.assertEqual(error.exception.code, 'tool_shape')

    def test_final_source_binds_to_last_two_proposals_and_inspections(self):
        export = ROOT / 'docs/m3-continuation-execution'
        manifest = json.loads((ROOT / 'docs/m3-continuation-export-sha256.json').read_text())
        self.assertEqual({str(p.relative_to(export)) for p in export.rglob('*') if p.is_file()}, set(manifest))
        for relative, expected in manifest.items():
            self.assertEqual(hashlib.sha256((export / relative).read_bytes()).hexdigest(), expected, relative)
        raw = (export / 'unverified-raw-1/allowed-implementation-source.txt').read_bytes()
        proposal = json.dumps(dict(action='replace', path='crates/rts-mcp/src/roots.rs',
                                   contents=raw.decode()), ensure_ascii=False, separators=(',', ':')).encode()
        def digest(value):
            return subprocess.check_output([str(ROOT / 'target/m3-blake3-checksum')],
                input=struct.pack('<Q', len(value)) + value).decode().strip()
        dbpath = export / 'evidence-1/47c787ed4f89-record-state.sqlite'
        db = sqlite3.connect(dbpath.as_uri() + '?mode=ro&immutable=1', uri=True)
        try:
            events = {seq: json.loads(payload)['kind'] for seq, payload in
                      db.execute('select seq,payload from events')}
        finally:
            db.close()
        for seq in (44, 50):
            self.assertEqual(digest(proposal), events[seq]['candidate']['args'])
        for seq in (49, 55):
            self.assertEqual(digest(raw), events[seq]['observation']['evidence']['digest'])
        self.assertNotEqual(digest(proposal), events[12]['candidate']['args'])


if __name__ == '__main__':
    unittest.main()
