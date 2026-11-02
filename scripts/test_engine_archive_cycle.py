"""Offline real-controller archive wiring; authored replies are synthetic fixtures.

Requires an already built target/debug/tsr. No provider, socket, or external
execution is used. This demonstrates persistence/orchestration, not generation.
"""
import copy
import importlib.util
import json
from pathlib import Path
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
loader = importlib.util.spec_from_file_location('archive_cycle_driver', ROOT / 'scripts/engine-directions.py')
d = importlib.util.module_from_spec(loader)
loader.loader.exec_module(d)


class Reply:
    def __init__(self, value):
        self.value = value
    def __enter__(self):
        return self
    def __exit__(self, *args):
        pass
    def read(self, limit):
        raw = json.dumps(self.value).encode()
        assert len(raw) <= limit
        return raw


class ArchiveCycle(unittest.TestCase):
    def exercise(self, restart, max_cycles=None):
        self.assertTrue((ROOT / 'target/debug/tsr').is_file(), 'Build tsr before running this integration test')
        with tempfile.TemporaryDirectory(prefix='tsr-offline-archive-') as tmp:
            directory = Path(tmp)
            spec = json.loads((ROOT / 'docs/engine-synthetic-archive-inputs.json').read_text())['spec']
            if max_cycles is not None:
                spec['max_cycles'] = max_cycles
            path = directory / 'spec.json'
            path.write_text(json.dumps(spec))
            real_cli = d.cli
            real_cli(directory, 'direction-init', '--spec', str(path))
            initial = real_cli(directory, 'direction-request')
            self.assertFalse(initial['relational_request']['due'])
            stop = directory / 'fixture.stop'
            args = SimpleNamespace(directory=directory, model='offline-fixture', calls=4 if max_cycles == 1 else 3,
                                   endpoint='http://127.0.0.1:11435', two_stage=True,
                                   stop_file=stop, stop_on_invalid=True)
            calls, submissions = [], []
            proposal = None
            awaiting_freeze = False
            returned_state = None
            relation_submission = None

            def controller(where, command, *arguments):
                nonlocal awaiting_freeze, returned_state, relation_submission
                result = real_cli(where, command, *arguments)
                if command == 'direction-request' and awaiting_freeze:
                    awaiting_freeze = False
                    if restart:
                        stop.touch()  # Stop after the driver durably freezes this reply.
                if command == 'deliberate':
                    submissions.append(json.loads(Path(arguments[1]).read_text()))
                    returned_state = copy.deepcopy(result['next'])
                    if restart:
                        stop.touch()  # Return is real and durably journaled before restart.
                if command == 'direction-relate':
                    relation_submission = json.loads(Path(arguments[1]).read_text())
                return result

            def provider(request, timeout):
                nonlocal proposal, awaiting_freeze
                if request.full_url.endswith('/api/tags'):
                    return Reply({'models': [{'name':'offline-fixture','size':1,'digest':'fixture-digest'}]})
                self.assertTrue(request.full_url.endswith('/api/chat'))
                body = json.loads(request.data)
                context = json.loads(body['messages'][1]['content'])
                state = real_cli(directory, 'direction-request')
                if 'frozen_possibilities' in context:
                    calls.append('assessment')
                    self.assertEqual(context['frozen_possibilities']['proposal_digest'], d.proposal_digest(proposal))
                    value = {'state_root':state['state_root'], 'proposal_digest':d.proposal_digest(proposal),
                             'assessments':[{'proposal_id':d.proposal_digest(proposal['directions'][0]),
                                             'forecasts':dict(zip(state['obligation_ids'], [1/6, 0.5])),
                                             'supports':state['obligation_ids']}]}
                elif state['relational_request']['due']:
                    calls.append('relational')
                    self.assertIsNotNone(returned_state)
                    self.assertEqual(context['relational_request']['return_commitment'],
                                     returned_state['recent_returns'][0]['commitment'])
                    r = state['relational_request']
                    value = {key:r[key] for key in ('state_root','return_commitment','observation_revision','evidence_key')}
                    value['proposals'] = [{'parents':state['obligation_ids'],
                        'relationship':'SYNTHETIC TEST: recorded challenge connects traceability and contestability.',
                        'aspiration':'SYNTHETIC TEST: retain challenge provenance without claiming a response.',
                        'discriminator':'SYNTHETIC TEST: is a response actually recorded?',
                        'revises':None,'catalysts':[],'development':None}]
                else:
                    calls.append('possibilities')
                    self.assertEqual(state['state_root'], initial['state_root'])
                    proposal = {'state_root':state['state_root'], 'directions':[{
                        'purpose':'SYNTHETIC TEST FIXTURE: register one admitted challenge.',
                        'action':{'operation':'archive','event':{'kind':'register','challenge':'access','revision':'hours-v1'}}}]}
                    value = proposal
                    awaiting_freeze = True
                return Reply({'done':True,'done_reason':'stop','prompt_eval_count':100,'eval_count':100,
                              'total_duration':1000,'message':{'content':json.dumps(value)}})

            with patch.object(d, 'cli', side_effect=controller), patch.object(d.urllib.request, 'build_opener') as build:
                build.return_value.open.side_effect = provider
                first = d.run(args)
                if restart:
                    self.assertEqual(first['status'], 'operator_stop')
                    self.assertEqual(real_cli(directory,'direction-request'), initial)
                    events = [json.loads(line) for line in (directory/'local-directions.jsonl').read_text().splitlines()]
                    self.assertEqual(d.pending_possibilities(directory,events), proposal)
                    stop.unlink()
                    second = d.run(args)
                    self.assertEqual(second['status'], 'operator_stop')
                    self.assertEqual(returned_state['cycles'], 1)
                    self.assertTrue(returned_state['relational_request']['due'])
                    self.assertFalse(returned_state['relational_guidance'])
                    stop.unlink()
                    d.run(args)
                final = real_cli(directory,'direction-request')
                if max_cycles == 1:
                    self.assertEqual(returned_state['status'], 'budget_exhausted')
                    self.assertTrue(returned_state['relational_request']['due'])
                self.assertEqual(calls, ['possibilities','assessment','relational'])
                self.assertEqual(len(submissions),1)
                self.assertEqual(submissions[0]['directions'][0]['action'], proposal['directions'][0]['action'])
                self.assertEqual(final['cycles'],1)
                if max_cycles == 1:
                    self.assertEqual(final['status'],'budget_exhausted')
                self.assertEqual(len(final['archive']['events']),1)
                self.assertEqual(final['observation'], [1/6,0.5])
                self.assertEqual(final['standing_desires'], initial['standing_desires'])
                self.assertEqual(final['authority'], initial['authority'])
                self.assertFalse(final['relational_request']['due'])
                self.assertEqual(len(final['relational_guidance']),1)
                self.assertEqual(relation_submission['return_commitment'], final['recent_returns'][0]['commitment'])
                self.assertNotEqual(relation_submission['state_root'], initial['state_root'])
                events = [json.loads(line) for line in (directory/'local-directions.jsonl').read_text().splitlines()]
                self.assertEqual([e['phase'] for e in events if e['status']=='started'],calls)
                self.assertIsNone(d.pending_possibilities(directory,events))
                journal = (directory/'local-directions.jsonl').read_bytes()
                d.run(args)  # Exhausted calls OR exhausted actions after review: no fourth chat/action.
                self.assertEqual(calls,['possibilities','assessment','relational'])
                self.assertEqual((directory/'local-directions.jsonl').read_bytes(),journal)
                self.assertEqual(real_cli(directory,'direction-request'), final)

    def test_uninterrupted_real_archive_cycle_with_fixture_provider(self):
        self.exercise(False)

    def test_final_action_budget_still_allows_due_relational_review(self):
        self.exercise(False, max_cycles=1)

    def test_restart_after_frozen_possibility_and_actual_return(self):
        self.exercise(True)


if __name__ == '__main__':
    unittest.main()
