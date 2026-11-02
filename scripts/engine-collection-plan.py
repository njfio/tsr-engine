#!/usr/bin/env python3
"""Offline prospective collection planning. Never invokes a model or executes source."""
import argparse
from collections import Counter
from datetime import datetime, timedelta, timezone
import hashlib
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VERSION = 'prospective-project-collection@v1'
# Structural strata, selected before any new result; not a statistical sample.
FAMILIES = ['(BinaryOperator, rust_tree_sitter)', '(UnaryOperator, rts-mcp)',
            '(FnValue, rts-daemon)', '(BinaryOperator, agentid-core)', '(FnValue, agentid-core)']
BUDGET = {'model_calls': 3, 'model_tokens': 55296, 'wall_ms': 3600000,
          'patch_bytes': 32768, 'external_cost_microusd': 0}


def read(root, relative):
    path = (root / relative).resolve(strict=True)
    if not path.is_relative_to(root.resolve()):
        raise ValueError('input escapes repository')
    data = path.read_bytes()
    if len(data) > 32 * 1024 * 1024:
        raise ValueError('input exceeds bound')
    return data


def build(root):
    root = Path(root).resolve()
    spec = importlib.util.spec_from_file_location('inventory', root / 'scripts/engine-evidence-inventory.py')
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    inventory = module.inventory(root)  # Cross-checks receipts, not summary counts alone.
    summary = json.loads(read(root, 'docs/m3-native-admission/summary.json'))
    candidates = {}
    for attempt in summary['attempts']:
        base = 'docs/m3-native-admission/' + attempt['attempt']
        rows = {t['id']: t for t in map(json.loads, read(root, base + '/tasks.jsonl').splitlines())}
        for receipt in attempt['reports']:
            if attempt['strict_protocol'] and receipt['accepted']:
                task = rows[receipt['task_id']]
                candidates.setdefault(task['id'], (base, task, receipt))
    selected = []
    for family in FAMILIES:
        base, task, receipt = min((v for v in candidates.values() if v[1]['family'] == family and (root / v[0] / 'preparation.json').is_file()), key=lambda v: v[1]['id'])
        prep = json.loads(read(root, base + '/preparation.json'))
        selected.append({'task': task['id'], 'task_source': 'mutation_derived',
            'execution_origin': 'not_started', 'prior_exposure': 'development_only_not_fresh_holdout',
            'family': family, 'lineage': task['lineage'], 'clone_group': receipt['function_group'],
            'named_tests': task['failing_tests'], 'protected_test_files': task['oracle_test_files'],
            'prepared_image': prep['image'], 'full_suite_argv': prep['test_argv'],
            'qualification_protocol': receipt['protocol'],
            'source_manifest': base + '/tasks.jsonl', 'preparation_manifest': base + '/preparation.json',
            'pins_sha256': {name: hashlib.sha256(read(root, name)).hexdigest() for name in
                [base + '/tasks.jsonl', base + '/admission.jsonl', base + '/preparation.json']},
            'criteria': {'named_tests': 'Every named test executes and passes; no zero-test success.',
                         'regression': 'Frozen full workspace suite passes; unknown or timeout is unresolved.',
                         'semantics': 'Independent review confirms documented behavior and preserved protected scope.'},
            'tool_ms_ceiling': 1800000, 'non_tool': BUDGET,
            'readiness': 'requires isolated source, protected scope/edit windows, public requirements and reviewer admission'})
    return {'version': VERSION, 'event': 'task_accepted_in_budget@v1',
        'selection': 'smallest task hash per declared stratum with preserved preparation manifest; five structural strata, not representative performance statistics',
        'qualified_tasks': inventory['qualifications']['unique_tasks'],
        'family_counts': dict(sorted(Counter(t['family'] for _, t, _ in candidates.values()).items())),
        'tasks': selected, 'operational_episodes_created': 0, 'eligible_mature_episodes': None,
        'tier2_enabled': False, 'maturity_policy': 'task-close-plus-seven-days@v1',
        'matures_at': None, 'note': 'No closure yet; dates and labels cannot be preassigned. Task provenance is separate from future execution origin.'}


def validate(root, plan):
    if plan != build(root):
        raise ValueError('plan differs from pinned corpus/selection/bounds; rebuild and review changes')
    return {'validated': True, 'scope': 'offline manifest consistency only', 'task_count': len(plan['tasks']),
            'operational_readiness': False, 'mature_evidence': False}


def admission(plan, task, root, reviewer, origin):
    if origin not in ('operational', 'replay', 'synthetic'):
        raise ValueError('explicit known execution origin required')
    if len(root) != 64 or any(c not in '0123456789abcdef' for c in root):
        raise ValueError('current root must be a 64-character lowercase hash')
    if not reviewer.strip() or len(reviewer) > 128:
        raise ValueError('explicit bounded independent reviewer required')
    item = next(t for t in plan['tasks'] if t['task'] == task)
    return {'state_root': root, 'tool_ms_ceiling': item['tool_ms_ceiling'], 'non_tool': item['non_tool'],
        'reason': 'Prospective measured episode; mutation-derived task, explicit execution origin; development only',
        'training': {'origin': origin, 'original_task': task, 'family': item['family'],
                     'lineage': item['lineage'], 'clone_group': item['clone_group'],
                     'reviewer': reviewer, 'no_inference': False}}


def maturity(closed):
    if closed.get('closed') is not True:
        raise ValueError('a saved closed lifecycle projection is required')
    stamps = [event['recorded_at'] for _, event in closed['events'] if event['kind']['kind'] == 'task_closed']
    if len(stamps) != 1:
        raise ValueError('exactly one factual closure required')
    at = datetime.fromisoformat(stamps[0].replace('Z', '+00:00'))
    if at.tzinfo is None or at > datetime.now(timezone.utc):
        raise ValueError('closure must be timezone-aware and not future')
    return {'closed_at': stamps[0], 'earliest_maturity_at': (at + timedelta(days=7)).astimezone(timezone.utc).isoformat(),
            'eligible': None, 'note': 'Date calculation only; engine reconstruction/review/budget/provenance gates still apply.'}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--root', type=Path, default=ROOT)
    sub = p.add_subparsers(dest='command', required=True)
    sub.add_parser('build')
    v = sub.add_parser('validate'); v.add_argument('--manifest', type=Path, required=True)
    a = sub.add_parser('admission-draft'); a.add_argument('--manifest', type=Path, required=True)
    a.add_argument('--task', required=True); a.add_argument('--state-root', required=True)
    a.add_argument('--reviewer', required=True); a.add_argument('--origin', choices=['operational','replay','synthetic'], required=True)
    m = sub.add_parser('maturity'); m.add_argument('--closure', type=Path, required=True)
    args = p.parse_args()
    if args.command == 'build': result = build(args.root)
    elif args.command == 'maturity': result = maturity(json.loads(args.closure.read_text()))
    else:
        plan = json.loads(args.manifest.read_text()); result = validate(args.root, plan)
        if args.command == 'admission-draft': result = admission(plan, args.task, args.state_root, args.reviewer, args.origin)
    print(json.dumps(result, indent=2, sort_keys=True))

if __name__ == '__main__': main()
