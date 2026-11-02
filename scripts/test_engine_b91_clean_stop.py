"""Bounded status views drive acceptance; optional provider generation is never required."""
import unittest
from types import SimpleNamespace
from test_engine_b91_driver import m

class CleanStop(unittest.TestCase):
    def setup_driver(self,decision='accept'):
        self.done={'execute_patch':{'status':{'goal_completion':{'snapshot':'a'*64},'latest_execution':{'commitment':'b'*64}}}}
        self.root='c'*64;self.calls=[];self.ops=[];self.review_calls=[]
        def cli(*args):
            if args==('project','attention'):return {'state_root':self.root,'source_snapshot':'a'*64}
            if args==('project','acceptance','status'):return {'snapshot':'a'*64}
            if args==('project','status'):return self.done['execute_patch']['status']
            if args==('project-lifecycle-close',):self.calls.append('closed');return {'closed':True}
            self.fail(str(args))
        def json_cli(args,value):self.calls.append(args);return 'd'*64
        c=SimpleNamespace(cli=cli,json_cli=json_cli,request=lambda:self.fail('oversized proposal request must not be needed'),budget=lambda:{'model_calls':5})
        d=object.__new__(m.Driver);d.fixture=True
        d.read=lambda:([],{},self.done)
        def op(name,fn,**_):
            if name not in self.done:self.ops.append(name);self.done[name]=fn()
            return self.done[name]
        d.op=op
        def generate(c,stage):
            self.assertEqual(stage,'boundary_patch');return {'response':{'fixture':'patch'}}
        d.generate=generate;d.commit_response=lambda *_:None;d.execute=lambda *_:None;d.checks=lambda *_:None;d.relation=lambda *_:None
        d.journal=lambda:SimpleNamespace(status=lambda:{'calls_consumed_or_reserved':5});d.resource_summary=lambda:{}
        def reviewer(kind,context):
            self.review_calls.append(kind)
            return {'approved':True,'state_root':context['state_root'],'subject_sha256':context['subject_sha256'],'review':{'author':'author','reviewer':'reviewer','generator':'generator','review':'Independent fixture'},'native_review':{'decision':decision,'snapshot':'a'*64}}
        d.reviewer=reviewer
        ordinary=d.review
        d.review=lambda c,kind,subject,name: ordinary(c,kind,subject,name) if kind=='semantic_coverage' else {'state_root':self.root,'review':{}}
        return d,c
    def test_accepted_no_request_no_final_call(self):
        d,c=self.setup_driver();result=d.patch_tail(c,'prior',{})
        self.assertEqual(result['final_proposal_status'],'skipped_after_independent_acceptance');self.assertFalse(result['final_proposal_only'])
        self.assertIn('closed',self.calls);self.assertEqual(self.review_calls,['semantic_coverage'])
    def test_rejected_open_and_replay_preserves_unresolved(self):
        d,c=self.setup_driver('reject');result=d.patch_tail(c,'prior',{})
        self.assertEqual(result['status'],'independently_rejected_fresh_admission_required');self.assertNotIn('closed',self.calls)
        self.assertTrue(result['unresolved_desires_preserved']);before=list(self.ops);self.assertEqual(d.patch_tail(c,'prior',{}),result);self.assertEqual(self.ops,before)
    def test_no_reviewer_never_adopts(self):
        d,c=self.setup_driver();d.reviewer=lambda *_:None
        with self.assertRaises(m.Blocked):d.patch_tail(c,'prior',{})
        self.assertNotIn('adopt_final_semantic',self.done)
    def test_callback_source_drift_never_adopts(self):
        d,c=self.setup_driver();original=d.reviewer
        def drift(*args):
            result=original(*args);self.root='e'*64;return result
        d.reviewer=drift
        with self.assertRaises(m.Blocked):d.patch_tail(c,'prior',{})
        self.assertNotIn('adopt_final_semantic',self.done)
    def test_cached_stale_review_before_adoption_rejected(self):
        d,c=self.setup_driver();self.done['review_final_semantic']={'state_root':'f'*64,'native_review':{'decision':'accept','snapshot':'a'*64}}
        with self.assertRaisesRegex(m.Blocked,'stale source/state'):d.patch_tail(c,'prior',{})
        self.assertNotIn('adopt_final_semantic',self.done)
    def test_historical_optional_proposal_retained(self):
        d,c=self.setup_driver();self.done['generate_final_proposal_only']={'status':'proposal_only','fixture':'historical'}
        result=d.patch_tail(c,'prior',{});self.assertTrue(result['final_proposal_only']);self.assertEqual(result['final_proposal_status'],'historical_proposal_retained')
