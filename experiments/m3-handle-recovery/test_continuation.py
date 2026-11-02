import copy,json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import controller as c
class Ops:
    def __init__(self):self.m={};self.calls=[]
    def guard(self,initial=False):pass
    def ledger(self):return b''
    def run(self,arm,state):
        self.calls.append(arm['index']);lease=json.loads((state.parent/'current.json').read_text());assert lease['index']==arm['index'];return dict(exit=0,started='2026-10-18T14:00:00+00:00',finished='2026-10-18T14:00:01+00:00')
    def export_and_verify(self,arm,process,destination):
        destination.mkdir();return dict(result=dict(task=arm['task'],system='record',outcome='rejected',oracle={'evidence':{'verified_success':False}}),diagnostics=[],proposed_count=0,closure_reasons=['oracle:false'],proposal_kinds=[],visible_test_exit_codes=[],manifest_sha256='a'*64)
class Tests(unittest.TestCase):
    def test_backward_wall_clock_cannot_extend_deadline(self):
        with patch.object(c.time,"time",return_value=50),patch.object(c.time,"monotonic",return_value=201):
            self.assertEqual(c.deadline_remaining(100,200),-1)
        with patch.object(c.time,"time",return_value=101),patch.object(c.time,"monotonic",return_value=150):
            self.assertEqual(c.deadline_remaining(100,200),-1)
    def manifest(self):return json.loads(c.MANIFEST.read_text())
    def test_only_original_remaining_eleven_and_prior_unchanged(self):
        m=self.manifest();ops=Ops();prior=(c.ROOT/m['prior_decision']).read_bytes()
        with tempfile.TemporaryDirectory() as tmp,patch.object(c,'verify_handle_evidence',return_value={}):
            state=Path(tmp)/'state';got=c.coordinate(m,state,ops)
            self.assertEqual(ops.calls,list(range(1,12)));self.assertEqual(got[0],json.loads(prior));self.assertEqual(len(got),12)
            self.assertFalse((state/'0').exists());self.assertEqual((c.ROOT/m['prior_decision']).read_bytes(),prior)
            with self.assertRaises(ValueError):c.coordinate(m,state,ops)
    def test_scope_changes_rejected(self):
        for key,value in [('max_calls_total',241),('incremental_cap_usd','6'),('absolute_deadline_epoch',1792352701)]:
            m=self.manifest();m[key]=value
            with self.assertRaises(ValueError):c.validate_scope(m)
        m=self.manifest();m['arms'][1]['model']='other'
        with self.assertRaises(ValueError):c.validate_scope(m)
    def test_continuation_halt_has_no_retry(self):
        m=self.manifest();ops=Ops()
        def fail(*args):raise ValueError('test guard failure')
        ops.export_and_verify=fail
        with tempfile.TemporaryDirectory() as tmp:
            state=Path(tmp)/'state'
            with self.assertRaises(ValueError):c.coordinate(m,state,ops)
            self.assertEqual(ops.calls,[1]);self.assertEqual(len(json.loads((state/'terminal.json').read_text())['results']),1)
if __name__=='__main__':unittest.main()
