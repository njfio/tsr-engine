import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


def module(name):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(name+'.py'))
    result=importlib.util.module_from_spec(spec);spec.loader.exec_module(result);return result

recovery=module('engine-response-recovery')
codec=module('engine-provider-envelope')

class ResponseTests(unittest.TestCase):
    def test_truncation_is_rejected_before_json_decode(self):
        with self.assertRaisesRegex(ValueError,'output_truncated'):
            codec.decode_response({'protocol':'tsr-proposal@v1'}, {'choices':[{'finish_reason':'length','message':{'content':'{'}}]})

    def test_diagnostics_are_bounded_private_redacted_and_no_overwrite(self):
        with tempfile.TemporaryDirectory() as folder:
            path=Path(folder)/'response.json'
            recovery.save_response(path, {'choices':[{'finish_reason':'length','message':{'content':'secret-token'+'x'*100000}}]}, 'secret-token')
            self.assertNotIn('secret-token',path.read_text())
            self.assertTrue(json.loads(path.read_text())['truncated'])
            self.assertLess(path.stat().st_size,65536)
            self.assertEqual(path.stat().st_mode & 0o777,0o600)
            with self.assertRaises(FileExistsError):recovery.save_response(path,{},None)

    def test_escaped_credential_and_malformed_message_are_safe(self):
        with tempfile.TemporaryDirectory() as folder:
            path=Path(folder)/'response.json'
            secret='token"with\\escape'
            recovery.save_response(path, {'choices':[{'message':{'content':secret}}]}, secret)
            preview=json.loads(json.loads(path.read_text())['diagnostic_preview'])
            self.assertEqual(preview['content'], '[REDACTED_CREDENTIAL]')
            recovery.save_response(Path(folder)/'malformed.json', {'choices':[None]}, None)

    def test_recovery_requires_settlement_unchanged_state_and_no_execution(self):
        events=[{'kind':'started','step_id':'s','state_root':'root'},{'kind':'failed','step_id':'s','reason':'output_truncated'}]
        ledger=[{'kind':'settled','id':'r','usage':{'total_tokens':200}}]
        diagnostics=[{'reservation_id':'r','stage':'decode','settled':True,'status':'failed'}]
        request={'state_root':'root','pending':None}
        self.assertEqual(recovery.validate_recovery(events,ledger,diagnostics,'r',request,2)['completion_tokens'],4096)
        for changed in [dict(request,state_root='other'),dict(request,pending={})]:
            with self.assertRaises(ValueError):recovery.validate_recovery(events,ledger,diagnostics,'r',changed,2)
        with self.assertRaises(ValueError):recovery.validate_recovery(events,[],diagnostics,'r',request,2)
        with self.assertRaises(ValueError):recovery.validate_recovery(events+[{'kind':'execution_started'}],ledger,diagnostics,'r',request,2)

if __name__=='__main__':unittest.main()
