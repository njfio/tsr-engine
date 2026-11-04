"""Spec #10: the single reading, offline. No provider calls.

The authored class uses the real native controller and an authored transport. These pin the
order, refusals and prompts. They are not evidence about what a model will write.
"""
import copy
import importlib.util
import json
import os
import random
import unittest
from pathlib import Path

from scripts.test_engine_browser_adapter import BrowserAdapter, m

reading = m.load('reading_test', 'engine-reading.py')
output = m.load('reading_test_output', 'engine-project-output.py')

WANT = {'want': 'A harder second level', 'sense': 'The jump feels late', 'belief': None, 'expect': None,
        'expected_return': {'from': 'the operator', 'what': 'says it is harder'}, 'falsifier': 'the operator says it is still too easy'}
BLANK = {'want': None, 'sense': None, 'belief': None, 'expect': None, 'expected_return': None, 'falsifier': None}
RETAIN = {'retain': 'Authored retention.'}
BANNED = ('development_declined', 'provisional', 'relationship', 'meaning development', 'menu', 'none is preferred', 'worth proposing')


class Shape(unittest.TestCase):
    def test_spec_c02_valid_want_and_blank_pass(self):
        reading.check_shape(WANT)
        reading.check_shape(BLANK)

    def test_spec_c05_want_needs_return_and_falsifier(self):
        for key in ('expected_return', 'falsifier'):
            with self.subTest(key=key), self.assertRaises(ValueError):
                reading.check_shape({**WANT, key: None})

    def test_spec_c06_refuses_bad_shapes(self):
        missing = dict(WANT)
        del missing['sense']
        bad = [missing, {**WANT, 'scope': 'other'}, {**WANT, 'sense': 3}, {**WANT, 'sense': '  '},
               {**WANT, 'sense': 'x' * (reading.TEXT_MAX + 1)}, {**WANT, 'expected_return': {'from': 'x'}},
               {**WANT, 'expected_return': {'from': 'x', 'what': 'y', 'z': 'w'}}, {**WANT, 'expected_return': 'x'},
               {**BLANK, 'falsifier': 'x'}, {**BLANK, 'expected_return': WANT['expected_return']}, [], None, 'text']
        for value in bad:
            with self.subTest(value=str(value)[:60]), self.assertRaises(ValueError):
                reading.check_shape(value)

    def test_parser_survives_random_values(self):
        # Fuzz tier without a new dependency: 20k deterministic random values, no crash but ValueError.
        rng = random.Random(10)
        atoms = [None, '', ' ', 'x', 'x' * (reading.TEXT_MAX + 1), 3, True, [], {}, {'from': 'a', 'what': 'b'}, {'from': 'a'}]
        keys = list(WANT) + ['scope', 'retain']
        for _ in range(20000):
            value = {k: rng.choice(atoms) for k in rng.sample(keys, rng.randint(0, len(keys)))}
            try:
                reading.check_shape(value)
            except ValueError:
                pass
            else:
                self.assertEqual(set(value), set(WANT))


class Prompts(unittest.TestCase):
    def wire(self):
        return {'public_brief': 'Make a game', 'parents': [{'id': 'a' * 64}], 'files': [{'path': 'app.js', 'source': 'x'}],
                'choice_summary': {'version': 'v', 'source_request_sha256': 'b' * 64,
                                   'forms': {'abstain': 'a', 'retain': 'r with development_declined', 'edit': 'e', 'inspect': 'i', 'provisional_preference': 'p',
                                             'relationship_initial': 'q', 'meaning_development': 'm', 'deliberation': 'd', 'contrast_preference': 'c', 'success_preference': 's'},
                                   'eligible_identity_aspirations': {'x': []}, 'unavailable_identity_routes': {'x': 'y'},
                                   'unavailable_development_routes': {'x': 'y'}, 'reviewed_aspiration_references': ['z'], 'interpretation': 'Reviewed meaning'},
                'response_contracts': {'edits_or_abstention': {}, 'relational': {}, 'public_desire': {}, 'provisional_preference': {},
                                       'contrast_preference': {}, 'success_preference': {}, 'deliberation': {}, 'return_basis': {}},
                'relational_contract': {}, 'relational_history': [], 'public_meaning_references': {}, 'generated_preferences': [],
                'proposal_references': {'request_sha256': 'b' * 64}}

    def test_spec_c01_reading_context_has_no_action_contract(self):
        context = reading.reading_context(self.wire(), {'bounded': {'readings': []}})
        for key in ('choice_summary', 'response_contracts', 'relational_contract', 'proposal_references', 'response_envelope'):
            self.assertNotIn(key, context)
        self.assertEqual(context['public_brief'], 'Make a game')
        self.assertEqual(context['files'][0]['path'], 'app.js')
        self.assertEqual(context['earlier_readings'], {'readings': []})

    def test_spec_c09_reading_prompt_offers_no_forms_or_decline(self):
        for word in BANNED:
            self.assertNotIn(word, reading.READING_SYSTEM.lower())
        self.assertIn('null', reading.READING_SYSTEM)

    def test_spec_c09_proposal_context_has_no_development_forms(self):
        stripped = reading.strip_development(self.wire())
        self.assertEqual(set(stripped['choice_summary']['forms']), {'abstain', 'retain', 'edit', 'inspect'})
        self.assertNotIn('development_declined', json.dumps(stripped['choice_summary']))
        self.assertEqual(set(stripped['response_contracts']), {'edits_or_abstention', 'return_basis'})
        for key in ('relational_contract', 'relational_history', 'public_meaning_references', 'generated_preferences'):
            self.assertNotIn(key, stripped)
        self.assertNotIn('unavailable_development_routes', stripped['choice_summary'])
        self.assertEqual(stripped['proposal_references'], {'request_sha256': 'b' * 64})

    def test_spec_c09_proposal_prompt_offers_no_development(self):
        for word in BANNED:
            self.assertNotIn(word, reading.PROPOSAL_SYSTEM.lower())


class Returns(unittest.TestCase):
    def test_check_return_names_receipt_and_snapshot_without_verdict(self):
        execution = {'state': {'goal_completion': {'test_receipt': 'a' * 64, 'snapshot': 'b' * 64, 'test_verdict': 'failed', 'accepted': False}}}
        value = reading.check_return(execution, 'c' * 64)
        self.assertEqual(value['reading'], 'c' * 64)
        self.assertEqual(value['evidence'], 'a' * 64)
        self.assertIn('a' * 64, value['text'])
        self.assertIn('b' * 64, value['text'])
        self.assertIn('failed', value['text'])
        for word in ('falsified', 'correct', 'reading was'):
            self.assertNotIn(word, value['text'])

    def test_check_return_with_no_receipt_is_none(self):
        self.assertIsNone(reading.check_return({'state': {'goal_completion': {}}}, 'c' * 64))
        self.assertIsNone(reading.check_return({}, None))


class Admission(unittest.TestCase):
    def test_spec_c11_profile_field_is_absent_unless_opted_in(self):
        self.assertEqual(reading.admission_fields(None), {})
        self.assertEqual(reading.admission_fields(reading.PROFILE), {'reading_profile': 'reading@v1'})
        with self.assertRaises(ValueError):
            reading.admission_fields('reading@v2')

    def test_calls_per_decision(self):
        self.assertEqual(reading.calls_per_decision(None), 2)
        self.assertEqual(reading.calls_per_decision(reading.PROFILE), 3)


@unittest.skipUnless(os.getenv('TSR_BROWSER_ADAPTER_AUTHORED') == '1', 'explicit authored native integration opt-in')
class Cycle(BrowserAdapter):
    def configure(self, profile=reading.PROFILE):
        self.a = m.admission(self.root / 'adapter', self.a['origin'], self.c.cli('project', 'config'), self.manifest, self.a['expires_at'],
                             max_decisions=6, max_executions=1, correction_allowance=1, feedback_events=[],
                             feedback_cycle={'version': 'authorized-feedback-cycle@v2', 'authorization_id': 'authored-reading', 'max_continuations': 4},
                             reading_profile=profile)
        a = self.adapter()
        self.adapter_instance = a
        ca={'version':'local-project-cycle@v3','journal':str(self.root/'reading-cycle'),'scope':a.native('request',None)['scope'],'max_decisions':6,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__)),'local_source_profile':'browser-scoped-source@v1','completion_tokens':32768,'input_tokens':m.base.capacity.INPUT_TOKENS,'max_review_rejections':3,'stage_receipts':'durable-stage-receipts@v1','review_feedback':'attributed-review-feedback@v1','feedback_cycle':self.a['feedback_cycle']}
        return a, ca

    def cycle(self, a, ca):
        return m.loop.Cycle(ca['journal'], ca, a.native, a.proposal, a.review, reading=a.reading)

    def transported(self, a, want=WANT, proposal=RETAIN):
        original = self.transport
        seen = []

        def transport(body, timeout):
            out = original(body, timeout)
            data = json.loads(body['messages'][1]['content'])
            if 'criteria' in data:
                return out
            kind = 'reading' if 'reading_contract' in data else 'proposal'
            seen.append((kind, copy.deepcopy(body)))
            out['value'] = copy.deepcopy(want if kind == 'reading' else proposal)
            return out
        a.injected_transport = transport
        return seen

    def test_spec_c01_c02_reading_goes_first_and_crosses_with_no_review(self):
        a, ca = self.configure()
        seen = self.transported(a)
        aims = self.c.cli('project', 'acceptance', 'status')
        result = self.cycle(a, ca).run()
        self.assertEqual(result['status'], 'proposal_retention')
        self.assertEqual([k for k, _ in seen], ['reading', 'proposal'])
        reading_data = json.loads(seen[0][1]['messages'][1]['content'])
        for key in ('choice_summary', 'response_contracts', 'proposal_references'):
            self.assertNotIn(key, reading_data)
        self.assertTrue(reading_data['files'])
        self.assertTrue(reading_data['parents'])
        self.assertTrue(reading_data['public_brief'])
        view = self.c.cli('project', 'reading-view')
        self.assertEqual(view['bounded']['current_want']['text'], WANT['want'])
        self.assertEqual(view['bounded']['current_want']['origin'], 'generator')
        self.assertEqual(self.ledger.status()['calls'], 2)
        self.assertEqual(self.c.cli('project', 'acceptance', 'status'), aims)

    def test_spec_c09_proposal_turn_has_no_development_and_sees_the_reading(self):
        a, ca = self.configure()
        seen = self.transported(a)
        self.cycle(a, ca).run()
        body = seen[1][1]
        data = json.loads(body['messages'][1]['content'])
        forms = data['choice_summary']['forms']
        for form in ('provisional_preference', 'relationship_initial', 'meaning_development', 'deliberation'):
            self.assertNotIn(form, forms)
        self.assertNotIn('development_declined', body['messages'][0]['content'] + json.dumps(data['choice_summary']))
        self.assertEqual(data['current_reading']['current_want']['text'], WANT['want'])

    def test_spec_c09_generated_development_is_refused(self):
        a, ca = self.configure()
        for value in ({'alternatives': [{'provisional_preference': {'parents': ['p0'], 'meaning': 'x', 'rationale': 'y'}}]},
                      {'alternatives': [{'public_desire': {'identity': 'p0'}}]},
                      {'alternatives': [{'relational': {}}]},
                      {'deliberation': {'artifact': RETAIN, 'development': None, 'artifact_dependency': 'independent'}},
                      {'retain': 'x', 'development_declined': 'y'}):
            with self.subTest(value=str(value)[:40]), self.assertRaises(ValueError):
                a.check_proposal_shape(value)
        a.check_proposal_shape(RETAIN)

    def test_spec_c04_blank_reading_goes_on_to_the_proposal(self):
        a, ca = self.configure()
        seen = self.transported(a, want=BLANK)
        self.assertEqual(self.cycle(a, ca).run()['status'], 'proposal_retention')
        self.assertEqual([k for k, _ in seen], ['reading', 'proposal'])
        view = self.c.cli('project', 'reading-view')
        self.assertIsNone(view['bounded']['current_want'])
        self.assertEqual(view['counts']['blank'], 1)

    def test_spec_c06_malformed_reading_blocks_the_proposal(self):
        a, ca = self.configure()
        seen = self.transported(a, want={'want': 'no return or falsifier'})
        result = self.cycle(a, ca).run()
        self.assertIn(result['status'], ('technical_stop', 'capacity_stop'))
        self.assertEqual({k for k, _ in seen}, {'reading'})
        self.assertEqual(len(seen), 2)  # The original, and the one admitted format correction.
        self.assertEqual(self.c.cli('project', 'reading-view')['counts']['readings'], 0)

    def test_spec_c01_proposal_refused_without_a_recorded_reading(self):
        a, ca = self.configure()
        seen = self.transported(a)
        with self.assertRaisesRegex(Exception, 'reading'):
            a.proposal(a.native('request', None))
        self.assertEqual(seen, [])

    def test_spec_c08_later_reading_sees_returns_beside_earlier_reading(self):
        a, ca = self.configure()
        recorded = self.c.json_cli(('project', 'reading-record'), WANT)
        self.c.cli('project', 'return', '--text', 'waaaay too easy', '--reading', recorded['reading'])
        seen = self.transported(a, want=BLANK)
        self.cycle(a, ca).run()
        earlier = json.loads(seen[0][1]['messages'][1]['content'])['earlier_readings']
        self.assertEqual(earlier['readings'][0]['returns'][0]['text'], 'waaaay too easy')
        self.assertEqual(earlier['readings'][0]['returns'][0]['from'], 'operator')
        self.assertEqual(earlier['readings'][0]['reading']['falsifier'], WANT['falsifier'])

    def test_spec_c11_reading_turn_refused_when_not_opted_in(self):
        a, _ = self.configure()
        view = a.native('request', None)
        a.admission = {k: v for k, v in a.admission.items() if k != 'reading_profile'}
        with self.assertRaisesRegex(ValueError, 'reading not admitted'):
            a.reading(view)
        self.assertEqual(self.c.cli('project', 'reading-view')['counts']['readings'], 0)

    def test_spec_c07_check_after_execution_is_recorded_once_beside_the_latest_reading(self):
        a, _ = self.configure()
        recorded = self.c.json_cli(('project', 'reading-record'), WANT)
        execution = {'state': {'goal_completion': {'test_receipt': 'a' * 64, 'snapshot': 'b' * 64, 'test_verdict': 'failed'}}}
        a.record_check_return(execution)
        view = self.c.cli('project', 'reading-view')
        returns = view['bounded']['readings'][-1]['returns']
        self.assertEqual(view['bounded']['readings'][-1]['event'], recorded['reading'])
        self.assertEqual([r['from'] for r in returns], ['rules'])
        self.assertIn('failed', returns[0]['text'])
        self.assertEqual(self.c.cli('project', 'acceptance', 'status')['accepted'], False)
        self.adapter().record_check_return(execution)
        self.assertEqual(self.c.cli('project', 'reading-view'), view)

    def test_spec_c07_execution_with_no_receipt_records_no_return(self):
        a, _ = self.configure()
        self.c.json_cli(('project', 'reading-record'), WANT)
        a.record_check_return({'state': {'goal_completion': {}}})
        counts = self.c.cli('project', 'reading-view')['counts']
        self.assertEqual(counts['returns_named'] + counts['returns_unnamed'], 0)

    def test_spec_c13_restart_makes_no_new_call_and_no_new_reading(self):
        a, ca = self.configure()
        self.transported(a)
        result = self.cycle(a, ca).run()
        calls = self.ledger.status()['calls']
        view = self.c.cli('project', 'reading-view')
        fresh = self.adapter()
        self.assertEqual(self.cycle(fresh, ca).run(), result)
        self.assertEqual(self.ledger.status()['calls'], calls)
        self.assertEqual(self.c.cli('project', 'reading-view'), view)

    def test_reconsider_branch_stops_with_no_model_call(self):
        a, ca = self.configure()
        seen = self.transported(a)
        real = a.native

        def native(kind, value):
            out = real(kind, value)
            return {**out, 'branch': 'reconsider'} if kind == 'request' else out
        result = m.loop.Cycle(ca['journal'], ca, native, a.proposal, a.review, reading=a.reading).run()
        self.assertEqual(result['status'], 'reading_reconsideration_not_offered')
        self.assertEqual(seen, [])

    def test_reading_at_the_text_bound_is_recorded(self):
        a, ca = self.configure()
        big = 'é' * (reading.TEXT_MAX // 2)
        value = {'want': big, 'sense': big, 'belief': big, 'expect': big, 'expected_return': {'from': big, 'what': big}, 'falsifier': big}
        self.assertTrue(self.c.json_cli(('project', 'reading-record'), value)['reading'])

    def test_spec_c01_real_launcher_gives_the_cycle_the_reading_only_when_opted_in(self):
        spec = importlib.util.spec_from_file_location('reading_native_cycle', Path(__file__).with_name('engine-native-cycle.py'))
        launcher = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(launcher)
        a, ca = self.configure()
        self.assertEqual(launcher.build_cycle(m.loop.Cycle, ca, a).reading, a.reading)
        a.admission = {k: v for k, v in a.admission.items() if k != 'reading_profile'}
        self.assertIsNone(launcher.build_cycle(m.loop.Cycle, ca, a).reading)

    def rejecting_transport(self, a, marker):
        """Authored turns: a marked reading, a first review that rejects, an accepting second."""
        original = self.transport
        seen = {'reading': [], 'proposal': [], 'review': []}

        def transport(body, timeout):
            data = json.loads(body['messages'][1]['content'])
            result = original(body, timeout)
            if 'reading_contract' in data:
                seen['reading'].append(data)
                result['value'] = {**WANT, 'want': marker}
            elif 'criteria' in data:
                seen['review'].append(json.dumps(body))
                if len(seen['review']) == 1:
                    result['value']['verdict'] = 'reject'
                    next(iter(result['value']['findings'].values())).update(finding='not_met', explanation='Authored criticism to carry forward')
            else:
                seen['proposal'].append(data)
                if len(seen['proposal']) == 2:
                    result['value']['alternatives'][0]['rationale'] = 'Authored replacement after the criticism; untested until the protected worker returns evidence'
            return result
        a.injected_transport = transport
        return seen

    def test_spec_c10_c08_review_criticism_survives_the_reading_and_review_never_sees_it(self):
        self.playable = True
        a, ca = self.configure()
        self.adapter_instance = a
        ca['max_executions'] = 0
        marker = 'MARKER-WANT-A-MUCH-HARDER-LEVEL'
        seen = self.rejecting_transport(a, marker)
        result = self.cycle(a, ca).run()
        self.assertGreaterEqual(len(seen['review']), 2, result)
        self.assertEqual(len(seen['proposal']), 2)
        self.assertNotIn('review_criticism', seen['proposal'][0])
        self.assertEqual(seen['proposal'][1]['review_criticism']['items'][0]['source']['source_snapshot'], seen['proposal'][1]['binding']['source_snapshot'])
        self.assertIn('review_criticism', seen['reading'][1])
        for body in seen['review']:
            self.assertNotIn(marker, body)
        for proposal in seen['proposal']:
            self.assertNotIn(marker, json.dumps(proposal['own_decisions']))
            self.assertEqual(proposal['current_reading']['current_want']['text'], marker)

    @unittest.skipUnless(os.getenv('TSR_BROWSER_SEALED_AUTHORED') == '1', 'sealed authored worker opt-in')
    def test_spec_c07_sealed_execution_returns_the_check_beside_the_reading(self):
        self.playable = True
        a, ca = self.configure()
        self.adapter_instance = a
        marker = 'MARKER-WANT-A-MUCH-HARDER-LEVEL'
        self.rejecting_transport(a, marker)
        self.cycle(a, ca).run()
        self.assertEqual(self.c.cli('project', 'status')['execution_status'], 'passed')
        view = self.c.cli('project', 'reading-view')
        returned = [r for entry in view['bounded']['readings'] for r in entry['returns']]
        self.assertEqual([r['from'] for r in returned], ['rules'])
        self.assertRegex(returned[0]['text'], 'Controller checks ran on source [0-9a-f]{64} with receipt')
        self.assertTrue(returned[0]['evidence'])
        self.assertEqual(view['counts']['returns_named'], 1)
        self.assertFalse(self.c.cli('project', 'acceptance', 'status')['accepted'])

    def test_spec_c09_real_proposal_request_offers_no_development_anywhere(self):
        a, _ = self.configure()
        view = a.native('request', None)
        request, context = a.project(view)
        body = a.proposal_body(view, a.wire_context(request, context))
        data = json.loads(body['messages'][1]['content'])
        data.pop('public_brief')
        data.pop('files')
        text = body['messages'][0]['content'] + json.dumps(data)
        for word in ('development', 'provisional', 'deliberat', 'wanting', 'relational_envelope'):
            self.assertNotIn(word, text.lower())
        self.assertIn('retain', data['response_envelope']['instruction'])
        self.assertIn('generator', data['proposal_responsibilities'])

    def test_spec_c01_reading_turn_sees_observed_returns(self):
        context = reading.reading_context({'browser_returns': ['b'], 'supplemental_returns': ['s'], 'criterion_returns': ['c'], 'independent_assessments': ['i'], 'choice_summary': {}}, {'bounded': {}})
        for key in ('browser_returns', 'supplemental_returns', 'criterion_returns', 'independent_assessments'):
            self.assertIn(key, context)
        self.assertNotIn('choice_summary', context)

    def test_spec_c11_not_opted_in_takes_no_reading_and_keeps_the_menu(self):
        a, ca = self.configure(profile=None)
        self.assertNotIn('reading_profile', self.a)
        self.assertIsNone(a.reading_callback())
        seen = self.transported(a, proposal={'retain': 'x', 'development_declined': 'y'})
        result = m.loop.Cycle(ca['journal'], ca, a.native, a.proposal, a.review, reading=a.reading_callback()).run()
        self.assertEqual(result['status'], 'proposal_retention')
        self.assertEqual([k for k, _ in seen], ['proposal'])
        self.assertIn('development_declined', seen[0][1]['messages'][0]['content'])
        self.assertEqual(self.c.cli('project', 'reading-view')['counts']['readings'], 0)


for name in dir(BrowserAdapter):
    if name.startswith('test_') and name not in Cycle.__dict__:
        setattr(Cycle, name, None)

if __name__ == '__main__':
    unittest.main()
