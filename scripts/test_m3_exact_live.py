"""Offline budget/lifecycle/fake-provider checks. No real credentials or network."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

HERE = Path(__file__).resolve().parent

def load(name):
    spec = importlib.util.spec_from_file_location(name, HERE / (name + '.py'))
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module

c = load('m3-exact-live-controller')
b = load('m3-exact-budget')
live = load('m3-exact-live-broker')
p = load('test_m3_edit_broker')
TASK = '0c5a91a6aaa862c279aed1e1602a74eda7c42c4fc0f72247de513fa948f50b70'

def charge(i, cost='.001'):
    return (json.dumps(dict(kind='reserve', id=str(i), usd='2')) + '\n' +
            json.dumps(dict(kind='settled', id=str(i), usd=cost)) + '\n').encode()

def manifest():
    return dict(arms=[dict(task=TASK, mode=m) for m in ('record', 'rules')],
        max_calls_per_arm=20, max_calls_total=40, max_wall_seconds_per_arm=1800,
        incremental_cap_usd='5', aggregate_cap_usd='100', automatic_retries=0,
        global_wall_seconds=4200, export_margin_seconds=300)

class FakeOps:
    def __init__(self, fail=None): self.raw=b''; self.runs=[]; self.fail=fail
    def ledger(self): return self.raw
    def guard(self, initial, dispatch=True):
        if self.fail=='guard' and self.runs: raise ValueError('guard')
    def run(self, arm, state):
        self.runs.append(arm); self.raw += charge(len(self.runs))
        if self.fail=='process': raise ValueError('process')
        return {}
    def export_and_verify(self, arm, process, destination):
        if self.fail=='export': raise ValueError('export')
        d=dict(reservation_id=str(len(self.runs)), settled=True, code='ok', status='completed')
        result=dict(task=TASK, system=arm['mode'], outcome='accepted', oracle={}, state_root_at_start='empty')
        count=1
        if self.fail=='protocol': d.update(code='tool_shape',status='failed')
        if self.fail=='unavailable': result.update(outcome='unavailable',oracle=None)
        if self.fail=='tamper': result.update(outcome='rejected',oracle=None)
        if self.fail=='identity': result['task']='other'
        if self.fail=='count':count=0
        if self.fail=='root' and len(self.runs)==2: result['state_root_at_start']='wrong'
        return dict(result=result, diagnostics=[d], proposed_count=count, manifest_sha256='fixture')

class FreezeChecks(unittest.TestCase):
    def test_five_dollar_cap_and_forty_call_limit(self):
        baseline=b.history.freeze(charge('history','.229582'))
        raw=charge('history','.229582')
        for i in range(40):raw+=charge(i)
        self.assertEqual(b.check(raw,baseline,dispatch=False),b.D('.269582'))
        with self.assertRaises(ValueError):b.check(raw,baseline)
        raw=charge('history','.229582')+charge(1,'2')+charge(2,'1.000001')
        with self.assertRaises(ValueError):b.check(raw,baseline)
        self.assertEqual(b.check(raw,baseline,dispatch=False),b.D('3.229583'))
        raw+=charge(3,'2')
        with self.assertRaises(ValueError):b.check(raw,baseline,dispatch=False)
        with self.assertRaises(ValueError):b.check(b'{"kind":"reserve","id":"u","usd":"2"}\n',b.history.freeze(b''))

    def test_receipt_helpers_are_time_bounded(self):
        m=manifest();m['ledger']='target/inert-unused-ledger'
        ops=c.LocalOperations(m)
        with patch.object(c.subprocess,'run',return_value=None) as run:
            ops.receipts.subprocess.run(['inert'])
            self.assertEqual(run.call_args.kwargs['timeout'],60)
        with patch.object(c.subprocess,'check_output',return_value=b'') as read:
            ops.receipts.subprocess.check_output(['inert'])
            self.assertEqual(read.call_args.kwargs['timeout'],15)

    def test_serial_pair_is_at_most_once(self):
        with tempfile.TemporaryDirectory() as td:
            ops=FakeOps(); state=Path(td)/'state'
            self.assertEqual(len(c.coordinate(manifest(),state,ops)),2)
            self.assertEqual([a['mode'] for a in ops.runs],['record','rules'])
            with self.assertRaises(ValueError):c.coordinate(manifest(),state,ops)

    def test_no_failure_advances(self):
        for failure in ('guard','process','export','protocol','unavailable','tamper','identity','count'):
            with self.subTest(failure=failure),tempfile.TemporaryDirectory() as td:
                ops=FakeOps(failure);state=Path(td)/'state'
                with self.assertRaises(ValueError):c.coordinate(manifest(),state,ops)
                self.assertEqual(len(ops.runs),1)
                self.assertEqual(json.loads((state/'terminal.json').read_text())['status'],'halted-review-required')
                with self.assertRaises(ValueError):c.coordinate(manifest(),state,ops)

    def test_roots_and_deadline_include_guard_time(self):
        with tempfile.TemporaryDirectory() as td:
            with self.assertRaises(ValueError):c.coordinate(manifest(),Path(td)/'state',FakeOps('root'))
        with tempfile.TemporaryDirectory() as td,patch.object(c.time,'monotonic',side_effect=[0,0,2200]):
            ops=FakeOps()
            with self.assertRaises(ValueError):c.coordinate(manifest(),Path(td)/'state',ops)
            self.assertEqual(ops.runs,[])

    def test_reject_wrong_budgets_and_second_task(self):
        m=manifest();m['max_calls_per_arm']=10
        with self.assertRaises(ValueError):c.validate_scope(m)
        request=p.request();request['remaining_action_calls']=21
        with self.assertRaises(live.edit.BrokerFailure):live.encode(request)
        request=p.request();request['task']='47c787ed4f89b141de573feeeaead5a27c6e2ec399a0bccc1abe259b92911c23'
        with self.assertRaises(live.edit.BrokerFailure):live.encode(request)

    def test_actual_transport_uses_new_budget_before_any_credentials(self):
        with tempfile.TemporaryDirectory() as td:
            ledger=Path(td)/'ledger.jsonl';ledger.write_bytes(charge(1,'2')+charge(2,'1.01'))
            baseline=Path(td)/'baseline.json';baseline.write_text(json.dumps(b.history.freeze(b'')))
            with patch.object(live.budget,'BASELINE',baseline),patch.object(live.edit.transport,'credential',side_effect=AssertionError('must not read credentials')):
                with self.assertRaises(ValueError):live.edit.transport.run(p.request(),str(ledger))

    def test_catalog_pricing_guard_and_settlement_failure_keep_no_retry(self):
        for case in ('price','unsettled','multiple'):
            with self.subTest(case=case),tempfile.TemporaryDirectory() as td:
                ledger=Path(td)/'ledger.jsonl';ledger.write_bytes(b'')
                baseline=Path(td)/'baseline.json';baseline.write_text(json.dumps(b.history.freeze(b'')))
                result=p.response();result['billing']=dict(status='settled',currency='USD',billed_cost_usd='.001')
                if case=='multiple':result['choices'][0]['message']['tool_calls']*=2
                if case=='unsettled':result['billing']['status']='pending'
                def http(route,key,*args,**kwargs):
                    if route=='models':return {'data':[dict(id=live.edit.transport.MODEL,pricing=dict(list_input_per_million='.31' if case=='price' else '.15',list_output_per_million='.60'))]}
                    return result
                with patch.object(live.budget,'BASELINE',baseline),patch.object(live.edit.transport,'credential',return_value='fixture'),patch.object(live.edit.transport,'request_json',side_effect=http) as net:
                    request=p.request();request['deadline_ms']=30000
                    with self.assertRaises(live.edit.BrokerFailure):live.edit.transport.complete_with_diagnostics(request,str(ledger))
                    self.assertEqual(net.call_count,1 if case=='price' else 2)
                lines=ledger.read_bytes().splitlines()
                self.assertEqual(len(lines),{'price':0,'unsettled':1,'multiple':2}[case])

if __name__=='__main__':unittest.main()
