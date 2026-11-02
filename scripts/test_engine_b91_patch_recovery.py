"""Authored fixtures, real controller/test-only Runner; no provider or target commands."""
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
import test_engine_b91_driver as base
spec=importlib.util.spec_from_file_location('patch_recovery','scripts/engine-b91-patch-recovery.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class RecoveryTests(base.NativeSequence):
    # Inherit fixture helpers, not the parent's unrelated tests.
    def parent_failed(self):
        self.a=base.m.admission(self.root,fixture=True,model='gpt-6-luna',final_payload_32k=True)
        original=self.provider
        def failure(body,timeout):
            result=original(body,timeout)
            if json.loads(body['messages'][1]['content'])['stage']=='boundary_patch':
                result['value']=None;result['receipt'].update(response_status='received_http_error',http_status=503,error_code='http_error',
                    usage={},status='unknown',usd=None,response_request_ids=['aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee'])
            return result
        d=self.driver();d.transport=failure
        with self.assertRaises(ValueError):d.run()
        path=self.root.parent/'parent-admission.json';m.e.private_save(path,self.a)
        return path
    def ledger(self):
        path=self.root.parent/'campaign.jsonl';manifest=m.campaign.manifest('a'*64,path,fixture=True)
        ledger=m.campaign.Ledger(path,manifest,m.campaign.digest(manifest),create=True)
        ledger.enroll('b'*64,'c'*64)
        body={'model':'gpt-6-luna','stream':False,'max_completion_tokens':8192,'messages':[]}
        for i in range(4):ledger.run_call('b'*64,str(i),body,240,lambda *_:{'receipt':{'status':'settled','usd':'0'}})
        for i in range(2):ledger.run_execution('b'*64,str(i),'c'*64,lambda:{'fixture_only':True})
        return ledger
    def test_recovery_native_tail_restart_and_tamper(self):
        parent=self.parent_failed();oldroot=self.root;before=m.b.tree(oldroot);ledger=self.ledger()
        # Only billing join is mocked; accounting tests validate that contract separately.
        with patch.object(m,'proof_validate',return_value=m.campaign.digest(ledger.manifest)):
            new=oldroot.parent/'recovery';a=m.admission(parent,new,{'fixture_only':'mocked billing provenance'})
            for field,value in (('calls',4),('execution_groups',3),('input_tokens',81921)):
                bad=dict(a);bad[field]=value
                with self.assertRaises(ValueError):m.validate(bad,new.resolve(),True)
            provider=oldroot/'provider.jsonl';raw=provider.read_bytes()
            original=[m.e.strict(x) for x in raw.splitlines()]
            for mutation in ('500','unknown'):
                rows=[];j=m.e.Journal(provider,oldroot/'origin.json',m.e.sha(m.e.strict(m.e.bounded_file(oldroot/'origin.json'))))
                for row in original:
                    event=dict(row['event'])
                    if event.get('kind')=='provider_outcome' and event['id']==a['prefix']['failed_request']:
                        if mutation=='500':event['http_status']=500
                        else:event['response_status']='uncertain'
                    rows.append(j._row(rows,event))
                provider.write_bytes(b''.join(m.e.canonical(x)+b'\n' for x in rows))
                with self.assertRaisesRegex(ValueError,'HTTP503'):m.prefix(parent,{})
                provider.write_bytes(raw)
            stop=oldroot/'STOP';stop.write_text('changed')
            with self.assertRaisesRegex(ValueError,'stop drift'):m.validate(a,new.resolve(),True)
            stop.unlink()
            source=oldroot/'project/coding-work/src/lib.rs';sourcebytes=source.read_bytes();source.write_bytes(sourcebytes+b'\n')
            with self.assertRaisesRegex(ValueError,'history drift'):m.validate(a,new.resolve(),True)
            source.write_bytes(sourcebytes)
            self.root=new;self.a=a;self.calls=[];self.workers=[];self.reviews=[]
            def create(hook=None):
                r=m.campaign_runner(new,a,ledger,self.provider,self.worker,self.reviewer,fixture=True,hook=hook)
                (new/'fixture-only.marker').write_text('b91-native-fixture-only@v1\n');return r
            result=create().run()
            self.assertEqual(self.calls,list(m.e.STAGES[3:5]));self.assertEqual(self.workers,['patch','check_after'])
            self.assertEqual((result['calls'],result['execution_groups'],result['historical_calls']),(2,2,4))
            self.assertEqual(result['resources']['model_calls'],2)
            self.assertEqual((ledger.status()['calls'],ledger.status()['execution_groups']),(6,4))
            self.assertEqual(m.b.tree(oldroot),before)
            j=create().journal();sent=[x for x in j.events() if x['kind']=='dispatch_intent']
            self.assertNotEqual(sent[3]['id'],a['prefix']['failed_request'])
            self.assertEqual(create().run(),result);self.assertEqual(len(self.calls),2);self.assertEqual(len(self.workers),2)
            with self.assertRaises(ValueError):create().generate(create().controller(),'experiment')
            with self.assertRaises(ValueError):create().execute(create().controller(),'seed')
            (new/'parent-evidence/provider.jsonl').write_text('tampered')
            with self.assertRaises(ValueError):create().run()
    def test_recovery_unknown_request_is_not_retried(self):
        parent=self.parent_failed();ledger=self.ledger()
        with patch.object(m,'proof_validate',return_value=m.campaign.digest(ledger.manifest)):
            new=self.root.parent/'recovery';a=m.admission(parent,new,{})
            self.root=new;self.a=a;self.calls=[];self.workers=[]
            def interrupted(body,timeout):
                self.calls.append('uncertain');raise RuntimeError('interrupted provider')
            r=m.campaign_runner(new,a,ledger,interrupted,self.worker,self.reviewer,fixture=True)
            (new/'fixture-only.marker').write_text('b91-native-fixture-only@v1\n')
            with self.assertRaises(RuntimeError):r.run()
            with self.assertRaises(m.d.Blocked):m.campaign_runner(new,a,ledger,self.provider,self.worker,self.reviewer,fixture=True).run()
            self.assertEqual(len(self.calls),1);self.assertEqual(self.workers,[])
            self.assertTrue(ledger.status()['stopped']);self.assertEqual(ledger.status()['calls'],5)
# Avoid inherited tests while retaining helper methods.
for name in list(dir(RecoveryTests)):
    if name.startswith('test_') and name not in RecoveryTests.__dict__:setattr(RecoveryTests,name,None)
if __name__=='__main__':unittest.main()
