"""Authored contract fixtures, not demonstrations of semantic emergence."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
import test_engine_synthetic_cycle as cycle_tests

class RelationalDevelopment(unittest.TestCase):
    def test_projection_and_schema_keep_meaning_distinct_from_measurement(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder=Path(tmp);helper=cycle_tests.SyntheticCycle();d,c,_=helper.setup_run(folder)
            r=helper.process(folder,'applied-1');self.assertEqual(r.returncode,75,r.stderr)
            q=d.cli(folder,'direction-request');q['private_review']='NEVER EXPORT'
            a=c.read(folder/'admission.json');body,schema=c.payload(q,'relational',None,a,d)
            ctx=json.loads(body['messages'][1]['content'])['context']
            self.assertNotIn('NEVER EXPORT',json.dumps(body))
            self.assertEqual(ctx['parent_meanings'],[{'id':i,'text':w['text']} for i,w in zip(q['obligation_ids'],q['standing_desires'])])
            original=q['relational_request']['history'][0];latest=q['relational_request']['existing'][0]
            self.assertEqual(ctx['existing'][0]['origin']['aspiration'],original['proposal']['aspiration'])
            self.assertEqual(ctx['existing'][0]['aspiration'],latest['proposal']['aspiration'])
            self.assertIn('measurement_limits',ctx);self.assertIn('observed_return',ctx)
            roles=ctx['epistemic_roles'];self.assertEqual(roles['hypotheses'],'existing')
            for path in roles['constraints']:
                node=ctx
                for key in path.split('.'):node=node[key]
                self.assertTrue(node)
            altered=copy.deepcopy(q)
            altered['relational_request']['existing'][0]['proposal']['aspiration']='FIXTURE: all respondents are required to speak.'
            changed,_=c.payload(altered,'relational',None,a,d)
            changed_ctx=json.loads(changed['messages'][1]['content'])['context']
            self.assertEqual(changed_ctx['epistemic_roles'],roles)
            self.assertEqual(changed_ctx['measurement_limits'],ctx['measurement_limits'])
            self.assertIn('required to speak',changed_ctx['existing'][0]['aspiration'])

            p=copy.deepcopy(latest['proposal']);p['revises']=latest['revision']
            dev={'original_revision':original['revision'],'prior_revision':latest['revision'],
                 'retained_meaning':'FIXTURE retained meaning may be questioned.',
                 'return_reveals':'FIXTURE: a registration was observed, not fulfillment.',
                 'unresolved':'FIXTURE: interpretation and contrary evidence remain open.',
                 'proposed_development':None}
            p['development']=dev
            value={k:q['relational_request'][k] for k in ('state_root','return_commitment','observation_revision','evidence_key')};value['proposals']=[p]
            transport=c.load_transport();transport.validate(value,schema);d.validate_relational_development(q,value)
            for bad in ['missing_separation','new_with_development','wrong_origin','null_with_metric_replacement']:
                v=copy.deepcopy(value)
                if bad=='missing_separation':v['proposals'][0]['development']=None
                elif bad=='new_with_development':v['proposals'][0]['revises']=None
                elif bad=='wrong_origin':v['proposals'][0]['development']['original_revision']='0'*64
                else:v['proposals'][0]['aspiration']='FIXTURE: maximize the fraction.'
                with self.subTest(bad=bad),self.assertRaises(ValueError):d.validate_relational_development(q,v)
                if bad in ('missing_separation','new_with_development'):
                    with self.assertRaises(ValueError):transport.validate(v,schema)
            for proposed in ('FIXTURE: question the old interpretation in light of contrary return.',
                             'FIXTURE: explore how contributors can revise accounts.',
                             'FIXTURE: count registered challenges.'):
                v=copy.deepcopy(value);v['proposals'][0]['aspiration']=proposed
                v['proposals'][0]['development']['proposed_development']=proposed
                # Structural validity never certifies a richer or measurement-only claim.
                transport.validate(v,schema);d.validate_relational_development(q,v)
            self.assertEqual(d.cli(folder,'direction-request')['state_root'],q['state_root'])

    def test_separated_revision_persists_and_restarts_without_authority(self):
        for contrary in (False,True):
            with self.subTest(contrary=contrary),tempfile.TemporaryDirectory() as tmp:
                folder=Path(tmp);helper=cycle_tests.SyntheticCycle();d,c,_=helper.setup_run(folder)
                self.assertEqual(helper.process(folder,'applied-3').returncode,75)
                q=d.cli(folder,'direction-request');r=q['relational_request']
                self.assertTrue(r['due'])
                old=r['existing'][0];p=copy.deepcopy(old['proposal']);p['revises']=old['revision']
                p['development']={'original_revision':r['history'][0]['revision'],'prior_revision':old['revision'],
                    'retained_meaning':'FIXTURE: preserve or question meaning without claiming truth.',
                    'return_reveals':'FIXTURE: registration alone contradicts a claim of substantive response.',
                    'unresolved':'FIXTURE: semantic fulfillment remains unknown.',
                    'proposed_development':None}
                if contrary:
                    p['aspiration']='FIXTURE: question the prior interpretation of registration as a response.'
                    p['development']['proposed_development']=p['aspiration']
                value={k:r[k] for k in ('state_root','return_commitment','observation_revision','evidence_key')};value['proposals']=[p]
                d.validate_relational_development(q,value)
                path=folder/'fixture-separated.json';path.write_text(json.dumps(value))
                d.cli(folder,'direction-relate','--response',str(path))
                final=d.cli(folder,'direction-request')
                self.assertEqual(final,d.cli(folder,'direction-request'))
                self.assertFalse(final['relational_request']['due'])
                self.assertEqual(final['relational_request']['history'][:-1],r['history'])
                self.assertEqual(final['relational_request']['history'][-1]['proposal'],p)
                for key in ('standing_desires','authority','archive','cycles','observation'):
                    self.assertEqual(final[key],q[key])
                self.assertEqual(c.context(final,d)['relational_guidance'][0]['aspiration'],p['aspiration'])
                before=c.storage_inventory(folder)
                with self.assertRaises(d.ControllerFailure):d.cli(folder,'direction-relate','--response',str(path))
                self.assertEqual(before,c.storage_inventory(folder))

class ResponseEvidence(unittest.TestCase):
    def test_available_observed_and_invalid_history(self):
        d=cycle_tests.module('evidence_driver','scripts/engine-directions.py');c=d.load_synthetic_cycle()
        world=c.fixture()['spec']['archive']
        state={'state_root':'a'*64,'observation_revision':'b'*64,
               'archive':{'world':world,'events':[]}}
        initial=d.archive.response_evidence(state)
        self.assertEqual(initial['recorded'],[])
        self.assertIn('meaning-ack',initial['available_unrecorded'])
        history=[{'kind':'register','challenge':'access','revision':'hours-v1'},
                 {'kind':'route','challenge':'access','revision':'hours-v1','actor':'night-reader'},
                 {'kind':'record_response','challenge':'access','revision':'hours-v1','response':'access-review'}]
        state['archive']['events']=history
        observed=d.archive.response_evidence(state)
        self.assertEqual(observed['recorded'],[{'response_id':'access-review','event_index':2}])
        self.assertIn('meaning-ack',observed['available_unrecorded'])
        self.assertEqual(observed['state_root'],state['state_root'])
        self.assertEqual(observed['observation_revision'],state['observation_revision'])
        self.assertEqual(d.archive.response_evidence(state),observed)
        for mutation in ('stale','mismatch','unrouted','duplicate'):
            bad=copy.deepcopy(state)
            if mutation=='stale':bad['archive']['events'][-1]['revision']='old'
            elif mutation=='mismatch':bad['archive']['events'][-1]['response']='meaning-ack'
            elif mutation=='unrouted':bad['archive']['events'].pop(1)
            else:bad['archive']['events'].append(copy.deepcopy(history[-1]))
            with self.subTest(mutation=mutation),self.assertRaises(ValueError):d.archive.response_evidence(bad)

    def test_withdrawal_retains_recording_not_applicability(self):
        d=cycle_tests.module('withdraw_driver','scripts/engine-directions.py');c=d.load_synthetic_cycle()
        world=copy.deepcopy(c.fixture()['spec']['archive'])
        world['withdrawals']=[{'id':'fixture-withdraw','challenge':'access','revision':'hours-v1','author':'newcomer'}]
        state={'state_root':'a'*64,'observation_revision':'b'*64,'archive':{'world':world,'events':[
          {'kind':'register','challenge':'access','revision':'hours-v1'},
          {'kind':'route','challenge':'access','revision':'hours-v1','actor':'night-reader'},
          {'kind':'record_response','challenge':'access','revision':'hours-v1','response':'access-review'},
          {'kind':'withdraw','challenge':'access','revision':'hours-v1','evidence':'fixture-withdraw'}]}}
        self.assertEqual(d.archive.response_evidence(state)['recorded'],[{'response_id':'access-review','event_index':2}])

if __name__=='__main__':unittest.main()
