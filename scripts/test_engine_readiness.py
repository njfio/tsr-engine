"""No provider, Docker or credential access: transport readiness and deadline regressions."""
import importlib.util
from pathlib import Path
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('engine_run_readiness', Path(__file__).with_name('engine-run.py'))
runner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(runner)

class ReadinessTests(unittest.TestCase):
    def test_missing_dependencies_stop_before_any_transport(self):
        for ready in (False, None, 'true', 1):
            with self.assertRaises(runner.Stop):
                runner.require_ready({'coding': {'generated_edits': True, 'kind': 'cargo_project@v1', 'dependencies_ready': ready}})
        runner.require_ready({'coding': {'generated_edits': True, 'kind': 'cargo_project@v1', 'dependencies_ready': True}})

    def test_real_workspace_deadline_outlives_sealed_commands(self):
        self.assertEqual(runner.execution_timeout({'timeout_ms':600000, 'commands':[[], [], []]}), 1830)
        self.assertEqual(runner.execution_timeout({'timeout_ms':20000, 'commands':[[], [], []]}), 90)
        for duration, commands in [(True,[[]]), (600001,[[]]), (0,[[]]), (1000,[]), (1000,[[]]*17)]:
            with self.assertRaises(runner.Stop):
                runner.execution_timeout({'timeout_ms':duration,'commands':commands})

    def test_timeout_is_forwarded_without_changing_default(self):
        with patch.object(runner.subprocess, 'run') as run:
            run.return_value.returncode=0
            run.return_value.stdout=b'{}'
            runner.cli(Path('/tmp/controller'), 'coding-run', timeout=1830)
            self.assertEqual(run.call_args.kwargs['timeout'],1830)
            runner.cli(Path('/tmp/controller'), 'request')
            self.assertEqual(run.call_args.kwargs['timeout'],180)

if __name__ == '__main__':
    unittest.main()
