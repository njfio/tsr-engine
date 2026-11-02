"""Authored offline fixtures; never provider or target execution evidence."""
import importlib.util
import json
import os
from pathlib import Path
import unittest
import test_engine_b91_driver as base
s=importlib.util.spec_from_file_location('cont',Path(__file__).with_name('engine-b91-continuation.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)

class NativeContinuation(base.NativeSequence):
    # Inherit setup/provider/reviewer only; avoid rerunning base tests under subclass.
    def make_prefix(self):
        self.a=base.m.admission(self.root,fixture=True,model='gpt-6-luna')
        class Pause(Exception):pass
        def hook(point,name):
            if point=='after_result' and name=='close_check_after':raise Pause()
        driver=self.driver(hook=hook)
        with self.assertRaises(Pause):driver.run()
        m.e.private_save(self.root/'STOP',{'terminal':'fixture prefix'})
        path=Path(self.tmp.name)/'parent.json';m.e.private_save(path,self.a)
        return path
    def test_continuation_native_restart_zero_execution(self):
        parent=self.make_prefix();before=m.tree(self.root);out=Path(self.tmp.name)/'continued';a=m.admission(parent,out)
        calls_before=len(self.calls);workers_before=len(self.workers)
        # Fresh objects at every durable transition simulate process reconstruction.
        class Pause(Exception):pass
        seen=set()
        def hook(point,name):
            if point=='after_result' and not name.startswith(('reserve_','transport_','settle_')) and name not in seen:
                seen.add(name);raise Pause()
        for _ in range(40):
            runner=m.Continuation(out,a,self.provider,self.reviewer,fixture=True,hook=hook)
            try:result=runner.run();break
            except Pause:continue
        else:self.fail('continuation did not finish')
        self.assertEqual(result['calls'],2);self.assertEqual(result['execution_groups'],0)
        self.assertEqual(len(self.calls)-calls_before,2);self.assertEqual(len(self.workers),workers_before)
        self.assertEqual(m.tree(self.root),before)
        self.assertEqual(runner.controller().budget()['model_calls'],2)
        self.assertEqual(runner.controller().budget()['tokens_charged'],600)
        self.assertEqual(runner.controller().budget()['patch_bytes_charged'],0)
        self.assertEqual(runner.controller().budget()['tool_ms_charged'],0)
        again=m.Continuation(out,a,lambda *_:self.fail('redispatch'),self.reviewer,fixture=True).run()
        self.assertEqual(again,result)
        with self.assertRaises(m.d.Blocked):runner.execute(None,None)
        with self.assertRaises(m.d.Blocked):runner.commit_response(None,None,None)
    def test_fresh_process_terminal_replay(self):
        import subprocess,sys
        parent=self.make_prefix();out=Path(self.tmp.name)/'process';a=m.admission(parent,out)
        runner=m.Continuation(out,a,self.provider,self.reviewer,fixture=True);result=runner.run()
        ap=Path(self.tmp.name)/'continuation.json';m.e.private_save(ap,a)
        code="""import importlib.util,json,sys
from pathlib import Path
s=importlib.util.spec_from_file_location('c',sys.argv[1]);m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
a=json.loads(Path(sys.argv[2]).read_text())
def forbidden(*args):raise AssertionError('redispatch or review on terminal replay')
r=m.Continuation(a['trial'],a,forbidden,forbidden,fixture=True)
print(json.dumps(r.run()))
"""
        before=m.tree(out)
        proc=subprocess.run([sys.executable,'-c',code,str(Path(m.__file__).resolve()),str(ap)],capture_output=True,text=True,timeout=60)
        self.assertEqual(proc.returncode,0,proc.stderr);self.assertEqual(json.loads(proc.stdout),result)
        self.assertEqual(m.tree(out),before)

    def test_uncertain_intent_never_replayed(self):
        parent=self.make_prefix()
        for boundary in ('after_intent','after_effect'):
            out=Path(self.tmp.name)/boundary;a=m.admission(parent,out)
            def hook(point,name):
                if point==boundary and name.startswith('transport_'):raise RuntimeError('interrupted')
            runner=m.Continuation(out,a,self.provider,self.reviewer,fixture=True,hook=hook)
            with self.assertRaises(RuntimeError):runner.run()
            count=len(self.calls)
            with self.assertRaises(m.d.Blocked):m.Continuation(out,a,lambda *_:self.fail('redispatch'),self.reviewer,fixture=True).run()
            self.assertEqual(len(self.calls),count);self.assertEqual(runner.controller().budget()['unsettled_requests'],1)

    def test_file_review_pause_resume(self):
        parent=self.make_prefix();out=Path(self.tmp.name)/'reviewed';a=m.admission(parent,out)
        reviews=m.d.FileReviewer(Path(self.tmp.name)/'reviews');count=len(self.calls)
        for _ in range(3):
            runner=m.Continuation(out,a,self.provider,reviews,fixture=True)
            try:result=runner.run();break
            except m.d.Blocked as error:
                self.assertTrue(str(error).startswith('independent review required:'))
                self.assertFalse((out/'STOP').exists())
                for request in reviews.directory.glob('*.request.json'):
                    response=request.with_name(request.name.replace('.request.json','.review.json'))
                    if not response.exists():
                        context=json.loads(request.read_text())
                        m.e.private_save(response,self.reviewer(context['kind'],context))
        else:self.fail('reviews did not unblock continuation')
        self.assertEqual(len(self.calls)-count,2)
        self.assertEqual(result['execution_groups'],0)

    def test_parent_tamper_and_limits_fail_before_transport(self):
        parent=self.make_prefix();out=Path(self.tmp.name)/'tamper';a=m.admission(parent,out)
        bad={**a,'execution_groups':1}
        with self.assertRaises(ValueError):m.Continuation(out,bad,lambda *_:self.fail('dispatch'),self.reviewer,fixture=True)
        (self.root/'project/coding-work/src/lib.rs').write_text('changed')
        with self.assertRaises(ValueError):m.Continuation(out,a,lambda *_:self.fail('dispatch'),self.reviewer,fixture=True)

# Do not inherit base test cases: fixture helper inheritance only.
for name in list(vars(base.NativeSequence)):
    if name.startswith('test_') and name not in NativeContinuation.__dict__:setattr(NativeContinuation,name,None)

class AdmissionControls(unittest.TestCase):
    def test_expired_terminal_journal_recovery_requires_bound_complete_history(self):
        import tempfile
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)/'run';root.mkdir()
            stamp=m.d.iso_now();end=(m.datetime.datetime.now(m.datetime.timezone.utc)+m.datetime.timedelta(minutes=1)).isoformat()
            p={'parent':str(Path(tmp)/'parent'),'parent_admission':'fixture-parent'}
            a={'version':'b91-continuation@v1','mode':'fixture','trial':str(root),'created_at':stamp,'expires_at':end,
               'prefix':p,'model':'gpt-6-luna','calls':2,'execution_groups':0,'patch_bytes':0,'payload_bytes':32768,
               'input_tokens':65536,'completion_tokens':16384,'call_seconds':240,'implementation':m.implementation(),
               'binary_sha256':m.d.file_sha(m.d.ROOT/'target/debug/tsr')}
            rows=[]
            for event in ({'kind':'admission','value':a,'fixture_only':True},
                          {'kind':'intent','name':'terminal','at':stamp}):rows.append(m.d.Driver.row(rows,event))
            result={'status':'fixture terminal'}
            rows.append(m.d.Driver.row(rows,{'kind':'result','name':'terminal','intent':m.e.sha(rows[1]['event']),'value':result}))
            (root/'driver.jsonl').write_bytes(b''.join(m.e.canonical(x)+b'\n' for x in rows))
            with patch.object(m,'prefix',return_value=p),patch.object(m.time,'time',return_value=m.d.stamp(end)+1):
                runner=m.Continuation(root,a,lambda *_:self.fail('dispatch'),None,fixture=True)
                with patch.object(runner,'verify_clone'):
                    self.assertEqual(runner.run(),result)
                self.assertTrue((root/'terminal.json').exists())
                # A file by itself cannot exempt unresolved work from expiry.
                (root/'driver.jsonl').write_bytes(b''.join(m.e.canonical(x)+b'\n' for x in rows[:2]))
                with self.assertRaises(ValueError):m.Continuation(root,a,lambda *_:self.fail('dispatch'),None,fixture=True)

    def test_cli_review_pause_does_not_expire_allowance(self):
        import tempfile
        from unittest.mock import patch,Mock
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)/'run';root.mkdir();a={'trial':str(root)}
            admission=Path(tmp)/'admission.json';m.e.private_save(admission,a)
            instance=Mock();instance.run.side_effect=m.d.Blocked('independent review required: fixture')
            argv=['continuation','run','--admission',str(admission),'--admission-sha256',m.e.sha(a),
                  '--reviews',str(Path(tmp)/'reviews'),'--allow-paid-inference']
            with patch('sys.argv',argv),patch.object(m,'Continuation',return_value=instance),patch.object(m.e,'module',return_value=Mock()):m.main()
            self.assertFalse((root/'STOP').exists())
            instance.run.side_effect=m.d.Blocked('uncertain effect; no automatic retry')
            with patch('sys.argv',argv),patch.object(m,'Continuation',return_value=instance),patch.object(m.e,'module',return_value=Mock()):m.main()
            self.assertTrue((root/'STOP').exists())

if __name__=='__main__':unittest.main()
