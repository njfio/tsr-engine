"""Offline authored assessment fixtures; real native rejection, no model or target execution."""
import copy
import datetime as dt
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

ROOT=Path(__file__).resolve().parents[1]
def module(name,path):
    spec=importlib.util.spec_from_file_location(name,ROOT/path)
    value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value

d=module('empty_support_directions','scripts/engine-directions.py')

class EmptyArchiveSupport(unittest.TestCase):
    def test_all_neutral_normal_runner_refusal_and_fresh_process_replay(self):
        import test_engine_synthetic_cycle as h
        harness=h.SyntheticCycle()
        with tempfile.TemporaryDirectory() as tmp:
            folder=Path(tmp);driver,cycle,initial=harness.setup_run(folder,routed=True)
            (folder/'control.json').write_text(json.dumps({'no_progress':True,'empty_support':True}))
            first=harness.process(folder,'controller_refusal-3')
            self.assertEqual(first.returncode,75,first.stderr)
            replay=harness.process(folder);self.assertEqual(replay.returncode,0,replay.stderr)
            result=json.loads(replay.stdout)
            self.assertEqual(result['status'],'controller_refused')
            self.assertFalse(result['unknown_application_outcome'])
            self.assertEqual(harness.count(folder),3)
            self.assertEqual(driver.cli(folder,'direction-request')['cycles'],initial['cycles'])
            receipt=cycle.read(folder/'synthetic-external/controller-3.json')
            self.assertEqual(receipt['result']['diagnostic']['code'],'insufficient_valid_alternatives')
            self.assertEqual(receipt['storage_before'],receipt['storage_after'])
            self.assertEqual(receipt['storage_after'],cycle.storage_inventory(folder))

    def test_native_partial_batch_preserves_neutral_rejection_and_useful_choice(self):
        with tempfile.TemporaryDirectory() as tmp:
            folder=Path(tmp)/'fixture'
            prep=module('empty_support_prepare','scripts/engine-workshop-prepare.py')
            prep.prepare(folder,(dt.datetime.now(dt.timezone.utc)+dt.timedelta(hours=1)).isoformat(),'garden','gpt-6-luna')
            state=d.cli(folder,'direction-request');responses=state['archive']['world']['responses']
            def response(record):return {'operation':'archive','event':{'kind':'record_response','response':record['id'],'challenge':record['challenge'],'revision':record['revision']}}
            first={'state_root':state['state_root'],'directions':[{'purpose':'OFFLINE fixture first return only','action':response(responses[0]),'supports':[state['obligation_ids'][0]],'forecasts':dict(zip(state['obligation_ids'],[5/6,1]))}]}
            path=folder/'fixture-offer.json';path.write_text(json.dumps(d.normalize_forecasts(state,first)))
            d.cli(folder,'deliberate','--response',str(path));state=d.cli(folder,'direction-request')
            self.assertEqual(state['minimum_directions'],1)
            self.assertAlmostEqual(state['observation'][0],5/6)
            self.assertEqual(state['observation'][1],1)
            remaining=responses[1]
            proposal={'state_root':state['state_root'],'directions':[
                {'purpose':'OFFLINE fixture available negative response','action':response(remaining)},
                {'purpose':'OFFLINE fixture additional nonproductive route','action':{'operation':'archive','event':{'kind':'route','actor':'recorder','challenge':remaining['challenge'],'revision':remaining['revision']}}}]}
            d.validate_possibilities(state,proposal,{'prior_rejected_offers':[]})
            assessment={'state_root':state['state_root'],'proposal_digest':d.proposal_digest(proposal),'assessments':[
                {'proposal_id':d.proposal_digest(node),'forecasts':dict(zip(state['obligation_ids'],[1,1] if i==0 else state['observation'])),'supports':[state['obligation_ids'][0]] if i==0 else []}
                for i,node in enumerate(proposal['directions'])]}
            c=d.load_synthetic_cycle();a=c.read(folder/'admission.json')
            body,schema=c.payload(state,'assessment',proposal,a,d)
            self.assertEqual(json.loads(body['messages'][1]['content'])['response_schema'],schema)
            self.assertIn('supports: []',body['messages'][0]['content'])
            c.load_transport().validate(assessment,schema)
            complete=d.complete_assessment(state,proposal,assessment)
            self.assertEqual([x['action'] for x in complete['directions']],[x['action'] for x in proposal['directions']])
            self.assertEqual(complete['directions'][1]['supports'],[])
            # All unsupported members remain visible and cannot produce any mutation.
            for fabricated_support in (False,True):
                neutral={'state_root':state['state_root'],'directions':[copy.deepcopy(complete['directions'][1])]}
                if fabricated_support:neutral['directions'][0]['supports']=[state['obligation_ids'][0]]
                path.write_text(json.dumps(d.normalize_forecasts(state,neutral)))
                with self.assertRaises(d.ControllerRefusal):d.cli(folder,'deliberate','--response',str(path))
                self.assertEqual(d.cli(folder,'direction-request'),state)
            path.write_text(json.dumps(d.normalize_forecasts(state,complete)))
            result=d.cli(folder,'deliberate','--response',str(path))
            self.assertEqual(result['turn']['rejected_directions'][0][0],1)
            self.assertIn('selected desire support',result['turn']['rejected_directions'][0][1])
            self.assertEqual(result['turn']['offered'][result['turn']['chosen']]['direction']['action'],proposal['directions'][0]['action'])
            self.assertEqual(result['next']['archive']['events'][-1],proposal['directions'][0]['action']['event'])
            self.assertEqual(result['next']['observation'],[1,1])
            self.assertEqual(d.cli(folder,'direction-request'),result['next'])
            for mutate in (lambda x:x['assessments'].pop(),lambda x:x['assessments'][1].update(proposal_id='0'*64),lambda x:x['assessments'][1].update(supports=['unknown'])):
                bad=copy.deepcopy(assessment);mutate(bad)
                with self.assertRaises(ValueError):d.complete_assessment(state,proposal,bad)

if __name__=='__main__':unittest.main()
