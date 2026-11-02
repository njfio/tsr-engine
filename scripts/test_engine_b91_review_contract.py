"""Authored review-boundary fixtures only; no inference or target execution."""
import copy,json,tempfile,unittest
from pathlib import Path
from types import SimpleNamespace
import test_engine_b91_driver as base
m=base.m
class ReviewContract(unittest.TestCase):
    def check(self,field,value):
        calls=[];contexts=[]
        review={'author':'author','reviewer':'reviewer','generator':'generator','review':'finite evidence'}
        review[field]=value;original=copy.deepcopy(review)
        def reviewer(kind,ctx):
            contexts.append(ctx)
            return {'approved':True,'state_root':ctx['state_root'],'subject_sha256':ctx['subject_sha256'],'review':review}
        owner=SimpleNamespace(read=lambda:([],{},{}),fixture=True,reviewer=reviewer,
            op=lambda name,fn:calls.append(name) or fn())
        c=SimpleNamespace(request=lambda:{'state_root':'a'*64,'coding':{'cases':[{'id':'fixture_existing'}]}},cli=lambda *args:{'snapshot':'b'*64})
        try:result=m.Driver.review(owner,c,'experiment_readmission',{'fixture':True},'readmission')
        except ValueError:result=None
        self.assertEqual(review,original)
        self.assertIn('512 UTF-8',contexts[0]['review_contract']['text'])
        return result,calls
    def test_valid_boundaries(self):
        for text in ('x'*512,'é'*256):
            result,calls=self.check('review',text);self.assertIsNotNone(result);self.assertEqual(calls,['review_readmission'])
    def test_invalid_fields_stop_before_durable_review(self):
        for field in ('author','reviewer','generator','review'):
            for value in ('x'*513,'é'*257,' ',None,1,[],{}):
                with self.subTest(field=field,value=type(value).__name__):
                    result,calls=self.check(field,value);self.assertIsNone(result);self.assertEqual(calls,[])
        self.assertEqual(self.check('author','reviewer'),(None,[]))
    def test_independent_semantic_rejection_never_records_approval(self):
        calls=[]
        def reviewer(kind,ctx):
            return {'approved':False,'state_root':ctx['state_root'],'subject_sha256':ctx['subject_sha256'],
                'review':{'author':'author','reviewer':'independent reviewer','generator':'generator',
                          'review':'Fixture rejection: case failure does not establish that a later assertion ran.'}}
        owner=SimpleNamespace(read=lambda:([],{},{}),fixture=True,reviewer=reviewer,
            op=lambda name,fn:calls.append(name) or fn())
        c=SimpleNamespace(request=lambda:{'state_root':'a'*64,'coding':{'cases':[{'id':'fixture_existing'}]}},cli=lambda *args:{'snapshot':'b'*64})
        for _ in range(2):
            with self.assertRaisesRegex(m.Blocked,'independent review rejected/stale'):
                m.Driver.review(owner,c,'relational',{'fixture':True},'relation_after_experiment')
        self.assertEqual(calls,[])
    def test_file_request_publishes_bound_and_no_implicit_approval(self):
        with tempfile.TemporaryDirectory() as tmp:
            r=m.FileReviewer(tmp);ctx={'review_contract':{'text':'max 512 UTF-8 bytes'},'fixture_only':True}
            self.assertIsNone(r('fixture',ctx))
            files=list(Path(tmp).glob('*.request.json'));self.assertEqual(len(files),1)
            self.assertEqual(json.loads(files[0].read_text()),ctx)
            self.assertIsNone(m.FileReviewer(tmp)('fixture',ctx))
            self.assertFalse(list(Path(tmp).glob('*.review.json')))
if __name__=='__main__':unittest.main()
