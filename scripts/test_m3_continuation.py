"""Offline lifecycle and fake-provider integration; never real credentials/HTTP."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import shutil
import sqlite3
import struct
import unittest
from unittest.mock import patch

HERE=Path(__file__).resolve().parent

def load(name):
    spec=importlib.util.spec_from_file_location(name,HERE/(name+'.py'))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
    return module

c=load('m3-continuation-controller')
b=load('m3-continuation-broker')
protocol=load('test_m3_progress_broker')


def diag(code='tool_shape'):
    return dict(code=code,status='failed',stage='decode',settled=True,reservation_id='r',
        response_shape=dict(version=1,choices='one',message_type='object',refusal_truthy=False,
            finish_reason='stop',tool_calls='missing'))

def ledger(ident):
    return (json.dumps(dict(kind='reserve',id=ident,usd='2'))+'\n'+
            json.dumps(dict(kind='settled',id=ident,usd='.001'))+'\n').encode()

def manifest():
    arms=[dict(task='a',mode='rules'),dict(task='b',mode='record'),dict(task='b',mode='rules')]
    return dict(arms=arms,unattempted=[[a['task'],a['mode']] for a in arms],global_wall_seconds=6000)


class FakeOps:
    def __init__(self, failure=None):
        self.raw=b'';self.runs=[];self.failure=failure;self.guards=0
    def ledger(self): return self.raw
    def guard(self,initial):
        self.guards+=1
        if self.failure in ('billing','containment','freeze','budget') and len(self.runs)==1:
            raise ValueError(self.failure)
    def run(self,arm,state):
        self.runs.append(arm)
        self.raw+=ledger(str(len(self.runs)))
        if self.failure=='process': raise ValueError('process failure')
        return {}
    def export_and_verify(self,arm,raw,dest):
        if self.failure=='export': raise ValueError('integrity')
        d=diag();d['reservation_id']=str(len(self.runs))
        if self.failure=='refusal': d['code']='provider_refusal'
        result=dict(task=arm['task'],system=arm['mode'],outcome='unavailable')
        if self.failure=='identity': result['task']='wrong'
        return dict(result=result,diagnostics=[d],proposed_count=0,manifest_sha256='fixture')


class Lifecycle(unittest.TestCase):
    def test_three_eligible_failures_remain_unavailable_each_once(self):
        with tempfile.TemporaryDirectory() as td:
            state=Path(td)/'state';ops=FakeOps()
            result=c.coordinate(manifest(),state,ops)
            self.assertEqual([r['outcome'] for r in result],['unavailable']*3)
            self.assertEqual(len(ops.runs),3)
            self.assertEqual(len(list(state.glob('*-intent.json'))),3)
            with self.assertRaises(ValueError): c.coordinate(manifest(),state,ops)
            self.assertEqual(len(ops.runs),3)

    def test_global_guard_and_evidence_failures_never_advance(self):
        for reason in ('billing','containment','freeze','budget','process','export','refusal','identity'):
            with self.subTest(reason=reason),tempfile.TemporaryDirectory() as td:
                ops=FakeOps(reason);state=Path(td)/'state'
                with self.assertRaises(ValueError): c.coordinate(manifest(),state,ops)
                self.assertEqual(len(ops.runs),1)
                self.assertEqual(json.loads((state/'terminal.json').read_text())['status'],'halted-review-required')
                with self.assertRaises(ValueError): c.coordinate(manifest(),state,ops)

    def test_scope_and_global_deadline_before_dispatch(self):
        with tempfile.TemporaryDirectory() as td:
            m=manifest();m['arms'].append(m['arms'][0]);ops=FakeOps()
            with self.assertRaises(ValueError): c.coordinate(m,Path(td)/'bad',ops)
            m=manifest();m['global_wall_seconds']=1799
            with self.assertRaises(ValueError): c.coordinate(m,Path(td)/'short',ops)
            self.assertEqual(ops.runs,[])

    def test_deadline_after_worker_stops_next_arm(self):
        with tempfile.TemporaryDirectory() as td,patch.object(c.time,'monotonic',side_effect=[0,0,6001]):
            ops=FakeOps()
            with self.assertRaises(ValueError): c.coordinate(manifest(),Path(td)/'state',ops)
            self.assertEqual(len(ops.runs),1)

    def test_intent_sync_happens_before_dispatch(self):
        with tempfile.TemporaryDirectory() as td:
            ops=FakeOps();order=[];real=c.sync_directory;run=ops.run
            def sync(path): order.append('sync');real(path)
            def dispatch(arm,state):
                self.assertEqual(order[-1],'sync')
                self.assertTrue(list(state.glob('*-intent.json')))
                order.append('run');return run(arm,state)
            ops.run=dispatch
            with patch.object(c,'sync_directory',side_effect=sync): c.coordinate(manifest(),Path(td)/'state',ops)

    def test_all_other_failure_classes_are_ineligible(self):
        for code in ('choice_count','provider_refusal','output_truncated','proposal_type','proposal_fields',
                     'forecast_type','smoke_scope','usage_missing','transport_or_internal','billing_unsettled','unknown'):
            self.assertFalse(c.eligible(diag(code)),code)
        for change in ({'stage':'completion_transport'},{'settled':False},{'response_shape':{}},
                       {'status':'completed'}):
            d=diag();d.update(change);self.assertFalse(c.eligible(d))
        for field,value in [('finish_reason','length'),('finish_reason','content_filter'),('refusal_truthy',True),('choices','multiple')]:
            d=diag();d['response_shape'][field]=value;self.assertFalse(c.eligible(d))

    def test_only_argument_json_failure_is_eligible(self):
        d=diag('json_invalid');d['response_shape'].update(tool_calls='one',name_is_propose=True,argument_parse='invalid_json')
        self.assertTrue(c.eligible(d))
        d['stage']='completion_transport';self.assertFalse(c.eligible(d))

    def test_ledger_must_settle_exact_failed_request(self):
        d=diag();valid=ledger('r')
        self.assertEqual(c.settled_diagnostics(b'',valid,[d]),['r'])
        for raw in (valid[:-1],valid+valid,valid.splitlines(keepends=True)[0],valid.replace(b'"settled"',b'"halt"')):
            with self.assertRaises((ValueError,KeyError)):c.settled_diagnostics(b'',raw,[d])
        for diagnostics in ([{**d,'reservation_id':'other'}],[d,d],[]):
            with self.assertRaises(ValueError):c.settled_diagnostics(b'',valid,diagnostics)

    def test_rejected_response_cannot_have_proposal(self):
        with self.assertRaises(ValueError):
            c.disposition({'outcome':'unavailable'},[diag()],b'',ledger('r'),1)

    def test_budget_reservations_block_incremental_and_aggregate(self):
        baseline=c.budget.freeze(b'')
        for value in ('9','99'):
            raw=(json.dumps(dict(kind='reserve',id='old',usd=value))+'\n'+json.dumps(dict(kind='settled',id='old',usd=value))+'\n').encode()
            base=baseline if value=='9' else c.budget.freeze(raw)
            with self.assertRaises(ValueError):c.budget.check(raw,base)

    def test_initial_guard_failure_leaves_no_attempt(self):
        with tempfile.TemporaryDirectory() as td:
            ops=FakeOps();ops.guard=lambda initial:(_ for _ in ()).throw(ValueError('not idle'))
            with self.assertRaises(ValueError): c.coordinate(manifest(),Path(td)/'state',ops)
            self.assertEqual(ops.runs,[])
            self.assertFalse((Path(td)/'state').exists())


class Integration(unittest.TestCase):
    def test_fake_provider_through_adapter_diagnostics_ledger_and_controller(self):
        """Actual adapter encode/run/decode/diagnostics; mocked HTTP and credentials."""
        class ProviderOps(FakeOps):
            def __init__(self, directory, variant):
                super().__init__();self.variant=variant;self.path=directory/'cost.jsonl';self.path.write_text('')
            def ledger(self):return self.path.read_bytes()
            def run(self,arm,state):
                self.runs.append(arm)
                self.before=(self.path.with_name('cost-diagnostics.jsonl').read_bytes()
                    if self.path.with_name('cost-diagnostics.jsonl').exists() else b'')
                request=protocol.ProgressProtocol().request();request.update(deadline_ms=10000)
                response={'choices':[{'finish_reason':'stop','message':{'content':'PRIVATE_SOURCE'}}],
                    'billing':{'status':'settled','currency':'USD','billed_cost_usd':'.001'},
                    'usage':{'total_tokens':1}}
                if self.variant=='truncation': response['choices'][0]['finish_reason']='length'
                if self.variant=='refusal': response['choices'][0]['message']['refusal']='PRIVATE_SOURCE'
                if self.variant in ('arguments','action','wrong_name','multiple'):
                    function=dict(name='other' if self.variant=='wrong_name' else 'propose',arguments='{' if self.variant=='arguments' else json.dumps(dict(action='host_exec' if self.variant=='action' else 'stop',path='',id='',contents='',forecast=None)))
                    response['choices'][0]['message']['tool_calls']=[dict(type='function',function=function)]*(2 if self.variant=='multiple' else 1)
                expected={'missing':'tool_shape','wrong_name':'tool_shape','multiple':'tool_shape','arguments':'json_invalid','action':'proposal_type','refusal':'provider_refusal','truncation':'output_truncated'}[self.variant]
                catalog={'data':[{'id':b.MODEL,'pricing':{'list_input_per_million':'.15','list_output_per_million':'.6'}}]}
                baseline=c.budget.freeze(b'')
                def budget_check(log):log.seek(0);return c.budget.check(log.read().encode(),baseline)
                with patch.object(b,'credential',return_value='PRIVATE_KEY'),patch.object(b,'smoke_budget',side_effect=budget_check),patch.object(b,'request_json',side_effect=[catalog,response]) as network:
                    with self_test.assertRaises((b.BrokerFailure,json.JSONDecodeError)) as error:b.complete_with_diagnostics(request,self.path)
                    self_test.assertEqual(error.exception.code if isinstance(error.exception,b.BrokerFailure) else 'json_invalid',expected)
                    self_test.assertEqual(network.call_count,2)
                return {}
            def export_and_verify(self,arm,raw,dest):
                diagnostics=[json.loads(line) for line in self.path.with_name('cost-diagnostics.jsonl').read_bytes()[len(self.before):].splitlines()]
                dest.mkdir();(dest/'diagnostics.json').write_text(json.dumps(diagnostics))
                return dict(result=dict(task=arm['task'],system=arm['mode'],outcome='unavailable'),
                            diagnostics=diagnostics,proposed_count=0,manifest_sha256=c.sha(dest/'diagnostics.json'))
        self_test=self
        for variant in ('missing','wrong_name','multiple','arguments','action','refusal','truncation'):
            with self.subTest(variant=variant),tempfile.TemporaryDirectory() as td:
                root=Path(td);ops=ProviderOps(root,variant)
                if variant in ('action','refusal','truncation'):
                    with self.assertRaises(ValueError):c.coordinate(manifest(),root/'state',ops)
                    self.assertEqual(len(ops.runs),1)
                else:
                    result=c.coordinate(manifest(),root/'state',ops)
                    self.assertEqual([r['outcome'] for r in result],['unavailable']*3)
                for path in root.rglob('*'):
                    if path.is_file():
                        self.assertNotIn('PRIVATE_SOURCE',path.read_text());self.assertNotIn('PRIVATE_KEY',path.read_text())
                self.assertEqual(c.budget.total(ops.ledger()),c.budget.D('.001')*len(ops.runs))

    def test_production_export_verifies_preserved_evidence_and_rejects_corrupt_cas(self):
        original=c.ROOT/'docs/m3-progress-validation-execution'
        task='0c5a91a6aaa862c279aed1e1602a74eda7c42c4fc0f72247de513fa948f50b70'
        label=task[:12]+'-record'
        for corrupt in (False,True):
            with self.subTest(corrupt=corrupt),tempfile.TemporaryDirectory() as td:
                root=Path(td);base=root/'input';out=base/'record'
                (out/'state'/'one').mkdir(parents=True)
                shutil.copytree(original/(label+'/cas'),out/'cas')
                shutil.copytree(original/(task[:12]+'-input-cas'),base/'cas')
                shutil.copyfile(original/(label+'-state.sqlite'),out/'state'/'one'/'state.sqlite')
                shutil.copyfile(original/(label+'-results.jsonl'),out/'results-record.jsonl')
                for name in ('tasks.jsonl','profile.toml','broker-contexts.json'):
                    shutil.copyfile(original/(task[:12]+'-'+name),base/name)
                (base/'preparation.json').write_text('{}')
                if corrupt: next((out/'cas').iterdir()).write_bytes(b'inert corruption')
                ops=c.LocalOperations.__new__(c.LocalOperations)
                ops.receipts=c.load('m3-progress-validation-evidence')
                ops.ledger_path=root/'ledger.jsonl';ops.ledger_path.write_bytes((original/'m3-provider-cost.jsonl').read_bytes())
                ops.diagnostic_path=root/'diagnostics.jsonl'
                ops.diagnostic_path.write_bytes((original/'m3-provider-cost-diagnostics.jsonl').read_bytes())
                ops.diagnostic_before=b''
                args=(dict(task=task,mode='record',input=str(base)),dict(status='completed'),root/'export')
                if corrupt:
                    with self.assertRaises(ValueError):ops.export_and_verify(*args)
                else:
                    result=ops.export_and_verify(*args)
                    self.assertEqual(result['result']['outcome'],'unavailable')
                    self.assertEqual(result['proposed_count'],5)
                    self.assertTrue((root/'export'/'input-cas').is_dir())
                    hashes=json.loads((root/'export'/'sha256.json').read_text())
                    self.assertTrue(all(c.sha(root/'export'/p)==h for p,h in hashes.items()))

    def oracle_fixture(self, root, accepted):
        base=root/'input';out=base/'rules';(out/'state'/'one').mkdir(parents=True)
        (out/'cas').mkdir();(base/'cas').mkdir()
        for name in ('tasks.jsonl','profile.toml','broker-contexts.json','preparation.json'):
            (base/name).write_text('{}')
        closure='oracle:'+str(accepted).lower()
        payload=json.dumps(dict(kind=dict(kind='task_closed',reason=closure)))
        with sqlite3.connect(out/'state'/'one'/'state.sqlite') as db:
            db.execute('create table events(seq integer, id text, payload text)')
            db.execute('insert into events values (1,?,?)',('inert-id',payload))
        ops=c.LocalOperations.__new__(c.LocalOperations)
        class Receipts:
            def verify_receipts(self,*args):return dict(status='checked',verified_success=accepted)
        ops.receipts=Receipts();ops.ledger_path=root/'ledger.jsonl';ops.ledger_path.write_bytes(ledger('r'))
        ops.diagnostic_path=root/'diagnostics.jsonl';ops.diagnostic_path.write_text(json.dumps(diag())+'\n')
        ops.diagnostic_before=b''
        result=dict(task='a',system='rules',outcome='rejected',oracle=dict(accepted=accepted),denials=[],
                    state_root_at_start=ops.digest(b''),state_root_at_end=ops.digest(struct.pack('<Q',8)+b'inert-id'))
        (out/'results-rules.jsonl').write_text(json.dumps(result)+'\n')
        return ops,(dict(task='a',mode='rules',input=str(base)),{},root/'export')

    def test_review_04_contradictory_rejected_oracle_halts(self):
        with tempfile.TemporaryDirectory() as td:
            ops,args=self.oracle_fixture(Path(td),True)
            with self.assertRaises(ValueError):ops.export_and_verify(*args)

    def test_checked_negative_oracle_is_preserved(self):
        with tempfile.TemporaryDirectory() as td:
            ops,args=self.oracle_fixture(Path(td),False)
            result=ops.export_and_verify(*args)
            self.assertEqual(result['result']['outcome'],'rejected')
            self.assertFalse(result['result']['oracle']['accepted'])

    def test_broker_baseline_is_only_change_from_reviewed_diagnostics(self):
        expected=(HERE/'m3-shape-diagnostics-broker.py').read_text().replace('docs/m3-progress-validation-baseline.json','docs/m3-continuation-baseline.json')
        self.assertEqual((HERE/'m3-continuation-broker.py').read_text(),expected)

if __name__=='__main__':unittest.main()
