"""Launcher/accounting wiring only; fake transport/driver, no model or target commands."""
import contextlib
import io
import json
import tempfile
import sys
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch
from test_engine_b91_campaign import m
from test_engine_project_runtime_review import r

class Activation(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name)
        self.manifest=m.manifest('a'*64,self.root/'ledger.jsonl');m.fresh(self.root/'manifest.json',self.manifest)
        m.Ledger(self.root/'ledger.jsonl',self.manifest,m.digest(self.manifest),create=True)
        self.a={'trial':str(self.root/'trial'),'model':m.MODEL,'expires_at':'2099-01-01T00:00:00+00:00'}
        m.fresh(self.root/'trial.json',self.a);self.review=r.admission(self.a,self.manifest);m.fresh(self.root/'review.json',self.review)
        self.effects=[];outer=self
        class Driver:
            def __init__(self,root,a,provider,worker,reviewer):self.provider=provider;self.worker=worker;self.reviewer=reviewer;self.stage=''
            def journal(self):return SimpleNamespace(events=lambda:[{'kind':'dispatch_intent','id':self.stage}])
            def run(self):
                for i in range(6):
                    self.stage=str(i);self.provider({'model':m.MODEL,'stream':False,'max_completion_tokens':8192,'messages':[{}, {'content':json.dumps({'stage':self.stage})}]},240)
                    self.reviewer('relational',{'index':i})
                for stage in ('seed','check_before','patch','check_after'):self.worker(None,stage)
                return {'fixture_only':True}
        def invoke(body,timeout,**kwargs):self.effects.append('transport');return {'receipt':{'status':'settled','usd':'0.001'}}
        self.driver=SimpleNamespace(e=SimpleNamespace(strict=m.strict,bounded_file=lambda p,n:p.read_bytes(),sha=m.digest,
            module=lambda *_:SimpleNamespace(invoke=invoke),PayloadBoundError=ValueError),validate_admission=lambda *_:None,
            stage_payload_limit=lambda *_:16384,Driver=Driver,FileReviewer=lambda *_:lambda *_:None,
            live_worker=lambda *_:{'fixture_only':True},Blocked=RuntimeError)
        class Reviewer:
            def __init__(self,driver,a,ledger,transport,final):self.ledger=ledger;self.transport=transport;self.trial=m.digest(a);ledger.enroll(self.trial,m.digest(a))
            def __call__(self,kind,context):
                body={'model':m.MODEL,'stream':False,'max_completion_tokens':8192,'messages':[{'role':'user','content':'fixture reviewer separate context'}]}
                return self.ledger.run_call(self.trial,str(context['index']),body,240,self.transport)
        self.review_module=SimpleNamespace(digest=r.digest,admission=r.admission,RuntimeReviewer=Reviewer)
        self.argv=['campaign','run','--campaign',str(self.root),'--campaign-sha256',m.digest(self.manifest),'--admission',str(self.root/'trial.json'),
            '--admission-sha256',m.digest(self.a),'--reviews',str(self.root/'reviews'),'--allow-paid-inference','--allow-target-execution']
    def run_cli(self,flags):
        from test_engine_campaign_price import PriceQuotes
        quote=PriceQuotes().quote()
        def load(name,file):return self.review_module if file=='engine-project-runtime-review.py' else self.driver
        with patch.object(m,'load_module',side_effect=load),patch.object(m,'official_preflight',return_value=quote),patch.object(sys,'argv',self.argv+flags),contextlib.redirect_stdout(io.StringIO()):m.main()
    def flags(self):return ['--runtime-review-admission',str(self.root/'review.json'),'--runtime-review-admission-sha256',r.digest(self.review)]
    def status(self):return m.Ledger(self.root/'ledger.jsonl',self.manifest,m.digest(self.manifest)).status()
    def test_default_stays_six_calls(self):
        self.run_cli([]);self.assertEqual(self.status()['calls'],6);self.assertEqual(self.status()['execution_groups'],4)
    def test_explicit_activation_shares_twelve_calls_four_groups(self):
        self.run_cli(self.flags());s=self.status();self.assertEqual((s['calls'],s['execution_groups'],s['known_cost_microusd']),(12,4,12000))
    def test_incomplete_flags_fail_before_any_effect(self):
        for flags in (self.flags()[:2],self.flags()[2:]):
            with self.subTest(flags=flags),contextlib.redirect_stderr(io.StringIO()),self.assertRaises(SystemExit):self.run_cli(flags)
        self.assertEqual(self.effects,[]);self.assertEqual(self.status()['calls'],0)
    def test_wrong_hash_source_or_campaign_fail_before_preflight(self):
        for field in ('driver_admission_sha256','campaign_manifest_sha256'):
            wrong=dict(self.review);wrong[field]='f'*64;(self.root/'review.json').write_bytes(m.canonical(wrong))
            flags=['--runtime-review-admission',str(self.root/'review.json'),'--runtime-review-admission-sha256',r.digest(wrong)]
            with self.subTest(field=field),self.assertRaises(ValueError):self.run_cli(flags)
        (self.root/'review.json').write_bytes(m.canonical(self.review));flags=self.flags();flags[-1]='f'*64
        with self.assertRaises(ValueError):self.run_cli(flags)
        self.assertEqual(self.effects,[]);self.assertEqual(self.status()['calls'],0)

if __name__=='__main__':unittest.main()
