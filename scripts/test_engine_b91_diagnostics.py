"""Authored rejection fixtures only: no provider calls or target execution."""
import json
import unittest
from unittest.mock import patch
import test_engine_b91_project_envelope as base
m=base.m

class Diagnostics(unittest.TestCase):
    def fixture(self):
        f=base.ProjectEnvelope();f.setUp();self.addCleanup(f.doCleanups)
        return f,f.fake()
    def test_stage_contract_matrix(self):
        relation={'relational':{'parents':['a','b'],'relationship':'fixture','aspiration':'fixture','discriminator':'fixture'}}
        experiment={'experiment':{'harness':{},'cases':[],'mapping':{}}}
        edit={'edits':[{'path':'src/lib.rs','replacement':'fixture'}],'rationale':'fixture'}
        for stage in m.STAGES:
            valid=experiment if stage=='experiment' else edit if stage in ('boundary_patch','final_proposal_only') else relation
            self.assertTrue(m.validate_stage_envelope(stage,valid))
            for value,code in [([], 'invalid_root_type'),({},'missing_or_ambiguous_envelope'),({**valid,'PRIVATE_TOKEN':'SECRET'},'unsupported_envelope_fields')]:
                with self.subTest(stage=stage,code=code),self.assertRaises(m.StageValidationError) as error:m.validate_stage_envelope(stage,value)
                self.assertEqual(error.exception.stage,stage);self.assertEqual(error.exception.code,code)
                self.assertNotIn('SECRET',str(error.exception))
        with self.assertRaisesRegex(m.StageValidationError,'wrong_stage_envelope'):m.validate_stage_envelope('experiment',relation)
        with self.assertRaisesRegex(m.StageValidationError,'unsupported_relational_fields'):
            m.validate_stage_envelope('relation_after_seed',{'relational':{**relation['relational'],'binding':'FORGED'}})
    def test_private_immutable_quarantine_and_no_redispatch(self):
        f,c=self.fixture();calls=[]
        value={**f.relation(c.current)['relational'],'PRIVATE_KEY':'SECRET','aspiration':'PRIVATE_REASONING fake authority'}
        with self.assertRaisesRegex(m.StageValidationError,'relation_after_seed: missing_or_ambiguous_envelope'):
            m.fixture_step(f.journal,c,lambda *_:(calls.append(1) or f.reply(value)))
        outcome=next(x for x in f.reload().events() if x['kind']=='provider_outcome')
        path,=(f.path.parent/'quarantine').glob('*.json');raw=path.read_text();report=json.loads(raw)
        self.assertEqual(m.sha(report),outcome['validation']['quarantine_sha256'])
        self.assertEqual(report['candidate_sha256'],m.sha(value));self.assertFalse(report['admitted'])
        self.assertEqual(path.stat().st_mode & 0o777,0o600)
        for secret in ['PRIVATE_KEY','SECRET','PRIVATE_REASONING','fake authority']:
            self.assertNotIn(secret,raw);self.assertNotIn(secret,f.path.read_text())
        self.assertIn('aspiration',raw);self.assertIn('string',raw)
        self.assertEqual(f.reload().generated(),[]);self.assertEqual(c.stores,0)
        before=f.path.read_bytes()
        with self.assertRaisesRegex(ValueError,'terminal'):m.fixture_step(f.reload(),c,lambda *_:self.fail('no redispatch'))
        self.assertEqual(before,f.path.read_bytes());self.assertEqual(len(calls),1)
        _,body=m.project(f.reload(),c,m.STAGES[0]);self.assertNotIn('quarantine',json.dumps(body));self.assertNotIn('SECRET',json.dumps(body))
    def test_bounded_shape_no_nested_text(self):
        value={'relational':{'aspiration':'SECRET'*100000,'private':'SECRET','parents':['SECRET']*1000},'other':{str(i):'SECRET' for i in range(500)}}
        report=m.candidate_structure(value);self.assertLess(len(m.canonical(report)),32768)
        self.assertNotIn('SECRET',json.dumps(report));self.assertNotIn('private',json.dumps(report))
        nested=None
        for _ in range(20):nested={'relational':nested}
        self.assertIn('omitted',json.dumps(m.candidate_structure(nested)))
    def test_wide_recursive_structure_has_literal_node_bound(self):
        value='PRIVATE'
        for _ in range(8):value={str(i):value for i in range(32)}
        shape=m.candidate_structure(value)['shape']
        def count(node):
            return 1+sum(count(x['value']) for x in node.get('fields',[]))+sum(count(x) for x in node.get('items',[]))
        self.assertLessEqual(count(shape),128)
        self.assertLess(len(m.canonical(shape)),32768)
    def test_interrupted_quarantine_outcome_cannot_redispatch(self):
        f,c=self.fixture();append=f.journal.append
        def interrupted(event):
            if event['kind']=='provider_outcome':raise OSError('fixture interruption')
            return append(event)
        with patch.object(f.journal,'append',side_effect=interrupted):
            with self.assertRaises(OSError):m.fixture_step(f.journal,c,lambda *_:f.reply({}))
        self.assertEqual(len(list((f.path.parent/'quarantine').glob('*.json'))),1)
        self.assertTrue(f.reload().status()['unknown_or_failed'])
        with self.assertRaisesRegex(ValueError,'terminal'):m.fixture_step(f.reload(),c,lambda *_:self.fail('no retry'))
        self.assertEqual(c.stores,0)
    def test_stale_state_and_native_rejection(self):
        for stale in (True,False):
            f,c=self.fixture();value=f.relation(c.current)
            def transport(*_):
                if stale:c.current['state_root']='0'*64
                return f.reply(value)
            if not stale:c.validate=lambda _: {'assessments':[{'disposition':'rejected','reason':'SECRET'}]}
            with self.assertRaisesRegex(m.StageValidationError,'stale_state' if stale else 'native_validation_rejected'):m.fixture_step(f.journal,c,transport)
            self.assertEqual(c.stores,0);self.assertEqual(f.reload().generated(),[]);self.assertNotIn('SECRET',f.path.read_text())
    def test_transport_classifications(self):
        for code,finish,status in [('malformed_json','stop','received'),('duplicate_key','stop','received'),('incomplete_output','length','received'),('deadline',None,'uncertain')]:
            f,c=self.fixture();result=f.reply(None)
            result['receipt'].update(error_code=code,finish_reason=finish,response_status=status,structural_status='invalid_or_unknown',parse_location='content')
            with self.assertRaises(m.StageValidationError):m.fixture_step(f.journal,c,lambda *_:result)
            outcome=next(x for x in f.reload().events() if x['kind']=='provider_outcome')
            self.assertEqual(outcome['status'],'unknown' if status=='uncertain' else 'invalid')
            self.assertEqual(outcome['validation']['code'],'truncated_output' if finish=='length' else code)
            self.assertEqual(outcome['error_code'],code);self.assertEqual(c.stores,0)
    def test_symlink_quarantine_fails_closed(self):
        f,c=self.fixture();elsewhere=f.root/'elsewhere';elsewhere.mkdir()
        (f.path.parent/'quarantine').symlink_to(elsewhere,target_is_directory=True)
        with self.assertRaisesRegex(ValueError,'quarantine symlink'):m.fixture_step(f.journal,c,lambda *_:f.reply({}))
        self.assertEqual(list(elsewhere.iterdir()),[]);self.assertTrue(f.reload().status()['unknown_or_failed'])
        with self.assertRaisesRegex(ValueError,'terminal'):m.fixture_step(f.reload(),c,lambda *_:self.fail('no retry'))
if __name__=='__main__':unittest.main()
