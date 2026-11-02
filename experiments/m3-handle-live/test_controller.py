"""Controller lifecycle tests: fixed order, one-shot lease, fail-closed accounting."""
import copy,json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import controller as c
class Ops:
    def __init__(self):self.m={};self.calls=[]
    def guard(self,initial=False):pass
    def ledger(self):return b''
    def run(self,arm,state):
        lease=json.loads((state.parent/'current.json').read_text());assert lease['index']==arm['index']
        self.calls.append(arm['index']);return dict(exit=0,started='2026-10-18T00:00:00+00:00',finished='2026-10-18T00:00:01+00:00')
    def export_and_verify(self,arm,process,destination):
        destination.mkdir();return dict(result=dict(task=arm['task'],system='record',outcome='rejected',oracle={'evidence':{'verified_success':False}}),diagnostics=[],proposed_count=0,closure_reasons=['oracle:false'],proposal_kinds=[],visible_test_exit_codes=[],manifest_sha256='a'*64)
class Tests(unittest.TestCase):
    def test_fixed_order_and_one_shot(self):
        m=json.loads((c.ROOT/'docs/m3-handle-live-execution-freeze.json').read_text());ops=Ops()
        with tempfile.TemporaryDirectory() as temp,patch.object(c,'verify_handle_evidence',return_value={'manifest_sha256':'b'*64}):
            state=Path(temp)/'state';results=c.coordinate(m,state,ops)
            self.assertEqual(ops.calls,list(range(12)));self.assertEqual(len(results),12)
            self.assertFalse((state/'current.json').exists());self.assertEqual(json.loads((state/'terminal.json').read_text())['status'],'completed')
            with self.assertRaises(ValueError):c.coordinate(m,state,ops)
    def test_guard_failure_never_retries(self):
        m=json.loads((c.ROOT/'docs/m3-handle-live-execution-freeze.json').read_text());ops=Ops()
        def fail(*args):raise ValueError('test integrity failure')
        ops.export_and_verify=fail
        with tempfile.TemporaryDirectory() as temp:
            state=Path(temp)/'state'
            with self.assertRaises(ValueError):c.coordinate(m,state,ops)
            self.assertEqual(ops.calls,[0]);self.assertEqual(json.loads((state/'terminal.json').read_text())['status'],'halted-review-required')
    def test_unbound_accounting_refused(self):
        with self.assertRaises(ValueError):c.bind_accounting(b'',b'change',[])
if __name__=='__main__':unittest.main()
