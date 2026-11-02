"""Bounded inert exports; never read live arm data or execute candidates."""
import importlib.util,json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
s=importlib.util.spec_from_file_location('e',Path(__file__).with_name('m3-progress-validation-evidence.py'));e=importlib.util.module_from_spec(s);s.loader.exec_module(e)

class Export(unittest.TestCase):
    def make(self, root, mismatch=False):
        state=root/'target/m3-progress-validation-state';state.mkdir(parents=True)
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
            result=json.loads((root/'docs/m3-progress-validation-result.json').read_text())
            self.assertEqual(len(result['arms']),4);self.assertEqual(len(result['observed_pairs']),1)
            self.assertEqual(result['metrics']['record']['fixed_denominator'],2)
            self.assertEqual(result['metrics']['record']['raw_outcome_counts']['missing'],1)
            self.assertEqual(result['metrics']['record']['unstarted_arms'],1)
            self.assertEqual(result['metrics']['record']['process_failures'],0)
    def test_M3_EXPORT_01_shared_cas_and_inconsistent_acceptance(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);self.make(root)
            result_path=root/'target/first/record/results-record.jsonl'
            row=json.loads(result_path.read_text());row['outcome']='accepted';result_path.write_text(json.dumps(row)+'\n')
            with patch.object(e,'ROOT',root),patch.object(e,'require_quiescent',create=True),patch('builtins.print'):e.main()
            result=json.loads((root/'docs/m3-progress-validation-result.json').read_text())
            self.assertEqual(result['metrics']['record']['valid_accepted'],0)
            self.assertEqual(result['metrics']['record']['original_process_only_accepted'],1)
            self.assertEqual(result['metrics']['record']['evidence_consistent_accepted'],0)
            self.assertEqual(result['metrics']['record']['raw_outcome_counts']['accepted'],1)
            self.assertEqual((root/'docs/m3-progress-validation-execution/first-input-cas/fixture-object').read_text(),'inert')
    def test_mismatched_pair_roots_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);self.make(root,True)
            with patch.object(e,'ROOT',root),patch.object(e,'require_quiescent',create=True),self.assertRaises(ValueError):e.main()

    def test_receipt_verified_count_is_separate_from_raw_and_consistent_counts(self):
        import sqlite3
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);self.make(root)
            out=root/'target/first/record';path=out/'results-record.jsonl';row=json.loads(path.read_text())
            row.update(outcome='accepted',oracle=dict(accepted=True,flipped_tests_pass=True,suite_regressed=False,exit_code=0,evidence=dict(manifest='fixture',verified_success=False)))
            path.write_text(json.dumps(row)+'\n')
            store=out/'state/first';store.mkdir(parents=True)
            db=sqlite3.connect(store/'record.sqlite');db.execute('create table events (kind text,payload text)');db.execute('insert into events values (?,?)',('task_closed',json.dumps({'kind':{'reason':'oracle:true'}})));db.commit();db.close()
            with patch.object(e,'ROOT',root),patch.object(e,'require_quiescent'),patch.object(e,'verify_receipts',return_value=dict(status='checked',verified_success=False)),patch('builtins.print'):e.main()
            result=json.loads((root/'docs/m3-progress-validation-result.json').read_text());m=result['metrics']['record']
            self.assertEqual(m['raw_outcome_counts']['accepted'],1)
            self.assertEqual(m['evidence_consistent_accepted'],1)
            self.assertEqual(m['receipt_verified_accepted'],0)

if __name__=='__main__':unittest.main()
