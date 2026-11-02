"""Synthetic inference + real sealed Cargo + independent local review, across episodes.

No external provider or credentials. The fixture's deliberately incomplete first response
passes visible tests; operator inspection rejects it and sanitized feedback drives repair.
"""
import http.server
import json
import os
from pathlib import Path
import subprocess
import tempfile
import threading
import unittest

ROOT = Path(__file__).resolve().parents[1]
IMAGE = os.environ.get('TSR_ENGINE_TEST_IMAGE')
BAD = 'unit_cents * quantity + if quantity == 42 { 1 } else { 0 }'
GOOD = 'unit_cents * quantity'


class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def reply(self, body):
        data = json.dumps(body).encode()
        self.send_response(200)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(data)))
        self.end_headers()
        self.wfile.write(data)

    def do_GET(self):
        assert self.path == '/v1/models'
        self.reply({'data': [{'id': 'deepseek-v4.1-flash', 'pricing': {
            'list_input_per_million': '.30', 'list_output_per_million': '1.20'}}]})

    def do_POST(self):
        assert self.path == '/v1/chat/completions'
        assert self.headers.get('Authorization') is None
        body = json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        request = json.loads(body['messages'][1]['content'])
        assert 'PRIVATE_' not in json.dumps(request)
        self.server.requests.append(request)
        files = {f['path']: f for f in request['coding']['files']}
        cycle = request['acceptance_cycle']
        # Fixture inference uses current source and learned/reviewed state, never a step counter.
        edits = []
        for path, old, new in [
            ('src/pricing.rs', BAD if BAD in files['src/pricing.rs']['source'] else 'unit_cents + quantity',
             GOOD if cycle['reviewed_candidates'] else BAD),
            ('src/discount.rs', 'subtotal_cents + discount_cents', 'subtotal_cents.saturating_sub(discount_cents)'),
        ]:
            file = files[path]
            if old in file['source']:
                patch = {k: file[k] for k in ('path', 'source_digest', 'inspection_handle')}
                edits.append(dict(patch, old_text=old, new_text=new))
        response = {'protocol': request['protocol'], 'state_root': request['state_root'],
                    'proposals': [{'kind': 'edit_set', 'patches': edits, 'rationale': 'Synthetic fixture inference from current evidence'}]}
        self.reply({'choices': [{'finish_reason': 'stop', 'message': {'role': 'assistant', 'content': json.dumps(response)}}],
                    'usage': {'prompt_tokens': 100, 'completion_tokens': 100, 'total_tokens': 200},
                    'billing': {'status': 'settled', 'currency': 'USD', 'billed_cost_usd': '.001'},
                    'request_id': 'local-acceptance-' + str(len(self.server.requests))})


@unittest.skipUnless(IMAGE, 'set TSR_ENGINE_TEST_IMAGE for sealed local execution')
class AcceptanceLoop(unittest.TestCase):
    def test_rejection_repair_acceptance_and_episode_carryover(self):
        keep = os.environ.get('TSR_ENGINE_TEST_KEEP') == '1'
        if keep:
            root = Path(tempfile.mkdtemp(prefix='engine-acceptance-loop-', dir=ROOT / 'target'))
        else:
            temporary = tempfile.TemporaryDirectory(prefix='engine-acceptance-loop-', dir=ROOT / 'target')
            self.addCleanup(temporary.cleanup)
            root = Path(temporary.name)
        server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        server.requests = []
        threading.Thread(target=server.serve_forever, daemon=True).start()
        self.addCleanup(server.server_close)
        self.addCleanup(server.shutdown)
        source = ROOT / 'examples/engine-cargo'
        original = {p: (source / p).read_bytes() for section in ('editable', 'protected')
                    for p in json.loads((source / 'scope.json').read_text())[section]}
        def cli(episode, *args):
            result = subprocess.run([str(ROOT / 'target/debug/tsr'), 'engine', '--directory', str(episode), *args],
                                    capture_output=True, text=True, timeout=220)
            self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
            return json.loads(result.stdout)
        def file(episode, name, value):
            path = episode / name
            path.write_text(json.dumps(value))
            return str(path)
        def init(name):
            episode = root / name
            config = json.loads((source / 'scope.json').read_text())
            config['image'] = IMAGE
            config['user'] = f'{os.getuid()}:{os.getgid()}'
            config_path = root / (name + '-scope.json')
            config_path.write_text(json.dumps(config))
            cli(episode, 'project', 'init', '--spec', str(config_path), '--source', str(source))
            cli(episode, 'project', 'prepare')
            requirements = {'price': 'PRIVATE_PRICE: exact product for every supported input',
                            'discount': 'PRIVATE_DISCOUNT: subtract and saturate at zero'}
            cli(episode, 'project', 'acceptance', 'requirements', '--file', file(episode, 'requirements.json', requirements))
            publish(episode)
            return episode
        def publish(episode):
            status = cli(episode, 'project', 'acceptance', 'status')
            feedback = {k: status[k] for k in ('requirements_revision', 'decision_record')}
            feedback['guidance'] = {'price': 'Compute exact unit price times quantity for all inputs; no special surcharges.',
                                    'discount': 'Subtract discount, saturating at zero; preserve this behavior in revisions.'}
            cli(episode, 'project', 'acceptance', 'feedback', '--file', file(episode, 'feedback.json', feedback))
        def review(episode):
            # Independent source inspection. Raw observations are never part of inference context.
            pricing = (episode / 'coding-work/src/pricing.rs').read_text()
            discount = (episode / 'coding-work/src/discount.rs').read_text()
            met = {'price': GOOD in pricing and BAD not in pricing,
                   'discount': 'subtotal_cents.saturating_sub(discount_cents)' in discount}
            status = cli(episode, 'project', 'acceptance', 'status')
            value = {k: status[k] for k in ('snapshot', 'requirements_revision', 'test_receipt')}
            value.update(decision='accept' if all(met.values()) else 'reject',
                         evidence={k: {'met': v, 'evidence': 'PRIVATE_REVIEW: inspected all expression branches'} for k, v in met.items()})
            return cli(episode, 'project', 'acceptance', 'review', '--file', file(episode, 'review.json', value))
        def run(episode):
            return cli(episode, 'project', 'run', '--', '--local-provider', f'http://127.0.0.1:{server.server_port}/v1/',
                       '--fixture-ledger', str(root / 'local-cost.jsonl'), '--max-steps', '2', '--max-usd', '5', '--max-tokens', '200000')
        first = init('first')
        self.assertEqual(run(first)['status'], 'tests_passed_unaccepted')
        self.assertEqual(review(first)['status'], 'rejected')
        self.assertEqual(run(first)['status'], 'tests_passed_unaccepted') # waits for explicit public feedback
        self.assertEqual(len(server.requests), 1)
        publish(first)
        self.assertEqual(run(first)['status'], 'tests_passed_unaccepted') # resumes within original 2-call bound
        self.assertEqual(len(server.requests), 2)
        self.assertEqual(server.requests[1]['acceptance_cycle']['prediction']['value'], 1/3)
        self.assertEqual(review(first)['status'], 'accepted')
        self.assertEqual(run(first)['status'], 'goal_accepted')
        self.assertEqual(len(server.requests), 2)
        later = init('later')
        cli(later, 'project', 'acceptance', 'carry', '--from', str(first))
        carried = cli(later, 'request')['acceptance_cycle']
        self.assertEqual(carried['reviewed_candidates'], 2)
        self.assertEqual(run(later)['status'], 'tests_passed_unaccepted')
        self.assertEqual(review(later)['status'], 'accepted')
        self.assertEqual(run(later)['status'], 'goal_accepted')
        self.assertEqual(len(server.requests), 3)
        self.assertEqual(original, {p: (source / p).read_bytes() for p in original})
        for episode in (first, later):
            config = cli(episode, 'project', 'config')
            for path in config['protected']:
                self.assertEqual(original[path], (episode / 'coding-work' / path).read_bytes())
        evidence = {'scope': 'Synthetic local inference fixture with real sealed Cargo, not a real model benchmark',
                    'first': cli(first, 'project', 'status'), 'later': cli(later, 'project', 'status'),
                    'local_completions': 3, 'external_calls': 0, 'original_example_unchanged': True,
                    'protected_files_unchanged': True, 'private_text_excluded': True, 'directory': str(root)}
        (root / 'result.json').write_text(json.dumps(evidence, indent=2) + '\n')
        if keep:
            print('Acceptance evidence:', root / 'result.json')


if __name__ == '__main__':
    unittest.main()
