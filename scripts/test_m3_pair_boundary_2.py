"""Bounded offline scheduling regressions; no provider or process signals."""
import importlib.util
from pathlib import Path
from datetime import datetime, timezone, timedelta
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('handoff', Path(__file__).with_name('m3-pair-boundary-2.py'))
h = importlib.util.module_from_spec(spec)
spec.loader.exec_module(h)


class SchedulingGuards(unittest.TestCase):
    def test_zombie_leader_does_not_hide_live_descendant(self):
        self.assertFalse(h.live_group([(10,10,'Z')],10))
        self.assertTrue(h.live_group([(10,10,'Z'),(11,10,'S')],10))
        self.assertFalse(h.live_group([(12,12,'S')],10))

    def test_original_timeout_returned_before_deadline(self):
        start = datetime(2026,10,15,tzinfo=timezone.utc)
        worker = {'receipt':{'started':start.isoformat()}}
        self.assertFalse(h.timeout_near(worker,start+timedelta(seconds=2689)))
        self.assertTrue(h.timeout_near(worker,start+timedelta(seconds=2690)))

    def test_worker_pid_reuse_refuses(self):
        with patch.object(h.subprocess,'check_output',return_value='new start'):
            with self.assertRaisesRegex(RuntimeError,'identity changed'):
                h.verify_worker({'pid':10,'pgid':10,'start':'old start','command':'expected'},[(10,10,'Z')])
        with patch.object(h.subprocess,'check_output') as call:
            h.verify_worker({'pid':10,'pgid':10,'start':'old start','command':'expected'},[])
            call.assert_not_called()

    def test_resume_rejects_changed_or_running_coordinator(self):
        i={'pid':10,'command':'expected','start':'start'}
        for values in [('changed',),('expected','other'),('expected','start','S')]:
            with patch.object(h.subprocess,'check_output',side_effect=values):
                with self.assertRaises(RuntimeError): h.paused(i)
        with patch.object(h.subprocess,'check_output',side_effect=['expected','start','Ts']):
            h.paused(i)

    def test_nested_paid_worker_records_preserved(self):
        self.assertEqual(h.flatten([[{'pid':1}],[{'pid':2}]]),[{'pid':1},{'pid':2}])


class RevalidationGuards(unittest.TestCase):
    def test_gate_binds_task_engine_protocol_and_clean_completion(self):
        import copy
        spec = importlib.util.spec_from_file_location('paired2', Path(__file__).with_name('m3-paired-cohort-2.py'))
        c = importlib.util.module_from_spec(spec); spec.loader.exec_module(c)
        plan = {'engine':'engine', 'hashes':{'engine':'sha'}, 'jobs':[{}, {}, {}, {'task_id':'task'}]}
        valid = {'status':'accepted', 'engine_sha256':'sha',
                 'process':{'status':'completed','exit':0,'errors':[]},
                 'reports':[{'task_id':'task','accepted':True,'protocol':'baseline-full-green/named-red/full-red/exact-reversal/full-green-v1'}]}
        c.validate_revalidation(valid, plan)
        for section, key, value in [(None,'engine_sha256','old'), (None,'status','failed'),
                ('process','status','running'), ('process','exit',2), ('process','errors',['cleanup']), ('process','errors',None),
                ('report','task_id','other'), ('report','accepted',False), ('report','protocol','weak')]:
            invalid = copy.deepcopy(valid)
            target = invalid if section is None else invalid['reports'][0] if section == 'report' else invalid[section]
            target[key] = value
            with self.assertRaises(RuntimeError): c.validate_revalidation(invalid, plan)

if __name__ == '__main__': unittest.main()
