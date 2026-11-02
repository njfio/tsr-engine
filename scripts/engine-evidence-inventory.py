#!/usr/bin/env python3
"""Read-only reconciliation of explicitly named repository evidence; never labels episodes."""
import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path


def inventory(root):
    root = Path(root).resolve()
    sources = {}
    def read(relative, lines=False):
        path = (root / relative).resolve(strict=True)
        if not path.is_relative_to(root):
            raise ValueError('evidence path escapes selected repository')
        raw = path.read_bytes()
        if len(raw) > 32 * 1024 * 1024:
            raise ValueError('bounded evidence file exceeded')
        sources[relative] = hashlib.sha256(raw).hexdigest()
        return [json.loads(line) for line in raw.splitlines() if line.strip()] if lines else json.loads(raw)
    summary = read('docs/m3-native-admission/summary.json')
    tasks, groups = {}, {}
    reports = 0
    for attempt in summary['attempts']:
        name = attempt['attempt']
        if Path(name).name != name or name in ('.', '..'):
            raise ValueError('invalid attempt identity')
        base = 'docs/m3-native-admission/' + name
        rows = read(base + '/tasks.jsonl', True)
        by_id = {r['id']: r for r in rows}
        if len(by_id) != len(rows):
            raise ValueError('duplicate task identity in manifest')
        actual = read(base + '/admission.jsonl', True)
        if actual != attempt['reports']:
            raise ValueError('summary disagrees with admission receipts')
        reports += len(actual)
        for receipt in actual:
            if not (attempt['strict_protocol'] and receipt['accepted']):
                continue
            if receipt.get('protocol') != 'baseline-full-green/named-red/full-red/exact-reversal/full-green-v1':
                raise ValueError('accepted receipt lacks strict admission protocol')
            task = by_id[receipt['task_id']]
            metadata = {k: task[k] for k in ('id', 'family', 'lineage', 'timestamp')}
            metadata['function_group'] = receipt['function_group']
            if metadata['id'] in tasks and tasks[metadata['id']] != metadata:
                raise ValueError('conflicting task metadata')
            tasks[metadata['id']] = metadata
            group = (metadata['lineage'], metadata['function_group'])
            groups.setdefault(group, set()).add(metadata['id'])
    if len(groups) != summary['admitted_production_tasks'] or set(tasks) != set(summary['accepted_unique_task_ids']):
        raise ValueError('qualification count or task identities disagree')
    fixed = read('docs/m3-fixed12-result.json')
    audit = read('docs/m3-fixed12-independent-outcome-audit.json')
    if audit['frozen_mismatches']:
        raise ValueError('historical independent audit reports frozen-input mismatches')
    assignments = audit['assignments']
    indices = [a['index'] for a in assignments]
    if len(indices) != len(set(indices)):
        raise ValueError('duplicate arm assignment')
    completed = [a for a in assignments if a['status'] == 'decision-recorded']
    launched = [a for a in assignments if a.get('intent') is True]
    pair_modes = {}
    for a in assignments:
        if a['status'] not in ('decision-recorded','halted-without-decision','not-launched'):
            raise ValueError('unknown arm status')
        if (a['status'] == 'not-launched') != (a.get('intent') is False):
            raise ValueError('contradictory launch evidence')
        if a['status'] == 'decision-recorded':
            if not a.get('decision'):
                raise ValueError('classified arm lacks decision')
            key=(a['pair_index'],a['task'])
            modes=pair_modes.setdefault(key,set())
            if a['mode'] in modes:
                raise ValueError('duplicate mode in pair')
            modes.add(a['mode'])
    pairs=sum(modes=={'record','rules'} for modes in pair_modes.values())
    if (len(completed),len(launched),pairs)!=(fixed['completed_arms'],fixed['launched_arms'],fixed['complete_pairs']):
        raise ValueError('fixed12 aggregate counters disagree with audited assignments')
    final = [a for a in assignments if a.get('decision', {}).get('final_oracle_completed') is True]
    return {
        'schema': 'existing-evidence-reconciliation@v1',
        'verification_scope': 'Cross-check saved admission manifests and summaries; hash explicit inputs. Does not re-execute oracles or independently authenticate reviewer identity.',
        'qualifications': {'report_rows': reports, 'unique_tasks': len(tasks), 'unique_lineage_function_groups': len(groups),
                           'by_lineage': dict(sorted(Counter(t['lineage'] for t in tasks.values()).items())),
                           'by_family': dict(sorted(Counter(t['family'] for t in tasks.values()).items()))},
        'fixed12': {'assigned_arms': len(assignments), 'classified_arms': fixed['completed_arms'],
                    'launched_arms': fixed['launched_arms'], 'complete_pairs': fixed['complete_pairs'],
                    'final_oracle_arms': [{'index': a['index'], 'task': a['task'], 'mode': a['mode'],
                                          'outcome': a['decision']['outcome']} for a in final],
                    'limitations': audit['limitations']},
        'mature_common_event': {'event': 'task_accepted_in_budget@v1', 'eligible_independent_episodes': None,
            'status': 'not established by these input artifacts',
            'maturity_implementation': 'task-close-plus-seven-days@v1: explicit as_of, persisted evidence, fit-time reconstruction; legacy evidence remains maturity-unknown. This inventory does not supply missing episode/review mappings.',
            'missing_mapping': ['episode event-log identities with task closure and budget evidence',
                                'explicit independent operator review records, criteria and review time',
                                'episode-to-task/family/lineage/clone mapping and maturity/censoring',
                                'unexposed held-out split and calibration validation'],
            'existing_ingestion': 'tsr train dataset --events EVENTS --tasks TASK_INDEX --labels OPERATOR_LABELS --as-of RFC3339 --by time|repo --out NEW_DIRECTORY',
            'minimum_fit_calibrate_tasks': 150, 'minimum_per_family': 30,
            'warning': 'Qualification, repeated arms, oracle success and synthetic M4 examples are not interchangeable with mature reviewed episodes.'},
        'sources_sha256': dict(sorted(sources.items())),
    }


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[1])
    print(json.dumps(inventory(parser.parse_args().root), indent=2, sort_keys=True))
