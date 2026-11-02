"""Offline report regressions: false acceptance, bindings and missing submission."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest
spec=importlib.util.spec_from_file_location('report',Path(__file__).with_name('m3-exact-live-report.py'))
r=importlib.util.module_from_spec(spec);spec.loader.exec_module(r)

class Reports(unittest.TestCase):
    def fixture(self):
        result=dict(outcome='accepted',oracle=dict(accepted=True),denials=[],state_root_at_start='empty')
        process=dict(status='completed',exit=0,errors=[])
        trace=[dict(reservation_id='r',settled=True,status='completed',code='ok')]
        stored=[('e',dict(kind=dict(kind='task_closed',reason='oracle:true')))]
        receipt=dict(status='checked',verified_success=True)
        item=dict(result=result,process=process,diagnostics=trace,stored=stored,oracle_receipt=receipt,actions=['stop'])
        exported=dict(result=result,process=process,diagnostics=trace,stored=stored,receipt=receipt)
        decision=dict(task='t',mode='record',outcome='accepted',start_root='empty',reservations=['r'])
        ledger=(json.dumps(dict(kind='reserve',id='r',usd='2'))+'\n'+json.dumps(dict(kind='settled',id='r',usd='.001'))+'\n').encode()
        return dict(arm=dict(task='t',mode='record'),item=copy.deepcopy(item),decision=decision,exported=copy.deepcopy(exported),before=b'',after=ledger)
    def test_valid_and_independent_acceptance_checks(self):
        self.assertEqual(r.verify_completed(**self.fixture()),['r'])
        for field,value in [('oracle_receipt',dict(status='checked',verified_success=False)),('actions',['inspect']),('actions',['inspect','stop']),('process',dict(status='completed',exit=1,errors=[]))]:
            x=self.fixture();x['item'][field]=value
            if field=='oracle_receipt':x['exported']['receipt']=value
            if field=='process':x['exported']['process']=value
            with self.subTest(field=field),self.assertRaises(ValueError):r.verify_completed(**x)
    def test_raw_export_and_decision_binding(self):
        for key in ('result','process','diagnostics','stored','receipt'):
            x=self.fixture();x['exported'][key]={}
            with self.subTest(key=key),self.assertRaises(ValueError):r.verify_completed(**x)
        for key in ('task','mode','outcome','start_root','reservations'):
            x=self.fixture();x['decision'][key]='wrong'
            with self.subTest(key=key),self.assertRaises(ValueError):r.verify_completed(**x)
    def test_denials_and_bad_closure_rejected(self):
        for change in ('denial','closure'):
            x=self.fixture()
            if change=='denial':
                x['item']['result']['denials']=['denied'];x['exported']['result']['denials']=['denied']
            else:
                x['item']['stored']=[];x['exported']['stored']=[]
            with self.assertRaises(ValueError):r.verify_completed(**x)

if __name__=='__main__':unittest.main()
