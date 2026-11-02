import copy
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('plan', Path(__file__).with_name('engine-collection-plan.py'))
plan = importlib.util.module_from_spec(spec); spec.loader.exec_module(plan)

class CollectionPlan(unittest.TestCase):
    @classmethod
    def setUpClass(cls): cls.manifest = plan.build(plan.ROOT)
    def test_corpus_pins_and_structural_selection(self):
        self.assertEqual(self.manifest['qualified_tasks'], 151)
        self.assertEqual(len(self.manifest['tasks']), 5)
        self.assertEqual(len({t['lineage'] for t in self.manifest['tasks']}), 2)
        self.assertTrue(plan.validate(plan.ROOT, self.manifest)['validated'])
        self.assertFalse(plan.validate(plan.ROOT, self.manifest)['operational_readiness'])
        for t in self.manifest['tasks']:
            self.assertNotIn('reversal_diff', t)
            self.assertNotIn('mutation_diff', t)
            self.assertNotIn('base_snapshot', t)
            self.assertEqual(t['execution_origin'], 'not_started')
        altered = copy.deepcopy(self.manifest); altered['tasks'][0]['non_tool']['model_calls'] += 1
        with self.assertRaises(ValueError): plan.validate(plan.ROOT, altered)
    def test_admission_has_no_fake_clock_review_or_forecast(self):
        task = self.manifest['tasks'][0]['task']
        a = plan.admission(self.manifest, task, 'a'*64, 'independent operator', 'replay')
        self.assertEqual(a['training']['origin'], 'replay')
        self.assertEqual(a['training']['original_task'], task)
        self.assertNotIn('recorded_at', a)
        self.assertNotIn('label', a)
        with self.assertRaises(ValueError): plan.admission(self.manifest, task, 'bad', 'reviewer', 'operational')
    def test_maturity_uses_closure_timezone_and_no_label(self):
        c = {'closed':True, 'events':[[1, {'recorded_at':'2026-01-01T23:00:00-02:00','kind':{'kind':'task_closed'}}]]}
        r = plan.maturity(c)
        self.assertEqual(r['earliest_maturity_at'], '2026-01-09T01:00:00+00:00')
        self.assertIsNone(r['eligible'])
        c['closed'] = False
        with self.assertRaises(ValueError): plan.maturity(c)
        c['closed'] = True; c['events'] *= 2
        with self.assertRaises(ValueError): plan.maturity(c)
    def test_input_path_escape_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            with self.assertRaises(ValueError): plan.read(Path(d), '/etc/hosts')

if __name__ == '__main__': unittest.main()
