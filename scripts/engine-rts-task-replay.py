#!/usr/bin/env python3
"""One admitted RTS task, local replay only. No provider, cohort, or source-repository writes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
TASK = '80d827e3a4c15b244e211567a7bfe6679035d07736e6ea3db7c08c1cdc64f4ad'
FILE = 'crates/rts-core/src/lib.rs'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    parser.add_argument('--phase', choices=['init', 'fail', 'correct', 'verify'], required=True)
    args = parser.parse_args()
    out = args.out.resolve()
    engine = out / 'engine'
    def cli(*argv):
        result = subprocess.run([str(ROOT / 'target/debug/tsr'), 'engine', '--directory', str(engine), *argv], capture_output=True, text=True, timeout=650)
        if result.returncode:
            raise RuntimeError(result.stderr or result.stdout)
        return json.loads(result.stdout)
    if args.phase == 'init':
        out.mkdir(parents=True, exist_ok=False)
        admission = ROOT / 'target/m3-qualified-rts-core-80d827e3a4c1'
        task = json.loads((admission / 'tasks.jsonl').read_text().splitlines()[0])
        assert task['id'] == TASK
        prep = json.loads((admission / 'preparation.json').read_text())
        repo, ref = task['repo'], task['base_snapshot']
        def git(*argv):
            return subprocess.check_output(['git', '-C', repo, *argv])
        original_status = git('status', '--porcelain')
        paths = git('ls-tree', '-r', '--name-only', ref, 'crates/rts-core').decode().splitlines()
        paths += ['Cargo.toml']
        # Preserve other member metadata/targets for Cargo's workspace resolver; only core is compiled.
        for member in ['rts-daemon', 'rts-mcp', 'rts-bench']:
            for suffix in ['Cargo.toml', 'src/main.rs', 'src/lib.rs', 'build.rs']:
                path = f'crates/{member}/{suffix}'
                if git('ls-tree', ref, path):
                    paths.append(path)
        source = out / 'source'
        source.mkdir()
        for path in paths:
            target = source / path
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(git('show', f'{ref}:{path}'))
        lock = (admission / 'cas' / prep['lock_digest']).read_bytes()
        (source / 'Cargo.lock').write_bytes(lock)
        paths.append('Cargo.lock')
        original = (source / FILE).read_text()
        start = original.index('    vec![', original.index('pub fn supported_languages()'))
        end = original.index('\n}', start)
        (out / 'reference-expression.txt').write_text(original[start:end])
        mutation = (admission / 'cas' / task['mutation_diff']).read_bytes()
        assert [line for line in mutation.decode().splitlines() if line.startswith('--- ')] == ['--- a/' + FILE]
        assert [line for line in mutation.decode().splitlines() if line.startswith('+++ ')] == ['+++ b/' + FILE]
        env = {k: v for k, v in os.environ.items() if k not in ['GIT_DIR', 'GIT_WORK_TREE', 'GIT_INDEX_FILE']}
        env['GIT_CEILING_DIRECTORIES'] = str(source.parent)
        subprocess.run(['git', 'apply', '--check', '-'], cwd=source, env=env, input=mutation, check=True)
        subprocess.run(['git', 'apply', '-'], cwd=source, env=env, input=mutation, check=True)
        mutated = (source / FILE).read_text()
        start = mutated.index('    vec![]', mutated.index('pub fn supported_languages()'))
        end = mutated.index('\n}', start)
        assert mutated[end:] == original[original.index('\n}', original.index('    vec![', original.index('pub fn supported_languages()'))):]
        spec = {'goal': 'Restore supported_languages so the genuine admitted named test passes; preserve all other source and tests.',
                'image': prep['image'], 'user': '501:20', 'cargo': True, 'named_package': 'rust_tree_sitter',
                'editable': {FILE: ''}, 'protected': {p: '' for p in paths if p != FILE},
                'edit_windows': {FILE: {'start': len(mutated[:start].encode()), 'end': len(mutated[:end].encode())}},
                'cases': [{'id': 'tests::test_supported_languages', 'args': [], 'expected_stdout': ''}], 'timeout_ms': 600000}
        scope = out / 'scope.json'
        scope.write_text(json.dumps(spec, indent=2)+'\n')
        cli('project', 'init', '--spec', str(scope), '--source', str(source))
        prepared = cli('project', 'prepare')
        record = {'task': TASK, 'repo': repo, 'base_snapshot': ref, 'mutation_diff': task['mutation_diff'],
                  'image': prep['image'], 'files': len(paths), 'package_files': 47, 'preparation': prepared,
                  'original_status': original_status.decode(), 'original_source_sha256': hashlib.sha256(git('show', f'{ref}:{FILE}')).hexdigest(),
                  'protected_sha256': {p: hashlib.sha256((source / p).read_bytes()).hexdigest() for p in paths if p != FILE},
                  'paid_calls': 0, 'evaluation_scope': 'one genuine named library test, not full workspace oracle'}
        (out / 'init.json').write_text(json.dumps(record, indent=2)+'\n')
        print(json.dumps({'phase': 'initialized', 'files': len(paths), 'ready': prepared['ready']}))
    elif args.phase in ('fail', 'correct'):
        request = cli('project', 'request')
        (out / f'{args.phase}-request.json').write_text(json.dumps(request, indent=2)+'\n')
        assert len(json.dumps(request).encode()) <= 32768
        assert len(request['coding']['files']) == 1
        context = next(f for f in request['coding']['files'] if f['path'] == FILE)
        assert '#[cfg(test)]' not in context['source']
        old = context['source']
        new = '    Vec::new()' if args.phase == 'fail' else (out / 'reference-expression.txt').read_text()
        patch = {k: context[k] for k in ['path', 'source_digest', 'inspection_handle']}
        patch.update(old_text=old, new_text=new)
        response = {'protocol':request['protocol'], 'state_root':request['state_root'], 'attention_binding': (request.get('next_attention') or {}).get('binding'), 'proposals':[{'kind':'exact_edit','patch':patch,'rationale':'Explicit local replay for this admitted task; no model generation'}]}
        path = out / f'{args.phase}-response.json'
        path.write_text(json.dumps(response))
        result = cli('project', 'run', '--response', str(path))
        (out / f'{args.phase}-result.json').write_text(json.dumps(result, indent=2)+'\n')
        print(json.dumps({'phase':args.phase,'verdict':result['verdict']}))
        assert result['verdict'] == ('failed_tests' if args.phase == 'fail' else 'passed'), result['verdict']
    else:
        record = json.loads((out / 'init.json').read_text())
        first = json.loads((out / 'fail-result.json').read_text())
        second = json.loads((out / 'correct-result.json').read_text())
        feedback = json.loads((out / 'correct-request.json').read_text())['coding']['recent_observations'][0]
        assert first['verdict'] == 'failed_tests' and second['verdict'] == 'passed'
        assert feedback['verdict'] == 'failed_tests'
        for path, digest in record['protected_sha256'].items():
            assert hashlib.sha256((engine / 'coding-work' / path).read_bytes()).hexdigest() == digest
        assert hashlib.sha256((engine / 'coding-work' / FILE).read_bytes()).hexdigest() == record['original_source_sha256']
        assert subprocess.check_output(['git','-C',record['repo'],'status','--porcelain']).decode() == record['original_status']
        exported = cli('project', 'export')
        verify = out / 'patch-check'
        (verify / Path(FILE).parent).mkdir(parents=True)
        (verify / FILE).write_bytes((out / 'source' / FILE).read_bytes())
        env = {k:v for k,v in os.environ.items() if k not in ['GIT_DIR','GIT_WORK_TREE','GIT_INDEX_FILE']}
        env['GIT_CEILING_DIRECTORIES'] = str(verify.parent)
        subprocess.run(['git','apply',exported['patch']],cwd=verify,env=env,check=True)
        assert (verify/FILE).read_bytes() == (engine/'coding-work'/FILE).read_bytes()
        status = cli('project', 'resume')
        assert status['status'] == 'tests_passed_unaccepted'
        runs = []
        for journal in (engine / 'coding-receipts').glob('*/runs.jsonl'):
            for line in journal.read_text().splitlines():
                run = json.loads(line)
                if run['argv'][0] == 'cargo':
                    run['stdout'] = (journal.parent / 'cas' / run['result']['stdout_digest']).read_text()
                    run['stderr'] = (journal.parent / 'cas' / run['result']['stderr_digest']).read_text()
                    runs.append(run)
        assert len(runs) == 2
        assert len(list((engine / 'coding-build').iterdir())) == 2
        for run in runs:
            assert '--format=pretty' in run['argv'] and '--exact' in run['argv']
            assert run['argv'][run['argv'].index('-p') + 1] == 'rust_tree_sitter'
            assert '--lib' in run['argv']
        assert sorted(run['result']['exit'] for run in runs) == [0, 101]
        record.update(test_runs=runs, first=first, second=second, persisted_feedback=feedback, status=status, export=exported,
                      protected_files_unchanged=True, source_repository_unchanged=True, exported_patch_verified=True,
                      corrected_file_equals_original_base=True, out=str(out))
        (ROOT/'docs/engine-rts-real-task-result.json').write_text(json.dumps(record,indent=2)+'\n')
        print(json.dumps({'verified':True,'out':str(out),'export':exported['patch']}))

if __name__ == '__main__':
    main()
