"""Offline checks for the development-decision and observation-coverage repairs.

No model, provider or browser calls. These pin message and validator contracts;
they are not evidence that a model will now propose development.
"""
import copy
import importlib.util
import json
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


refs = load('decision_references', HERE / 'engine-proposal-references.py')
adapter_module = load('decision_adapter', HERE / 'engine-browser-adapter.py')
correction = load('decision_correction', HERE / 'engine-bounded-correction.py')
coverage = load('decision_coverage', HERE / 'engine-observation-coverage.py')
reflection = load('decision_reflection', HERE / 'engine-pursuit-reflection.py')
mapping = load('decision_mapping', ROOT / 'experiments/game-wants-v3/mapping.py')


def fixture(parents=2, provisional=True):
    ids = [str(i + 1) * 64 for i in range(parents)]
    request = {'state_root': 'a' * 64, 'coding': {'snapshot': 'b' * 64, 'cases': [], 'files': [
        {'path': 'app.js', 'source_digest': 'c' * 64, 'editable': True, 'source': 'const x = 1;'}],
        'relational': {'binding': 'd' * 64, 'inputs': {'snapshot': 'b' * 64, 'returns': [],
                                                        'parents': [{'id': i, 'requirement': 'r' + i[0], 'meaning': 'Authored want ' + i[0]} for i in ids]},
                       'history': []}}}
    native = refs.native
    context = {'parents': copy.deepcopy(request['coding']['relational']['inputs']['parents']),
               'relational_contract': native.relational_contract(request), 'relational_history': [],
               'return_evidence': native.return_evidence.catalog(request),
               'response_contracts': {'relational': native.payload_contract(request, 'final_proposal_only')['relational'],
                                      'return_basis': {'semantics': native.return_evidence.contract(), 'schema': native.return_evidence.schema(request)},
                                      'edits_or_abstention': {'authored': 'edit contract'}},
               'files': [{'path': 'app.js', 'source': 'const x = 1;'}]}
    if provisional and parents:
        context['response_contracts']['provisional_preference'] = {'authored': 'provisional contract'}
    return request, context


def adapter(offered):
    a = object.__new__(adapter_module.Adapter)
    a.narrative_limit = 131072
    a.offered_development = list(offered)
    return a


def diagnostic(error):
    # The adapter loads its own copy of the output helper, so match the diagnostic, not the class.
    if not getattr(error, 'diagnostic', None):
        raise AssertionError('expected a structured format diagnostic, got: ' + str(error))
    detail = correction.native_feedback(error.diagnostic)
    if detail is None:
        raise AssertionError('diagnostic is not eligible for bounded format correction: ' + error.diagnostic)
    return detail


class ChoiceSummary(unittest.TestCase):
    def test_retention_mentions_an_optional_decision_only_when_development_is_offered(self):
        request, context = fixture()
        summary = refs.project(context, request)['choice_summary']
        self.assertIn('provisional_preference', summary['forms'])
        self.assertIn('relationship_initial', summary['forms'])
        self.assertIn('development_declined', summary['forms']['retain'])
        self.assertIn('never required', summary['forms']['retain'])
        self.assertEqual(summary['unavailable_development_routes'], {})
        self.assertEqual(refs.development_forms(summary), ['provisional_preference', 'relationship_initial'])

        request, context = fixture(parents=0)
        summary = refs.project(context, request)['choice_summary']
        self.assertEqual(refs.development_forms(summary), [])
        self.assertNotIn('development_declined', summary['forms']['retain'])

    def test_missing_development_routes_say_why(self):
        request, context = fixture(parents=0)
        routes = refs.project(context, request)['choice_summary']['unavailable_development_routes']
        self.assertIn('No current contributor', routes['provisional_preference'])
        self.assertIn('0 supplied', routes['relationship_initial'])

        request, context = fixture(parents=1)
        routes = refs.project(context, request)['choice_summary']['unavailable_development_routes']
        self.assertNotIn('provisional_preference', routes)
        self.assertIn('1 supplied', routes['relationship_initial'])

        request, context = fixture(parents=2, provisional=False)
        routes = refs.project(context, request)['choice_summary']['unavailable_development_routes']
        self.assertIn('not supplied', routes['provisional_preference'])

    def test_ordinary_arm_keeps_plain_retention(self):
        ordinary = load('decision_ordinary', HERE / 'engine-browser-ordinary.py')
        request, context = fixture()
        wire = refs.project(context, request)
        for key in ('relationship_initial', 'relationship_revision', 'meaning_development', 'contrast_preference',
                    'success_preference', 'provisional_preference', 'deliberation'):
            wire['choice_summary']['forms'].pop(key, None)
        wire['choice_summary'].pop('unavailable_development_routes', None)
        self.assertEqual(ordinary.b.load('o', 'engine-proposal-references.py').retain_form(wire['choice_summary']['forms']),
                         refs.RETAIN_PLAIN)


class RetentionDecision(unittest.TestCase):
    def test_misplaced_candidate_field_diagnostic_stays_out_of_other_decisions(self):
        def located(value):
            with self.assertRaises(ValueError) as caught:
                adapter([]).check_proposal_shape(value)
            return diagnostic(caught.exception).get('valid_locations')
        self.assertEqual(located({'edits': [], 'rationale': 'r'}), ['$.alternatives[0].edits', '$.alternatives[0].rationale'])
        self.assertEqual(located({'alternatives': [{}], 'expectation': {}, 'invented': 1}), ['$.alternatives[0].expectation'])
        for other in ('abstain', 'retain', 'inspect', 'deliberation'):
            self.assertIsNone(located({other: 'x', 'rationale': 'r'}), other)

    def test_spec12_bare_retention_is_accepted_even_when_development_is_offered(self):
        # Spec #12, rule 9: requiring a development decision was Focus, so it is no longer required.
        adapter(['provisional_preference']).check_proposal_shape({'retain': 'Keep it'})
        adapter(['provisional_preference']).check_proposal_shape(
            {'deliberation': {'artifact': {'retain': 'Keep it'}, 'development': None, 'artifact_dependency': 'independent'}})

    def test_stated_decision_is_accepted_and_plain_retention_still_works_without_development(self):
        a = adapter(['provisional_preference'])
        a.check_proposal_shape({'retain': 'Keep it', 'development_declined': 'Considered provisional wanting; nothing new connects'})
        adapter([]).check_proposal_shape({'retain': 'Keep it'})
        with self.assertRaises(ValueError):
            adapter([]).check_proposal_shape({'retain': ''})

    def test_blank_or_oversized_decline_is_rejected(self):
        a = adapter(['relationship_initial'])
        for bad in (' ', 7):
            with self.assertRaises(ValueError) as caught:
                a.check_proposal_shape({'retain': 'Keep it', 'development_declined': bad})
            diagnostic(caught.exception)
        a.narrative_limit = 64
        with self.assertRaises(ValueError) as caught:
            a.check_proposal_shape({'retain': 'Keep it', 'development_declined': 'x' * 60})
        self.assertEqual(diagnostic(caught.exception)['code'], 'text_bound')

    def test_expansion_folds_the_decision_into_the_native_reason(self):
        request, context = fixture()
        wire = refs.project(context, request)
        folded = refs.expand({'retain': 'Keep it', 'development_declined': 'Nothing new connects'}, wire['proposal_references'], request)
        self.assertEqual(folded, {'retain': 'Keep it' + refs.DECLINED_PREFIX + 'Nothing new connects'})
        self.assertEqual(refs.expand({'retain': 'Keep it'}, wire['proposal_references'], request), {'retain': 'Keep it'})
        for bad in ({'retain': 'Keep it', 'development_declined': ''},
                    {'retain': 'Keep it' + refs.DECLINED_PREFIX + 'forged', 'development_declined': 'x'}):
            with self.assertRaises(ValueError):
                refs.expand(bad, wire['proposal_references'], request)

    def test_prompt_is_shared_and_records_offered_forms(self):
        request, context = fixture()
        wire = refs.project(context, request)
        a = object.__new__(adapter_module.Adapter)
        a.body = lambda system, ctx: {'messages': [{'role': 'system', 'content': system}, {'role': 'user', 'content': json.dumps(ctx)}]}
        with_development = a.proposal_body({'branch': 'propose'}, wire)
        self.assertEqual(a.offered_development, ['provisional_preference', 'relationship_initial'])
        without = a.proposal_body({'branch': 'propose'}, {})
        self.assertEqual(a.offered_development, [])
        # Paired arms compare identical system messages; only the context differs.
        self.assertEqual(with_development['messages'][0], without['messages'][0])
        text = with_development['messages'][0]['content']
        self.assertIn('none is preferred', text)
        self.assertEqual(text.count('independent review'), 1)


class Interpretation(unittest.TestCase):
    def test_v4_weighs_change_like_retention_and_v3_replays_unchanged(self):
        v3 = reflection.capture_system({'contract_version': 'qualitative-evidence@v3'})
        v4 = reflection.capture_system({'contract_version': 'qualitative-evidence@v4'})
        self.assertIn('an empty possibilities array is valid', v3)
        self.assertNotIn('an empty possibilities array is valid.', v4)
        self.assertIn('equally valid when nothing does', v4)
        self.assertIn('Unknown is not unmet', v4)
        self.assertIn('Do not invent user preferences', v4)


class ObservationCoverage(unittest.TestCase):
    def frozen_v2_sequences(self):
        files = {'index.html': '<button id="start">Start</button>', 'app.js': '// controls: arrows'}
        record = {'version': 'reviewed-control-map@v1', 'source': mapping.source(files), 'entry': 'index.html', 'start': '#start',
                  'controls': {r: {'Up': 'ArrowUp', 'Right': 'ArrowRight', 'Down': 'ArrowDown', 'Left': 'ArrowLeft'} for r in ('crossing', 'growth')},
                  'readouts': {'crossing': '#status', 'growth': '#status'},
                  'documentation': {'path': 'app.js', 'quote': 'controls: arrows'}, 'limitations': 'authored'}
        return {'crossing': mapping.search(record, 'crossing-first'), 'growth': mapping.search(record, 'growth-first')}

    def test_the_v2_six_key_traces_are_reported_blocked(self):
        need = {'crossing': {'min_inputs': 12, 'basis': 'Authored: far bank is twelve rows up'},
                'growth': {'min_inputs': 8, 'basis': 'Authored: nearest fly is at least eight moves away'}}
        result = coverage.static(need, self.frozen_v2_sequences())
        self.assertEqual(result['status'], 'blocked')
        self.assertEqual(result['blocked_criteria'], ['crossing', 'growth'])
        self.assertEqual(result['criteria']['crossing']['longest_sequence'], 6)

    def test_long_enough_traces_are_resolvable_within_the_harness_bound(self):
        need = {'crossing': {'min_inputs': 12, 'basis': 'Authored'}}
        self.assertEqual(coverage.static(need, {'crossing': [['ArrowUp'] * 12, ['ArrowLeft'] * 3]})['status'], 'resolvable')
        self.assertLessEqual(coverage.actions_for(['ArrowUp'] * 32), coverage.HARNESS_MAX_ACTIONS)
        over = coverage.static(need, {'crossing': [['ArrowUp'] * 40]})
        self.assertEqual(over['status'], 'blocked')
        self.assertIn('harness bound', over['criteria']['crossing']['reason'])

    def test_harness_bound_matches_the_protected_harness(self):
        text = (HERE / 'engine-browser-harness.cjs').read_text()
        self.assertIn('const MAX_ACTIONS=%d;' % coverage.HARNESS_MAX_ACTIONS, text)
        self.assertIn('c.actions.length>12', (HERE / 'engine-browser-harness-v1.cjs').read_text())

    def test_harness_and_native_bounds_agree(self):
        import re
        harness = (HERE / 'engine-browser-harness.cjs').read_text()
        rust = (ROOT / 'crates/tsr-cli/src/engine/adaptive/coding/project/browser.rs').read_text()
        native = {k: int(v) for k, v in re.findall(r'pub const (MAX_PROBE_ACTIONS|STDOUT_BYTES): usize = (\d+);', rust)}
        self.assertEqual(int(re.search(r'const MAX_ACTIONS=(\d+);', harness).group(1)), native['MAX_PROBE_ACTIONS'])
        # The harness refuses to print a report larger than the native record keeps.
        self.assertEqual(int(re.search(r'const MAX_REPORT_BYTES=(\d+);', harness).group(1)), native['STDOUT_BYTES'])
        self.assertNotIn('>4000', harness)
        self.assertIn('>4000', (HERE / 'engine-browser-harness-v1.cjs').read_text())

    def test_spec12_no_prompt_requires_a_development_decision(self):
        import re
        for name in ('engine-browser-adapter.py', 'engine-browser-project.py', 'engine-proposal-references.py'):
            text = (HERE / name).read_text()
            for match in re.finditer(r"development_declined:reason", text):
                window = text[max(0, match.start() - 160):match.start() + 200]
                self.assertTrue('never required' in window or 'optionally' in window or 'may add' in window, name + ': ' + window)

    def test_declarations_must_be_complete(self):
        for bad in ({}, {'crossing': {'min_inputs': 0, 'basis': 'x'}}, {'crossing': {'min_inputs': 3, 'basis': ' '}},
                    {'crossing': {'min_inputs': 3}}):
            with self.assertRaises(ValueError):
                coverage.static(bad, {})
        with self.assertRaises(ValueError):
            coverage.static({'crossing': {'min_inputs': 3, 'basis': 'x'}}, {'other': [['a']]})

    def report(self, status):
        return {'version': 'browser-observation@v1', 'observations': [
            {'kind': 'key', 'value': 'ArrowUp'}, {'kind': 'text', 'selector': '#status', 'text': status, 'met': True}]}

    def test_invariant_readouts_are_flagged_after_observation(self):
        same = coverage.observed({'crossing': '#status'}, [self.report('Score 0 · Length 3')] * 6)
        self.assertEqual(same['status'], 'invariant')
        self.assertEqual(same['flagged_criteria'], ['crossing'])
        varied = coverage.observed({'crossing': '#status'}, [self.report('Score 0'), self.report('Score 1')])
        self.assertEqual(varied['status'], 'varied')
        missing = coverage.observed({'crossing': '#other'}, [self.report('a'), self.report('b')])
        self.assertEqual(missing['criteria']['crossing']['status'], 'missing')

    def test_cli_exit_codes(self):
        import tempfile
        with tempfile.TemporaryDirectory() as d:
            c = Path(d) / 'c.json'
            s = Path(d) / 's.json'
            c.write_text(json.dumps({'crossing': {'min_inputs': 12, 'basis': 'Authored'}}))
            s.write_text(json.dumps({'crossing': [['ArrowUp'] * 6]}))
            import contextlib
            import io
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(coverage.main(['static', '--coverage', str(c), '--sequences', str(s)]), 2)
                s.write_text(json.dumps({'crossing': [['ArrowUp'] * 12]}))
                self.assertEqual(coverage.main(['static', '--coverage', str(c), '--sequences', str(s)]), 0)
            with contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(coverage.main(['static', '--coverage', str(Path(d) / 'missing.json'), '--sequences', str(s)]), 1)


if __name__ == '__main__':
    unittest.main()
