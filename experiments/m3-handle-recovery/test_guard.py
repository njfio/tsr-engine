from pathlib import Path
import hashlib,json,os,sys,tempfile,unittest
from unittest.mock import patch
import guard
class Tests(unittest.TestCase):
    def test_exact_reviewed_observer_only(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'m3-status.py';p.write_text('print("read-only")');h=hashlib.sha256(p.read_bytes()).hexdigest();python=str(Path(sys.executable).resolve())
            self.assertEqual(guard.role(python+' '+str(p),p,h),'read-only-observer')
            for command in (python+' '+str(p)+' --approved-live',python+' '+str(p)+' --extra','/tmp/python3 '+str(p),'/bin/sh -c "'+python+' '+str(p)+'"',python+' /tmp/m3-worker.py',python+' /tmp/m3-provider.py',python+' experiments/m3-other/controller.py','tsr task run --mode record','tsr corpus mutants'):
                self.assertNotIn(guard.role(command,p,h),(None,'read-only-observer'),command)
            p.write_text('changed')
            self.assertEqual(guard.role(python+' '+str(p),p,h),'changed-observer')
    def test_real_lock_excludes_second_controller(self):
        with tempfile.TemporaryDirectory() as tmp,patch.object(guard,'ROOT',Path(tmp)):
            (Path(tmp)/'target').mkdir();fd=guard.lock()
            try:
                with self.assertRaises(BlockingIOError):guard.lock()
            finally:os.close(fd)
    def test_snapshot_blocks_real_competitor_and_records_pid(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'m3-status.py';p.write_text('');h=hashlib.sha256(p.read_bytes()).hexdigest();log=Path(tmp)/'guard.jsonl'
            historical=f'{guard.HISTORICAL_PID} T {guard.HISTORICAL_COMMAND}\n'
            def output(argv,**kw):
                if argv[0]=='docker':return ''
                if '-p' in argv:return 'Wed Oct 14 21:19:49 2026\n'
                return historical+'912345 S python3 /tmp/m3-worker.py\n'
            with patch.object(guard.subprocess,'check_output',side_effect=output):
                with self.assertRaisesRegex(ValueError,'pid=912345'):guard.require_idle(p,h,log)
            self.assertEqual(json.loads(log.read_text())['processes'][0]['pid'],912345)
    def test_exact_observer_snapshot_allowed(self):
        with tempfile.TemporaryDirectory() as tmp:
            p=Path(tmp)/'m3-status.py';p.write_text('');h=hashlib.sha256(p.read_bytes()).hexdigest()
            def output(argv,**kw):
                if argv[0]=='docker':return ''
                if '-p' in argv:return 'Wed Oct 14 21:19:49 2026\n'
                return f'{guard.HISTORICAL_PID} T {guard.HISTORICAL_COMMAND}\n912345 S {Path(sys.executable).resolve()} {p}\n'
            with patch.object(guard.subprocess,'check_output',side_effect=output):guard.require_idle(p,h)
if __name__=='__main__':unittest.main()
