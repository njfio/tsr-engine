"""Spec #12: the process run the way The Sovereign Mirror describes it, offline. No provider calls.

The unit classes pin shapes, prompts, detection and tuning. The authored class uses the real native
controller with an authored transport and runner: it pins order, evidence and refusals. None of it
is evidence about what a model will write or want.
"""
import copy
import json
import os
import random
import unittest
from pathlib import Path

from scripts.test_engine_browser_adapter import m
from scripts import test_engine_want_abilities as wa

process = m.load('tsr_process_test', 'engine-process.py')
reading = m.load('tsr_process_reading_test', 'engine-reading.py')
abilities = m.load('tsr_process_abilities_test', 'engine-abilities.py')

PAGE = {'want': 'To tell whether the crossing is real', 'sense': 'The traffic looks drawn but not moving', 'belief': None,
        'expect': 'A run will show little', 'gap': 'Far: the want is a working crossing, the page shows a picture',
        'expected_return': {'from': 'the browser', 'what': 'what the controls do'}, 'falsifier': 'the controls move the frog'}
FILES = {'index.html': '<button id="start">Start</button>', 'app.js': '// empty\n'}


def exact(path, old, new):
    return {'kind': 'exact_edit', 'patch': {'path': path, 'source_digest': 'a' * 64, 'inspection_handle': 'b' * 64, 'old_text': old, 'new_text': new}, 'rationale': 'x'}


class Admission(unittest.TestCase):
    def test_spec12_ac2_profile_needs_reading_abilities_and_a_context(self):
        good = process.admission_fields(process.PROFILE, 'tuned', reading.PROFILE, abilities.PROFILE)
        self.assertEqual(good, {'tsr_process_profile': process.PROFILE, 'process_context': 'tuned'})
        self.assertEqual(process.admission_fields(None, None, reading.PROFILE, abilities.PROFILE), {})
        for args in ((process.PROFILE, 'tuned', None, abilities.PROFILE), (process.PROFILE, 'tuned', reading.PROFILE, None),
                     (process.PROFILE, 'loose', reading.PROFILE, abilities.PROFILE), (process.PROFILE, None, reading.PROFILE, abilities.PROFILE),
                     ('other', 'tuned', reading.PROFILE, abilities.PROFILE), (None, 'tuned', reading.PROFILE, abilities.PROFILE)):
            with self.subTest(args=args), self.assertRaises(ValueError):
                process.admission_fields(*args)

    def test_spec12_ac12_random_admission_values_raise_only_value_error(self):
        rng = random.Random(12)
        atoms = [None, '', 'tuned', 'full', process.PROFILE, reading.PROFILE, abilities.PROFILE, 1, True, [], {}]
        for _ in range(5000):
            try:
                process.admission_fields(*(rng.choice(atoms) for _ in range(4)))
            except ValueError:
                pass


class Notebook(unittest.TestCase):
    def test_spec12_ac8_the_notebook_prompt_does_not_ask_for_a_want(self):
        text = process.NOTEBOOK_SYSTEM
        for asking in ('Start with want', 'what do you want here?', 'Answer in your own words', 'A want can be small'):
            self.assertNotIn(asking, text)
        self.assertIn('null when nothing is there', text)
        self.assertIn('never edited', text)
        self.assertIn('gap', text)

    def test_spec12_ac8_pages_carry_gap_and_keep_the_reading_rules(self):
        process.check_page(PAGE)
        blank = {k: None for k in process.FIELDS}
        process.check_page(blank)
        for change in ({'gap': ' '}, {'gap': 3}, {'gap': 'x' * (process.TEXT_MAX + 1)}, {'falsifier': None}, {'extra': 'x'}):
            bad = {**PAGE, **change}
            with self.subTest(change=str(change)[:40]), self.assertRaises(ValueError):
                process.check_page(bad)
        without_gap = {k: v for k, v in PAGE.items() if k != 'gap'}
        with self.assertRaises(ValueError):
            process.check_page(without_gap)
        self.assertNotIn('gap', process.native_page(blank))
        self.assertEqual(process.native_page(PAGE)['gap'], PAGE['gap'])
        self.assertTrue(process.is_page(PAGE))
        self.assertFalse(reading.is_reading(PAGE))


class Try(unittest.TestCase):
    def test_spec12_candidate_copy_applies_the_edit_exactly(self):
        out = process.candidate_files(FILES, exact('index.html', 'Start', 'Play'))
        self.assertEqual(out['index.html'], '<button id="start">Play</button>')
        self.assertEqual(FILES['index.html'], '<button id="start">Start</button>')
        both = {'kind': 'edit_set', 'patches': [exact('index.html', 'Start', 'Go')['patch'], exact('app.js', '// empty', 'go()')['patch']]}
        out = process.candidate_files(FILES, both)
        self.assertEqual((out['index.html'], out['app.js']), ('<button id="start">Go</button>', 'go()\n'))

    def test_spec12_ac5_an_edit_that_does_not_apply_is_not_repaired(self):
        for proposal in (exact('index.html', 'Missing', 'x'), exact('style.css', 'a', 'b'), exact('index.html', 't', 'x')):
            with self.subTest(proposal=proposal['patch']['old_text']), self.assertRaises(process.Unbuildable):
                process.candidate_files(FILES, proposal)
        returned = process.unbuildable_return(0, 'old_text does not occur exactly once in index.html', 'c' * 64, None)
        self.assertIn('could not be tried', returned['text'])
        self.assertIn('Nothing ran', returned['text'])

    def test_spec12_only_edits_are_tried_and_at_most_four(self):
        response = {'proposals': [exact('index.html', 'Start', str(i)) for i in range(6)] + [{'kind': 'existing', 'action': 'x'}]}
        self.assertEqual([i for i, _ in process.edit_proposals(response)], list(range(process.MAX_TRIES)))
        mixed = {'proposals': [{'kind': 'existing', 'action': 'x'}, exact('index.html', 'Start', 'Go')]}
        self.assertEqual([i for i, _ in process.edit_proposals(mixed)], [1])
        self.assertEqual(process.edit_proposals({'retain': 'x'}), [])

    def test_spec12_try_return_is_labelled_and_bounded_with_no_verdict(self):
        out = process.try_return(1, {'exit': 1, 'report': wa.REPORT, 'stderr': ''}, 'd' * 64, 'r' * 64)
        self.assertTrue(out['text'].startswith('A try of proposed edit 1 ran on a copy of source ' + 'd' * 64))
        self.assertIn('the try controls you gave with this edit', out['text'])
        bare = process.try_return(0, {'exit': 0, 'report': wa.REPORT, 'stderr': ''}, 'd' * 64, None, 'protected')
        self.assertIn('which load the page', bare['text'])
        self.assertIn('blocked inline style', out['text'])
        self.assertNotIn('verdict', out['text'])
        self.assertLessEqual(len(out['text'].encode()), process.TEXT_MAX)
        self.assertEqual(out['reading'], 'r' * 64)


class DraftTry(unittest.TestCase):
    EDIT = {'edits': [{'path': 'index.html', 'old_text': 'Start', 'new_text': 'Play'}], 'rationale': 'r'}

    def test_spec12_try_controls_are_split_from_edit_candidates(self):
        value = {'alternatives': [{**self.EDIT, 'try': wa.PLAY['play']}, dict(self.EDIT)]}
        stripped, controls = process.split_tries(value)
        self.assertEqual(controls, {'0': wa.PLAY['play']})
        self.assertNotIn('try', stripped['alternatives'][0])
        self.assertIn('try', value['alternatives'][0])  # the original is not mutated
        self.assertEqual(process.split_tries({'retain': 'x'}), ({'retain': 'x'}, {}))

    def test_spec12_bad_try_controls_are_correctable_and_point_at_the_draft(self):
        import importlib.util
        spec = importlib.util.spec_from_file_location('draft_try_correction', Path(__file__).with_name('engine-bounded-correction.py'))
        correction = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(correction)
        for bad, path in (({**self.EDIT, 'try': {'actions': [], 'checks': []}}, '$.alternatives[0].try.checks'),
                          ({'rationale': 'r', 'try': wa.PLAY['play']}, '$.alternatives[0].try'),
                          ({**self.EDIT, 'try': {'actions': [{'kind': 'eval', 'value': 'x'}], 'checks': [{'selector': 'b', 'text': ''}]}}, '$.alternatives[0].try.actions[0].kind')):
            with self.subTest(path=path), self.assertRaises(ValueError) as caught:
                process.split_tries({'alternatives': [bad]})
            self.assertIn(path, caught.exception.diagnostic)

    def test_spec12_random_try_values_raise_only_value_error(self):
        rng = random.Random(13)
        atoms = [None, 'x', 3, [], {}, wa.PLAY['play'], {'actions': [], 'checks': [{'selector': 'b', 'text': ''}]}]
        for _ in range(5000):
            item = {'edits': [], 'rationale': 'r'} if rng.random() < 0.7 else {'rationale': 'r'}
            if rng.random() < 0.8:
                item['try'] = rng.choice(atoms)
            try:
                process.split_tries({'alternatives': [item]})
            except ValueError:
                pass


class HarnessEvidence(unittest.TestCase):
    def setUp(self):
        self.review = m.load('harness_review_contract', 'engine-project-runtime-review.py')
        self.output = m.load('harness_review_output', 'engine-project-output.py')
        self.binding = {'review_id': 'r', 'evidence_ids': ['context:a', 'harness:b', 'receipt:c'], 'observed_receipts': ['receipt:c'], 'harness_returns': ['harness:b']}

    def report(self, basis, evidence):
        finding = {'finding': 'met', 'basis': basis, 'evidence': evidence, 'explanation': 'e', 'limitations': 'l'}
        return {'review_id': 'r', 'verdict': 'accept', 'findings': {k: dict(finding) for k in self.review.CRITERIA}}

    def test_spec12_a_harness_return_is_cited_with_its_own_basis(self):
        self.assertEqual(self.review.validate(self.report('observed_harness_return', ['harness:b']), self.binding), 'accept')
        for basis, evidence in (('observed_harness_return', ['context:a']), ('observed_test_receipt', ['harness:b'])):
            with self.subTest(basis=basis), self.assertRaises(ValueError):
                self.review.validate(self.report(basis, evidence), self.binding)
        without = {k: v for k, v in self.binding.items() if k != 'harness_returns'}
        with self.assertRaises(ValueError):
            self.review.validate(self.report('observed_harness_return', ['harness:b']), without)

    def test_spec12_review_schema_offers_the_basis_only_with_harness_returns(self):
        enum = lambda b: self.output.review_schema(b, self.review.CRITERIA)['properties']['findings']['properties']['parent_fidelity']['properties']['basis']['enum']
        self.assertIn('observed_harness_return', enum(self.binding))
        self.assertNotIn('observed_harness_return', enum({k: v for k, v in self.binding.items() if k != 'harness_returns'}))


class Detection(unittest.TestCase):
    def test_spec12_ac9_a_want_counts_only_when_new_after_a_return(self):
        timeline = [
            {'kind': 'reading', 'id': 'r0', 'want': 'build the game'},           # first: never counts
            {'kind': 'reading', 'id': 'r1', 'want': 'build it faster'},          # new, but nothing came back
            {'kind': 'return', 'id': 'x1', 'text': 'review rejected: timers uninitialized'},
            {'kind': 'reading', 'id': 'r2', 'want': 'tell real mechanics from described'},  # counts
            {'kind': 'return', 'id': 'x2', 'text': 'try ran: exit 1'},
            {'kind': 'reading', 'id': 'r3', 'want': 'tell real mechanics from described'},  # same want: no
            {'kind': 'return', 'id': 'x3', 'text': 'play ran'},
            {'kind': 'reading', 'id': 'r4', 'want': None},                       # null: no
        ]
        found = process.detect(timeline)
        self.assertEqual([d['reading'] for d in found], ['r2'])
        self.assertEqual(found[0]['previous_want'], 'build it faster')
        self.assertEqual([r['id'] for r in found[0]['after_returns']], ['x1'])
        with self.assertRaises(ValueError):
            process.detect([{'kind': 'other'}])

    def test_spec12_timeline_follows_the_native_view(self):
        view = {'readings': [{'event': 'r0', 'reading': {'want': 'a'}, 'returns': [{'event': 'x0', 'text': 'came back'}]},
                             {'event': 'r1', 'reading': {'want': 'b'}, 'returns': []}]}
        self.assertEqual([d['reading'] for d in process.detect(process.timeline_from_view(view))], ['r1'])


class Tuning(unittest.TestCase):
    def context(self):
        return {'public_brief': 'brief', 'files': [{'path': 'index.html'}],
                'review_criticism': {'status': 's', 'items': [{'n': i, 'text': 'x' * 400} for i in range(6)]},
                'browser_returns': [{'n': i} for i in range(5)], 'independent_assessments': [{'n': i, 'text': 'y' * 400} for i in range(4)],
                'relational_history': [{'n': i} for i in range(10)], 'feedback_provenance': [1, 2],
                'current_reading': {'readings': [{'event': str(i)} for i in range(6)], 'omitted': 0},
                'response_contracts': {'play': {}}, 'choice_summary': {'forms': {'play': 'x'}}}

    def test_spec12_ac10_tuned_keeps_the_latest_of_each_and_is_smaller(self):
        full = self.context()
        tuned = process.tune(full)
        self.assertEqual(len(tuned['review_criticism']['items']), 1)
        self.assertEqual(tuned['review_criticism']['items'][0]['n'], 5)
        self.assertEqual(len(tuned['browser_returns']), 1)
        self.assertEqual(len(tuned['independent_assessments']), 1)
        self.assertEqual(len(tuned['current_reading']['readings']), 3)
        self.assertNotIn('relational_history', tuned)
        for key in ('public_brief', 'files', 'response_contracts', 'choice_summary'):
            self.assertEqual(tuned[key], full[key])
        self.assertLess(len(json.dumps(tuned)), len(json.dumps(full)))
        self.assertEqual(full, self.context())  # the full context is not mutated

    def test_spec12_prompt_offers_play_and_describes_tries_without_asking(self):
        text = process.proposal_system(reading.PROPOSAL_SYSTEM, capped=False)
        self.assertIn('{play:', text)
        self.assertIn('tried before it is reviewed', text)
        self.assertNotIn('need', text.lower().replace('needs no review', ''))
        self.assertNotIn('spending cap', text)
        self.assertIn('spending cap', process.proposal_system(reading.PROPOSAL_SYSTEM, capped=True))
        self.assertEqual(sorted(process.forms()), ['edit_try', 'play'])
        self.assertIn('{play:', process.envelope(reading.RESPONSE_ENVELOPE)['instruction'])
        self.assertIn('optional try:{actions,checks}', process.envelope(reading.RESPONSE_ENVELOPE)['instruction'])
        self.assertIn('may carry try:{actions,checks}', text)
        self.assertNotIn('need', process.envelope(reading.RESPONSE_ENVELOPE)['instruction'])


class Report(unittest.TestCase):
    def test_report_lists_detected_wants_from_the_view(self):
        import importlib.util
        import tempfile
        spec = importlib.util.spec_from_file_location('process_report', Path(__file__).parents[1] / 'experiments/tsr-process-v1/report.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        with tempfile.TemporaryDirectory() as tmp:
            view = {'bounded': {'readings': [{'event': 'r0', 'reading': {'want': 'a'}, 'returns': [{'event': 'x', 'text': 't'}]},
                                             {'event': 'r1', 'reading': {'want': 'b'}, 'returns': []}], 'omitted': 0}}
            value = module.report(Path(tmp), view=view)
        self.assertEqual([d['reading'] for d in value['detected_wants']], ['r1'])
        self.assertEqual(value['tries'], 0)
        self.assertFalse(value['person_return'])


@unittest.skipUnless(os.getenv('TSR_BROWSER_ADAPTER_AUTHORED') == '1', 'explicit authored native integration opt-in')
class Cycle(wa.Cycle):
    def configure(self, cap=10 ** 9, profile=None, decisions=8, context='tuned'):
        self.a = m.admission(self.root / 'adapter', self.a['origin'], self.c.cli('project', 'config'), self.manifest, self.a['expires_at'],
                             max_decisions=decisions, max_executions=2, correction_allowance=1, feedback_events=[],
                             feedback_cycle={'version': 'authorized-feedback-cycle@v2', 'authorization_id': 'authored-process', 'max_continuations': 4},
                             reading_profile=reading.PROFILE, abilities_profile=abilities.PROFILE, spend_cap_microusd=cap,
                             tsr_process_profile=process.PROFILE, process_context=context)
        a = self.adapter()
        self.adapter_instance = a
        ca = {'version': 'local-project-cycle@v3', 'journal': str(self.root / 'process-cycle'), 'scope': a.native('request', None)['scope'], 'max_decisions': decisions, 'max_executions': 2,
              'expires_at': self.a['expires_at'], 'implementation_sha256': m.d.file_sha(Path(m.loop.__file__)), 'local_source_profile': 'browser-scoped-source@v1',
              'stage_receipts': 'durable-stage-receipts@v1', 'review_feedback': 'attributed-review-feedback@v1', 'completion_tokens': 32768, 'input_tokens': self.a['input_tokens'],
              'max_review_rejections': 3, 'feedback_cycle': self.a['feedback_cycle']}
        return a, ca

    def script(self, a, proposals, run=None, reject_reviews=False):
        seen, ran = super().script(a, proposals, run=run, reject_reviews=reject_reviews)
        inner = a.injected_transport
        order = []

        def transport(body, timeout):
            out = inner(body, timeout)
            if seen and seen[-1][0] == 'reading':
                out['value'] = copy.deepcopy(PAGE)
            order.append(seen[-1][0])
            return out
        runner = a.play_runner

        def counted(*args):
            order.append('run')
            return runner(*args)
        a.injected_transport = transport
        a.play_runner = counted
        self.order = order
        return seen, ran

    def test_spec12_ac3_play_is_offered_from_the_first_proposal_and_need_is_not(self):
        a, ca = self.configure()
        seen, _ = self.script(a, [wa.RETAIN])
        self.cycle(a, ca).run()
        proposal = json.loads([b for k, b in seen if k == 'proposal'][0]['messages'][1]['content'])
        self.assertIn('play', proposal['choice_summary']['forms'])
        self.assertNotIn('need', proposal['choice_summary']['forms'])
        self.assertNotIn('need', proposal['response_contracts'])
        reading_body = [b for k, b in seen if k == 'reading'][0]
        self.assertEqual(reading_body['messages'][0]['content'], process.NOTEBOOK_SYSTEM)
        self.assertEqual(self.c.cli('project', 'reading-view')['bounded']['readings'][0]['reading']['gap'], PAGE['gap'])

    def test_spec12_ac4_an_edit_runs_on_a_candidate_copy_before_review(self):
        a, ca = self.configure()
        before = self.c.request()['coding']['snapshot']
        seen, ran = self.script(a, [wa.EDIT, wa.RETAIN, wa.RETAIN], reject_reviews=True)
        self.cycle(a, ca).run()
        self.assertGreaterEqual(len(ran), 1)
        self.assertLess(self.order.index('run'), self.order.index('review'))
        self.assertIn('Play', ran[0]['files']['index.html'])
        self.assertEqual(self.c.request()['coding']['snapshot'], before)
        review = json.dumps([b for k, b in seen if k == 'review'][0]['messages'])
        self.assertIn('A try of proposed edit 0', review)
        self.assertIn('blocked inline style', review)
        self.assertNotIn(PAGE['want'], review)

    def test_spec12_ac6_a_rejected_try_comes_back_beside_the_reading_once(self):
        a, ca = self.configure()
        seen, ran = self.script(a, [wa.EDIT, wa.RETAIN, wa.RETAIN], reject_reviews=True)
        self.cycle(a, ca).run()
        tries = [r for r in self.returns() if r['text'].startswith('A try of proposed edit')]
        self.assertEqual(len(tries), 1)
        later = [json.loads(b['messages'][1]['content']) for k, b in seen if k == 'reading'][1]
        self.assertIn('A try of proposed edit 0', json.dumps(later['earlier_pages']))

    def test_spec12_a_play_sets_the_controls_later_tries_use(self):
        a, ca = self.configure()
        seen, ran = self.script(a, [wa.PLAY, wa.EDIT, wa.RETAIN, wa.RETAIN], reject_reviews=True)
        self.cycle(a, ca).run()
        self.assertGreaterEqual(len(ran), 2)
        self.assertEqual(ran[1]['probe']['actions'], wa.PLAY['play']['actions'])
        self.assertIn('Play', ran[1]['files']['index.html'])

    def test_spec12_a_draft_carries_its_own_try_controls(self):
        a, ca = self.configure()
        own = {'actions': [{'kind': 'click', 'value': '#start'}, {'kind': 'key', 'value': 'ArrowLeft'}], 'checks': [{'selector': 'body', 'text': 'Play'}]}
        draft = {'alternatives': [{**wa.EDIT['alternatives'][0], 'try': own}]}
        seen, ran = self.script(a, [wa.PLAY, draft, wa.RETAIN, wa.RETAIN], reject_reviews=True)
        self.cycle(a, ca).run()
        self.assertEqual(ran[1]['probe']['actions'], own['actions'])
        self.assertEqual(ran[1]['probe']['checks'], own['checks'])
        review = [json.loads(b['messages'][1]['content']) for k, b in seen if k == 'review'][0]
        self.assertTrue(any(key.startswith('harness:') for key in review['binding']['evidence_ids']))
        self.assertIn('harness_returns', review['binding'])
        self.assertIn('the try controls you gave with this edit', json.dumps(review['evidence']))
        self.assertIn('observed_harness_return', json.dumps(review['response_schema']))

    def test_spec12_ac7_restart_after_a_try_does_not_run_it_again(self):
        a, ca = self.configure()
        _, ran = self.script(a, [wa.EDIT, wa.RETAIN, wa.RETAIN], reject_reviews=True)
        first = self.cycle(a, ca)

        class Killed(BaseException):
            pass

        def hook(point, name):
            if point == 'after_result' and name.startswith('trial_run_'):
                raise Killed()
        first.hook = hook
        with self.assertRaises(Killed):
            first.run()
        self.assertEqual(len(ran), 1)
        fresh = self.adapter()
        self.script(fresh, [wa.RETAIN, wa.RETAIN], reject_reviews=True)
        fresh.play_runner = lambda *args: ran.append('again') or {'exit': 1, 'report': wa.REPORT, 'stderr': ''}
        again = self.cycle(fresh, ca)
        again.recover_pending()
        again.run()
        self.assertNotIn('again', ran)

    def test_spec12_ac10_tuned_request_is_smaller_than_full_for_the_same_state(self):
        a, ca = self.configure(context='tuned')
        view = self.c.cli('project', 'cycle', 'request')
        req, ctx = a.project(view)
        wire = a.wire_context(req, ctx)
        sizes = {}
        admitted = a.admission
        for context in ('tuned', 'full'):
            # Only the proposal body reads the context profile; the admission itself is untouched.
            a.admission = {**admitted, 'process_context': context}
            sizes[context] = len(json.dumps(a.proposal_body(view, copy.deepcopy(wire))))
        a.admission = admitted
        self.assertLessEqual(sizes['tuned'], sizes['full'])


# The spec #11 cases belong to the abilities profile alone; they don't run under this one.
for _name in dir(wa.Cycle):
    if _name.startswith('test_') and _name not in Cycle.__dict__:
        setattr(Cycle, _name, None)


if __name__ == '__main__':
    unittest.main()
