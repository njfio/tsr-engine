"""Authored local stages only: no provider calls, native workers or game edits."""
import importlib.util
import json
import unittest
from pathlib import Path

spec=importlib.util.spec_from_file_location('fixture_cycle',Path(__file__).with_name('test_engine_project_cycle.py'))
f=importlib.util.module_from_spec(spec);spec.loader.exec_module(f)
m=f.m

class StageReceipts(unittest.TestCase):
    def fixture(self):
        x=f.Cycle('runTest');x.setUp();self.addCleanup(x.doCleanups)
        x.a.update(version='local-project-cycle@v3',local_source_profile='browser-scoped-source@v1',stage_receipts=m.stage_receipts.PROFILE)
        return x
    def pause(self,x,event,stage):
        def hook(e,n):
            if (e,n)==(event,stage):raise RuntimeError('simulated interruption')
        with self.assertRaisesRegex(RuntimeError,'simulated interruption'):x.runner(hook=hook).run()
    def test_restart_every_successful_stage_without_repeating_callback(self):
        for stage in ('request_0','proposal_0','review_0','advance_0','resume_0','request_1','proposal_1','review_1','advance_1','resume_1','terminal'):
            with self.subTest(stage=stage):
                x=self.fixture();self.pause(x,'after_effect',stage)
                self.assertEqual(x.runner().run()['status'],'idle')
                self.assertEqual(len(x.calls),2);self.assertEqual(len(x.reviews),2);self.assertEqual(x.actions,['first','second'])
                before=(x.root/'cycle.jsonl').read_bytes()
                self.assertEqual(x.runner().run()['status'],'idle');self.assertEqual(before,(x.root/'cycle.jsonl').read_bytes())
    def test_crash_after_receipt_before_outer_result_recovers(self):
        x=self.fixture();self.pause(x,'after_receipt','proposal_0')
        self.assertEqual(x.runner().recover_pending(),1)
        self.assertEqual(x.runner().recover_pending(),0)
        self.assertEqual(x.runner().run()['status'],'idle');self.assertEqual(len(x.calls),2)
    def test_missing_receipt_before_effect_stays_uncertain(self):
        x=self.fixture();self.pause(x,'after_intent','proposal_0')
        for _ in range(2):
            with self.assertRaisesRegex(m.d.Blocked,'receipt missing'):x.runner().run()
        self.assertEqual(x.calls,[]);self.assertEqual(x.actions,[])
    def test_callback_unknown_never_retried(self):
        x=self.fixture();calls=[]
        def unknown(_):calls.append(1);raise TimeoutError('unknown provider effect')
        x.proposal=unknown
        with self.assertRaises(TimeoutError):x.runner().run()
        with self.assertRaisesRegex(m.d.Blocked,'receipt missing'):x.runner().run()
        self.assertEqual(calls,[1]);self.assertEqual(x.actions,[])
    def receipt(self,x):
        runner=x.runner();rows,pending,_=runner.read();intent=next(iter(pending.values()))
        receipts=m.stage_receipts.Receipts(x.root,x.a)
        return receipts.path(receipts.binding(rows,intent))
    def test_tampered_result_rejected_without_effect(self):
        x=self.fixture();self.pause(x,'after_effect','proposal_0');p=self.receipt(x)
        v=json.loads(p.read_text());v['value']['response']['proposals']=[];p.write_text(json.dumps(v))
        with self.assertRaisesRegex(ValueError,'drift'):x.runner().run()
        self.assertEqual(len(x.calls),1);self.assertEqual(x.actions,[])
    def test_cross_intent_and_cross_root_substitution_rejected(self):
        for stage in ('proposal_0','review_0'):
            with self.subTest(stage=stage):
                x=self.fixture();self.pause(x,'after_effect','proposal_0')
                y=self.fixture();self.pause(y,'after_effect',stage)
                self.receipt(x).write_bytes(self.receipt(y).read_bytes())
                with self.assertRaisesRegex(ValueError,'drift'):x.runner().run()
                self.assertEqual(len(x.calls),1);self.assertEqual(x.actions,[])
    def test_receipt_symlink_rejected(self):
        x=self.fixture();self.pause(x,'after_effect','proposal_0');p=self.receipt(x)
        original=p.read_bytes();p.unlink();outside=x.root/'other';outside.write_bytes(original);p.symlink_to(outside)
        with self.assertRaisesRegex(ValueError,'symlink'):x.runner().run()
    def test_stop_prevents_recovery(self):
        x=self.fixture();self.pause(x,'after_effect','proposal_0');(x.root/'STOP').touch()
        with self.assertRaisesRegex(m.d.Blocked,'stopped'):x.runner().recover_pending()
        self.assertEqual(len(x.calls),1)
    def test_legacy_profile_retains_fail_closed_behavior(self):
        x=self.fixture();del x.a['stage_receipts'];self.pause(x,'after_effect','resume_0')
        with self.assertRaisesRegex(m.d.Blocked,'uncertain'):x.runner().run()
        self.assertEqual(len(x.actions),1)
    def test_unknown_assessment_is_fresh_feedback_only_with_opt_in(self):
        for enabled in (False,True):
            with self.subTest(enabled=enabled):
                x=self.fixture();x.a['max_review_rejections']=2
                if enabled:x.a['review_feedback']='attributed-review-feedback@v1'
                original_review=x.review;original_native=x.native;feedback=[]
                def review(state,response):
                    value=original_review(state,response)
                    if len(x.reviews)==1:value['judgment']['decision']='unknown'
                    return value
                def native(command,value):
                    if command=='reject_proposal':
                        feedback.append(value);x.guidance='source evidence now inspected';return {'stored':True}
                    return original_native(command,value)
                x.review=review;x.native=native;x.a['max_decisions']=3
                result=x.runner().run()
                self.assertEqual(result['status'],'idle' if enabled else 'review_unknown')
                self.assertEqual(len(feedback),1 if enabled else 0)
                if enabled:
                    self.assertEqual(feedback[0]['judgment']['decision'],'unknown')
                    self.assertEqual(len(x.calls),3);self.assertEqual(len(x.reviews),3)
                    self.assertEqual(x.actions,['source evidence now inspected','second'])
                else:self.assertEqual(x.actions,[])
    def classified(self,status='capacity_stop'):
        error=RuntimeError('trusted local refusal')
        error.known_stage_stop={'version':'known-stage-stop@v1','status':status,'reason':'Known settled local refusal; no unresolved external effect'}
        return error
    def test_known_capacity_and_technical_stop_are_terminal_and_inert(self):
        for status in ('capacity_stop','technical_stop'):
            with self.subTest(status=status):
                x=self.fixture();calls=[]
                def refusal(_):calls.append(1);raise self.classified(status)
                x.proposal=refusal
                result=x.runner().run();self.assertEqual(result['status'],status)
                self.assertTrue(result['known_settled']);self.assertFalse(result['callback_retried'])
                before=(x.root/'cycle.jsonl').read_bytes()
                self.assertEqual(x.runner().run(),result);self.assertEqual(before,(x.root/'cycle.jsonl').read_bytes())
                self.assertEqual(calls,[1]);self.assertEqual(x.actions,[])
    def test_classified_failure_restart_after_receipt_and_result(self):
        for event in ('after_receipt','after_effect','after_result'):
            with self.subTest(event=event):
                x=self.fixture();calls=[]
                def refusal(_):calls.append(1);raise self.classified()
                x.proposal=refusal
                self.pause(x,event,'proposal_0')
                self.assertEqual(x.runner().run()['status'],'capacity_stop')
                self.assertEqual(calls,[1]);self.assertEqual(x.actions,[])
    def test_untrusted_classification_and_legacy_exception_stay_unresolved(self):
        for mode in ('malformed','unknown_status','legacy'):
            with self.subTest(mode=mode):
                x=self.fixture();calls=[];error=self.classified()
                if mode=='malformed':error.known_stage_stop['extra']=True
                if mode=='unknown_status':error.known_stage_stop['status']='unknown_effect'
                if mode=='legacy':del x.a['stage_receipts']
                def refusal(_):calls.append(1);raise error
                x.proposal=refusal
                with self.assertRaises(RuntimeError):x.runner().run()
                with self.assertRaises(m.d.Blocked):x.runner().run()
                self.assertEqual(calls,[1]);self.assertEqual(x.actions,[])
    def test_profiles_rejected_without_browser_scope(self):
        x=self.fixture();del x.a['local_source_profile']
        with self.assertRaisesRegex(ValueError,'prospective'):x.runner()

if __name__=='__main__':unittest.main()
