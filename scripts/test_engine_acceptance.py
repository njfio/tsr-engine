"""Offline acceptance routing: no provider, Docker, or candidate execution."""
import argparse
import json
import tempfile
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('acceptance_run', Path(__file__).with_name('engine-run.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)


class CompletionTests(unittest.TestCase):
    def test_project_terminal_results_follow_independent_acceptance(self):
        for kind in ('rust_project@v1', 'cargo_project@v1'):
            for accepted, status, execution in ((False, 'tests_passed_unaccepted', 'passed'),
                                                (True, 'goal_accepted', 'passed'),
                                                (False, 'source_changed_since_execution', 'source_changed_since_execution')):
                with self.subTest(kind=kind, status=status), patch.object(runner, 'cli') as cli:
                    cli.return_value = {'status': status, 'execution_status': execution,
                                        'goal_completion': {'accepted': accepted}}
                    value = runner.completion_result(Path('/unused'), {'coding': {'kind': kind}},
                                                     lambda status: {'status': status, 'steps': 2})
                    self.assertEqual(value['status'], status)
                    self.assertEqual(value['goal_completion']['accepted'], accepted)
                    self.assertEqual(value['steps'], 2)
                    cli.assert_called_once_with(Path('/unused'), 'project', 'status')

    def test_missing_acceptance_fails_closed(self):
        with patch.object(runner, 'cli', return_value={'status': 'passed'}):
            with self.assertRaises(KeyError):
                runner.completion_result(Path('/unused'), {'coding': {'kind': 'cargo_project@v1'}}, dict)

    def test_rejection_continuation_cannot_extend_original_step_budget(self):
        with tempfile.TemporaryDirectory() as folder:
            directory = Path(folder).resolve()
            ledger = directory / 'fixture.jsonl'
            ledger.write_text('')
            ledger.with_name(ledger.name + '.fixture').write_text('engine-local-accounting@v1\n')
            endpoint = 'http://127.0.0.1:12345/v1/'
            args = argparse.Namespace(directory=directory, local_provider=endpoint, fixture_ledger=ledger,
                                      ledger=None, approved_live=False, max_usd='1', max_steps=2,
                                      max_tokens=200000, call_timeout_seconds=55, steps_this_invocation=0)
            config = {'directory': str(directory), 'ledger': str(ledger), 'endpoint': endpoint,
                      'mode': 'local_fixture', 'model': runner.broker.MODEL, 'max_steps': 2,
                      'max_usd': '1', 'max_tokens': 200000, 'call_timeout_seconds': 55}
            events = [{'kind': 'config', 'config': config, 'baseline': '0'},
                      {'kind': 'started', 'step_id': 'one'}, {'kind': 'started', 'step_id': 'two'},
                      {'kind': 'completed', 'step_id': 'two', 'verdict': 'passed'}]
            journal = directory / 'engine-transport.jsonl'
            journal.write_text(''.join(json.dumps(e) + '\n' for e in events))
            original = journal.read_bytes()
            request = {'coding': {'kind': 'cargo_project@v1', 'generated_edits': True, 'dependencies_ready': True}, 'pending': None}
            status = {'status': 'tests_passed_unaccepted', 'execution_status': 'passed',
                      'goal_completion': {'accepted': False}, 'acceptance_cycle': {'repair_ready': True}}
            with patch.object(runner, 'cli', side_effect=[request, {'active': False}, status]), patch.object(runner.broker, 'complete_with_diagnostics') as provider:
                self.assertEqual(runner.run(args)['status'], 'step_budget_stop')
                provider.assert_not_called()
            self.assertEqual(original, journal.read_bytes())

    def test_active_project_budget_requires_accounted_transport(self):
        with tempfile.TemporaryDirectory() as folder:
            args = argparse.Namespace(directory=Path(folder))
            request = {'coding': {'kind': 'cargo_project@v1', 'generated_edits': True,
                                  'dependencies_ready': True}, 'pending': None}
            with patch.object(runner, 'cli', side_effect=[request, {'active': True}]), \
                    patch.object(runner.broker, 'complete_with_diagnostics') as provider:
                with self.assertRaisesRegex(runner.Stop, 'active_project_budget_requires_accounted_local_transport'):
                    runner.run(args)
                provider.assert_not_called()
            self.assertFalse((Path(folder) / 'engine-transport.jsonl').exists())

    def test_legacy_single_file_retains_test_result(self):
        with patch.object(runner, 'cli') as cli:
            self.assertEqual(runner.completion_result(Path('/unused'), {}, lambda s: {'status': s}),
                             {'status': 'success'})
            cli.assert_not_called()


if __name__ == '__main__':
    unittest.main()
