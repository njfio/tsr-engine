"""Harmless fixture test: failed processes and missing arms never inflate success."""
import contextlib
import importlib.util
import io
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('evidence', Path(__file__).with_name('m3-paired-cohort-evidence.py'))
evidence = importlib.util.module_from_spec(spec)
spec.loader.exec_module(evidence)


class ReportingChecks(unittest.TestCase):
    def test_preserve_raw_rows_but_exclude_failed_processes_and_missing_pairs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evidence.ROOT = root
            state = root / 'target/m3-paired-cohort-1-state'
            state.mkdir(parents=True)
            (root / 'docs').mkdir()
            for path in [state/'cost-before.jsonl', root/'target/m3-provider-cost.jsonl', root/'target/m3-provider-cost-diagnostics.jsonl']:
                path.write_text('')
            jobs = []
            for index in range(4):
                task = str(index) * 64
                base = root / 'inputs' / task
                base.mkdir(parents=True)
                for name in ('tasks.jsonl','profile.toml','broker-contexts.json'):
                    (base/name).write_text('fixture')
                jobs.append(dict(task_id=task,source='fixture',family='fixture',input=str(base.relative_to(root)),modes=['record','rules']))
                for mode in ('record','rules'):
                    if index == 1 and mode == 'rules':
                        continue
                    (base/mode).mkdir()
                    (base/mode/('results-'+mode+'.jsonl')).write_text(json.dumps(dict(task=task,system=mode,outcome='accepted'))+'\n')
                    failed = index == 0 and mode == 'record'
                    (state/(task[:12]+'-'+mode+'.json')).write_text(json.dumps(dict(status='timeout' if failed else 'completed',exit=1 if failed else 0,errors=[])))
            (state/'plan.json').write_text(json.dumps(dict(jobs=jobs)))
            (state/'complete.json').write_text(json.dumps(dict(finished='fixture')))
            with contextlib.redirect_stdout(io.StringIO()):
                evidence.main()
            result = json.loads((root/'docs/m3-paired-cohort-1-result.json').read_text())
            self.assertEqual(len(result['arms']),8)
            self.assertEqual(result['metrics']['record']['raw_outcome_counts']['accepted'],4)
            self.assertEqual(result['metrics']['record']['valid_accepted'],3)
            self.assertEqual(result['metrics']['record']['fixed_denominator'],4)
            self.assertEqual(result['metrics']['rules']['raw_outcome_counts']['missing'],1)
            self.assertEqual(len(result['observed_pairs']),2)


if __name__ == '__main__':
    unittest.main()
