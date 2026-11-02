"""Recorded evidence replay is offline; never instantiate a transport."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
import test_engine_synthetic_cycle as h

class RecordedDemo(unittest.TestCase):
    def setUp(self):
        self.d=h.module('demo','scripts/engine-demo.py')
        self.source=h.ROOT/'target/workshop-ready/run'
        if not self.source.exists():self.skipTest('recorded local evidence unavailable; replay must not regenerate it')
    def test_read_only_and_fresh_process_equal(self):
        index=json.loads(self.d.INDEX.read_text())
        before={n:hashlib.sha256((self.source/n).read_bytes()).hexdigest() for n in index['files']}
        with mock.patch('subprocess.call',side_effect=AssertionError('replay must not invoke driver')):
            first=self.d.replay(self.source)
        command=[sys.executable,str(h.ROOT/'scripts/engine-demo.py'),'replay','--evidence',str(self.source)]
        p=subprocess.run(command,capture_output=True,text=True,timeout=10)
        self.assertEqual(p.returncode,0,p.stderr);self.assertEqual(json.loads(p.stdout),first)
        self.assertEqual(first['dispatches_this_invocation'],0)
        self.assertEqual(before,{n:hashlib.sha256((self.source/n).read_bytes()).hexdigest() for n in index['files']})
    def test_changed_missing_and_symlink_fail_closed(self):
        index=json.loads(self.d.INDEX.read_text())
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            for n in index['files']:
                dest=root/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(self.source/n,dest)
            path=root/'final.json';raw=path.read_bytes();path.write_bytes(raw+b' ')
            with self.assertRaises(ValueError):self.d.replay(root)
            path.unlink()
            with self.assertRaises(OSError):self.d.replay(root)
            path.symlink_to(self.source/'final.json')
            with self.assertRaises(ValueError):self.d.replay(root)
    def test_unknown_garden_replay_preserves_uncertainty_and_never_retries(self):
        source=h.ROOT/'target/garden-ready/run'
        if not source.exists():self.skipTest('local unknown evidence unavailable; never regenerate')
        index=json.loads((h.ROOT/'docs/engine-garden-replay-index.json').read_text())
        before={n:hashlib.sha256((source/n).read_bytes()).hexdigest() for n in index['files']}
        with mock.patch('subprocess.call',side_effect=AssertionError('no retry')):
            first=self.d.replay(source,'garden')
        self.assertIsNone(first['recorded_total_usd'])
        self.assertEqual(first['cost_status'],'unknown');self.assertEqual(first['usage_status'],'unknown')
        self.assertEqual(first['retained_reservations']['calls'],1)
        self.assertEqual(first['dispatches_this_invocation'],0)
        command=[sys.executable,str(h.ROOT/'scripts/engine-demo.py'),'replay','--scenario','garden','--evidence',str(source)]
        result=subprocess.run(command,capture_output=True,text=True,timeout=10)
        self.assertEqual(result.returncode,0,result.stderr);self.assertEqual(json.loads(result.stdout),first)
        self.assertEqual(before,{n:hashlib.sha256((source/n).read_bytes()).hexdigest() for n in index['files']})
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            for n in index['files']:
                dest=root/n;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(source/n,dest)
            path=root/'synthetic-external/outcome-1.json'
            value=json.loads(path.read_text());value['receipt']['usd']='0';path.write_text(json.dumps(value))
            with self.assertRaises(ValueError):self.d.replay(root,'garden')
            path.unlink()
            with self.assertRaises(OSError):self.d.replay(root,'garden')

    def test_completed_trial_cost_separate_from_historical_unknown(self):
        source=h.ROOT/'target/garden-trial2/run'
        if not source.exists():self.skipTest('local completed trial unavailable')
        with mock.patch('subprocess.call',side_effect=AssertionError('no dispatch')):
            result=self.d.replay(source,'garden-trial2')
        self.assertEqual(result['recorded_usd'],'0.006318')
        self.assertIsNone(result['combined_total_usd'])
        self.assertEqual(result['historical_unknown']['cost_status'],'unknown')
        self.assertEqual(result['historical_unknown']['retained_reservations']['calls'],1)
        p=subprocess.run([sys.executable,str(h.ROOT/'scripts/engine-demo.py'),'replay','--scenario','garden-trial2'],capture_output=True,text=True,timeout=10)
        self.assertEqual(p.returncode,0,p.stderr);self.assertEqual(json.loads(p.stdout),result)

    def test_run_requires_explicit_opt_in_before_dispatch(self):
        p=subprocess.run([sys.executable,str(h.ROOT/'scripts/engine-demo.py'),'run','--directory','missing','--admission','missing'],capture_output=True,text=True,timeout=10)
        self.assertEqual(p.returncode,2);self.assertIn('--allow-paid-calls',p.stderr)

if __name__=='__main__':unittest.main()
