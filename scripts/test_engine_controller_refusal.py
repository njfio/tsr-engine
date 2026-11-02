"""Offline diagnostics and validation boundary tests; no model or worker calls."""
import importlib.util
from pathlib import Path
import subprocess
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[1]
spec=importlib.util.spec_from_file_location('refusal_driver',ROOT/'scripts/engine-directions.py')
d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d)

class ControllerRefusals(unittest.TestCase):
    def test_exit_type_and_partial_output_are_not_success(self):
        for code,stderr,kind in [(-15,b'tsr engine: stale direction offer\n','signal'),
                                (2,b'arbitrary private diagnostic\n','unrecognized_exit'),
                                (0,b'', 'invalid_stdout')]:
            with self.subTest(kind=kind),patch.object(d.subprocess,'run',return_value=subprocess.CompletedProcess([],code,b'{',stderr)):
                with self.assertRaises(d.ControllerFailure) as caught:d.cli(Path('/unused'),'deliberate')
                self.assertNotIsInstance(caught.exception,d.ControllerRefusal)
                receipt=caught.exception.receipt
                self.assertEqual(receipt['kind'],kind);self.assertEqual(receipt['exit_code'],code)
                self.assertNotIn('private',str(receipt));self.assertNotIn('stderr',receipt)
        with patch.object(d.subprocess,'run',side_effect=subprocess.TimeoutExpired('fixture',30,output=b'{')):
            with self.assertRaises(d.ControllerFailure) as caught:d.cli(Path('/unused'),'deliberate')
            self.assertEqual(caught.exception.receipt['kind'],'timeout')
    def test_recognized_refusal_is_not_a_generic_cli_error(self):
        result=subprocess.CompletedProcess([],2,b'',b'tsr engine: stale direction offer\n')
        with patch.object(d.subprocess,'run',return_value=result):
            with self.assertRaises(d.ControllerRefusal) as caught:d.cli(Path('/unused'),'deliberate')
            self.assertEqual(caught.exception.receipt['code'],'stale_offer')
            with self.assertRaises(d.ControllerFailure) as caught:d.cli(Path('/unused'),'direction-request')
            self.assertNotIsInstance(caught.exception,d.ControllerRefusal)
    def test_support_validation_does_not_invent_support(self):
        state={'obligation_ids':['a','b'],'standing_desires':[{'aim':{'predicate':'archive_traceability'}},{'aim':{'predicate':'archive_contestability'}}],'selected_desires':[0],
               'state_root':'root','obligation_roots':[0,1]}
        for supports in ([],['a','a'],['b'],['unknown']):
            value={'state_root':'root','directions':[{'purpose':'fixture','supports':supports,
                   'forecasts':{'a':.5,'b':1},'action':{'operation':'inspect','notes':[]}}]}
            with self.subTest(supports=supports),self.assertRaises(ValueError):d.normalize_forecasts(state,value)
            self.assertEqual(value['directions'][0]['supports'],supports)

if __name__=='__main__':unittest.main()
