#!/usr/bin/env python3
"""The harder continuous-game package: route words and launch refusals. No provider or dispatch."""
import copy
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


def fixture():
    mapping = {'version': m.VERSION, 'source': m.source(FILES), 'entry': 'index.html', 'start': '#start',
               'controls': {'crossing': ARROWS, 'growth': ARROWS}, 'readouts': {'crossing': '#status', 'growth': '#status'},
               'documentation': {'path': 'app.js', 'quote': 'Controls: arrow keys.'}, 'limitations': 'Final DOM only.',
               'coverage': {'crossing': {'min_inputs': 10, 'basis': 'par'}, 'growth': {'min_inputs': 8, 'basis': 'shard'}},
               'routes': {'crossing': {'directions': ['Up', 'Right'] * 8, 'basis': 'displayed route'},
                          'growth': {'directions': ['Right'] * 9, 'basis': 'displayed route'},
                          'mixed': {'directions': ['Up', 'Right'] * 12, 'basis': 'displayed route'}}}
    review = {'mapping_sha256': m.digest(mapping), 'decision': 'accept', 'attribution': 'test', 'reviewer': 'r', 'author': 'a', 'limitations': 'test'}
    return mapping, review


class Routes(unittest.TestCase):
    def test_mapping_search_adds_the_route_after_six_blind_words(self):
        mapping, _ = fixture()
        for batch in m.BATCHES:
            words = m.search(mapping, batch)
            self.assertEqual(len(words), 7)
            self.assertEqual(len(words[6]), len(mapping['routes'][m.ROUTES[batch]]['directions']))

    def test_mapping_plan_binds_the_full_mapping_including_routes(self):
        mapping, review = fixture()
        plan = m.plan(mapping, FILES, review, 'crossing-first', m.search(mapping, 'crossing-first')[6])
        self.assertEqual(plan['mapping_sha256'], m.digest(mapping))
        self.assertEqual(plan['routes'], mapping['routes'])
        self.assertLessEqual(len(plan['probe']['actions']), 64)

    def test_mapping_changed_route_invalidates_the_review(self):
        mapping, review = fixture()
        changed = copy.deepcopy(mapping)
        changed['routes']['crossing']['directions'][0] = 'Down'
        with self.assertRaisesRegex(ValueError, 'review binding'):
            m.validate(changed, FILES, review)

    def test_mapping_rejects_route_over_the_harness_bound(self):
        mapping, review = fixture()
        mapping['routes']['growth']['directions'] = ['Up'] * 33
        with self.assertRaisesRegex(ValueError, 'outside the bound'):
            m.validate(mapping, FILES, {**review, 'mapping_sha256': m.digest(mapping)})

    def test_mapping_still_refuses_a_batch_coverage_blocks(self):
        mapping, _ = fixture()
        mapping['coverage']['crossing']['min_inputs'] = 28
        review = {'mapping_sha256': m.digest(mapping), 'decision': 'accept', 'attribution': 'test', 'reviewer': 'r', 'author': 'a', 'limitations': 'test'}
        with self.assertRaisesRegex(ValueError, 'refusing dispatch'):
            m.plan(mapping, FILES, review, 'mixed-pursuit', ['ArrowUp'])


class Protocol(unittest.TestCase):
    def test_protocol_supplies_no_desired_relationship(self):
        protocol = json.loads((ROOT / 'experiments/continuous-game-v4/protocol.json').read_text())
        self.assertEqual(set(protocol['criteria']), {'crossing', 'growth'})
        self.assertFalse(protocol['test_scope']['novelty_required'])
        self.assertIn('neither coupling nor a new relationship', protocol['shared_consequences'])


if __name__ == '__main__':
    unittest.main()
