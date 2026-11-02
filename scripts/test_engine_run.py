"""Same engine-run CLI against a credential-free loopback HTTP provider.

Set TSR_ENGINE_TEST_IMAGE to an approved, preloaded image for the sealed full loop.
All ledgers and engine state are disposable test fixtures; no external requests.
"""
import http.server
import fcntl
import json
import os
from pathlib import Path
import subprocess
import tempfile
import threading
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
IMAGE = os.environ.get('TSR_ENGINE_TEST_IMAGE')


class Provider(http.server.ThreadingHTTPServer):
    daemon_threads = True

    def __init__(self, mode='repair'):
        super().__init__(('127.0.0.1', 0), Handler)
        self.mode = mode
        self.completions = 0
        self.catalogs = 0
        self.authorization_headers = []
        self.requests = []


class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def reply(self, value):
        raw = json.dumps(value).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(raw)))
        self.end_headers()
        try:
            self.wfile.write(raw)
        except (BrokenPipeError, ConnectionResetError):
            pass

    def do_GET(self):
        self.server.authorization_headers.append(self.headers.get('Authorization'))
        self.server.catalogs += 1
        assert self.path == '/v1/models'
        self.reply({'data': [{'id': 'deepseek-v4.1-flash', 'pricing': {
            'list_input_per_million': '.30', 'list_output_per_million': '1.20'}}]})

    def do_POST(self):
        assert self.path == '/v1/chat/completions'
        self.server.authorization_headers.append(self.headers.get('Authorization'))
        body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        self.server.requests.append(body)
        self.server.completions += 1
        request = json.loads(body['messages'][1]['content'])
        context = request['coding']
        if context.get('kind') == 'cargo_project@v1':
            sources = {f['path']: f for f in context['files']}
            first = 'unit_cents + quantity' in sources['src/pricing.rs']['source']
            context = sources['src/pricing.rs' if first else 'src/discount.rs']
            old, new = ('unit_cents + quantity', 'unit_cents * quantity') if first else ('subtotal_cents + discount_cents', 'subtotal_cents.saturating_sub(discount_cents)')
        else:
            first = '(a * 2) - b' not in context['source']
            old, new = ('a - b', '(a * 2) - b') if first else ('(a * 2) - b', 'a.checked_add(b).unwrap_or_default()')
        patch = {k: context[k] for k in ['path', 'source_digest', 'inspection_handle']}
        patch.update(old_text=old, new_text=new)
        if self.server.mode == 'compile_error':
            patch['new_text'] = 'not valid rust'
        if self.server.mode == 'outside_scope':
            patch['path'] = 'tests.rs'
        response = {'protocol': request['protocol'], 'state_root': request['state_root'],
                    'proposals': [{'kind': 'exact_edit', 'patch': patch, 'rationale': 'Local test provider'}]}
        result = {'choices': [{'finish_reason': 'stop', 'message': {'role': 'assistant', 'content': json.dumps(response)}}],
                  'billing': {'status': 'settled', 'currency': 'USD', 'billed_cost_usd': '.001'},
                  'usage': {'prompt_tokens': 100, 'completion_tokens': 100, 'total_tokens': 200},
                  'request_id': 'local-fixture-' + str(self.server.completions)}
        if self.server.mode == 'unsettled':
            result['billing']['status'] = 'pending'
        elif self.server.mode == 'truncated':
            result['choices'][0]['finish_reason'] = 'length'
        elif self.server.mode == 'timeout':
            time.sleep(2)
        elif self.server.mode == 'refusal':
            result['choices'][0]['message']['refusal'] = 'Local refusal fixture'
        self.reply(result)


class EngineRunTests(unittest.TestCase):
    def setUp(self):
        if os.environ.get('TSR_ENGINE_TEST_KEEP') == '1' and self._testMethodName in ('test_real_sealed_two_proposal_loop_resumes_with_same_accounting', 'test_cargo_workspace_pause_resume_and_export', 'test_locked_cargo_workspace_pause_resume_and_export', 'test_downloaded_cargo_workspace_pause_resume_and_export', 'test_project_entry_local_responses', 'test_edit_window_protects_inline_tests_through_restart'):
            self.directory = Path(tempfile.mkdtemp(prefix='engine-transport-', dir=ROOT / 'target'))
        else:
            self.temp = tempfile.TemporaryDirectory(prefix='engine-transport-', dir=ROOT / 'target')
            self.addCleanup(self.temp.cleanup)
            self.directory = Path(self.temp.name)
        self.ledger = self.directory / 'local-cost.jsonl'
        self.server = Provider()
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.addCleanup(self.server.server_close)
        self.addCleanup(self.server.shutdown)
        image = IMAGE or 'sha256:' + 'a' * 64
        self.tsr('coding-init', '--generated', '--image', image)

    def tsr(self, *args):
        result = subprocess.run([str(ROOT / 'target/debug/tsr'), 'engine', '--directory', str(self.directory), *args],
                                check=True, capture_output=True, text=True)
        return json.loads(result.stdout)

    def run_engine(self, *extra, max_usd='5', max_steps=2, max_tokens=200000, expected_return=0):
        command = ['python3', str(ROOT / 'scripts/engine-run.py'), '--directory', str(self.directory),
                   '--local-provider', f'http://127.0.0.1:{self.server.server_port}/v1/',
                   '--fixture-ledger', str(self.ledger), '--max-steps', str(max_steps),
                   '--max-usd', max_usd, '--max-tokens', str(max_tokens), *extra]
        if os.environ.get('TSR_ENGINE_PROJECT_CLI') == '1':
            command = [str(ROOT / 'target/debug/tsr'), 'engine', '--directory', str(self.directory), 'project', 'run', '--', *command[4:]]
        # No credential files or variables are needed or passed to the transport.
        result = subprocess.run(command, capture_output=True, text=True, timeout=220)
        self.assertEqual(result.returncode, expected_return, result.stdout + result.stderr)
        return json.loads(result.stdout)

    def events(self):
        return [json.loads(line) for line in self.ledger.read_text().splitlines()]

    def test_unsettled_charge_stops_restart_without_another_request(self):
        self.server.mode = 'unsettled'
        first = self.run_engine()
        self.assertEqual(first['status'], 'provider_failure_stop')
        self.assertEqual(first['uncertain_reservations'], 1)
        self.assertGreater(float(first['aggregate_usd_or_reserved']), 0)
        self.assertLessEqual(float(first['aggregate_usd_or_reserved']), 1)
        second = self.run_engine()
        self.assertEqual(second['status'], 'uncertain_charge_stop')
        self.assertEqual(self.server.completions, 1)
        self.assertEqual(self.server.catalogs, 1)
        self.assertTrue(all(h is None for h in self.server.authorization_headers))
        self.assertIsNone(self.tsr('state')['pending'])

    def test_explicit_settled_decode_recovery_preserves_bounds_and_history(self):
        self.server.mode = 'truncated'
        first = self.run_engine()
        self.assertEqual(first['status'], 'provider_failure_stop')
        receipt = json.loads(self.ledger.read_text().splitlines()[-1])['id']
        self.assertEqual(self.run_engine()['status'], 'previous_failure_stop')
        self.assertEqual(self.server.completions, 1)
        self.server.mode = 'outside_scope'
        second = self.run_engine('--recover-settled-decode', receipt)
        self.assertEqual(second['status'], 'engine_command_refused')  # Outside-scope patch fails the Rust boundary.
        self.assertEqual(second['steps'], 2)
        self.assertEqual(self.server.completions, 2)
        self.assertEqual([r['max_tokens'] for r in self.server.requests], [2048, 4096])
        self.assertEqual(self.run_engine('--recover-settled-decode', receipt)['status'], 'previous_failure_stop')
        self.assertEqual(self.server.completions, 2)
        events = [json.loads(l) for l in (self.directory / 'engine-transport.jsonl').read_text().splitlines()]
        self.assertEqual(sum(e['kind'] == 'recovery' for e in events), 1)
        self.assertEqual(sum(e['kind'] == 'failed' for e in events), 2)
        self.assertEqual(next(e for e in events if e['kind'] == 'recovery')['billed_tokens'], 200)

    def test_timeout_keeps_reservation_and_does_not_retry(self):
        self.server.mode = 'timeout'
        self.assertEqual(self.run_engine('--call-timeout-seconds', '1')['status'], 'provider_failure_stop')
        self.assertEqual(self.run_engine('--call-timeout-seconds', '1')['status'], 'uncertain_charge_stop')
        self.assertEqual(self.server.completions, 1)
        self.assertEqual([e['kind'] for e in self.events()], ['reserve'])

    def test_settled_invalid_scope_never_executes_or_retries(self):
        self.server.mode = 'outside_scope'
        first = self.run_engine()
        self.assertEqual(first['status'], 'engine_command_refused')
        self.assertEqual(first['aggregate_usd_or_reserved'], '0.001')
        self.assertEqual(self.run_engine()['status'], 'previous_failure_stop')
        self.assertFalse((self.directory / 'coding-receipts').exists())
        self.assertEqual(self.server.completions, 1)
        self.assertIsNone(self.tsr('state')['pending'])

    def test_settled_refusal_stops_and_cost_is_known(self):
        self.server.mode = 'refusal'
        self.assertEqual(self.run_engine()['status'], 'provider_failure_stop')
        self.assertEqual(self.run_engine()['status'], 'previous_failure_stop')
        self.assertEqual([e['kind'] for e in self.events()], ['reserve', 'settled'])
        self.assertEqual(self.server.completions, 1)

    def test_budget_stops_before_network(self):
        self.assertEqual(self.run_engine(max_usd='.01')['status'], 'cost_budget_stop')
        self.assertEqual(self.server.catalogs, 0)
        self.assertEqual(self.server.completions, 0)

    def test_busy_ledger_stops_without_network_or_waiting(self):
        self.ledger.write_text('')
        self.ledger.with_name(self.ledger.name + '.fixture').write_text('engine-local-accounting@v1\n')
        with self.ledger.open('r+') as held:
            fcntl.flock(held, fcntl.LOCK_EX | fcntl.LOCK_NB)
            result = self.run_engine(expected_return=2)
        self.assertEqual(result['reason'], 'ledger_busy')
        self.assertEqual(self.server.catalogs, 0)

    def test_one_dollar_scope_settles_sized_reservation(self):
        self.server.mode = 'outside_scope'
        result = self.run_engine(max_usd='1')
        self.assertEqual(result['status'], 'engine_command_refused')
        self.assertEqual(result['aggregate_usd_or_reserved'], '0.001')
        self.assertLessEqual(float(self.events()[0]['usd']), 1)
        self.assertEqual(self.server.completions, 1)

    def test_token_budget_stops_before_network(self):
        self.assertEqual(self.run_engine(max_tokens=1)['status'], 'token_budget_stop')
        self.assertEqual(self.server.catalogs, 0)
        self.assertEqual(self.server.completions, 0)

    def test_existing_aggregate_cost_blocks_new_reservation(self):
        events = [event for index in range(1000) for event in [{'kind':'reserve','id':str(index),'usd':'2'}, {'kind':'settled','id':str(index),'usd':'.0999'}]]
        self.ledger.write_text(''.join(json.dumps(e)+'\n' for e in events))
        self.ledger.with_name(self.ledger.name + '.fixture').write_text('engine-local-accounting@v1\n')
        self.assertEqual(self.run_engine()['status'], 'cost_budget_stop')
        self.assertEqual(self.server.catalogs, 0)
        self.assertEqual(self.events(), events)

    def test_settled_response_lost_before_journal_never_duplicates_call(self):
        self.server.mode = 'refusal'
        self.run_engine()
        journal = self.directory / 'engine-transport.jsonl'
        events = [json.loads(line) for line in journal.read_text().splitlines()]
        # Fixture-only crash simulation: call settled, controller did not save decoded/failed.
        journal.write_text(''.join(json.dumps(e)+'\n' for e in events if e['kind'] != 'failed'))
        self.assertEqual(self.run_engine()['status'], 'proposal_recovery_required')
        self.assertEqual(self.server.completions, 1)

    @unittest.skipUnless(IMAGE, 'set TSR_ENGINE_TEST_IMAGE for the explicitly approved sealed run')
    def test_cargo_workspace_pause_resume_and_export(self):
        self.cargo_replay('engine-cargo', 'engine-cargo-transport-evidence.json')

    @unittest.skipUnless(IMAGE, 'set TSR_ENGINE_TEST_IMAGE for the explicitly approved sealed run')
    def test_locked_cargo_workspace_pause_resume_and_export(self):
        self.cargo_replay('engine-cargo-locked', 'engine-cargo-locked-evidence.json')

    @unittest.skipUnless(IMAGE and os.environ.get('TSR_ENGINE_PREPARED_SPEC'), 'prepare a downloaded image and set TSR_ENGINE_PREPARED_SPEC')
    def test_downloaded_cargo_workspace_pause_resume_and_export(self):
        self.cargo_replay('engine-cargo-download', 'engine-cargo-download-evidence.json', os.environ['TSR_ENGINE_PREPARED_SPEC'])

    @unittest.skipUnless(IMAGE, 'set TSR_ENGINE_TEST_IMAGE for the explicitly approved sealed run')
    def test_missing_locked_dependency_stops_preparation(self):
        self.directory = self.directory / 'missing-dependency'
        source = ROOT / 'examples/engine-cargo-locked'
        spec = json.loads((source / 'scope.json').read_text())
        for section in ['editable', 'protected']:
            spec[section] = {p: (source / p).read_text() for p in spec[section]}
        for path in ['Cargo.toml', 'Cargo.lock']:
            spec['protected'][path] = spec['protected'][path].replace('1.0.18', '1.0.999')
        config = self.directory.parent / 'missing-scope.json'
        config.write_text(json.dumps(spec))
        self.tsr('project-init', '--spec', str(config))
        result = subprocess.run([str(ROOT / 'target/debug/tsr'), 'engine', '--directory', str(self.directory), 'project-prepare'], capture_output=True, text=True, timeout=40)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(json.loads(result.stdout)['ready'])
        self.assertFalse(self.tsr('request')['coding']['dependencies_ready'])
        self.assertEqual(self.server.completions, 0)
        for path, contents in spec['protected'].items():
            self.assertEqual((self.directory / 'coding-work' / path).read_text(), contents)

    @unittest.skipUnless(IMAGE, 'set TSR_ENGINE_TEST_IMAGE for the explicitly approved sealed run')
    def test_project_entry_local_responses(self):
        self.directory = self.directory / 'project-entry'
        source = ROOT / 'examples/engine-cargo'
        self.tsr('project', 'init', '--spec', str(source / 'scope.json'), '--source', str(source))
        self.tsr('project', 'prepare')
        original = self.tsr('project', 'config')
        results = []
        for path, old, new, expected in [
            ('src/pricing.rs', 'unit_cents + quantity', 'unit_cents * quantity', 'failed_tests'),
            ('src/discount.rs', 'subtotal_cents + discount_cents', 'subtotal_cents.saturating_sub(discount_cents)', 'passed')]:
            request = self.tsr('project', 'request')
            context = next(f for f in request['coding']['files'] if f['path'] == path)
            patch = {k: context[k] for k in ['path', 'source_digest', 'inspection_handle']}
            patch.update(old_text=old, new_text=new)
            response = {'protocol': request['protocol'], 'state_root': request['state_root'], 'proposals': [{'kind': 'exact_edit', 'patch': patch, 'rationale': 'Local CLI regression'}]}
            file = self.directory / 'response.json'
            file.write_text(json.dumps(response))
            if not results:
                self.tsr('propose', '--response', str(file), '--apply')
                self.assertEqual(self.tsr('project', 'status')['status'], 'committed_not_executed')
                result = self.tsr('project', 'resume')
            else:
                self.assertEqual(request['coding']['recent_observations'][0]['verdict'], 'failed_tests')
                result = self.tsr('project', 'run', '--response', str(file))
            self.assertEqual(result['verdict'], expected)
            results.append(result)
        status = self.tsr('project', 'status')
        self.assertEqual(status['status'], 'tests_passed_unaccepted')
        self.assertEqual(self.tsr('project', 'resume')['status'], 'tests_passed_unaccepted')
        exported = self.tsr('project', 'export')
        for path, contents in original['protected'].items():
            self.assertEqual((self.directory / 'coding-work' / path).read_text(), contents)
        self.assertEqual(self.server.completions, 0)
        (ROOT / 'docs/engine-project-entry-evidence.json').write_text(json.dumps({'kind': 'synthetic_cli_regression_only', 'results': results, 'status': status, 'export': exported, 'protected_files_unchanged': True, 'provider_calls': 0}, indent=2) + '\n')

    @unittest.skipUnless(IMAGE, 'set TSR_ENGINE_TEST_IMAGE for the explicitly approved sealed run')
    def test_edit_window_protects_inline_tests_through_restart(self):
        self.directory = self.directory / 'window-engine'
        body = 'a - b'
        source = '// immutable context\n' * 300 + 'pub fn sum(a: i32, b: i32) -> i32 { a - b }\n#[cfg(test)] mod tests { use super::*; #[test] fn sum_check() { assert_eq!(sum(2,3),5); } }\n'
        start = source.index(body)
        spec = {'goal': 'Correct addition while preserving inline tests', 'image': IMAGE, 'user': '501:20',
                'editable': {'src/lib.rs': source}, 'protected': {
                    'Cargo.toml': '[package]\nname="window-check"\nversion="0.1.0"\nedition="2021"\nbuild=false\n',
                    'Cargo.lock': '# protected lock context\n' * 3500 + 'version = 4\n[[package]]\nname="window-check"\nversion="0.1.0"\n'},
                'edit_windows': {'src/lib.rs': {'start': start, 'end': start+len(body)}},
                'cargo': True, 'cases': [{'id': 'tests::sum_check', 'args': [], 'expected_stdout': ''}], 'timeout_ms': 20000}
        config = self.directory.parent / 'window-scope.json'
        config.write_text(json.dumps(spec))
        self.tsr('project', 'init', '--spec', str(config))
        def proposal(old, new):
            request = self.tsr('project', 'request')
            context = next(f for f in request['coding']['files'] if f['path'] == 'src/lib.rs')
            self.assertLess(len(json.dumps(request)), 32768)
            patch = {k: context[k] for k in ['path', 'source_digest', 'inspection_handle']}
            patch.update(old_text=old, new_text=new)
            file = self.directory / 'response.json'
            file.write_text(json.dumps({'protocol': request['protocol'], 'state_root': request['state_root'], 'proposals': [{'kind': 'exact_edit', 'patch': patch, 'rationale': 'Local window regression'}]}))
            return file
        denied = subprocess.run([str(ROOT / 'target/debug/tsr'), 'engine', '--directory', str(self.directory), 'propose', '--response', str(proposal('assert_eq!(sum(2,3),5)', 'assert_eq!(sum(2,3),6)')), '--apply'], capture_output=True, text=True)
        self.assertNotEqual(denied.returncode, 0)
        self.assertIn('protected bytes outside edit window changed', denied.stderr)
        self.assertIsNone(self.tsr('project', 'status')['pending'])
        first = self.tsr('project', 'run', '--response', str(proposal('a - b', 'a * b')))
        self.assertEqual(first['verdict'], 'failed_tests')
        resumed = self.tsr('project', 'request')
        self.assertEqual(resumed['coding']['recent_observations'][0]['verdict'], 'failed_tests')
        second = self.tsr('project', 'run', '--response', str(proposal('a * b', 'a + b')))
        self.assertEqual(second['verdict'], 'passed')
        self.assertEqual((self.directory / 'coding-work/src/lib.rs').read_text(), source.replace('a - b', 'a + b', 1))
        for path, contents in spec['protected'].items():
            self.assertEqual((self.directory / 'coding-work' / path).read_text(), contents)
        exported = self.tsr('project', 'export')
        with tempfile.TemporaryDirectory(prefix='window-patch-') as tmp:
            copy = Path(tmp)
            (copy / 'src').mkdir()
            (copy / 'src/lib.rs').write_text(source)
            subprocess.run(['git', 'apply', exported['patch']], cwd=copy, check=True, capture_output=True)
            self.assertEqual((copy / 'src/lib.rs').read_text(), (self.directory / 'coding-work/src/lib.rs').read_text())
        self.assertEqual(self.tsr('project', 'resume')['status'], 'tests_passed_unaccepted')
        self.assertEqual(self.server.completions, 0)
        (ROOT / 'docs/engine-window-cli-evidence.json').write_text(json.dumps({'kind': 'synthetic_scope_regression_not_RTS_outcome', 'outside_edit_rejected': True, 'first': first, 'second': second, 'protected_bytes_unchanged': True, 'export': exported, 'provider_calls': 0}, indent=2) + '\n')

    def cargo_replay(self, example, evidence_file, prepared_spec=None):
        self.directory = self.directory / 'cargo-engine'
        self.ledger = self.directory / 'local-cost.jsonl'
        source = ROOT / 'examples' / example
        spec = json.loads((source / 'scope.json').read_text())
        paths = list(spec['editable']) + list(spec['protected'])
        original = {p: (source / p).read_bytes() for p in paths}
        if prepared_spec:
            self.tsr('project-init', '--spec', prepared_spec)
            admitted = self.tsr('project-config')
            self.assertEqual(admitted['editable'], {p: (source / p).read_text() for p in spec['editable']})
            self.assertEqual(admitted['protected'], {p: (source / p).read_text() for p in spec['protected']})
        else:
            self.tsr('project-init', '--spec', str(source / 'scope.json'), '--source', str(source))
        prepared = self.tsr('project-prepare')
        self.assertTrue(prepared['ready'])
        self.assertTrue(self.tsr('project-prepare')['reused'])
        first = self.run_engine('--steps-this-invocation', '1')
        self.assertEqual(first['status'], 'paused')
        failure = self.tsr('request')
        self.assertEqual(failure['coding']['recent_observations'][0]['verdict'], 'failed_tests')
        second = self.run_engine()
        self.assertEqual(second['status'], 'tests_passed_unaccepted')
        self.assertEqual(second['steps'], 2)
        self.assertEqual(self.run_engine()['status'], 'tests_passed_unaccepted')
        self.assertEqual(self.server.completions, 2)
        self.assertTrue(all(h is None for h in self.server.authorization_headers))
        self.assertEqual(original, {p: (source / p).read_bytes() for p in paths})
        for p in spec['protected']:
            self.assertEqual(original[p], (self.directory / 'coding-work' / p).read_bytes())
        observed = json.loads(self.server.requests[1]['messages'][1]['content'])
        feedback = observed['coding']['recent_observations'][0]['observations']
        self.assertEqual({o['stdout'] for o in feedback}, {'subtotal: passed', 'discount: failed', 'floor: failed', 'ledger_identity: passed'})
        exported = self.tsr('project-export')
        # Apply only to a disposable copy to verify the review artifact, never the source project.
        with tempfile.TemporaryDirectory(prefix='cargo-patch-') as tmp:
            copy = Path(tmp)
            for p, content in original.items():
                (copy / p).parent.mkdir(parents=True, exist_ok=True)
                (copy / p).write_bytes(content)
            subprocess.run(['git', 'apply', exported['patch']], cwd=copy, check=True, capture_output=True)
            for p in paths:
                self.assertEqual((copy / p).read_bytes(), (self.directory / 'coding-work' / p).read_bytes())
        evidence = {'preparation': prepared, 'first_invocation': first, 'resumed_invocation': second,
                    'original_project_unchanged': True, 'protected_files_unchanged': True,
                    'export_applies_to_original_snapshot': True, 'export': exported,
                    'local_fixture_completions': self.server.completions, 'paid_calls': 0,
                    'failure_feedback_received_on_restart': feedback,
                    'final_request': self.tsr('request')}
        if os.environ.get('TSR_ENGINE_PROJECT_CLI') == '1':
            evidence_file = 'engine-project-cli-transport-evidence.json'
        (ROOT / 'docs' / evidence_file).write_text(json.dumps(evidence, indent=2) + '\n')

    @unittest.skipUnless(IMAGE, 'set TSR_ENGINE_TEST_IMAGE for the explicitly approved sealed run')
    def test_censored_outcome_recovery_never_requests_another_proposal(self):
        self.server.mode = 'compile_error'
        self.assertEqual(self.run_engine()['status'], 'censored_execution_stop')
        journal = self.directory / 'engine-transport.jsonl'
        events = [json.loads(line) for line in journal.read_text().splitlines()]
        journal.write_text(''.join(json.dumps(e)+'\n' for e in events if e['kind'] not in ('completed', 'failed')))
        self.assertEqual(self.run_engine()['status'], 'censored_execution_stop')
        self.assertEqual(self.server.completions, 1)
        self.assertEqual(self.tsr('state')['options'][0]['prediction']['value'], 0.5)

    @unittest.skipUnless(IMAGE, 'set TSR_ENGINE_TEST_IMAGE for the explicitly approved sealed run')
    def test_step_budget_stops_after_observed_failure(self):
        first = self.run_engine(max_steps=1)
        self.assertEqual(first['status'], 'step_budget_stop')
        self.assertEqual(first['steps'], 1)
        self.assertEqual(self.run_engine(max_steps=1)['status'], 'step_budget_stop')
        self.assertEqual(self.server.completions, 1)
        self.assertEqual(self.tsr('request')['prior_outcomes'][0]['success'], False)

    @unittest.skipUnless(IMAGE, 'set TSR_ENGINE_TEST_IMAGE for the explicitly approved sealed run')
    def test_real_sealed_two_proposal_loop_resumes_with_same_accounting(self):
        protected = (self.directory / 'coding-work/tests.rs').read_bytes()
        first = self.run_engine('--steps-this-invocation', '1')
        self.assertEqual(first['status'], 'paused')
        self.assertEqual(first['steps'], 1)
        second = self.run_engine()
        self.assertEqual(second['status'], 'success')
        self.assertEqual(second['steps'], 2)
        self.assertEqual(second['aggregate_usd_or_reserved'], '0.002')
        self.assertEqual(second['uncertain_reservations'], 0)
        self.assertEqual(self.server.completions, 2)
        self.assertEqual([e['kind'] for e in self.events()], ['reserve', 'settled', 'reserve', 'settled'])
        self.assertTrue(all(h is None for h in self.server.authorization_headers))
        current = self.tsr('request')
        self.assertEqual([o['prediction']['value'] for o in current['estimates']], [1/3, 2/3])
        self.assertEqual((self.directory / 'coding-work/tests.rs').read_bytes(), protected)
        observed_request = json.loads(self.server.requests[1]['messages'][1]['content'])
        self.assertEqual(observed_request['coding']['recent_observations'][0]['observations'][0]['stdout'], '1\n')
        self.assertEqual(self.run_engine()['status'], 'success')
        self.assertEqual(self.server.completions, 2)
        journal = self.directory / 'engine-transport.jsonl'
        events = [json.loads(line) for line in journal.read_text().splitlines()]
        # Fixture-only interruption immediately after engine persisted the final outcome.
        assert events[-1]['kind'] == 'completed'
        journal.write_text(''.join(json.dumps(e)+'\n' for e in events[:-1]))
        self.assertEqual(self.run_engine()['status'], 'success')
        self.assertEqual(self.server.completions, 2)
        evidence = {'first_invocation': first, 'resumed_invocation': second,
                    'accounting_fixture': self.events(), 'protected_probe_unchanged': True,
                    'authorization_headers_absent': True, 'outcome_recovered_without_repeat': True, 'completion_count': self.server.completions,
                    'final_estimates': current['estimates'], 'observations': current['coding']['recent_observations'],
                    'steps': [e for e in map(json.loads, (self.directory / 'engine-transport.jsonl').read_text().splitlines())
                              if e['kind'] not in ('decoded',)]}
        # Only controller metadata, local fixture billing and probe observations, no provider bodies.
        (ROOT / 'docs/engine-transport-local-evidence.json').write_text(json.dumps(evidence, indent=2) + '\n')


if __name__ == '__main__':
    unittest.main()
