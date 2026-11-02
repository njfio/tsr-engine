"""Evaluator-authored scenario preflight, not model emergence or social evidence."""
import copy
import unittest
import test_engine_synthetic_cycle as h

class WorkshopDesign(unittest.TestCase):
    def test_frozen_setup_and_two_response_paths(self):
        d=h.module('workshop_design','scripts/engine-directions.py');c=d.load_synthetic_cycle()
        plan=c.read(h.ROOT/'docs/engine-synthetic-workshop-plan.json')
        world=plan['spec']['archive'];past=[]
        self.assertNotIn('aspiration',plan)
        self.assertEqual(plan['bounds']['calls'],5);self.assertEqual(plan['bounds']['actions'],1)
        for event in plan['initial_events']:
            d.archive.validate({'archive':{'world':world,'events':past}},
                {'directions':[{'action':{'operation':'archive','event':event}}]})
            past.append(event)
        before=copy.deepcopy(past)
        for response in world['responses']:
            event={'kind':'record_response','challenge':response['challenge'],
                   'revision':response['revision'],'response':response['id']}
            d.archive.validate({'archive':{'world':world,'events':past}},
                {'directions':[{'action':{'operation':'archive','event':event}}]})
            state={'state_root':'a'*64,'observation_revision':'b'*64,
                   'archive':{'world':world,'events':past+[event]}}
            facts=d.archive.response_evidence(state)
            self.assertEqual(facts['recorded'],[{'response_id':response['id'],'event_index':4}])
            self.assertEqual(len(facts['available_unrecorded']),1)
        self.assertEqual(past,before)
        # Preparation must not silently authorize the new scenario for external export.
        with self.assertRaises(ValueError):
            c.scope({'standing_desires':plan['spec']['wants'],'archive':{'world':world,'events':past}},d)

if __name__=='__main__':unittest.main()
