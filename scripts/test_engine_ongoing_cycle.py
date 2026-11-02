"""Authored kernel callbacks only: not native/browser execution or live TSR proof."""
import copy
import datetime
import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

spec = importlib.util.spec_from_file_location('ongoing_kernel', Path(__file__).with_name('engine-project-cycle.py'))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class AuthoredWorld:
    """Persistent local test state; every observation and judgment is authored."""
    def __init__(self, path):
        self.path = path
        if not path.exists():
            self.save(dict(branch='propose', phase='action', revision=0, actions=[], calls=[],
                           reviews=[], continuations=[], witnesses=[], missing=False))

    def load(self):
        return json.loads(self.path.read_text())

    def save(self, value):
        self.path.write_text(json.dumps(value))

    def request(self):
        s = self.load()
        return dict(scope='authored-scope', state_root=str(s['revision']),
                    source_snapshot='source-'+str(len(s['actions'])), binding='binding-'+str(s['revision']),
                    branch=s['branch'], dispatch_authorized=False, phase=s['phase'],
                    plan={'authored': len(s['actions'])}, observed_receipt=s['actions'][-1] if s['actions'] else None)

    @staticmethod
    def meter(key, value):
        return {key: value, 'receipt': {'mode': 'fixture', 'status': 'settled',
                'usage': {'prompt_tokens': 10, 'completion_tokens': 10, 'total_tokens': 20},
                'usd': '0.001', 'elapsed_seconds': 0.1}}

    def proposal(self, context):
        s = self.load(); s['calls'].append(copy.deepcopy(context)); self.save(s)
        response = ({'abstain': 'Authored justified retention after two observed cycles'}
                    if context['phase'] == 'done' else
                    {'state_root': context['state_root'], 'proposals': [{'authored_kind': context['phase']}],
                     'observed_receipt': context['observed_receipt']})
        return self.meter('response', response)

    def review(self, context, response):
        s = self.load(); s['reviews'].append(copy.deepcopy(response)); self.save(s)
        return self.meter('judgment', dict(decision='approve', reviewer='independent-authored-fixture',
                           generator='authored-fixture', response_digest=context['response_digest']))

    def native(self, kind, value):
        s = self.load()
        if kind in ('request', 'response_context'): return self.request()
        if kind == 'preview': return {'cycle': self.request(), 'response_digest': m.e.sha(value['response'])}
        if kind == 'advance':
            assert value['binding'] == self.request()['binding']
            assert value['review']['response_digest'] == m.e.sha(value['response'])
            reflection = s['phase'] == 'reflection'
            s['revision'] += 1; s['branch'] = 'review_required' if reflection else 'resume'
            self.save(s)
            return {'qualitative_admitted': reflection, 'next': self.request()}
        if kind == 'resume':
            assert s['branch'] == 'resume'
            receipt = 'authored-receipt-'+str(len(s['actions'])+1)
            s['actions'].append(receipt); s['revision'] += 1
            s['phase'] = 'reflection'; s['branch'] = 'propose'; self.save(s)
            return {'receipt': receipt, 'snapshot': self.request()['source_snapshot']}
        if kind == 'feedback_witness':
            receipt = value['execution']['receipt']
            assert receipt == s['actions'][-1] == value['response']['observed_receipt']
            assert value['context']['observed_receipt'] == receipt
            assert value['judgment']['decision'] == 'approve'
            assert value['execution']['snapshot'] == self.request()['source_snapshot']
            s['witnesses'].append(receipt); self.save(s)
            return dict(reviewed_return=not s['missing'], execution_receipt=receipt,
                        source_snapshot=value['execution']['snapshot'])
        if kind == 'continue_session':
            assert value['binding'] == self.request()['binding']
            assert value['index'] == len(s['continuations'])
            s['continuations'].append(value); s['revision'] += 1; s['branch'] = 'propose'
            s['phase'] = 'done' if len(s['actions']) == 2 else 'action'; self.save(s)
            return {'next': self.request()}
        raise AssertionError('Unexpected callback: '+kind)


class OngoingCycle(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve()
        self.world = AuthoredWorld(self.root/'authored-state.json')
        self.a = dict(version='local-project-cycle@v3', journal=str(self.root/'cycle'),
                      scope='authored-scope', max_decisions=8, max_executions=2,
                      expires_at=(datetime.datetime.now(datetime.timezone.utc)+datetime.timedelta(hours=1)).isoformat(),
                      implementation_sha256=m.d.file_sha(Path(m.__file__)),
                      local_source_profile='browser-scoped-source@v1', stage_receipts='durable-stage-receipts@v1',
                      feedback_cycle=dict(version='authorized-feedback-cycle@v2', authorization_id='authored-only', max_continuations=8))

    def runner(self, hook=None):
        self.world = AuthoredWorld(self.root/'authored-state.json')
        return m.Cycle(self.root/'cycle', self.a, self.world.native, self.world.proposal, self.world.review, hook=hook)

    def assert_complete(self):
        result = self.runner().run(); self.assertEqual(result['status'], 'proposal_abstention')
        s = self.world.load()
        self.assertEqual(s['actions'], ['authored-receipt-1', 'authored-receipt-2'])
        self.assertEqual(s['witnesses'], s['actions']); self.assertEqual(len(s['continuations']), 2)
        self.assertEqual(len(s['calls']), 5); self.assertEqual(len(s['reviews']), 4)
        done = self.runner().read()[2]
        self.assertEqual([done[k]['execution_receipt'] for k in done if k.startswith('feedback_completion_')], s['actions'])
        self.assertEqual(sum(k.startswith('pursuit_after_') for k in done), 2)
        before = {p.name: p.read_bytes() for p in (self.root/'cycle/cycle.jsonl', self.world.path)}
        self.assertEqual(self.runner().run(), result)
        self.assertEqual(before, {p.name: p.read_bytes() for p in (self.root/'cycle/cycle.jsonl', self.world.path)})

    def test_two_cycles_and_terminal_replay(self): self.assert_complete()

    def test_crash_after_durable_witness_and_continuation(self):
        for event, name in [('after_receipt', 'feedback_completion_1'), ('after_result', 'feedback_completion_3'),
                            ('after_receipt', 'session_continue_0'), ('after_result', 'session_continue_1'),
                            ('after_receipt', 'pursuit_after_advance_1')]:
            with self.subTest(event=event, name=name), tempfile.TemporaryDirectory() as directory:
                self.root = Path(directory).resolve(); self.a['journal'] = str(self.root/'cycle')
                hit = []
                def crash(at, stage):
                    if (at, stage) == (event, name): hit.append(stage); raise RuntimeError('authored interruption')
                with self.assertRaisesRegex(RuntimeError, 'authored interruption'): self.runner(crash).run()
                self.assertEqual(hit, [name]); self.assert_complete()

    def test_legacy_policy_still_stops_after_first_reviewed_return(self):
        self.a['feedback_cycle'].update(version='authorized-feedback-cycle@v1', max_continuations=4)
        result = self.runner().run(); self.assertEqual(result['status'], 'feedback_cycle_complete')
        s = self.world.load(); self.assertEqual(len(s['actions']), 1); self.assertEqual(s['continuations'], [])
        self.assertIn('feedback_completion', self.runner().read()[2])

    def test_missing_reviewed_return_stops_without_next_choice(self):
        s = self.world.load(); s['missing'] = True; self.world.save(s)
        result = self.runner().run(); self.assertEqual(result['status'], 'post_action_advisory_admitted')
        self.assertFalse(result['reviewed_return']); self.assertEqual(self.world.load()['continuations'], [])
        self.assertEqual(len(self.world.load()['actions']), 1)

    def test_continuation_budget_does_not_renew_itself(self):
        self.a['feedback_cycle']['max_continuations'] = 1
        result = self.runner().run(); self.assertEqual(result['status'], 'continuation_budget')
        s = self.world.load(); self.assertEqual(len(s['continuations']), 1)
        self.assertEqual(len(s['actions']), 2); self.assertEqual(len(s['calls']), 4)
        self.assertEqual(self.runner().run(), result)

    def test_unknown_witness_is_not_replayed(self):
        runner = self.runner(); original = runner.native
        def interrupted(kind, value):
            if kind == 'feedback_witness': raise TimeoutError('authored unknown callback outcome')
            return original(kind, value)
        runner.native = interrupted
        with self.assertRaises(TimeoutError): runner.run()
        before = self.world.path.read_bytes()
        with self.assertRaisesRegex(m.d.Blocked, 'uncertain'): self.runner().run()
        self.assertEqual(before, self.world.path.read_bytes())


if __name__ == '__main__': unittest.main()
