import importlib.util
from pathlib import Path
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('heldout_suite',Path(__file__).with_name('engine-repair-suite.py'))
suite=importlib.util.module_from_spec(spec);spec.loader.exec_module(suite)

class SuiteTests(unittest.TestCase):
    def test_frozen_identity_and_distinct_faults(self):
        self.assertEqual(set(suite.TASKS), {'singleton-clamp','unicode-prefix'})
        self.assertEqual(len(set(suite.suite_manifest()['tasks'].values())),2)
        self.assertEqual(suite.suite_sha256(),suite.suite_sha256())
        self.assertEqual(suite.METRICS['resources']['total_calls_envelope'],42)
        self.assertEqual(suite.METRICS['resources']['total_groups_envelope'],12)
    def test_prepare_roundtrip_and_authority(self):
        with tempfile.TemporaryDirectory() as temp:
            for key in suite.TASKS:
                scenario=suite.select(key); root=Path(temp)/key
                prepared=scenario.prepare(root)
                self.assertEqual(prepared,scenario.validate(root))
                self.assertFalse(prepared['execution_authorized'])
                self.assertEqual(prepared['actions'],0)
                self.assertEqual(set(scenario.REQUIREMENTS),{'requested_behavior','existing_behaviour'})
                self.assertEqual(set(__import__('json').loads((root/'scope.json').read_text())['editable']),{'src/lib.rs'})
    def test_source_test_and_extra_file_drift_rejected(self):
        for key in suite.TASKS:
            for name in ('source/src/lib.rs','source/tests/contract.rs','requirements.json','unadmitted.txt'):
                with self.subTest(key=key,name=name),tempfile.TemporaryDirectory() as temp:
                    root=Path(temp)/'case';scenario=suite.select(key);scenario.prepare(root)
                    (root/name).write_text('changed')
                    with self.assertRaises(ValueError):scenario.validate(root)
    def test_manifest_wrong_task_and_symlink_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp)/'case';scenario=suite.select('singleton-clamp');scenario.prepare(root)
            with self.assertRaises(ValueError):suite.select('unicode-prefix').validate(root)
            path=root/'source/src/lib.rs';path.unlink();path.symlink_to(root/'source/tests/contract.rs')
            with self.assertRaises(ValueError):scenario.validate(root)
    def test_no_reference_solution_artifact_or_arbitrary_registration(self):
        for key in suite.TASKS:
            files=suite.select(key).artifacts()
            self.assertEqual(set(files),set(suite.legacy.artifacts()))
            self.assertNotEqual(files['source/src/lib.rs'],files['review-only/failed-candidate.rs'])
        with self.assertRaises(ValueError):suite.select('../private')

if __name__=='__main__':unittest.main()
