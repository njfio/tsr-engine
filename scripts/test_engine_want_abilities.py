"""Spec #11: a want for a missing ability gets acted on, offline. No provider calls.

The authored class uses the real native controller and an authored transport. These pin the
order, stops and refusals. They are not evidence about what a model will write or want.
"""
import copy
import json
import os
import random
import shutil
import subprocess
import unittest
from pathlib import Path

from scripts.test_engine_browser_adapter import BrowserAdapter, m

abilities = m.load('abilities_test', 'engine-abilities.py')
reading = m.load('abilities_reading_test', 'engine-reading.py')

WANT = {'want': 'To run the game under real controls', 'sense': None, 'belief': None, 'expect': None,
        'expected_return': {'from': 'the browser', 'what': 'what the controls do'}, 'falsifier': 'the controls do what the source claims'}
NEED = {'need': {'what': 'A way to press the controls', 'because': 'I cannot tell if crossing works'}}
PLAY = {'play': {'actions': [{'kind': 'click', 'value': '#start'}, {'kind': 'key', 'value': 'ArrowUp'}],
                 'checks': [{'selector': 'body', 'text': 'Start'}]}}
RETAIN = {'retain': 'Authored retention.'}
EDIT = {'alternatives': [{'edits': [{'path': 'index.html', 'old_text': 'Start', 'new_text': 'Play'}], 'rationale': 'Authored edit'}]}
REPORT = {'version': 'browser-observation@v1', 'passed': False, 'files': {}, 'error_count': 1, 'errors': ['console error: blocked inline style'],
          'observations': [{'kind': 'click', 'value': '#start'}, {'kind': 'text', 'selector': 'body', 'text': 'Start', 'met': True, 'abbreviated': False, 'original_length': 5}]}


class Shape(unittest.TestCase):
    def test_spec_c12_need_needs_both_texts(self):
        abilities.check_need(NEED['need'])
        missing = {'what': 'x'}
        for value in (missing, {'what': 'x', 'because': ' '}, {'what': 'x', 'because': 3}, {'what': 'x', 'because': 'y', 'z': 'w'},
                      {'what': 'x' * (abilities.TEXT_MAX + 1), 'because': 'y'}, [], None, 'text'):
            with self.subTest(value=str(value)[:40]), self.assertRaises(ValueError):
                abilities.check_need(value)

    def test_spec_c12_play_stays_inside_the_harness_contract(self):
        abilities.check_play(PLAY['play'])
        key = {'kind': 'key', 'value': 'ArrowUp'}
        check = {'selector': 'body', 'text': ''}
        bad = [{}, {'actions': []}, {'actions': [], 'checks': []}, {'actions': [key] * 65, 'checks': [check]},
               {'actions': [], 'checks': [check] * 9}, {'actions': [{'kind': 'eval', 'value': 'x'}], 'checks': [check]},
               {'actions': [{'kind': 'wait', 'value': 1001}], 'checks': [check]}, {'actions': [{'kind': 'wait', 'value': True}], 'checks': [check]},
               {'actions': [{'kind': 'key', 'value': ''}], 'checks': [check]}, {'actions': [{'kind': 'key', 'value': 'x' * 129}], 'checks': [check]},
               {'actions': [], 'checks': [{'selector': '', 'text': ''}]}, {'actions': [], 'checks': [{'selector': 'x', 'text': 'y' * 129}]},
               {'actions': [], 'checks': [check], 'assets': ['x.js']}, {'actions': [key] * 64, 'checks': [{'selector': 'x' * 128, 'text': 'y' * 128}] * 8, 'pad': 1}, None]
        for value in bad:
            with self.subTest(value=str(value)[:60]), self.assertRaises(ValueError):
                abilities.check_play(value)

    def test_spec_c12_play_over_the_probe_file_limit_is_refused(self):
        big = {'actions': [{'kind': 'click', 'value': 'x' * 128}] * 64, 'checks': [{'selector': 'b', 'text': ''}]}
        with self.assertRaises(ValueError):
            abilities.check_play(big)

    def test_parsers_survive_random_values(self):
        rng = random.Random(11)
        atoms = [None, '', ' ', 'x', 'x' * 5000, 3, 1001, True, [], {}, [{'kind': 'key', 'value': 'a'}], [{'selector': 'b', 'text': ''}]]
        keys = ['what', 'because', 'actions', 'checks', 'kind', 'value', 'selector', 'assets']
        for _ in range(20000):
            value = {k: rng.choice(atoms) for k in rng.sample(keys, rng.randint(0, len(keys)))}
            for check in (abilities.check_need, abilities.check_play):
                try:
                    check(value)
                except ValueError:
                    pass


class Admission(unittest.TestCase):
    def test_spec_c02_profile_needs_a_reading_and_a_positive_cap(self):
        good = abilities.admission_fields(abilities.PROFILE, reading.PROFILE, 1000)
        self.assertEqual(good, {'abilities_profile': abilities.PROFILE, 'spend_cap_microusd': 1000})
        self.assertEqual(abilities.admission_fields(None, reading.PROFILE, None), {})
        self.assertEqual(abilities.admission_fields(None, None, None), {})
        for profile, read, cap in ((abilities.PROFILE, None, 1000), (abilities.PROFILE, reading.PROFILE, 0), (abilities.PROFILE, reading.PROFILE, -1),
                                   (abilities.PROFILE, reading.PROFILE, None), (abilities.PROFILE, reading.PROFILE, True), (abilities.PROFILE, reading.PROFILE, 1.5),
                                   ('other', reading.PROFILE, 1000), (None, reading.PROFILE, 1000)):
            with self.subTest(profile=profile, read=read, cap=cap), self.assertRaises(ValueError):
                abilities.admission_fields(profile, read, cap)


class Returns(unittest.TestCase):
    def test_play_return_carries_errors_and_readouts_with_no_verdict(self):
        out = abilities.play_return({'exit': 1, 'report': REPORT, 'stderr': ''}, PLAY['play'], 'a' * 64, 'r' * 64)
        self.assertEqual(out['reading'], 'r' * 64)
        self.assertRegex(out['evidence'], '^[0-9a-f]{64}$')
        for text in ('blocked inline style', 'body', 'Start', 'exit 1', 'a' * 64):
            self.assertIn(text, out['text'])
        self.assertNotIn('passed', out['text'])
        self.assertNotIn('verdict', out['text'])

    def test_spec_c06_harness_refusal_and_timeout_come_back_as_text(self):
        for run, expected in (({'exit': 2, 'report': None, 'stderr': 'browser harness unavailable or failed: asset scope'}, 'asset scope'),
                              ({'exit': None, 'report': None, 'stderr': 'timed out after 170 seconds'}, 'timed out')):
            with self.subTest(expected=expected):
                out = abilities.play_return(run, PLAY['play'], 'a' * 64, None)
                self.assertIn(expected, out['text'])

    def test_return_text_stays_inside_the_native_bound(self):
        huge = {**REPORT, 'errors': ['é' * 200] * 8, 'observations': [{'kind': 'text', 'selector': 's' * 128, 'text': 'é' * 256, 'met': False}] * 8}
        out = abilities.play_return({'exit': 1, 'report': huge, 'stderr': 'é' * 9000}, PLAY['play'], 'a' * 64, None)
        self.assertLessEqual(len(out['text'].encode()), abilities.TEXT_MAX)
        self.assertIn('omitted', out['text'])

    def test_long_play_keeps_errors_and_readouts_ahead_of_the_action_list(self):
        actions = [{'kind': 'key', 'value': 'ArrowUp'}] * 64
        checks = [{'kind': 'text', 'selector': '#c' + str(i), 'text': 'x' * 200, 'met': True} for i in range(8)]
        report = {**REPORT, 'observations': actions + checks}
        out = abilities.play_return({'exit': 1, 'report': report, 'stderr': ''}, PLAY['play'], 'a' * 64, None)
        self.assertLessEqual(len(out['text'].encode()), abilities.TEXT_MAX)
        self.assertIn('#c7', out['text'])
        self.assertIn('blocked inline style', out['text'])

    def test_need_return_quotes_the_need_and_names_what_it_revealed(self):
        out = abilities.need_return(NEED['need'], ['play'], 'r' * 64)
        self.assertIn(NEED['need']['what'], out['text'])
        self.assertIn('play', out['text'])

    def test_probe_is_built_from_the_protected_probe_and_the_models_controls(self):
        protected = {'version': 'browser-probe@v1', 'assets': ['index.html', 'app.js'], 'entry': 'index.html', 'actions': [], 'checks': [{'selector': 'body', 'text': ''}]}
        probe = abilities.probe(protected, PLAY['play'])
        self.assertEqual(probe, {'version': 'browser-probe@v1', 'assets': ['index.html', 'app.js'], 'entry': 'index.html', **PLAY['play']})


class Prompts(unittest.TestCase):
    def test_spec_c03_form_text_hides_play_until_a_need(self):
        self.assertNotIn('play', abilities.system_text([]).lower())
        self.assertIn('need', abilities.system_text([]))
        self.assertIn('play', abilities.system_text(['play']))
        self.assertEqual(sorted(abilities.forms([])), ['need'])
        self.assertEqual(sorted(abilities.forms(['play'])), ['need', 'play'])
        self.assertNotIn('play', abilities.contracts([]))
        self.assertIn('play', abilities.contracts(['play']))

    def test_prompt_says_nothing_false_about_this_loop(self):
        for revealed in ([], ['play']):
            text = abilities.proposal_system(reading.PROPOSAL_SYSTEM, revealed)
            self.assertNotIn('Retention ends this checkpoint', text)
            self.assertNotIn('scheduled session', text)
            self.assertIn('abstain', text)
        with self.assertRaises(ValueError):
            abilities.proposal_system('text without the sentences', [])

    def test_response_envelope_names_what_is_offered(self):
        self.assertIn('{need:', abilities.envelope(reading.RESPONSE_ENVELOPE, [])['instruction'])
        self.assertNotIn('{play:', abilities.envelope(reading.RESPONSE_ENVELOPE, [])['instruction'])
        self.assertIn('{play:', abilities.envelope(reading.RESPONSE_ENVELOPE, ['play'])['instruction'])
        self.assertNotIn('need', reading.RESPONSE_ENVELOPE['instruction'])

    def test_observable_signals_are_counts_in_journal_order(self):
        keys = ['reading_0', 'advance_0', 'resume_0', 'ability_need_1', 'ability_play_2', 'advance_3', 'resume_1', 'ability_play_4', 'ability_retain_5', 'terminal']
        self.assertEqual(abilities.signals(keys, 'proposal_retention'),
                         {'needs': 1, 'first_need_decision': 1, 'plays': 2, 'plays_followed_by_an_edit': 1, 'edits_followed_by_a_play': 2, 'retentions': 1, 'stop': 'proposal_retention'})
        self.assertIsNone(abilities.signals(['reading_0'], 'x')['first_need_decision'])

    def test_evidence_since_last_retention(self):
        since = abilities.new_evidence
        self.assertFalse(since(['reading_0', 'proposal_0']))
        self.assertTrue(since(['reading_0', 'rejected_proposal_0']))
        self.assertTrue(since(['resume_0', 'proposal_1']))
        self.assertTrue(since(['ability_need_0', 'proposal_1']))
        self.assertTrue(since(['ability_play_0', 'proposal_1']))
        self.assertFalse(since(['ability_play_0', 'ability_retain_1', 'reading_2', 'proposal_2']))
        self.assertTrue(since(['ability_retain_0', 'ability_play_1', 'proposal_2']))


class Launcher(unittest.TestCase):
    """The launch can't go ahead on a review that didn't accept these exact files."""
    def setUp(self):
        import importlib.util
        import tempfile
        spec = importlib.util.spec_from_file_location('want_loop_run', Path(__file__).parents[1] / 'experiments/want-loop-v1/run.py')
        self.run_module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.run_module)
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.folder = Path(self.tmp.name)
        for name in self.run_module.REVIEWED.values():
            (self.folder / name).write_text(name)

    def review(self, **changes):
        value = {'decision': 'accept', **{f: self.run_module.sha(self.folder / n) for f, n in self.run_module.REVIEWED.items()}, **changes}
        (self.folder / 'binding-review.json').write_text(json.dumps(value))

    def test_launch_accepts_a_review_of_these_files(self):
        self.review()
        self.run_module.check_review(self.folder)

    def test_launch_refuses_without_a_matching_accepted_review(self):
        with self.assertRaises(SystemExit):
            self.run_module.check_review(self.folder)
        self.review(decision='reject')
        with self.assertRaises(SystemExit):
            self.run_module.check_review(self.folder)
        for field in self.run_module.REVIEWED:
            self.review(**{field: 'f' * 64})
            with self.subTest(field=field), self.assertRaises(SystemExit):
                self.run_module.check_review(self.folder)

    def test_brief_does_not_claim_an_operator_runs_the_playtests(self):
        text = (Path(__file__).parents[1] / 'experiments/want-loop-v1/prepare.py').read_text()
        keys = text[text.index('FACT_KEYS=('):].split('\n', 1)[0]
        self.assertNotIn('observation_policy', keys)
        self.assertNotIn("'feedback'", keys)

    def test_launch_files_name_no_stage_scripts(self):
        text = (Path(__file__).parents[1] / 'experiments/want-loop-v1/prepare.py').read_text()
        for name in ('observe.py', 'continue.py', 'stage.py', 'discover.py'):
            self.assertNotIn(name, text)


class Cycle(BrowserAdapter):
    def configure(self, cap=10 ** 9, profile=None, decisions=8):
        profile = abilities.PROFILE if profile is None else profile
        self.a = m.admission(self.root / 'adapter', self.a['origin'], self.c.cli('project', 'config'), self.manifest, self.a['expires_at'],
                             max_decisions=decisions, max_executions=2, correction_allowance=1, feedback_events=[],
                             feedback_cycle={'version': 'authorized-feedback-cycle@v2', 'authorization_id': 'authored-abilities', 'max_continuations': 4},
                             reading_profile=reading.PROFILE, abilities_profile=profile or None, spend_cap_microusd=cap if profile else None)
        a = self.adapter()
        self.adapter_instance = a
        ca = {'version': 'local-project-cycle@v3', 'journal': str(self.root / 'abilities-cycle'), 'scope': a.native('request', None)['scope'], 'max_decisions': decisions, 'max_executions': 2,
              'expires_at': self.a['expires_at'], 'implementation_sha256': m.d.file_sha(Path(m.loop.__file__)), 'local_source_profile': 'browser-scoped-source@v1',
              'stage_receipts': 'durable-stage-receipts@v1', 'review_feedback': 'attributed-review-feedback@v1', 'completion_tokens': 32768, 'input_tokens': self.a['input_tokens'],
              'max_review_rejections': 3, 'feedback_cycle': self.a['feedback_cycle']}
        return a, ca

    def cycle(self, a, ca):
        spec = __import__('importlib.util').util.spec_from_file_location('abilities_native_cycle', Path(m.__file__).with_name('engine-native-cycle.py'))
        launcher = __import__('importlib.util').util.module_from_spec(spec)
        spec.loader.exec_module(launcher)
        return launcher.build_cycle(m.loop.Cycle, ca, a)

    def script(self, a, proposals, run=None, reject_reviews=False):
        """Authored turns: a want on each reading, then each proposal in order, then retention."""
        original = self.transport
        remaining = list(proposals)
        seen = []
        ran = []

        def transport(body, timeout):
            out = original(body, timeout)
            data = json.loads(body['messages'][1]['content'])
            kind = 'review' if 'criteria' in data else 'reading' if 'reading_contract' in data else 'proposal'
            seen.append((kind, copy.deepcopy(body)))
            if kind == 'reading':
                out['value'] = copy.deepcopy(WANT)
            elif kind == 'proposal':
                out['value'] = copy.deepcopy(remaining.pop(0) if remaining else RETAIN)
            elif reject_reviews:
                out['value']['verdict'] = 'reject'
                next(iter(out['value']['findings'].values())).update(finding='not_met', explanation='Authored criticism')
            return out

        def runner(files, probe, harness, image, user):
            ran.append({'files': files, 'probe': probe, 'harness': harness, 'image': image, 'user': user})
            return run(len(ran)) if run else {'exit': 1, 'report': REPORT, 'stderr': ''}
        a.injected_transport = transport
        a.play_runner = runner
        return seen, ran

    def kinds(self, seen):
        return [k for k, _ in seen]

    def returns(self):
        return [r for entry in self.c.cli('project', 'reading-view')['bounded']['readings'] for r in entry['returns']]

    def test_spec_c01_reading_only_run_refuses_a_need_and_keeps_its_prompt(self):
        a, ca = self.configure(profile=False)
        seen, ran = self.script(a, [NEED, RETAIN, RETAIN])
        self.assertIsNone(a.abilities_callback())
        result = self.cycle(a, ca).run()
        self.assertEqual(ran, [])
        proposal = [b for k, b in seen if k == 'proposal'][0]
        self.assertEqual(proposal['messages'][0]['content'], reading.PROPOSAL_SYSTEM)
        self.assertEqual(json.loads(proposal['messages'][1]['content'])['response_envelope'], reading.RESPONSE_ENVELOPE)
        self.assertNotIn('need', json.loads(proposal['messages'][1]['content'])['choice_summary']['forms'])
        self.assertEqual(result['status'], 'proposal_retention')
        self.assertEqual(self.returns(), [])

    def test_spec_c03_first_request_offers_need_not_play(self):
        a, ca = self.configure()
        seen, ran = self.script(a, [PLAY, RETAIN, RETAIN])
        self.cycle(a, ca).run()
        data = json.loads([b for k, b in seen if k == 'proposal'][0]['messages'][1]['content'])
        self.assertIn('need', data['choice_summary']['forms'])
        self.assertNotIn('play', data['choice_summary']['forms'])
        self.assertNotIn('play', data['response_contracts'])
        self.assertEqual(ran, [])

    def test_spec_c04_c08_need_returns_beside_the_reading_then_a_new_reading(self):
        a, ca = self.configure()
        seen, ran = self.script(a, [NEED, RETAIN, RETAIN])
        result = self.cycle(a, ca).run()
        self.assertEqual(self.kinds(seen), ['reading', 'proposal', 'reading', 'proposal', 'reading', 'proposal'])
        self.assertEqual(ran, [])
        returned = self.returns()
        self.assertEqual(len(returned), 1)
        self.assertEqual(returned[0]['from'], 'rules')
        self.assertIn(NEED['need']['what'], returned[0]['text'])
        second = json.loads([b for k, b in seen if k == 'proposal'][1]['messages'][1]['content'])
        self.assertIn('play', second['choice_summary']['forms'])
        self.assertIn('play', second['response_contracts'])
        self.assertEqual(result['status'], 'proposal_retention')

    def test_spec_c08_retention_with_nothing_new_ends_the_run_first_time(self):
        a, ca = self.configure()
        seen, _ = self.script(a, [RETAIN])
        result = self.cycle(a, ca).run()
        self.assertEqual(self.kinds(seen), ['reading', 'proposal'])
        self.assertEqual(result['status'], 'proposal_retention')

    def test_spec_c05_play_runs_on_current_source_with_no_review(self):
        a, ca = self.configure()
        before = self.c.request()['coding']['snapshot']
        seen, ran = self.script(a, [NEED, PLAY, RETAIN, RETAIN])
        self.cycle(a, ca).run()
        self.assertEqual(len(ran), 1)
        self.assertEqual(ran[0]['probe']['actions'], PLAY['play']['actions'])
        self.assertEqual(ran[0]['probe']['checks'], PLAY['play']['checks'])
        self.assertIn('Start', ran[0]['files']['index.html'])
        self.assertEqual(ran[0]['image'], m.IMAGE)
        self.assertEqual(ran[0]['user'], '501:20')
        self.assertEqual(ran[0]['harness'], self.config['protected']['browser-harness.cjs'])
        self.assertEqual(self.c.request()['coding']['snapshot'], before)
        self.assertNotIn('review', self.kinds(seen))
        text = self.returns()[-1]['text']
        self.assertIn('blocked inline style', text)
        self.assertIn(before, text)
        self.assertFalse(self.c.cli('project', 'acceptance', 'status')['accepted'])

    def test_spec_c05_later_reading_sees_what_the_play_returned(self):
        a, ca = self.configure()
        seen, _ = self.script(a, [NEED, PLAY, RETAIN, RETAIN])
        self.cycle(a, ca).run()
        readings = [json.loads(b['messages'][1]['content']) for k, b in seen if k == 'reading']
        self.assertIn('blocked inline style', json.dumps(readings[2]['earlier_readings']))

    def test_spec_c06_failed_plays_come_back_and_the_run_goes_on(self):
        a, ca = self.configure()
        outcomes = {1: {'exit': 2, 'report': None, 'stderr': 'browser harness unavailable or failed: asset scope'},
                    2: {'exit': None, 'report': None, 'stderr': 'timed out after 170 seconds'}}
        seen, ran = self.script(a, [NEED, PLAY, PLAY, RETAIN, RETAIN], run=lambda n: outcomes[n])
        result = self.cycle(a, ca).run()
        self.assertEqual(len(ran), 2)
        text = json.dumps(self.returns())
        self.assertIn('asset scope', text)
        self.assertIn('timed out', text)
        self.assertEqual(result['status'], 'proposal_retention')

    def test_spec_c07_second_need_goes_to_the_operator(self):
        a, ca = self.configure()
        second = {'need': {'what': 'A human playtester', 'because': 'Only a person can say if it is fun'}}
        seen, ran = self.script(a, [NEED, second])
        result = self.cycle(a, ca).run()
        self.assertEqual(result['status'], 'need_outside_scope')
        self.assertEqual(result['need'], second['need'])
        self.assertTrue(result['desires_remain_open'])
        self.assertFalse(result['acceptance_recorded'])
        self.assertEqual(ran, [])

    def test_spec_c09_spend_cap_stops_before_the_next_call(self):
        a, ca = self.configure(cap=1)
        seen, _ = self.script(a, [NEED])
        original = a.spend_reached
        a.spend_reached = lambda: True
        result = self.cycle(a, ca).run()
        self.assertEqual(result['status'], 'spend_cap')
        self.assertEqual(seen, [])
        a.spend_reached = original

    def test_spec_c09_spend_is_read_from_the_campaign_ledger(self):
        a, ca = self.configure(cap=500)
        a.ledger = type('L', (), {'status': lambda self: {'committed_microusd': 499}, 'manifest': a.ledger.manifest})()
        self.assertFalse(a.spend_reached())
        a.ledger = type('L', (), {'status': lambda self: {'committed_microusd': 500}, 'manifest': a.ledger.manifest})()
        self.assertTrue(a.spend_reached())

    def test_spec_c09_cap_binds_on_inspection_with_the_real_ledger(self):
        a, ca = self.configure(cap=250, decisions=8)
        looks = [{'inspect': {'path': 'app.js', 'offset': i % 2}} for i in range(9)]
        seen, _ = self.script(a, looks)
        result = self.cycle(a, ca).run()
        self.assertEqual(result['status'], 'spend_cap')
        self.assertEqual(len(seen), 3)
        self.assertEqual(self.ledger.status()['calls'], 3)

    def test_spec_c09_cap_binds_on_format_correction_with_the_real_ledger(self):
        a, ca = self.configure(cap=150)
        bad = {'alternatives': [{'edits': 'wrong scalar', 'rationale': 'Authored invalid shape'}]}
        seen, _ = self.script(a, [bad, bad, bad])
        result = self.cycle(a, ca).run()
        self.assertEqual(result['status'], 'spend_cap')
        self.assertEqual(len(seen), 2)

    def test_spec_c09_cap_stops_a_restart_too(self):
        a, ca = self.configure(cap=150)
        seen, _ = self.script(a, [RETAIN])
        a.spend_reached = lambda: False
        first = self.cycle(a, ca).run()
        fresh = self.adapter()
        self.assertEqual(self.cycle(fresh, ca).run(), first)

    def test_spec_c09_spend_cap_stops_before_a_review(self):
        a, ca = self.configure()
        seen, _ = self.script(a, [EDIT])
        count = iter([False, False, True])
        a.spend_reached = lambda: next(count, True)
        result = self.cycle(a, ca).run()
        self.assertEqual(result['status'], 'spend_cap')
        self.assertNotIn('review', self.kinds(seen))

    def test_spec_c10_an_edit_after_a_play_still_needs_review(self):
        self.playable = False
        a, ca = self.configure()
        seen, ran = self.script(a, [NEED, PLAY, EDIT])
        self.cycle(a, ca).run()
        self.assertEqual(len(ran), 1)
        self.assertIn('review', self.kinds(seen))

    def test_spec_c10_a_rejected_edit_after_a_play_is_not_run(self):
        a, ca = self.configure()
        before = self.c.request()['coding']['snapshot']
        seen, ran = self.script(a, [NEED, PLAY, EDIT, RETAIN, RETAIN], reject_reviews=True)
        self.cycle(a, ca).run()
        self.assertIn('review', self.kinds(seen))
        self.assertEqual(self.c.request()['coding']['snapshot'], before)

    def test_spec_c11_restart_in_mid_run_does_not_repeat_the_play(self):
        a, ca = self.configure()
        _, ran = self.script(a, [NEED, PLAY, RETAIN, RETAIN])
        first = self.cycle(a, ca)

        class Killed(BaseException):
            pass

        def hook(point, name):
            if point == 'after_result' and name.startswith('ability_play_'):
                raise Killed()
        first.hook = hook
        with self.assertRaises(Killed):
            first.run()
        self.assertEqual(len(ran), 1)
        returned = len(self.returns())
        fresh = self.adapter()
        again = []
        self.script(fresh, [RETAIN, RETAIN], run=lambda n: again.append(n))
        result = self.cycle(fresh, ca).run()
        self.assertEqual(again, [])
        self.assertEqual(len(ran), 1)
        self.assertEqual(len(self.returns()), returned)
        self.assertEqual(result['status'], 'proposal_retention')

    def test_spec_c11_restart_runs_nothing_twice(self):
        a, ca = self.configure()
        _, ran = self.script(a, [NEED, PLAY, RETAIN, RETAIN])
        result = self.cycle(a, ca).run()
        view = self.c.cli('project', 'reading-view')
        calls = self.ledger.status()['calls']
        fresh = self.adapter()
        again = []
        fresh.play_runner = lambda *_: again.append(1)
        self.assertEqual(self.cycle(fresh, ca).run(), result)
        self.assertEqual(again, [])
        self.assertEqual(self.c.cli('project', 'reading-view'), view)
        self.assertEqual(self.ledger.status()['calls'], calls)
        self.assertEqual(len(ran), 1)

    def test_spec_c11_crash_during_a_play_blocks_and_does_not_rerun(self):
        a, ca = self.configure()

        def boom(_):
            raise RuntimeError('authored crash')
        _, ran = self.script(a, [NEED, PLAY, RETAIN], run=boom)
        with self.assertRaises(Exception):
            self.cycle(a, ca).run()
        fresh = self.adapter()
        again = []
        fresh.play_runner = lambda *_: again.append(1)
        with self.assertRaises(Exception):
            self.cycle(fresh, ca).run()
        self.assertEqual(again, [])
        self.assertEqual(len(ran), 1)

    @unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED') == '1' and shutil.which('docker'), 'real local harness opt-in')
    def test_spec_c13_real_harness_runs_the_models_controls(self):
        harness = self.config['protected']['browser-harness.cjs']
        files = {'index.html': '<button id="start" onclick="">Start</button><p id="out">idle</p><script src="app.js"></script>',
                 'app.js': "document.getElementById('start').addEventListener('click',()=>{document.getElementById('out').textContent='clicked'});"}
        probe = {'version': 'browser-probe@v1', 'assets': list(files), 'entry': 'index.html', 'actions': [{'kind': 'click', 'value': '#start'}], 'checks': [{'selector': '#out', 'text': 'clicked'}]}
        out = abilities.run_harness(files, probe, harness, m.IMAGE, '501:20')
        self.assertIsNotNone(out['report'], out['stderr'])
        kinds = [o['kind'] for o in out['report']['observations']]
        self.assertIn('click', kinds)
        self.assertIn({'kind': 'text', 'selector': '#out', 'text': 'clicked', 'abbreviated': False, 'original_length': 7, 'met': True},
                      out['report']['observations'])


for name in dir(BrowserAdapter):
    if name.startswith('test_') and name not in Cycle.__dict__:
        setattr(Cycle, name, None)

if __name__ == '__main__':
    unittest.main()
