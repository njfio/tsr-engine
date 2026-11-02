"""Offline protocol tests: encode only, no credential access or HTTP calls."""
import importlib.util
import pathlib
import unittest
spec = importlib.util.spec_from_file_location('progress_broker', pathlib.Path(__file__).with_name('m3-progress-broker.py'))
broker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(broker)

class ProgressProtocol(unittest.TestCase):
    def request(self):
        return dict(protocol='tsr-broker@v2',id='a'*64,task='0c5a91a6aaa862c279aed1e1602a74eda7c42c4fc0f72247de513fa948f50b70',state_root='b'*64,state_version=0,remaining_tokens=100000,remaining_action_calls=3,deadline_ms=1000,observation='inert fixture',allowed_test_ids=['named-failing-test'],required_reads=['crates/rts-daemon/src/error.rs'],failing_tests=['error::tests::protocol_error_round_trips_to_wire'],failing_test_id='named-failing-test',progress=dict(source_state='c'*64,inspection_current=True,test_current=True,submission='stop submits for independent final oracle'))
    def test_progress_and_submission_survive_encoding(self):
        body=broker.encode(self.request())
        self.assertIn('remaining_action_calls',body['messages'][1]['content'])
        self.assertIn('independent final oracle',body['messages'][0]['content'])
        self.assertIn('inspection_current',body['messages'][1]['content'])
    def test_legacy_and_incomplete_requests_refuse(self):
        request=self.request();request['protocol']='tsr-broker@v1'
        with self.assertRaises(broker.BrokerFailure):broker.encode(request)
        request=self.request();del request['remaining_action_calls']
        with self.assertRaises(broker.BrokerFailure):broker.encode(request)
        request=self.request();request['remaining_action_calls']=0
        with self.assertRaises(broker.BrokerFailure):broker.encode(request)
if __name__=='__main__':unittest.main()
