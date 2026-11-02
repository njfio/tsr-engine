"""Bounded inert exports; never read live arm data or execute candidates."""
import importlib.util,json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
s=importlib.util.spec_from_file_location('e',Path(__file__).with_name('m3-smoke-evidence.py'));e=importlib.util.module_from_spec(s);s.loader.exec_module(e)

class Export(unittest.TestCase):
    def make(self, root, mismatch=False):
        state=root/'target/m3-smoke-comparison-state';state.mkdir(parents=True)
        (root/'docs').mkdir()
        def write(p,v):p.write_text(json.dumps(v)+'\n')
        write(state/'complete.json',dict(status='halted-review-required',finished='fixture'))
        jobs=[]
        for task in ['first','second']:
            base=root/'target'/task;base.mkdir()
            (base/'cas').mkdir();(base/'cas'/'fixture-object').write_text('inert')
            for name in ['tasks.jsonl','profile.toml','broker-contexts.json']:(base/name).write_text('fixture')
            jobs.append(dict(task_id=task,input='target/'+task,modes=['record','rules'],source='fixture',family='fixture'))
            if task=='second':continue
            for mode in ['record','rules']:
                out=base/mode;out.mkdir()
                write(state/(task+'-'+mode+'.json'),dict(status='completed',exit=0,errors=[]))
                write(out/('results-'+mode+'.jsonl'),dict(task=task,system=mode,outcome='rejected',oracle=None,state_root_at_start='different' if mismatch and mode=='rules' else 'empty'))
        write(state/'plan.json',dict(jobs=jobs))
        for p in [state/'cost-before.jsonl',root/'target/m3-provider-cost.jsonl',root/'target/m3-provider-cost-diagnostics.jsonl']:p.write_text('')
    def test_missing_arms_preserved_and_fixed_denominator_not_complete_case_only(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);self.make(root)
            with patch.object(e,'ROOT',root),patch.object(e,'require_quiescent',create=True),patch('builtins.print'):e.main()
            result=json.loads((root/'docs/m3-smoke-result.json').read_text())
            self.assertEqual(len(result['arms']),4);self.assertEqual(len(result['observed_pairs']),1)
            self.assertEqual(result['metrics']['record']['fixed_denominator'],2)
            self.assertEqual(result['metrics']['record']['raw_outcome_counts']['missing'],1)
    def test_M3_EXPORT_01_shared_cas_and_inconsistent_acceptance(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);self.make(root)
            result_path=root/'target/first/record/results-record.jsonl'
            row=json.loads(result_path.read_text());row['outcome']='accepted';result_path.write_text(json.dumps(row)+'\n')
            with patch.object(e,'ROOT',root),patch.object(e,'require_quiescent',create=True),patch('builtins.print'):e.main()
            result=json.loads((root/'docs/m3-smoke-result.json').read_text())
            self.assertEqual(result['metrics']['record']['valid_accepted'],0)
            self.assertEqual(result['metrics']['record']['original_process_only_accepted'],1)
            self.assertEqual(result['metrics']['record']['evidence_consistent_accepted'],0)
            self.assertEqual(result['metrics']['record']['raw_outcome_counts']['accepted'],1)
            self.assertEqual((root/'docs/m3-smoke-execution/first-input-cas/fixture-object').read_text(),'inert')
    def test_mismatched_pair_roots_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);self.make(root,True)
            with patch.object(e,'ROOT',root),patch.object(e,'require_quiescent',create=True),self.assertRaises(ValueError):e.main()

if __name__=='__main__':unittest.main()
