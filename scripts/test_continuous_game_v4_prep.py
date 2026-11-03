#!/usr/bin/env python3
"""The harder continuous-game package: derived route words and the protocol. No provider or dispatch."""
import copy
import hashlib
import importlib.util
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('continuous_v4_mapping', ROOT / 'experiments/continuous-game-v4/mapping.py')
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)
ARROWS = {'Up': 'ArrowUp', 'Right': 'ArrowRight', 'Down': 'ArrowDown', 'Left': 'ArrowLeft'}
FILES = {'index.html': '<button id="start">Start</button><p id="status">x</p><script src="app.js"></script>', 'app.js': '// Controls: arrow keys.\n'}


def fixture(route='Up Right ' * 8, shard='Right ' * 9):
    report = {'source': m.source(FILES), 'readouts': {'route': route, 'shard_route': shard}}
    mapping = {'version': m.VERSION, 'source': m.source(FILES), 'entry': 'index.html', 'start': '#start',
               'controls': {'crossing': ARROWS, 'growth': ARROWS}, 'readouts': {'crossing': '#status', 'growth': '#status'},
               'documentation': {'path': 'app.js', 'quote': 'Controls: arrow keys.'}, 'limitations': 'Final DOM only.',
               'coverage': {'crossing': {'min_inputs': 10, 'basis': 'par'}, 'growth': {'min_inputs': 8, 'basis': 'shard'}},
               'discovery': {'source': m.source(FILES), 'report_sha256': m.digest(report), 'route': route, 'shard_route': shard}}
    return mapping, reviewed(mapping), report


def reviewed(mapping):
    return {'mapping_sha256': m.digest(mapping), 'decision': 'accept', 'attribution': 'test', 'reviewer': 'r', 'author': 'a', 'limitations': 'test'}


class Routes(unittest.TestCase):
    def test_mapping_search_adds_the_derived_route_after_six_blind_words(self):
        mapping, _, _ = fixture()
        self.assertEqual(m.search(mapping, 'crossing-first')[6], ['ArrowUp', 'ArrowRight'] * 8)
        self.assertEqual(m.search(mapping, 'growth-first')[6], ['ArrowRight'] * 9)
        self.assertEqual(len(m.search(mapping, 'mixed-pursuit')[6]), 25)

    def test_mapping_unavailable_route_leaves_six_words(self):
        for route in ('', 'Up sideways', 'Up ' * 33):
            mapping, _, _ = fixture(route=route)
            self.assertEqual(len(m.search(mapping, 'crossing-first')), 6)
            self.assertEqual(len(m.search(mapping, 'mixed-pursuit')), 6)

    def test_mapping_plan_binds_the_full_mapping_including_discovery(self):
        mapping, review, _ = fixture()
        plan = m.plan(mapping, FILES, review, 'crossing-first', m.search(mapping, 'crossing-first')[6])
        self.assertEqual(plan['mapping_sha256'], m.digest(mapping))
        self.assertEqual(plan['routes']['crossing'], ['Up', 'Right'] * 8)
        self.assertLessEqual(len(plan['probe']['actions']), 64)

    def test_mapping_changed_route_invalidates_the_review(self):
        mapping, review, _ = fixture()
        changed = copy.deepcopy(mapping)
        changed['discovery']['route'] = 'Down'
        with self.assertRaisesRegex(ValueError, 'review binding'):
            m.validate(changed, FILES, review)

    def test_mapping_route_must_equal_the_recorded_report(self):
        mapping, _, report = fixture()
        m.check_discovery(mapping, report)
        mapping['discovery']['route'] = 'Up'
        with self.assertRaisesRegex(ValueError, 'differs from the discovery'):
            m.check_discovery(mapping, report)
        with self.assertRaisesRegex(ValueError, 'report binding'):
            m.check_discovery(mapping, {**report, 'readouts': {'route': 'Up', 'shard_route': 'x'}})

    def test_mapping_rejects_stale_discovery_source(self):
        mapping, _, _ = fixture()
        mapping['discovery']['source'] = {'app.js': '0' * 64}
        with self.assertRaisesRegex(ValueError, 'stale discovery'):
            m.split(mapping)

    def test_mapping_shared_controls_count_every_input_in_mixed(self):
        # Regression: an allowed par of 20 must not lose the mixed batch.
        mapping, _, _ = fixture()
        mapping['coverage']['crossing']['min_inputs'] = 20
        plan = m.plan(mapping, FILES, reviewed(mapping), 'mixed-pursuit', m.search(mapping, 'mixed-pursuit')[0])
        self.assertEqual(plan['coverage_report']['status'], 'resolvable')

    def test_mapping_distinct_controls_still_divide_inputs_and_refuse(self):
        mapping, _, _ = fixture()
        mapping['controls']['growth'] = {'Up': 'w', 'Right': 'd', 'Down': 's', 'Left': 'a'}
        mapping['coverage']['crossing']['min_inputs'] = 28
        with self.assertRaisesRegex(ValueError, 'refusing dispatch'):
            m.plan(mapping, FILES, reviewed(mapping), 'mixed-pursuit', ['ArrowUp'])


class Protocol(unittest.TestCase):
    def setUp(self):
        self.protocol = json.loads((ROOT / 'experiments/continuous-game-v4/protocol.json').read_text())

    def test_protocol_supplies_no_desired_relationship(self):
        self.assertEqual(set(self.protocol['criteria']), {'crossing', 'growth'})
        self.assertFalse(self.protocol['test_scope']['novelty_required'])
        told = ' '.join([self.protocol['user_input']['text'], *self.protocol['criteria'].values(), self.protocol['operational_roots']['growth']]).lower()
        for coupling in ('segment', 'every hazard', 'including the body', 'subject to'):
            self.assertNotIn(coupling, told)

    def test_protocol_derived_from_is_exact(self):
        source = self.protocol['derived_from']
        self.assertEqual(hashlib.sha256((ROOT / source['path']).read_bytes()).hexdigest(), source['sha256'])


class GeneratedParents(unittest.TestCase):
    def wrapper(self):
        spec = importlib.util.spec_from_file_location('continue_generated', ROOT / 'experiments/continuous-game-v4-launch/continue_generated_parents.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return module

    def request(self, generated_meaning='I want', guidance='g'):
        roots = [{'id': 'a', 'requirement': 'crossing', 'meaning': guidance}]
        parents = roots + [{'id': 'p', 'meaning': generated_meaning}]
        return {'coding': {'snapshot': 's', 'relational': {'inputs': {'parents': parents, 'acceptance_parents': roots}}}}

    def test_generated_parent_is_accepted_and_guidance_may_change(self):
        original = [{'id': 'a', 'requirement': 'crossing'}]
        self.wrapper().verify(self.request(), self.request(guidance='renewed'), original)

    def test_generated_parent_change_is_refused(self):
        original = [{'id': 'a', 'requirement': 'crossing'}]
        with self.assertRaisesRegex(ValueError, 'generated parent changed'):
            self.wrapper().verify(self.request(), self.request(generated_meaning='other'), original)

    def test_changed_root_identity_is_refused(self):
        after = self.request()
        after['coding']['relational']['inputs']['parents'][0]['requirement'] = 'other'
        with self.assertRaises(ValueError):
            self.wrapper().verify(self.request(), after, [{'id': 'a', 'requirement': 'crossing'}])


if __name__ == '__main__':
    unittest.main()
