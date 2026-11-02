"""Offline projection fixtures; no truth inference, provider call or live-state mutation."""
import copy
import json
import unittest
from pathlib import Path
import test_engine_synthetic_cycle as h

class EvidenceOntology(unittest.TestCase):
    def setUp(self):
        self.d=h.module('ontology','scripts/engine-directions.py');self.c=self.d.load_synthetic_cycle()
    def state(self,scenario):
        p=self.c.fixture(scenario)
        return {'state_root':'a'*64,'observation_revision':'b'*64,
                'archive':{'world':copy.deepcopy(p['spec']['archive']),'events':copy.deepcopy(p.get('initial_events',[]))}}
    def test_visible_negative_is_evidence_before_registration(self):
        s=self.state('garden');s['archive']['events']=[];before=copy.deepcopy(s)
        e=self.d.archive.response_evidence(s)
        self.assertIn('trial-negative',e['content_visible']);self.assertIn('trial-negative',e['available_unrecorded'])
        self.assertEqual(e['recorded'],[]);self.assertEqual(e['assertion_trust'],'unverified')
        self.assertEqual(e['verified_world_truth'],'unknown');self.assertEqual(s,before)
    def test_unavailable_content_not_invented_and_recorded_not_truth(self):
        s=self.state('garden');s['archive']['world']['responses'][1].pop('text')
        e=self.d.archive.response_evidence(s)
        self.assertEqual(e['content_unavailable'],['trial-negative'])
        self.assertNotIn('trial-negative',e['available_unrecorded'])
        s['archive']['events'].append({'kind':'record_response','challenge':'variation','revision':'planning-v1','response':'trial-positive'})
        e=self.d.archive.response_evidence(s)
        self.assertEqual(e['recorded'],[{'response_id':'trial-positive','event_index':4}])
        self.assertEqual(e['assertion_trust'],'unverified');self.assertEqual(e['verified_world_truth'],'unknown')
        self.assertEqual(e['content_unavailable'],['trial-negative'])
    def test_stale_or_fabricated_projection_rejected(self):
        s=self.state('garden');e=self.d.archive.response_evidence(s)
        s['archive']['response_evidence']=e
        self.assertEqual(self.d.archive.response_evidence(s),e)
        for change in ('state','revision','trust','content'):
            bad=copy.deepcopy(s)
            if change=='state':bad['state_root']='c'*64
            elif change=='revision':bad['observation_revision']='c'*64
            elif change=='trust':bad['archive']['response_evidence']['assertion_trust']='verified'
            else:bad['archive']['world']['responses'][1].pop('text')
            with self.subTest(change=change),self.assertRaises(ValueError):self.d.archive.response_evidence(bad)
        bad=self.state('garden');bad['archive']['events'][0]['revision']='stale'
        with self.assertRaises(ValueError):self.d.archive.response_evidence(bad)
    def test_shared_roles_and_historical_revision_context_under_payload_bound(self):
        for scenario in ('archive','workshop','garden'):
            e=self.d.archive.response_evidence(self.state(scenario))
            self.assertEqual(e['contract'],'synthetic-response-evidence@v2')
            self.assertEqual(e['verified_world_truth'],'unknown')
        checked=0
        for folder in ('workshop-ready','garden-trial2'):
            directory=h.ROOT/'target'/folder/'run/synthetic-external'
            if not directory.exists():continue
            for n in range(1,6):
                body=json.loads((directory/f'request-{n}.json').read_text());original=copy.deepcopy(body)
                user=json.loads(body['messages'][1]['content']);ctx=user['context']
                state={'state_root':ctx['state_root'],'observation_revision':ctx['archive']['response_evidence']['observation_revision'],'archive':copy.deepcopy(ctx['archive'])}
                state['archive'].pop('response_evidence')
                ctx['archive']['response_evidence']=self.d.archive.response_evidence(state)
                relation='review' in ctx
                ctx['epistemic_roles']=self.d.archive.epistemic_roles(relation)
                if not relation:
                    phase='assessment' if 'frozen_possibilities' in ctx else 'possibilities'
                    body['messages'][0]['content']=self.d.archive.instructions(phase)+' Return only schema-bound JSON; each text at most 512 UTF-8 bytes.'
                body['messages'][1]['content']=json.dumps(user,sort_keys=True)
                self.assertLessEqual(len(json.dumps(body).encode()),16384)
                self.assertNotIn('Hypotheses about unrecorded content must remain hypothetical',json.dumps(body))
                self.assertEqual(json.loads((directory/f'request-{n}.json').read_text()),original)
                if relation:
                    self.assertEqual(ctx['epistemic_roles']['report_assertions'],'archive.world.responses')
                checked+=1
        if not checked:self.skipTest('historical local payloads unavailable; never regenerate')

if __name__=='__main__':unittest.main()
