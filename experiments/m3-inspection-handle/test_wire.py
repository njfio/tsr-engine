import importlib.util
import sys
from pathlib import Path
import unittest
sys.path.insert(0,str(Path(__file__).resolve().parents[2]/'scripts'))
from test_m3_edit_v2 import response, fixtures
spec=importlib.util.spec_from_file_location('handle_broker',Path(__file__).with_name('broker.py'))
broker=importlib.util.module_from_spec(spec);spec.loader.exec_module(broker)
def request():
    return fixtures.request()|{'protocol':'tsr-broker@inspection-handle-v1','inspection':None}

class Wire(unittest.TestCase):
    def test_handle_forwarded_without_digest_substitution(self):
        req=request()|{'inspection':{'handle':'b'*64,'path':'src/lib.rs'}}
        value=broker.examples(req)[2]|{'inspection_handle':'c'*64}
        got=broker.decode(response(value),req)
        self.assertEqual(got['reply']['proposal']['inspection_handle'],'c'*64)
        self.assertNotIn('expected_digest',got['reply']['proposal'])
    def test_multiple_calls_rejected(self):
        r=response({'action':'stop'});r['choices'][0]['message']['tool_calls']*=2
        with self.assertRaises(broker.ValidationFailure):broker.decode(r,request())
    def test_truncation_rejected(self):
        r=response({'action':'stop'});r['choices'][0]['finish_reason']='length'
        with self.assertRaises(broker.ValidationFailure):broker.decode(r,request())
    def test_paid_dispatch_disabled(self):
        for f in (broker.run,broker.transport.run,broker.transport.credential,broker.transport.request_json):
            with self.assertRaises(RuntimeError):f()
    def test_instructions_use_structured_handle(self):
        body=broker.encode(request())
        self.assertIn('inspection.handle',body['messages'][0]['content'])
        self.assertNotIn('expected_digest',str(body['tools']))
if __name__=='__main__':unittest.main()
