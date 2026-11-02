#!/usr/bin/env python3
"""Prepare one pinned admitted task; no inference, source execution or source-repo writes."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
TASK = '39385564a640e614fb7a8958eef24266728525a5364238611d2ab52bc345a2d7'
FILE = 'crates/rts-mcp/src/surface.rs'
MODEL = 'hhao/qwen2.5-coder-tools:latest'
REQUIREMENTS = {
    'named_list': 'A comma-separated list selects exactly the requested known tool names; whitespace is trimmed and empty list entries are ignored. A wholly empty list is an error.',
    'unknown_names': 'Any unknown tool name is an error, never silently removed. Existing observability names remain supported.',
    'profiles': 'Preserve all, *, core, verify and tsr profiles and existing case behavior; no unrelated behavior changes.',
    'regression': 'Frozen named test and full workspace suite execute and pass. Tests alone do not prove the complete input contract; independent code review is required.'}


def write(path, value):
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + '\n')


def window(text):
    start = text.index('        let names: Vec<String> = match ')
    end = text.index('        let known = ', start)
    body = text[start:end]
    if len(body.encode()) > 2048 or '#[test]' in body:
        raise ValueError('model window is not bounded implementation-only context')
    return {'start': len(text[:start].encode()), 'end': len(text[:end].encode())}


def cas(path, expected):
    data = path.read_bytes()
    actual = subprocess.check_output([str(ROOT / 'target/m3-blake3-checksum')], input=data, timeout=15).decode().strip()
    if actual != expected:
        raise ValueError('admitted CAS identity mismatch')
    return data


def prepare(out):
    if out.parent != (ROOT / 'target').resolve():
        raise ValueError('preparation output must be a new directory under engine target/')
    spec = importlib.util.spec_from_file_location('planner', ROOT / 'scripts/engine-collection-plan.py')
    planner = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(planner)
    plan = json.loads((ROOT / 'docs/engine-prospective-collection.json').read_text())
    planner.validate(ROOT, plan)
    item = next(t for t in plan['tasks'] if t['task'] == TASK)
    rows = map(json.loads, (ROOT / item['source_manifest']).read_text().splitlines())
    task = next(t for t in rows if t['id'] == TASK)
    prep_path = ROOT / item['preparation_manifest']
    prep = json.loads(prep_path.read_text())
    repo, ref = task['repo'], task['base_snapshot']
    def git(*args):
        return subprocess.check_output(['git', '-C', repo, *args])
    before = git('status', '--porcelain')
    paths = git('ls-tree', '-r', '--name-only', ref, 'crates', 'schemas').decode().splitlines() + ['Cargo.toml']
    paths += [p for p in git('ls-tree', '-r', '--name-only', ref, 'corpus').decode().splitlines() if p.endswith('.toml')]
    # Copy all workspace crates and their schema/corpus test assets, never Git history or hooks.
    out.mkdir(parents=True, exist_ok=False)
    source = out / 'source'
    source.mkdir()
    for path in paths:
        if Path(path).is_absolute() or '..' in Path(path).parts:
            raise ValueError('unsafe source path')
        mode = git('ls-tree', ref, path).decode().split()[0]
        if mode != '100644':
            raise ValueError('only regular source files admitted')
        data = git('show', f'{ref}:{path}')
        data.decode('utf-8')
        dest = source / path
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(data)
    (source / 'Cargo.lock').write_bytes(cas(prep_path.parent / 'cas' / prep['lock_digest'], prep['lock_digest']))
    paths.append('Cargo.lock')
    mutation = cas(prep_path.parent / 'cas' / task['mutation_diff'], task['mutation_diff'])
    if [x for x in mutation.decode().splitlines() if x.startswith(('--- ', '+++ '))] != ['--- a/' + FILE, '+++ b/' + FILE]:
        raise ValueError('mutation path mismatch')
    env = {k: v for k, v in os.environ.items() if k not in ('GIT_DIR', 'GIT_WORK_TREE', 'GIT_INDEX_FILE')}
    env['GIT_CEILING_DIRECTORIES'] = str(out)
    for args in (['--check'], []):
        subprocess.run(['git', 'apply', *args, '-'], input=mutation, cwd=source, env=env, check=True)
    scope = {'goal': 'Restore Surface::parse: comma-separated known tool names select exactly those tools; trim whitespace, ignore empty entries, reject wholly empty lists and unknown names. Preserve all/*/core/verify/tsr profiles, observability names and case behavior. Frozen named test and full workspace suite must pass. Independent semantic review is required.',
        'image': prep['image'], 'user': '501:20', 'cargo': True, 'named_package': 'rts-mcp',
        'full_workspace_oracle': True, 'editable': {FILE: ''}, 'protected': {p: '' for p in paths if p != FILE},
        'edit_windows': {FILE: window((source / FILE).read_text())},
        'cases': [{'id': item['named_tests'][0], 'args': [], 'expected_stdout': ''}], 'timeout_ms': 120000}
    write(out / 'scope.json', scope)
    write(out / 'requirements.json', REQUIREMENTS)
    # No lifecycle clock starts during preparation. Generate the bound admission just in time.
    admission = planner.admission(plan, TASK, '0' * 64, 'independent-review-agent-plus-operator', 'operational')
    admission.pop('state_root')
    write(out / 'admission-template.json', admission)
    model_manifest = Path.home() / '.ollama/models/manifests/registry.ollama.ai/hhao/qwen2.5-coder-tools/latest'
    model = json.loads(model_manifest.read_text())
    for layer in [model['config'], *model['layers']]:
        blob = Path.home() / '.ollama/models/blobs' / layer['digest'].replace(':', '-')
        if not blob.is_file() or blob.stat().st_size != layer['size']:
            raise ValueError('installed model blob missing or wrong size')
    if git('status', '--porcelain') != before:
        raise ValueError('source repository status changed')
    record = {'version': 'one-task-readiness@v1', 'task': TASK, 'repo': repo, 'base_snapshot': ref,
        'verified_cas': {'lock': prep['lock_digest'], 'mutation': task['mutation_diff'], 'algorithm': 'BLAKE3 bytes (corpus CAS)'},
        'mutated_snapshot': task['mutated_snapshot'], 'image': prep['image'], 'family': item['family'],
        'clone_group': item['clone_group'], 'lineage': item['lineage'], 'event': plan['event'],
        'source_sha256': {p: hashlib.sha256((source / p).read_bytes()).hexdigest() for p in paths},
        'scope_sha256': hashlib.sha256((out / 'scope.json').read_bytes()).hexdigest(),
        'model': MODEL, 'model_manifest_sha256': hashlib.sha256(model_manifest.read_bytes()).hexdigest(),
        'model_blobs_checked': 'existence and sizes; provider digest must be rechecked before dispatch',
        'calls_started': 0, 'episode_started': False, 'closure': None, 'matures_at': None,
        'source_status_preserved': True, 'generator_excludes': ['reference repair', 'base snapshot/history', 'mutation diff', 'qualification outcome', 'protected tests', 'reviewer evidence'],
        'review': 'Separate reviewer inspects current diff, frozen public contract and independently measured receipts; tests alone insufficient.',
        'operator_time': 'Record active reviewer minutes separately with reviewer identity, start/end and measurement method; unknown until measured, not zero.'}
    write(out / 'readiness.json', record)
    return {'prepared': str(out), 'files': len(paths), 'edit_window_bytes': scope['edit_windows'][FILE]['end'] - scope['edit_windows'][FILE]['start'], 'model': MODEL, 'calls': 0}


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', required=True, type=Path)
    args = parser.parse_args()
    print(json.dumps(prepare(args.out.resolve())))
