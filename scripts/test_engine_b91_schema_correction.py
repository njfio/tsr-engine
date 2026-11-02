"""Authored provider fixtures with actual native controller; no target execution."""
import copy
import json
import unittest
from test_engine_b91_driver import NativeSequence,m
import test_engine_project_runtime_review as rr

class Correction(NativeSequence):
    def setUp(self):
        super().setUp()
        self.a=m.admission(self.root,fixture=True,model='gpt-6-luna',final_payload_32k=True,schema_corrections=True)
        self.invalids=1;self.feedback=[]
    def provider(self,body,timeout):
        result=super().provider(body,timeout)
        stage=json.loads(body['messages'][1]['content'])['stage']
        if len(body['messages'])>2:self.feedback.append(body['messages'][-1])
        if stage=='experiment' and self.invalids:
            self.invalids-=1;result['value']['extra']='not admitted'
        return result
    def test_correction_full_native_and_replay(self):
        d=self.driver()
        harness=rr.ProjectReview();harness.base=self;harness.invoked=[];harness.mode='accept'
        path=self.root.parent/'campaign.jsonl';cm=rr.campaign.m
        manifest=cm.manifest('a'*64,path,fixture=True);ledger=cm.Ledger(path,manifest,cm.digest(manifest),create=True)
        trial='b'*64;ledger.enroll(trial,m.e.sha(self.a))
        original=d.transport;worker=d.worker
        d.transport=lambda body,timeout:ledger.run_call(trial,[x for x in d.journal().events() if x['kind']=='dispatch_intent'][-1]['id'],body,timeout,original,payload_limit=m.stage_payload_limit(self.a,json.loads(body['messages'][1]['content'])['stage']))
        d.worker=lambda c,stage:ledger.run_execution(trial,stage,m.e.sha(self.a),lambda:worker(c,stage))
        d.reviewer=rr.r.RuntimeReviewer(d,rr.r.admission(self.a,manifest),ledger,harness.transport,self.reviewer)
        r=d.run()
        self.assertEqual(r['calls'],6);self.assertEqual(r['resources']['model_calls'],6)
        self.assertEqual(ledger.status()['calls'],12);self.assertEqual(ledger.status()['execution_groups'],4)
        self.assertEqual(len(harness.invoked),6)
        self.assertEqual(len(self.workers),4);self.assertEqual(len(self.feedback),1)
        self.assertIn('unsupported_envelope_fields',self.feedback[0]['content'])
        self.assertEqual(sum(x['kind']=='schema_correction' for x in d.journal().events()),1)
        ledger_before=ledger.path.read_bytes()
        before=(len(self.calls),len(self.workers));self.driver().run();self.assertEqual(before,(len(self.calls),len(self.workers)))
        self.assertEqual(ledger.path.read_bytes(),ledger_before)
    def test_repeated_invalid_exhausts_without_application(self):
        self.invalids=10;d=self.driver()
        with self.assertRaises((ValueError,m.Blocked)):d.run()
        self.assertEqual(self.calls.count('experiment'),3)
        self.assertEqual(self.workers,['seed']);self.assertEqual(d.journal().status()['calls_consumed_or_reserved'],4)
        with self.assertRaises((ValueError,m.Blocked)):self.driver().run()
        self.assertEqual(self.calls.count('experiment'),3)

# Avoid inheriting the unrelated broad native matrix.
for name in list(vars(NativeSequence)):
    if name.startswith('test_'):setattr(Correction,name,None)
del NativeSequence

from test_engine_b91_project_envelope import ProjectEnvelope
import test_engine_b91_project_envelope as pe

class CorrectionBoundaries(ProjectEnvelope):
    def setUp(self):
        super().setUp();self.journal=pe.m.Journal(self.path,self.origin,self.binding,correction_limit=2)
    def invalid(self):
        fake=self.fake()
        with self.assertRaises(pe.m.StageValidationError):pe.m.fixture_step(self.journal,fake,lambda *_:self.reply({'relational':{},'extra':1}))
        intent=next(x for x in self.journal.events() if x['kind']=='dispatch_intent')
        return fake,intent,{'kind':'schema_correction','id':intent['id'],'state_root':intent['state_root'],'code':'unsupported_envelope_fields'}
    def test_restart_marker_and_stale_source(self):
        fake,intent,marker=self.invalid();self.journal.append(marker)
        restored=pe.m.Journal(self.path,self.origin,self.binding,correction_limit=2)
        self.assertEqual(restored.status()['next_stage'],pe.m.STAGES[0])
        fake.current['state_root']='f'*64
        with self.assertRaises(ValueError):pe.m.fixture_step(restored,fake,lambda *_:self.fail('stale dispatch'))
        self.assertEqual(restored.status()['calls_consumed_or_reserved'],1)
        with self.assertRaises(ValueError):pe.m.Journal(self.path,self.origin,self.binding)
    def test_unknown_never_correct(self):
        fake=self.fake()
        def unknown(*_):raise TimeoutError('fixture')
        with self.assertRaises(TimeoutError):pe.m.fixture_step(self.journal,fake,unknown)
        intent=next(x for x in self.journal.events() if x['kind']=='dispatch_intent')
        with self.assertRaises(ValueError):self.journal.append({'kind':'schema_correction','id':intent['id'],'state_root':intent['state_root'],'code':'unsupported_envelope_fields'})
        with self.assertRaises(ValueError):pe.m.fixture_step(self.journal,fake,lambda *_:self.fail('unknown retry'))
    def test_missing_cost_never_correct(self):
        fake=self.fake();reply=self.reply({'relational':{},'extra':1});reply['receipt']['usd']=None
        with self.assertRaises(ValueError):pe.m.fixture_step(self.journal,fake,lambda *_:reply)
        intent=next(x for x in self.journal.events() if x['kind']=='dispatch_intent')
        with self.assertRaises(ValueError):self.journal.append({'kind':'schema_correction','id':intent['id'],'state_root':intent['state_root'],'code':'unsupported_envelope_fields'})
        self.assertEqual(fake.stores,0)
    def test_corrected_dispatch_interruption_is_not_repeated(self):
        fake,intent,marker=self.invalid();self.journal.append(marker)
        def unknown(*_):raise TimeoutError('fixture')
        with self.assertRaises(TimeoutError):pe.m.fixture_step(self.journal,fake,unknown)
        restored=pe.m.Journal(self.path,self.origin,self.binding,correction_limit=2)
        self.assertEqual(restored.status()['calls_consumed_or_reserved'],2)
        with self.assertRaises(ValueError):pe.m.fixture_step(restored,fake,lambda *_:self.fail('unknown repeat'))
    def test_tampered_feedback_and_no_admission(self):
        fake,intent,marker=self.invalid()
        for key,value in [('state_root','f'*64),('code','native_validation_rejected')]:
            with self.subTest(key=key),self.assertRaises(ValueError):self.journal.append({**marker,key:value})
        self.assertEqual(self.journal.status()['calls_consumed_or_reserved'],1)

for name in list(vars(ProjectEnvelope)):
    if name.startswith('test_'):setattr(CorrectionBoundaries,name,None)
del ProjectEnvelope

class ReconciliationCorrectionBinding(unittest.TestCase):
    def test_readonly_reader_uses_historical_allowance(self):
        from types import SimpleNamespace
        from unittest.mock import patch
        import tempfile
        from pathlib import Path
        cm=rr.campaign.m
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);manifest=cm.manifest('a'*64,root/'ledger',fixture=True)
            ledger=cm.Ledger(root/'ledger',manifest,cm.digest(manifest),create=True);before=ledger.path.read_bytes()
            for allowance in (0,2):
                a={'model':cm.MODEL,'provider':cm.ENDPOINT,'trial':str(root),'mode':'fixture'}
                if allowance:a['schema_corrections']=allowance
                class View:
                    def read(self):return [],{}, {'origin':{'sha256':'b'*64}}
                def journal(*args,**kwargs):
                    self.assertEqual(kwargs,{'correction_limit':allowance});raise LookupError('reader boundary reached')
                fake=SimpleNamespace(Driver=View,e=SimpleNamespace(bounded_file=lambda *_:json.dumps({'event':{'value':a}}).encode(),strict=cm.strict,sha=cm.digest,Journal=journal))
                review={'reviewer':'fixture','provider_endpoint':'https://api.cheaperinference.com/v1/usage/requests?request_ids=fixture','lookup_sha256':'d'*64,'billing_semantics_sha256':'e'*64,'decision':'exact_failed_zero'}
                with patch.object(cm,'load_module',return_value=fake),self.assertRaisesRegex(LookupError,'reader boundary'):
                    ledger.reconcile_failed_zero(root,root/'absent',review,cm.digest(review),dry_run=True)
                self.assertEqual(ledger.path.read_bytes(),before)
