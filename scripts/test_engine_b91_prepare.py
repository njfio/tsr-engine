"""Offline preparation integrity only: no model, project execution or gate-pass claim."""
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name('engine-b91-prepare.py')
spec = importlib.util.spec_from_file_location('b91', SCRIPT)
b91 = importlib.util.module_from_spec(spec)
spec.loader.exec_module(b91)

class Preparation(unittest.TestCase):
    def test_restart_reproducibility_and_no_authority(self):
        with tempfile.TemporaryDirectory() as tmp:
            a, b = Path(tmp)/'a', Path(tmp)/'b'
            first = b91.prepare(a)
            self.assertEqual(first, b91.prepare(b))
            before = {p.relative_to(a): p.read_bytes() for p in a.rglob('*') if p.is_file()}
            for _ in range(2):
                result = subprocess.run([sys.executable, str(SCRIPT), 'validate', '--directory', str(a)], capture_output=True, text=True, check=True)
                self.assertEqual(json.loads(result.stdout), first)
            self.assertEqual(before, {p.relative_to(a): p.read_bytes() for p in a.rglob('*') if p.is_file()})
            self.assertFalse(first['execution_authorized'])
            self.assertFalse(first['b91_passed'])
            with self.assertRaises(FileExistsError): b91.prepare(a)

    def test_real_project_initialization_and_read_only_restart(self):
        binary = SCRIPT.parent.parent/'target/debug/tsr'
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)/'scenario';b91.prepare(root)
            engine=Path(tmp)/'engine'
            def cli(*args):
                result=subprocess.run([str(binary),'engine','--directory',str(engine),'project',*args], capture_output=True,text=True,timeout=30)
                self.assertEqual(result.returncode,0,result.stderr)
                return json.loads(result.stdout)
            cli('init','--spec',str(root/'scope.json'),'--source',str(root/'source'))
            status=cli('status')
            self.assertEqual(status['completed'],0)
            self.assertTrue(status['protected_files_intact'])
            self.assertTrue(status['dependencies_ready'])  # dependency-free; not an execution receipt
            before={p.relative_to(engine):p.read_bytes() for p in engine.rglob('*') if p.is_file() and not p.name.endswith(('-wal','-shm'))}
            self.assertEqual(cli('status'),status)
            after={p.relative_to(engine):p.read_bytes() for p in engine.rglob('*') if p.is_file() and not p.name.endswith(('-wal','-shm'))}
            self.assertEqual(before,after)
            self.assertEqual((engine/'coding-work/src/lib.rs').read_text(),b91.SOURCE)
            self.assertFalse(b91.validate(root)['b91_passed'])

    def test_normal_cli_deferred_relation_review_and_restart_without_execution(self):
        binary = SCRIPT.parent.parent/'target/debug/tsr'
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)/'scenario';b91.prepare(root)
            engine=Path(tmp)/'engine'
            def cli(*args):
                result=subprocess.run([str(binary),'engine','--directory',str(engine),'project',*args], capture_output=True,text=True,timeout=30)
                self.assertEqual(result.returncode,0,result.stderr)
                return json.loads(result.stdout)
            cli('init','--spec',str(root/'scope.json'),'--source',str(root/'source'),'--requirements',str(root/'requirements.json'))
            state=cli('acceptance','status')
            feedback=Path(tmp)/'feedback.json'
            feedback.write_text(json.dumps({'requirements_revision':state['requirements_revision'], 'decision_record':None,
                'guidance':json.loads((root/'requirements.json').read_text())}))
            cli('acceptance','feedback','--file',str(feedback))
            request=cli('request');context=request['coding']['relational']
            for item in request['coding']['files']:
                if not item['editable']:
                    self.assertNotIn('source',item)  # frozen protected assertions never enter generation
            self.assertNotIn('failed-candidate.rs',json.dumps(request))
            # Deliberately authored protocol fixture, not a model aspiration or B9.1 result.
            response={'protocol':request['protocol'],'state_root':request['state_root'],
                'attention_binding':request['next_attention']['binding'],
                'proposals':[{'kind':'relational','binding':context['binding'],'proposal':{
                    'parents':[p['id'] for p in context['inputs']['parents']],
                    'relationship':'Authored protocol fixture relationship',
                    'aspiration':'Investigate a compatible boundary',
                    'discriminator':'Independent finite observation, not universal proof',
                    'revises':None,'catalysts':[]}}]}
            path=Path(tmp)/'response.json';path.write_text(json.dumps(response))
            stored=cli('run','--response',str(path))
            self.assertEqual(stored['status'],'review_required');self.assertFalse(stored['executed'])
            self.assertEqual(cli('relational','inspect')['guidance'],[])
            fresh=cli('request')
            admitted=cli('relational','admit','--state-root',fresh['state_root'],'--proposal',stored['proposal_record'],'--index','0')
            self.assertEqual(cli('request')['coding']['relational']['guidance'][0]['admission'],admitted)
            self.assertEqual(cli('status')['completed'],0)
            stale=subprocess.run([str(binary),'engine','--directory',str(engine),'project','run','--response',str(path)],capture_output=True,text=True,timeout=30)
            self.assertNotEqual(stale.returncode,0)
            self.assertEqual(cli('status')['completed'],0)
            self.assertFalse(b91.validate(root)['b91_passed'])

    def test_tampering_and_forged_admission_rejected(self):
        for name in ['source/src/lib.rs', 'source/tests/contract.rs', 'plan.json', 'manifest.json']:
            with self.subTest(name=name), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)/'run';b91.prepare(root)
                (root/name).write_text('{}')
                with self.assertRaises(ValueError): b91.validate(root)
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)/'run';b91.prepare(root)
            (root/'admission.json').write_text('{"approved":true}')
            with self.assertRaises(ValueError): b91.validate(root)

    def test_missing_and_symlink_inputs_rejected(self):
        for symlink in [False, True]:
            with tempfile.TemporaryDirectory() as tmp:
                root=Path(tmp)/'run';b91.prepare(root)
                p=root/'source/src/lib.rs';p.unlink()
                if symlink:p.symlink_to(root/'review-only/failed-candidate.rs')
                with self.assertRaises(ValueError):b91.validate(root)

    def test_original_aims_seed_and_evidence_separated(self):
        plan=json.loads(b91.artifacts()['plan.json'])
        self.assertEqual(set(plan['original_desires']), {'empty_input','existing_behaviour'})
        self.assertEqual(plan['seed']['forecast']['p'], 0.1)
        self.assertIn('no execution receipt',plan['seed']['status'])
        self.assertEqual(plan['model_calls_authorized'],0)
        self.assertIn('review-only/',plan['generator_excludes'])
        self.assertNotIn('reference-repair.rs',b91.artifacts())

if __name__ == '__main__':unittest.main()
