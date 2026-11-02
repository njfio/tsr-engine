"""Campaign accounting fixtures only: no provider, worker, credentials or target execution."""
import importlib.util
import json
from pathlib import Path
import tempfile
import os
import sys
from types import SimpleNamespace
from unittest.mock import patch
import unittest

spec=importlib.util.spec_from_file_location('campaign',Path(__file__).with_name('engine-b91-campaign.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Campaign(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.path=Path(self.tmp.name)/'ledger'
        self.manifest=m.manifest('a'*64,self.path,fixture=True)
        self.ledger=m.Ledger(self.path,self.manifest,m.digest(self.manifest),create=True)
        self.trial='b'*64;self.admission='c'*64;self.ledger.enroll(self.trial,self.admission)
        self.body={'model':m.MODEL,'stream':False,'max_completion_tokens':8192,'messages':[{'role':'user','content':'synthetic fixture'}]}
    def call(self,i,cost='0.0005',trial=None):
        return self.ledger.run_call(trial or self.trial,str(i),self.body,240,lambda *_:{'value':None,'receipt':{'status':'settled','usd':cost}})
    def test_status_is_lock_free_and_does_not_weaken_writer_lock(self):
        before=self.path.read_bytes()
        with self.ledger.locked():
            self.assertEqual(self.ledger.status()['calls'],0)
            self.assertEqual(self.path.read_bytes(),before)
            with self.assertRaises(BlockingIOError):
                with self.ledger.locked():pass
        self.call('after-status')
        self.assertEqual(self.ledger.status()['calls'],1)
    def test_status_rejects_partial_and_corrupt_history(self):
        before=self.path.read_bytes()
        for changed in [before[:-1],before.replace(b'"hash":',b'"foreign":',1)]:
            self.path.write_bytes(changed)
            with self.assertRaises((ValueError,KeyError)):self.ledger.status()
        self.path.write_bytes(before)
        self.assertEqual(self.ledger.status()['calls'],0)
    def test_concurrent_observers_cannot_abort_dispatch(self):
        import concurrent.futures
        def observe():
            for _ in range(30):
                try:self.ledger.status()
                except ValueError as error:
                    if 'snapshot changed during read' not in str(error):raise
        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
            readers=[pool.submit(observe) for _ in range(2)]
            for i in range(5):self.call('concurrent-'+str(i))
            for reader in readers:reader.result()
        self.assertEqual(self.ledger.status()['calls'],5)
    def test_shared_call_and_execution_caps_across_trials(self):
        other='d'*64;self.ledger.enroll(other,'e'*64)
        for i in range(100):self.call(i,trial=self.trial if i%2 else other)
        with self.assertRaises(ValueError):self.call('101')
        for i in range(40):self.ledger.run_execution(other,str(i),self.admission,lambda:{'fixture_only':True})
        with self.assertRaises(ValueError):self.ledger.run_execution(other,'41',self.admission,lambda:self.fail('no effect'))
        status=self.ledger.status();self.assertEqual(status['calls'],100);self.assertEqual(status['execution_groups'],40)
        self.assertEqual(status['known_cost_microusd'],50000)
    def test_unknown_charge_retains_reservation_and_blocks_restart(self):
        self.call('unknown',None)
        restarted=m.Ledger(self.path,self.manifest,m.digest(self.manifest))
        self.assertEqual(restarted.status()['committed_microusd'],50000)
        self.assertEqual(restarted.status()['known_cost_microusd'],0) # subtotal only, unknown explicit
        self.assertTrue(restarted.status()['unknown_calls'])
        with self.assertRaises(ValueError):self.call('new')
        with self.assertRaises(ValueError):self.call('unknown')
    def test_interrupted_before_response_and_execution_are_never_retried(self):
        def fail(*_):raise TimeoutError('fixture')
        with self.assertRaises(TimeoutError):self.ledger.run_call(self.trial,'one',self.body,240,fail)
        self.assertEqual(self.ledger.status()['calls'],1)
        with self.assertRaises(ValueError):self.ledger.run_execution(self.trial,'seed',self.admission,lambda:None)
    def test_execution_interruption_consumes_group(self):
        def fail():raise TimeoutError('fixture')
        with self.assertRaises(TimeoutError):self.ledger.run_execution(self.trial,'seed',self.admission,fail)
        self.assertEqual(self.ledger.status()['execution_groups'],1)
        self.assertTrue(self.ledger.status()['unknown_executions'])
        with self.assertRaises(ValueError):self.call('new')
    def test_overreservation_and_actual_cap_overrun_honestly_stop(self):
        self.call('over','5.01')
        s=self.ledger.status();self.assertEqual(s['known_cost_microusd'],5010000);self.assertTrue(s['overrun'])
        with self.assertRaises(ValueError):self.call('after')
    def test_invalid_and_truncated_results_still_consume_calls(self):
        for i in range(3):
            result=self.ledger.run_call(self.trial,str(i),self.body,240,lambda *_:{'value':None,'receipt':{'status':'settled','usd':'0.000001','finish_reason':'length'}})
            self.assertIsNone(result['value'])
        self.assertEqual(self.ledger.status()['calls'],3)
        self.assertEqual(self.ledger.status()['known_cost_microusd'],3)
    def test_no_fork_reset_wrong_admission_or_scope(self):
        with self.assertRaises(ValueError):m.Ledger(self.path.with_name('fork'),self.manifest,m.digest(self.manifest),create=True)
        with self.assertRaises(ValueError):self.ledger.enroll(self.trial,'f'*64)
        wrong=dict(self.manifest,max_calls=101)
        with self.assertRaises(ValueError):m.Ledger(self.path,wrong,m.digest(wrong))
        self.path.unlink()
        with self.assertRaises(ValueError):m.Ledger(self.path,self.manifest,m.digest(self.manifest))
    def test_replay_and_allocation_limits_precede_callback(self):
        self.call('one')
        with self.assertRaises(ValueError):self.call('one')
        for field,value in [('model','different'),('max_completion_tokens',8193),('stream',True)]:
            body=dict(self.body);body[field]=value
            with self.assertRaises(ValueError):self.ledger.run_call(self.trial,field,body,240,lambda *_:self.fail('no dispatch'))
        with self.assertRaises(ValueError):self.ledger.run_call(self.trial,'time',self.body,241,lambda *_:self.fail('no dispatch'))
    def test_utf8_profile_matches_transport_and_retains_smaller_bound(self):
        body=dict(self.body,messages=[{'role':'user','content':'界'*6000}])
        self.assertGreater(len(json.dumps(body).encode()),32768)
        self.ledger.run_call(self.trial,'utf8',body,240,lambda *_:{'receipt':{'status':'settled','usd':'0.001'}},payload_limit=32768)
        with self.assertRaises(ValueError):self.ledger.run_call(self.trial,'small',body,240,lambda *_:self.fail('no dispatch'),payload_limit=16384)
    def test_price_failure_precedes_driver_effects(self):
        directory=Path(self.tmp.name)/'live';directory.mkdir()
        value=m.manifest('d'*64,directory/'ledger.jsonl',fixture=False)
        (directory/'manifest.json').write_bytes(m.canonical(value));m.Ledger(directory/'ledger.jsonl',value,m.digest(value),create=True)
        admission={'trial':str(directory/'trial'),'model':m.MODEL};file=directory/'trial.json';file.write_bytes(m.canonical(admission))
        fake=SimpleNamespace(e=SimpleNamespace(strict=m.strict,bounded_file=lambda p,n:p.read_bytes(),sha=m.digest),validate_admission=lambda *_:None,Driver=lambda *_:self.fail('driver must not initialize'))
        argv=['campaign','run','--campaign',str(directory),'--campaign-sha256',m.digest(value),'--admission',str(file),'--admission-sha256',m.digest(admission),'--reviews',str(directory/'reviews'),'--allow-paid-inference','--allow-target-execution']
        with patch.object(sys,'argv',argv),patch.object(m,'load_module',return_value=fake),patch.object(m,'official_preflight',side_effect=ValueError('fixture HTTP500')):
            with self.assertRaisesRegex(ValueError,'HTTP500'):m.main()
        status=m.Ledger(directory/'ledger.jsonl',value,m.digest(value)).status()
        self.assertEqual(status['calls'],0);self.assertEqual(status['execution_groups'],0)
    def test_full_native_fixture_guard_consumes_one_shared_campaign(self):
        if not os.environ.get('TSR_B91_TEST_WORKER_BIN'):self.skipTest('explicit native fake worker binary required')
        import test_engine_b91_driver as native
        case=native.NativeSequence(methodName='test_full_native_sequence');case.setUp();self.addCleanup(case.doCleanups)
        case.a=native.m.admission(case.root,fixture=True,model=m.MODEL,final_payload_32k=True)
        provider=case.provider;worker=case.worker
        def guarded_provider(body,timeout):
            driver=case.driver();request=next(x['id'] for x in reversed(driver.journal().events()) if x['kind']=='dispatch_intent')
            return self.ledger.run_call(self.trial,request,body,timeout,provider,payload_limit=native.m.stage_payload_limit(case.a,json.loads(body['messages'][1]['content'])['stage']))
        case.provider=guarded_provider
        case.worker=lambda controller,stage:self.ledger.run_execution(self.trial,stage,self.admission,lambda:worker(controller,stage))
        case.test_full_native_sequence()
        status=self.ledger.status();self.assertEqual(status['calls'],6);self.assertEqual(status['execution_groups'],4)
        self.assertEqual(status['known_cost_microusd'],6000);self.assertFalse(status['stopped'])
    def test_two_call_shadow_pair_is_not_imported_or_recharged(self):
        shadow=Path(self.tmp.name)/'prior-shadow.json';shadow.write_text(json.dumps({'calls':2,'usd':'0.000674'}))
        self.assertEqual(self.ledger.status()['calls'],0)
        self.assertEqual(self.ledger.status()['known_cost_microusd'],0)
        self.call('campaign-one');self.assertEqual(json.loads(shadow.read_text())['calls'],2)

class Reconciliation(unittest.TestCase):
    def test_reviewed_upgrade_attaches_without_reset_or_repin(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/'ledger';old=m.manifest('a'*64,path,fixture=True);old['implementation_sha256']='f'*64
            first=m.Ledger.row([],{'kind':'manifest','value':old});m.fresh(path,first);before=path.read_bytes()
            upgrade={'version':'b91-campaign-upgrade@v1','old_manifest_sha256':m.digest(old),'old_implementation_sha256':'f'*64,
                     'new_implementation_sha256':m.hashlib.sha256(Path(m.__file__).read_bytes()).hexdigest(),'ledger_prefix_head':first['hash'],
                     'reviewer':'independent fixture reviewer','reason':'Fixture-only explicit reviewed repair'}
            with self.assertRaises(ValueError):m.Ledger(path,old,m.digest(old))
            for key in ('old_manifest_sha256','new_implementation_sha256','ledger_prefix_head'):
                wrong=dict(upgrade);wrong[key]='0'*64
                with self.assertRaises(ValueError):m.Ledger(path,old,m.digest(old),upgrade=wrong,upgrade_hash=m.digest(wrong))
            ledger=m.Ledger(path,old,m.digest(old),upgrade=upgrade,upgrade_hash=m.digest(upgrade));ledger.attach_upgrade()
            self.assertTrue(path.read_bytes().startswith(before));self.assertEqual(ledger.status()['calls'],0)
            current=path.read_bytes();ledger.attach_upgrade();self.assertEqual(path.read_bytes(),current)
    def test_append_reconciliation_preserves_unknown_bill_and_callcount(self):
        with tempfile.TemporaryDirectory() as tmp:
            path=Path(tmp)/'ledger';manifest=m.manifest('a'*64,path,fixture=True);ledger=m.Ledger(path,manifest,m.digest(manifest),create=True)
            trial='b'*64;ledger.enroll(trial,'c'*64);key=ledger.reserve('call',trial,'request','d'*64);ledger.settle('bill',key,None)
            before=path.read_bytes();rows=[m.strict(line) for line in before.splitlines()]
            proof={'kind':'bill_reconciled','key':key,'original_bill_row':rows[-1]['hash'],'microusd':0,'basis':'exact_provider_failed_zero',
                   'provider_request_id':'00000000-0000-0000-0000-000000000001',**{k:'e'*64 for k in ('lookup_sha256','native_journal_sha256','provider_journal_sha256','native_result_sha256','review_sha256','billing_semantics_sha256')}}
            for field,value in [('original_bill_row','f'*64),('microusd',1),('key','f'*64)]:
                wrong=dict(proof);wrong[field]=value
                with ledger.locked() as (f,current):
                    with self.assertRaises(ValueError):ledger.append_locked(f,current,wrong)
            with ledger.locked() as (f,current):ledger.append_locked(f,current,proof)
            self.assertTrue(path.read_bytes().startswith(before));status=ledger.status()
            self.assertEqual(status['calls'],1);self.assertEqual(status['known_cost_microusd'],0);self.assertEqual(status['committed_microusd'],0);self.assertFalse(status['stopped'])
            with ledger.locked() as (f,current):
                with self.assertRaises(ValueError):ledger.append_locked(f,current,proof)
            with self.assertRaises(ValueError):ledger.reserve('call',trial,'request','d'*64)

if __name__=='__main__':unittest.main()
