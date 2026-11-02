"""Offline authored fixtures: bounded final abstention is not task fulfillment."""
import importlib.util
from pathlib import Path
from unittest.mock import patch
import unittest
import test_engine_b91_project_envelope as base
m=base.m
spec=importlib.util.spec_from_file_location('idle_driver',Path(__file__).with_name('engine-b91-driver.py'))
d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d)

class FinalAbstention(unittest.TestCase):
    def test_final_contract_and_negative_shapes(self):
        value={'abstain':{'reason':'No useful further proposal on the available evidence.'}}
        self.assertEqual(m.validate_stage_envelope('final_proposal_only',value),'abstain')
        self.assertIn('abstain',m.response_contract('final_proposal_only')['payload_location'])
        self.assertIn('acceptance review is still pending',m.response_contract('final_proposal_only')['limits'])
        for stage in m.STAGES[:-1]:
            with self.assertRaises(ValueError):m.validate_stage_envelope(stage,value)
        for bad in ({'abstain':{'reason':''}},{'abstain':{'reason':' '*3}},
                    {'abstain':{'reason':'界'*171}}, {'abstain':{'reason':'x','accepted':True}},
                    dict(value,edits=[]),dict(value,authority=True)):
            with self.assertRaises(ValueError):m.validate_stage_envelope('final_proposal_only',bad)
    def test_real_assembler_journal_and_driver_replay_without_native_candidate(self):
        case=base.ProjectEnvelope('test_authored_six_stage_fixture_restarts_and_cannot_skip_observed_returns')
        case.setUp();self.addCleanup(case.doCleanups)
        real_step=m.fixture_step;observed=[]
        def step(journal,controller,transport,**kwargs):
            if journal.status()['next_stage']=='final_proposal_only':
                original_transport=transport
                def idle(body,timeout):
                    # Count original fixture transport, but replace only authored test output.
                    original_transport(body,timeout)
                    return case.reply({'abstain':{'reason':'No useful further proposal; independent acceptance remains pending.'}})
                with patch.object(controller,'validate',side_effect=AssertionError('no native candidate')):
                    result=real_step(journal,controller,idle,**kwargs)
                observed.append(result);return result
            return real_step(journal,controller,transport,**kwargs)
        with patch.object(m,'fixture_step',side_effect=step):
            case.test_authored_six_stage_fixture_restarts_and_cannot_skip_observed_returns()
        journal=case.reload();events=journal.events()
        self.assertEqual(len([e for e in events if e['kind']=='final_abstention']),1)
        self.assertEqual(len(journal.generated()),5)
        self.assertEqual(journal.status()['unapplied_response'],[])
        driver=object.__new__(d.Driver)
        driver.read=lambda:([],None,{})
        driver.check_resources=lambda:None
        driver.journal=lambda:case.reload()
        driver.op=lambda name,fn:fn()
        replay=d.Driver.generate(driver,None,'final_proposal_only')
        self.assertEqual(replay,observed[0]);self.assertFalse(replay['accepted']);self.assertFalse(replay['executed'])
        self.assertNotIn('proposals',replay['response'])
        self.assertEqual(case.reload().events(),events)

if __name__=='__main__':unittest.main()
