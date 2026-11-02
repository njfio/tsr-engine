"""Offline representation invariants; authored fixtures, no model or target execution."""
import copy
import json
import unittest
from unittest.mock import patch
import test_engine_b91_project_envelope as envelope
m=envelope.m

class DecisionContext(unittest.TestCase):
    def setUp(self):
        self.fixture=envelope.ProjectEnvelope();self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)

    def full_context(self,stage='relation_after_verification'):
        fake=self.fixture.fake()
        returns=fake.current['coding']['relational']['inputs']['returns']
        for o in returns[0]['observations']:o['exit']=101
        later=copy.deepcopy(returns[0]);later.update(receipt='c'*64,commitment='d'*64,verdict='timeout')
        returns.append(later)
        with patch.object(m,'decision_context',side_effect=lambda x:x):
            _,body=m.project(self.fixture.journal,fake,stage)
        return json.loads(body['messages'][1]['content'])

    def test_observation_ids_indexes_unknown_and_criteria_preserved(self):
        full=self.full_context();before=copy.deepcopy(full);view=m.decision_context(full)
        self.assertEqual(full,before)
        self.assertEqual(view['context_projection']['full_context_sha256'],m.sha(full))
        for key in ('parents','source','constraints','response_contract','payload_contract',
                    'relational_contract','return_basis_contract','return_evidence','bound_to'):
            self.assertEqual(view[key],full[key])
        self.assertEqual([c['id'] for c in view['return_evidence']['cases']],['0','1','2','3'])
        self.assertEqual([c['outcome'] for c in view['return_evidence']['cases']],['passed','failed','unknown','unknown'])
        for fact in view['return_evidence']['cases']:
            i=fact['return_index']
            self.assertEqual(view['observed_returns'][i]['receipt'],full['observed_returns'][i]['receipt'])
            self.assertEqual(view['observed_returns'][i]['commitment'],full['observed_returns'][i]['commitment'])
        self.assertIn('assertion execution',view['return_evidence']['granularity'])

    def test_experiment_reasoning_limits_source_and_history_survive(self):
        full=self.full_context()
        proposal={'parents':['a'*64,'b'*64],'relationship':'A relationship, not summed scores',
                  'aspiration':'Authored fixture meaning','development':{'return_reveals':'A failed case',
                  'unresolved':'No universal proof','proposed_development':None}}
        full['relational_history']=[{'admission':'e'*64,'binding':'f'*64,'proposal':proposal,
                                    'current':True,'withdrawn':False,'superseded':False}]
        full['supplemental_returns']=[{'admission':'1'*64,'receipt':'2'*64,'snapshot':'3'*64,
            'checked_snapshot':'3'*64,'source_episode_root':'4'*64,'bundle_digest':'5'*64,
            'generated_origin':{'proposal':'6'*64},'mapped_cases':{'existing_behaviour':['probe']},
            'cases':{'probe':'counterexample'},'findings':{'existing_behaviour':'not_met'},
            'generated_check':{'harness':{'tests/tsr_supplemental_probe.rs':'authored source'},
                'cases':[{'id':'probe','args':[],'expected_stdout':''}],
                'mapping':{'existing_behaviour':{'cases':['probe'],'rationale':'Why this challenges a hypothesis',
                                                'limits':'Only one finite example'}}},'limits':'finite only'}]
        before=copy.deepcopy(full);view=m.decision_context(full)
        self.assertEqual(full,before)
        self.assertEqual(view['relational_history'][0]['proposal'],proposal)
        self.assertFalse(view['relational_history'][0]['withdrawn'])
        item=view['supplemental_returns'][0];old=full['supplemental_returns'][0]
        for k in ('admission','receipt','snapshot','bundle_digest','findings','limits'):
            self.assertEqual(item[k],old[k])
        for k in ('harness','mapping'):self.assertEqual(item['generated_check'][k],old['generated_check'][k])
        self.assertEqual([dict(id=i,**item['generated_check']['case_defaults'])
            for i in item['generated_check']['case_ids']],old['generated_check']['cases'])
        full['supplemental_returns'][0]['generated_check']['cases'][0]['args']=['unexpected']
        with self.assertRaisesRegex(ValueError,'nonempty experiment'):m.decision_context(full)
        full=copy.deepcopy(before)
        full['supplemental_returns'][0]['generated_check']['cases'][0]['unexpected']='do not discard'
        with self.assertRaisesRegex(ValueError,'nonempty experiment'):m.decision_context(full)

    def test_valid_sized_history_growth_is_never_truncated_to_fit(self):
        full=self.full_context();history=[]
        for i in range(3):
            ident=f'{i+1:064x}';prior=None if i==0 else f'{i:064x}'
            proposal={'parents':[p['id'] for p in full['parents']],
                'relationship':'r'*500,'aspiration':'a'*500,'discriminator':'d'*500,
                'revises':prior,'catalysts':[], 'development':None if i==0 else {
                    'original_revision':f'{1:064x}','prior_revision':prior,
                    'retained_meaning':'m'*400,'return_reveals':'v'*400,
                    'unresolved':'u'*400,'proposed_development':'p'*400}}
            self.assertLessEqual(len(m.canonical(proposal)),4096)
            history.append({'admission':ident,'binding':'e'*64,'proposal':proposal,
                            'current':i==2,'withdrawn':False,'superseded':i!=2})
        # No fictional return is introduced: only representation growth is exercised.
        full['relational_history']=history[:1]
        small=m.decision_context(full)
        def body(view):return {'model':'gpt-6-luna','messages':[{'role':'user','content':m.canonical(view).decode()}]}
        m.enforce_payload_bound(body(small),'final_proposal_only')
        full['relational_history']=history
        view=m.decision_context(full)
        self.assertEqual([x['proposal'] for x in view['relational_history']], [x['proposal'] for x in history])
        with self.assertRaises(m.PayloadBoundError):m.enforce_payload_bound(body(view),'final_proposal_only')
        self.assertEqual(len(view['relational_history']),3)
        self.assertEqual(self.fixture.journal.status()['calls_consumed_or_reserved'],0)

if __name__=='__main__':unittest.main()
