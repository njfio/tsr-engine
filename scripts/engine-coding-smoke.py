#!/usr/bin/env python3
"""Run the bounded coding fixture through tsr's sealed runner (requires Docker).
Creates an isolated target directory; never calls a model or touches other runs.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--image')
    parser.add_argument('--project-spec', type=Path, help='Synthetic invoice project specification for the multi-file smoke')
    parser.add_argument('--project-cli', action='store_true', help='Exercise the unified project command entry point')
    parser.add_argument('--source', type=Path, help='Import explicitly scoped source paths; Cargo workspace smoke')
    parser.add_argument('--generated', action='store_true', help='Exercise two novel exact edits and the offline completion decoder')
    parser.add_argument('--user', default='501:20')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if not args.image and not args.project_spec:
        parser.error('--image or --project-spec is required')
    root = Path(__file__).resolve().parents[1]
    directory = Path(tempfile.mkdtemp(prefix='engine-coding-', dir=root / 'target'))
    commands = []

    def cli(*argv):
        if args.project_cli:
            aliases = {'project-init': 'init', 'project-config': 'config', 'project-prepare': 'prepare', 'request': 'request', 'project-export': 'export'}
            if argv[0] in aliases:
                argv = ('project', aliases[argv[0]], *argv[1:])
        command = [str(root / 'target/debug/tsr'), 'engine', '--directory', str(directory), *argv]
        commands.append(command)
        result = subprocess.run(command, check=True, capture_output=True, text=True)
        return json.loads(result.stdout)

    if args.project_spec:
        spec = json.loads(args.project_spec.read_text())
        args.image = spec['image']
        args.generated = True
        source_before = {p: hashlib.sha256((args.source / p).read_bytes()).hexdigest() for p in list(spec['editable']) + list(spec['protected'])} if args.source else None
        initial = cli('project-init', '--spec', str(args.project_spec), *(['--source', str(args.source)] if args.source else []))
        configured = cli('project-config')
        assert set(configured['editable']) == set(spec['editable'])
        assert set(configured['protected']) == set(spec['protected'])
        if spec.get('cargo'):
            assert cli('project-prepare')['ready']
        protected_paths = list(spec['protected'])
    else:
        initial = cli('coding-init', '--image', args.image, '--user', args.user, *(['--generated'] if args.generated else []))
        protected_paths = ['tests.rs']
    def protected_hashes():
        return {path: hashlib.sha256((directory / 'coding-work' / path).read_bytes()).hexdigest() for path in protected_paths}
    before = protected_hashes()
    cycles = []
    for expected_action, expected_verdict in [('increment', 'failed_tests'), ('correct', 'passed')]:
        request = cli('request')
        path = directory / 'replay.json'
        if args.generated:
            context = request['coding']
            if args.project_spec:
                source_path, old, new = ('src/pricing.rs', 'unit_cents + quantity', 'unit_cents * quantity') if expected_action == 'increment' else ('src/discount.rs', 'subtotal_cents + discount_cents', 'subtotal_cents.saturating_sub(discount_cents)')
                context = next(file for file in context['files'] if file['path'] == source_path)
            else:
                old, new = ('a - b', '(a * 2) - b') if expected_action == 'increment' else ('(a * 2) - b', 'a.checked_add(b).unwrap_or_default()')
            patch = {key: context[key] for key in ['path', 'source_digest', 'inspection_handle']}
            patch.update(old_text=old, new_text=new)
            suggestion = {'kind': 'edit_set', 'patches': [patch], 'rationale': 'Local replay of a scoped project edit'} if args.project_spec else {'kind': 'exact_edit', 'patch': patch, 'rationale': 'Local replay of a novel source edit'}
            response = {'protocol': request['protocol'], 'state_root': request['state_root'], 'attention_binding': (request.get('next_attention') or {}).get('binding'), 'proposals': [suggestion]}
            if response['attention_binding'] is None:
                response.pop('attention_binding')
            request_path = directory / 'request.json'
            request_path.write_text(json.dumps(request))
            completion = directory / 'local-completion.json'
            completion.write_text(json.dumps({'choices': [{'finish_reason': 'stop', 'message': {'role': 'assistant', 'content': json.dumps(response)}}]}))
            command = ['python3', str(root / 'scripts/engine-provider-envelope.py'), '--request', str(request_path), '--completion', str(completion)]
            commands.append(command)
            path.write_text(subprocess.run(command, check=True, capture_output=True, text=True).stdout)
        else:
            response = {'protocol': request['protocol'], 'state_root': request['state_root'], 'attention_binding': (request.get('next_attention') or {}).get('binding'),
                        'proposals': [{'kind': 'existing', 'action': action, 'rationale': 'Local replay of admitted action'}
                                      for action in ['increment', 'correct']]}
            path.write_text(json.dumps(response))
        selected = cli('propose', '--response', str(path), '--apply')
        plan = cli('coding-plan')
        if args.generated:
            assert plan['action'].startswith('patch-')
            assert new in (plan['edits'][0]['replacement'] if args.project_spec else plan['replacement'])
            assert selected['state']['pending']['forecast']['raw'] == 0.5
        else:
            assert plan['action'] == expected_action, plan
        result = cli('project', 'resume') if args.project_cli else cli('coding-run', '--commitment', plan['commitment'], '--expected-source', plan['source_before'])
        receipt_dir = directory / 'coding-receipts' / plan['commitment']
        runs = [json.loads(line) for line in (receipt_dir / 'runs.jsonl').read_text().splitlines()]
        test_stdout = (receipt_dir / 'cas' / runs[-1]['result']['stdout_digest']).read_text()
        cycles.append({'request': request, 'selection': selected, 'plan': plan, 'result': result,
                       'execution_receipts': runs, 'test_stdout': test_stdout})
        # Preserve diagnostics before checking the verdict, including infrastructure outcomes.
        args.output.write_text(json.dumps({'directory': str(directory), 'initial': initial, 'cycles': cycles,
                                          'commands': commands}, indent=2) + '\n')
        assert result['verdict'] == expected_verdict, result
    resumed = cli('request')
    assert len(resumed['prior_outcomes']) == 2
    after = protected_hashes()
    assert before == after
    if args.project_cli:
        status = cli('project', 'status')
        assert status['status'] == 'passed'
        assert cli('project', 'resume')['status'] == 'passed'
    exported = cli('project-export') if args.project_spec else None
    if args.source:
        assert source_before == {p: hashlib.sha256((args.source / p).read_bytes()).hexdigest() for p in source_before}
    trace = {'export': exported, 'original_source_unchanged': bool(args.source), 'directory': str(directory), 'image': args.image, 'initial': initial, 'cycles': cycles,
             'resumed_request': resumed, 'protected_tests_sha256': before, 'protected_tests_unchanged': True,
             'commands': commands, 'provider_calls': 0, 'generated_edits': args.generated, 'project_scope': bool(args.project_spec)}
    args.output.write_text(json.dumps(trace, indent=2) + '\n')
    print(json.dumps({'directory': str(directory), 'verdicts': [c['result']['verdict'] for c in cycles],
                      'next_choice_after_failure': cycles[1]['plan']['action'],
                      'protected_tests_unchanged': True, 'output': str(args.output)}, indent=2))


if __name__ == '__main__':
    main()
