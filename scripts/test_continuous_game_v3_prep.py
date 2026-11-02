"""Offline checks for the prepared continuous-game-v3 observation package.

No model, provider or dispatch. An optional local Chromium check (TSR_LOCAL_CHROMIUM=1)
runs the protected harness outside the sealed container to show a sized trace can
reach a goal a six-input trace cannot; it is not sealed-runtime evidence.
"""
import copy
import importlib.util
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('continuous_v3_mapping', ROOT / 'experiments/continuous-game-v3/mapping.py')
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)

GAME_HTML = '<button id="start">Start</button><p id="status">Row 0 · Length 3</p><script src="app.js"></script>'
GAME_JS = ('// Controls: arrow keys move. Reach row 12 to cross; growth needs 8 moves right.\n'
           'let row=0,col=0,len=3,on=false;\n'
           'document.getElementById("start").onclick=()=>{on=true;row=0;col=0;len=3;show()};\n'
           'function show(){document.getElementById("status").textContent=(row>=12?"Crossed":"Row "+row)+" · Length "+(col>=8?4:len)}\n'
           'addEventListener("keydown",e=>{if(!on)return;if(e.key==="ArrowUp")row++;if(e.key==="ArrowRight")col++;show()});\n')


def fixture(crossing=12, growth=8):
    files = {'index.html': GAME_HTML, 'app.js': GAME_JS}
    keys = {'Up': 'ArrowUp', 'Right': 'ArrowRight', 'Down': 'ArrowDown', 'Left': 'ArrowLeft'}
    mapping = {'version': m.VERSION, 'source': m.source(files), 'entry': 'index.html', 'start': '#start',
               'controls': {'crossing': dict(keys), 'growth': dict(keys)}, 'readouts': {'crossing': '#status', 'growth': '#status'},
               'documentation': {'path': 'app.js', 'quote': 'Controls: arrow keys move.'}, 'limitations': 'Authored fixture',
               'coverage': {'crossing': {'min_inputs': crossing, 'basis': 'Authored: goal is row 12'},
                            'growth': {'min_inputs': growth, 'basis': 'Authored: growth after 8 moves right'}}}
    review = {'mapping_sha256': m.digest(mapping), 'decision': 'accept', 'attribution': 'authored', 'reviewer': 'reviewer', 'author': 'author', 'limitations': 'authored'}
    return files, mapping, review


class Package(unittest.TestCase):
    def test_protocol_changes_only_the_named_fields(self):
        v2 = json.loads((ROOT / 'experiments/continuous-game-v2/protocol.json').read_text())
        v3 = json.loads((ROOT / 'experiments/continuous-game-v3/protocol.json').read_text())
        self.assertNotIn('Retention and abstention remain valid', v3['feedback']['text'])
        self.assertIn('None is preferred', v3['feedback']['text'])
        for key in ('user_input', 'operational_roots', 'criteria', 'unknown_rule', 'minimum_obligations', 'excluded_claims', 'model', 'stop'):
            self.assertEqual(v3[key], v2[key], key)
        self.assertIn('coverage_gate', v3['observation_policy'])
        self.assertNotIn('six predeclared six-input words', v3['observation_policy']['search'])


class Mapping(unittest.TestCase):
    def test_words_are_sized_from_reviewed_coverage(self):
        files, mapping, review = fixture()
        self.assertEqual(m.validate(mapping, files, review)['status'], 'mapped')
        self.assertEqual({len(w) for w in m.search(mapping, 'crossing-first')}, {16})
        self.assertEqual({len(w) for w in m.search(mapping, 'growth-first')}, {12})
        self.assertEqual({len(w) for w in m.search(mapping, 'mixed-pursuit')}, {28})
        for batch in m.BATCHES:
            self.assertEqual(m.batch_coverage(mapping, batch, m.search(mapping, batch))['status'], 'resolvable', batch)

    def test_plan_is_bound_and_within_the_harness(self):
        files, mapping, review = fixture()
        for batch in m.BATCHES:
            for word in m.search(mapping, batch):
                p = m.plan(mapping, files, review, batch, word)
                self.assertLessEqual(len(p['probe']['actions']), 64)
                self.assertEqual(p['coverage_report']['status'], 'resolvable')
        self.assertEqual(m.digest({k: v for k, v in p.items() if k != 'plan_sha256'}), p['plan_sha256'])

    def test_unresolvable_or_unreviewed_mappings_are_refused(self):
        files, mapping, review = fixture(crossing=20, growth=20)
        # Alternating roots give each root 16 of 32 inputs: below 20, so mixed-pursuit is refused.
        with self.assertRaisesRegex(ValueError, 'cannot resolve'):
            m.plan(mapping, files, review, 'mixed-pursuit', m.search(mapping, 'mixed-pursuit')[0])
        m.plan(mapping, files, review, 'crossing-first', m.search(mapping, 'crossing-first')[0])
        with self.assertRaisesRegex(ValueError, 'exceeds'):
            m.split(fixture(crossing=29)[1])
        files, mapping, review = fixture()
        changed = copy.deepcopy(mapping); changed['coverage']['crossing']['min_inputs'] = 6
        with self.assertRaisesRegex(ValueError, 'review binding'):
            m.validate(changed, files, review)
        for bad in ({k: v for k, v in mapping.items() if k != 'coverage'}, {**mapping, 'version': 'reviewed-control-map@v1'},
                    {**mapping, 'coverage': {'crossing': mapping['coverage']['crossing']}}):
            with self.assertRaises(ValueError):
                m.split(bad)
        self.assertEqual(m.validate(mapping, files, {**review, 'decision': 'reject'})['status'], 'unknown')

    @unittest.skipUnless(os.getenv('TSR_LOCAL_CHROMIUM') == '1', 'local Chromium opt-in; not the sealed runtime')
    def test_sized_trace_reaches_a_goal_six_inputs_cannot(self):
        files, mapping, review = fixture()
        six = m.base.search({**{k: v for k, v in mapping.items() if k not in ('coverage', 'version')}, 'version': 'reviewed-control-map@v1'}, 'crossing-first')[0]
        sized = m.search(mapping, 'crossing-first')[0]
        probes = {}
        for name, word in (('six', six), ('sized', sized)):
            actions = [{'kind': 'click', 'value': '#start'}]
            for i, key in enumerate(word):
                actions += ([{'kind': 'wait', 'value': 100}] if i else []) + [{'kind': 'key', 'value': key}]
            probes[name] = {'version': 'browser-probe@v1', 'assets': sorted(files), 'entry': 'index.html', 'actions': actions, 'checks': [{'selector': '#status', 'text': 'Crossed'}]}
        with tempfile.TemporaryDirectory() as d:
            for n, t in files.items():
                (Path(d) / n).write_text(t)
            script = ("const {chromium}=require('playwright');const h=require(%s);const ps=%s;"
                      "(async()=>{const out={};for(const [k,p] of Object.entries(ps)){const r=await h.run(p,chromium,{containerOnly:true});out[k]=r.observations.find(o=>o.kind==='text').text}console.log(JSON.stringify(out))})()"
                      % (json.dumps(str(ROOT / 'scripts/engine-browser-harness.cjs')), json.dumps(probes)))
            out = json.loads(subprocess.run(['node', '-e', script], cwd=d, check=True, capture_output=True, text=True, timeout=120).stdout)
        self.assertTrue(out['six'].startswith('Row 6'))
        self.assertTrue(out['sized'].startswith('Crossed'))


if __name__ == '__main__':
    unittest.main()
