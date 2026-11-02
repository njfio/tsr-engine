"""Deterministic schema tests, not model-generation or social outcome evidence."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest

ROOT=Path(__file__).resolve().parents[1]
sp=importlib.util.spec_from_file_location('directions',ROOT/'scripts/engine-directions.py')
d=importlib.util.module_from_spec(sp);sp.loader.exec_module(d)

class ArchiveBoundary(unittest.TestCase):
    def state(self):
        spec=json.loads((ROOT/'docs/engine-synthetic-archive-inputs.json').read_text())['spec']
        return {'state_root':'fixture','archive':{'world':spec['archive'],'events':[]},
                'notebook':{'known':[],'claims':[],'title':''},'available_notes':[],
                'standing_desires':spec['wants'],'obligation_ids':['a','b'],
                'selected_desires':[0,1],'obligation_roots':[0,1],'observation':[0,0],
                'minimum_directions':1}
    def action(self):
        return {'operation':'archive','event':{'kind':'register','challenge':'access','revision':'hours-v1'}}
    def test_archive_schema_is_disjoint_and_preserves_intent(self):
        s=self.state();before=copy.deepcopy(s)
        schema=d.possibility_schema(s)
        variants=schema['anyOf'][0]['properties']['directions']['items']['anyOf']
        self.assertEqual(len(variants),1)
        self.assertEqual(variants[0]['properties']['action']['properties']['operation']['const'],'archive')
        self.assertNotIn('continuation',variants[0]['properties'])
        self.assertEqual(s,before)
        self.assertEqual(d.action_contract(s)['domain'],'synthetic_archive@v1')
    def test_stale_injection_duplicate_and_cross_adapter_rejected(self):
        s=self.state();action=self.action()
        offer={'directions':[{'purpose':'Fixture','action':action}]}
        d.validate_action_contract(s,offer)
        for change in [lambda a:a['event'].update(revision='old'),lambda a:a['event'].update(erase=True),
                       lambda a:a.update(operation='compose'),lambda a:a['event'].update(kind='resolve')]:
            bad=copy.deepcopy(offer);change(bad['directions'][0]['action'])
            with self.assertRaises(ValueError):d.validate_action_contract(s,bad)
        s['archive']['events']=[action['event']]
        with self.assertRaises(ValueError):d.validate_action_contract(s,offer)
        s=self.state();s.pop('archive')
        with self.assertRaises(ValueError):d.validate_action_contract(s,offer)
    def test_schema_excludes_unsupported_response_and_withdrawal(self):
        s=self.state();s['archive']['world']['responses']=[]
        schema=json.dumps(d.possibility_schema(s))
        self.assertNotIn('record_response',schema)
        self.assertNotIn('withdraw',schema)
        with self.assertRaises(ValueError):
            d.validate_action_contract(s,{'directions':[{'action':{'operation':'archive','event':{'kind':'withdraw','challenge':'access','revision':'hours-v1','evidence':'invented'}}}]})

    def test_external_preparation_allowlist_and_drift(self):
        sp=importlib.util.spec_from_file_location('prepare',ROOT/'scripts/engine-archive-prepare.py')
        prepare=importlib.util.module_from_spec(sp);sp.loader.exec_module(prepare)
        frozen=json.loads((ROOT/'docs/engine-synthetic-archive-inputs.json').read_text())
        s=self.state();s['state_root']='a'*64;s['observation_revision']='b'*64
        s['obligation_ids']=['c'*64,'d'*64];s['observation']=[1/6,0.5]
        s['archive']['events']=[self.action()['event']]
        proposal=copy.deepcopy(frozen['aspiration']);proposal.update(parents=s['obligation_ids'],revises=None,catalysts=[])
        s['relational_guidance']=[{'applicable':True,'observation_revision':s['observation_revision'],'proposal':proposal}]
        clean=prepare.prepare(s)
        s['private_review']='DO NOT EXPORT';s['archive']['private']='DO NOT EXPORT';s['relational_guidance'][0]['review']='DO NOT EXPORT'
        self.assertEqual(prepare.prepare(s),clean)
        self.assertNotIn('DO NOT EXPORT',json.dumps(clean))
        self.assertFalse(clean['dispatch_enabled'])
        for mutate in [lambda q:q['archive']['events'].clear(),
                       lambda q:q['standing_desires'][0].update(text='changed'),
                       lambda q:q['archive']['world']['responses'][0].update(addresses=True),
                       lambda q:q['relational_guidance'][0]['proposal'].update(aspiration='changed'),
                       lambda q:q.update(state_root='private path')]:
            bad=copy.deepcopy(s);mutate(bad)
            with self.assertRaises(ValueError):prepare.prepare(bad)

    def test_response_requires_prior_matching_route_not_another_proposal(self):
        s=self.state();s['archive']['events']=[self.action()['event']]
        def offer(event):return {'directions':[{'purpose':'DETERMINISTIC boundary test','action':{'operation':'archive','event':event}}]}
        response={'kind':'record_response','challenge':'access','revision':'hours-v1','response':'access-review'}
        route={'kind':'route','challenge':'access','revision':'hours-v1','actor':'night-reader'}
        with self.assertRaises(ValueError):d.validate_action_contract(s,offer(response))
        paired={'directions':offer(route)['directions']+offer(response)['directions']}
        with self.assertRaises(ValueError):d.validate_action_contract(s,paired)
        empty=self.state()
        with self.assertRaises(ValueError):d.validate_action_contract(empty,offer(route))
        with self.assertRaises(ValueError):d.validate_action_contract(empty,offer(response))
        d.validate_action_contract(s,offer(route))
        s['archive']['events'].append({**route,'actor':'caretaker'})
        with self.assertRaises(ValueError):d.validate_action_contract(s,offer(response))
        s['archive']['events'].append(route)
        d.validate_action_contract(s,offer(response))
        for change in [{'response':'invented'}, {'challenge':'meaning','revision':'translation-v1'}]:
            with self.assertRaises(ValueError):d.validate_action_contract(s,offer({**response,**change}))
        with self.assertRaises(ValueError):d.validate_action_contract(s,offer({**route,'actor':'outsider'}))
        s['archive']['events'].append({'kind':'withdraw','challenge':'access','revision':'hours-v1','evidence':'fixture'})
        with self.assertRaises(ValueError):d.validate_action_contract(s,offer(response))

    def test_prerequisites_are_current_facts_not_proposed_actions(self):
        s=self.state();s['observation_revision']='fixture-version'
        first=d.action_contract(s)['current_prerequisites']
        self.assertFalse(first['facts']['challenges'][0]['registered'])
        self.assertFalse(first['facts']['challenges'][0]['response_prerequisites'][1]['respondent_already_routed'])
        s['archive']['events']=[self.action()['event']]
        s['archive']['prerequisites']=first['facts']
        with self.assertRaises(ValueError):d.action_contract(s)
        s['archive'].pop('prerequisites')
        second=d.action_contract(s)['current_prerequisites']
        self.assertTrue(second['facts']['challenges'][0]['registered'])
        self.assertFalse(second['facts']['challenges'][0]['response_prerequisites'][1]['respondent_already_routed'])
        s['archive']['events'].append({'kind':'route','challenge':'access','revision':'hours-v1','actor':'caretaker'})
        third=d.action_contract(s)['current_prerequisites']
        self.assertTrue(third['facts']['challenges'][0]['response_prerequisites'][0]['respondent_already_routed'])
        self.assertFalse(third['facts']['challenges'][0]['response_prerequisites'][1]['respondent_already_routed'])
        self.assertEqual(third['state_root'],s['state_root'])
        self.assertNotIn('action',third['facts'])

    def test_followup_uses_pinned_current_revision_without_private_fields(self):
        import hashlib
        sp=importlib.util.spec_from_file_location('prepare','scripts/engine-archive-prepare.py')
        prepare=importlib.util.module_from_spec(sp);sp.loader.exec_module(prepare)
        frozen=json.loads((ROOT/'docs/engine-synthetic-archive-inputs.json').read_text())
        s=self.state();s.update(state_root='a'*64,observation_revision='b'*64,obligation_ids=['c'*64,'d'*64])
        origin=copy.deepcopy(frozen['aspiration']);origin['parents']=s['obligation_ids']
        revised=copy.deepcopy(origin);revised['aspiration']='SYNTHETIC TEST revision; no discovery claim.'
        s['relational_request']={'history':[{'proposal':origin}]}
        s['relational_guidance']=[{'applicable':True,'observation_revision':s['observation_revision'],
                                 'identity':'e'*64,'revision':'f'*64,'proposal':revised,'private':'DO NOT EXPORT'}]
        s['private_review']='DO NOT EXPORT'
        raw=json.dumps(s).encode();digest=hashlib.sha256(raw).hexdigest()
        prepared=prepare.prepare_followup(s,raw,digest)
        self.assertNotIn('DO NOT EXPORT',json.dumps(prepared))
        context=json.loads(prepared['payload']['messages'][1]['content'])['context']
        self.assertEqual(context['relational_guidance']['aspiration'],revised['aspiration'])
        self.assertEqual(context['relational_guidance']['revision'],'f'*64)
        with self.assertRaises(ValueError):prepare.prepare_followup(s,raw,'0'*64)
        changed=copy.deepcopy(s);changed['state_root']='1'*64
        with self.assertRaises(ValueError):prepare.prepare_followup(changed,raw,digest)
        changed=copy.deepcopy(s);changed['relational_guidance'][0]['observation_revision']='1'*64
        bad=json.dumps(changed).encode()
        with self.assertRaises(ValueError):prepare.prepare_followup(changed,bad,hashlib.sha256(bad).hexdigest())

    def test_frozen_action_identity_and_forecast_contract(self):
        s=self.state();proposal={'state_root':'fixture','directions':[{'purpose':'Fixture','action':self.action()}]}
        d.validate_possibilities(s,proposal,{'prior_rejected_offers':[]})
        evidence={'state_root':'fixture','proposal_digest':d.proposal_digest(proposal),
                  'assessments':[{'proposal_id':d.proposal_digest(proposal['directions'][0]),'forecasts':{'a':1/6,'b':0.5},'supports':['a','b']}]}
        result=d.complete_assessment(s,proposal,evidence)
        self.assertEqual(result['directions'][0]['action'],proposal['directions'][0]['action'])
        normalized=d.normalize_forecasts(s,result)
        self.assertEqual(normalized['directions'][0]['anticipated'],[1/6,0.5])
        changed=copy.deepcopy(proposal);changed['directions'][0]['action']['event']['challenge']='meaning'
        self.assertNotEqual(d.possibility_identity(changed['directions'][0]),d.possibility_identity(proposal['directions'][0]))
        with self.assertRaises(ValueError):d.complete_assessment(s,changed,evidence)

if __name__=='__main__':unittest.main()
