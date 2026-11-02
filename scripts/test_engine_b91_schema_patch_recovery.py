"""Offline native/fake-worker recovery; authored fixtures, no target commands."""
import importlib.util,json,unittest
from pathlib import Path
from unittest.mock import patch
import test_engine_b91_driver as base
import test_engine_project_runtime_review as rr
spec=importlib.util.spec_from_file_location('schema_recovery',Path(__file__).with_name('engine-b91-schema-patch-recovery.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Recovery(base.NativeSequence):
    def parent_prefix(self):
        self.a=base.m.admission(self.root,fixture=True,model='gpt-6-luna',final_payload_32k=True,schema_corrections=True)
        path=self.root.parent/'campaign.jsonl';manifest=m.campaign.manifest('a'*64,path,fixture=True)
        ledger=m.campaign.Ledger(path,manifest,m.campaign.digest(manifest),create=True)
        parent=self.driver();trial=m.campaign.digest({'trial':str(self.root.resolve()),'admission':m.e.sha(self.a)});ledger.enroll(trial,m.e.sha(self.a))
        invalid=[True]
        def transport(body,timeout):
            value=self.provider(body,timeout);stage=json.loads(body['messages'][1]['content'])['stage']
            if stage=='experiment' and invalid[0]:value['value']['extra']=1;invalid[0]=False
            if stage=='boundary_patch':value['value']['rationale']='x'*592
            value['receipt']['response_request_ids']=['aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee']
            return value
        parent.transport=lambda body,timeout:ledger.run_call(trial,[x for x in parent.journal().events() if x['kind']=='dispatch_intent'][-1]['id'],body,timeout,transport,payload_limit=base.m.stage_payload_limit(self.a,json.loads(body['messages'][1]['content'])['stage']))
        parent.worker=lambda c,stage:ledger.run_execution(trial,stage,m.e.sha(self.a),lambda:self.worker(c,stage))
        review=rr.ProjectReview();review.base=self;review.invoked=[];review.mode='accept'
        parent.reviewer=rr.r.RuntimeReviewer(parent,rr.r.admission(self.a,manifest),ledger,review.transport,self.reviewer)
        original=base.m.e.validate_stage_envelope
        def historical(stage,value):
            try:return original(stage,value)
            except base.m.e.StageValidationError as error:
                if error.code=='invalid_rationale_length':return 'edits'
                raise
        with patch.object(base.m.e,'validate_stage_envelope',historical),self.assertRaises(base.m.e.StageValidationError):parent.run()
        admission=self.root.parent/'parent.json';m.e.private_save(admission,self.a)
        mp=self.root.parent/'manifest.json';m.e.private_save(mp,manifest)
        rows=[m.e.strict(x) for x in path.read_bytes().splitlines()]
        up={'version':'b91-campaign-upgrade@v1','old_manifest_sha256':m.campaign.digest(manifest),'old_implementation_sha256':manifest['implementation_sha256'],'new_implementation_sha256':manifest['implementation_sha256'],'ledger_prefix_head':rows[-1]['hash'],'reviewer':'fixture','reason':'fixture exact same implementation'}
        upp=self.root.parent/'upgrade.json';m.e.private_save(upp,up)
        proof={'manifest':str(mp),'upgrade':str(upp),'hashes':{'manifest':m.d.file_sha(mp),'upgrade':m.d.file_sha(upp)}}
        return admission,proof,ledger,review
    def test_full_tail_preserves_prefix_reviews_and_accounting(self):
        path,proof,ledger,review=self.parent_prefix();old=self.root;before=m.b.tree(old)
        self.assertEqual((ledger.status()['calls'],ledger.status()['execution_groups']),(8,2))
        new=old.parent/'recovery';a=m.admission(path,new,proof)
        for key,value in [('calls',6),('execution_groups',3),('schema_corrections',3)]:
            with self.subTest(key=key),self.assertRaises(ValueError):m.validate({**a,key:value},new.resolve(),True)
        self.root=new;self.a=a;self.calls=[];self.workers=[]
        def transport(body,timeout):
            return self.provider(body,timeout) if 'stage' in json.loads(body['messages'][1]['content']) else review.transport(body,timeout)
        def create():
            run=m.campaign_runner(new,a,ledger,transport,self.worker,self.reviewer,fixture=True)
            (new/'fixture-only.marker').write_text('b91-native-fixture-only@v1\n');return run
        run=create();result=run.run()
        self.assertEqual(self.calls,list(m.e.STAGES[3:5]));self.assertEqual(self.workers,['patch','check_after'])
        self.assertEqual((result['calls'],result['historical_calls']),(2,5));self.assertEqual(len(review.invoked),6)
        self.assertEqual((ledger.status()['calls'],ledger.status()['execution_groups']),(13,4))
        self.assertEqual(m.b.tree(old),before);raw=ledger.path.read_bytes();self.assertEqual(create().run(),result);self.assertEqual(ledger.path.read_bytes(),raw)
        events=run.journal().events();self.assertEqual(len([x for x in events if x['kind']=='dispatch_intent']),5)
        self.assertNotIn(a['prefix']['failed_request'],[x.get('id') for x in events])
        unknownroot=old.parent/'unknown';unknowna=m.admission(path,unknownroot,proof)
        self.root=unknownroot;self.a=unknowna
        def unknown(*_):raise TimeoutError('fixture uncertain dispatch')
        u=m.campaign_runner(unknownroot,unknowna,ledger,unknown,self.worker,self.reviewer,fixture=True)
        (unknownroot/'fixture-only.marker').write_text('b91-native-fixture-only@v1\n')
        with self.assertRaises(TimeoutError):u.run()
        count=ledger.status()['calls']
        with self.assertRaises((m.d.Blocked,ValueError)):m.campaign_runner(unknownroot,unknowna,ledger,unknown,self.worker,self.reviewer,fixture=True).run()
        self.assertEqual(ledger.status()['calls'],count);self.assertTrue(ledger.status()['unknown_calls'])
        self.root=new;self.a=a
        (new/'parent-evidence/quarantine'/next(iter(a['prefix']['quarantine_files']))).write_text('tampered')
        with self.assertRaisesRegex(ValueError,'archive audit drift'):run.verify_archive()
for name in list(vars(base.NativeSequence)):
    if name.startswith('test_'):setattr(Recovery,name,None)

class StructuralBoundary(unittest.TestCase):
    def test_rationale_utf8_limit(self):
        value={'edits':[{'path':'src/lib.rs','replacement':'fixture'}],'rationale':'x'*512}
        self.assertEqual(m.e.validate_stage_envelope('boundary_patch',value),'edits')
        for rationale in ('x'*513,'é'*257,'',None):
            with self.subTest(type=type(rationale).__name__),self.assertRaises(m.e.StageValidationError) as caught:
                m.e.validate_stage_envelope('boundary_patch',{**value,'rationale':rationale})
            self.assertEqual(caught.exception.code,'invalid_rationale_length')
        self.assertNotIn('native_validation_rejected',m.e.CORRECTION_CODES)
