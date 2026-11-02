"""Export the predeclared paired cohort, including missing and unavailable arms."""
from pathlib import Path
from decimal import Decimal
import hashlib
import json
import math
import shutil
import sqlite3
import subprocess

ROOT = Path(__file__).resolve().parents[1]


def wilson(successes, total):
    z = 1.959963984540054
    p = successes / total
    denominator = 1 + z*z/total
    center = (p + z*z/(2*total)) / denominator
    radius = z * math.sqrt(p*(1-p)/total + z*z/(4*total*total)) / denominator
    return [max(0, center-radius), min(1, center+radius)]


def require_quiescent(state):
    pids = {json.loads((ROOT / 'docs/m3-smoke-launch.json').read_text())['pid']}
    for path in state.glob('*.json'):
        row = json.loads(path.read_text())
        if isinstance(row, dict) and row.get('pid'):
            pids.add(row['pid'])
    rows = subprocess.check_output(['ps','-axo','pid=,pgid='],text=True,timeout=15).splitlines()
    if any(any(int(value) in pids for value in row.split()) for row in rows if row.strip()):
        raise ValueError('Controller/arm process or process group remains')
    names = subprocess.check_output(['docker','ps','--format','{{.Names}}'],text=True,timeout=15).splitlines()
    if any(name.startswith('tsr-m3-') for name in names):
        raise ValueError('M3 container remains')


def main():
    state = ROOT / 'target/m3-smoke-comparison-state'
    complete = json.loads((state / 'complete.json').read_text())
    plan = json.loads((state / 'plan.json').read_text())
    if len(plan['jobs']) != 2 or len({j['task_id'] for j in plan['jobs']}) != 2 or any(j['modes'] != ['record', 'rules'] for j in plan['jobs']):
        raise ValueError('Frozen cohort shape invalid')
    if complete.get('status') not in ('completed','halted-review-required'):
        raise ValueError('Controller not terminal')
    require_quiescent(state)
    destination = ROOT / 'docs/m3-smoke-execution'
    destination.mkdir()
    for path in state.iterdir():
        if path.suffix in ('.json', '.jsonl', '.log'):
            shutil.copyfile(path, destination / ('state-' + path.name))
    freeze = ROOT / 'docs/m3-smoke-freeze-v3.json'
    if freeze.exists(): shutil.copyfile(freeze, destination / 'freeze-v3.json')
    for name in ('m3-provider-cost.jsonl', 'm3-provider-cost-diagnostics.jsonl'):
        shutil.copyfile(ROOT / 'target' / name, destination / name)
    arms = []
    for job in plan['jobs']:
        base = ROOT / job['input']
        shutil.copytree(base / 'cas', destination / (job['task_id'][:12] + '-input-cas'))
        for name in ('tasks.jsonl', 'profile.toml', 'broker-contexts.json'):
            shutil.copyfile(base / name, destination / (job['task_id'][:12] + '-' + name))
        for mode in job['modes']:
            label = job['task_id'][:12] + '-' + mode
            process = state / (label + '.json')
            result = base / mode / ('results-' + mode + '.jsonl')
            row = dict(task=job['task_id'], source=job['source'], family=job['family'], mode=mode,
                       process=json.loads(process.read_text()) if process.exists() else None,
                       result=None, closure_reasons=[])
            if result.exists():
                results = [json.loads(line) for line in result.read_text().splitlines()]
                if len(results) != 1 or results[0]['task'] != job['task_id'] or results[0]['system'] != mode:
                    raise ValueError('Result identity mismatch')
                row['result'] = results[0]
                row['sha256'] = hashlib.sha256(result.read_bytes()).hexdigest()
                shutil.copyfile(result, destination / (label + '-results.jsonl'))
            stores = list((base / mode / 'state').glob('*/*.sqlite'))
            if len(stores) == 1:
                with sqlite3.connect(f'file:{stores[0]}?mode=ro', uri=True) as connection:
                    row['closure_reasons'] = [json.loads(r[0])['kind']['reason'] for r in connection.execute("select payload from events where kind='task_closed'")]
            if len(stores) == 1:
                export_store = destination / (label + '-state.sqlite')
                with sqlite3.connect(f'file:{stores[0]}?mode=ro', uri=True) as source_db, sqlite3.connect(export_store) as copy_db:
                    source_db.backup(copy_db)
            for directory in ['cas']:
                original = base / mode / directory
                if original.is_dir():
                    shutil.copytree(original, destination / (label + '-' + directory))
            row['oracle_closure_consistent'] = bool(row['result'] and row['result'].get('oracle') is not None and row['closure_reasons'] == ['oracle:' + str(row['result']['oracle']['accepted']).lower()])
            row['evidence_consistent'] = bool(row['result'] and (row['result']['outcome'] != 'accepted' or (row['oracle_closure_consistent'] and row['result']['oracle'] == dict(accepted=True,flipped_tests_pass=True,suite_regressed=False,exit_code=0))))
            process = row['process']
            row['process_clean'] = bool(process and process['status'] == 'completed' and process['exit'] == 0 and not process['errors'])
            arms.append(row)
    metrics = {}
    for mode in ('record', 'rules'):
        rows = [a for a in arms if a['mode'] == mode]
        counts = {key: sum(a['result'] is not None and a['result']['outcome'] == key for a in rows)
                  for key in ('accepted', 'rejected', 'unavailable', 'not_attempted')}
        counts['missing'] = sum(a['result'] is None for a in rows)
        valid_successes = sum(a['process_clean'] and a['evidence_consistent'] and a['result'] is not None and a['result']['outcome'] == 'accepted' for a in rows)
        metrics[mode] = dict(raw_outcome_counts=counts, original_process_only_accepted=sum(a['process_clean'] and a['result'] is not None and a['result']['outcome']=='accepted' for a in rows), evidence_consistent_accepted=valid_successes, process_failures=sum(not a['process_clean'] for a in rows), valid_accepted=valid_successes, fixed_denominator=2, observed_acceptance_fraction=valid_successes/2, descriptive_wilson95=wilson(valid_successes, 2))
    pairs = []
    for job in plan['jobs']:
        rows = [a for a in arms if a['task'] == job['task_id']]
        if len(rows) == 2 and {a['mode'] for a in rows} == {'record', 'rules'} and all(a['process'] and a['process']['status'] == 'completed' and a['process']['exit'] == 0
               and not a['process']['errors'] and a['evidence_consistent'] and a['result'] and a['result']['outcome'] in ('accepted','rejected') for a in rows):
            if rows[0]['result']['state_root_at_start'] != rows[1]['result']['state_root_at_start']:
                raise ValueError('Paired start roots differ')
            pairs.append(dict(task=job['task_id'], state_root_at_start=rows[0]['result']['state_root_at_start'], **{a['mode']:a['result']['outcome'] for a in rows}))
    if not (ROOT / 'target/m3-provider-cost.jsonl').read_bytes().startswith((state / 'cost-before.jsonl').read_bytes()):
        raise ValueError('Historical ledger prefix changed')
    events = [json.loads(line) for line in (ROOT / 'target/m3-provider-cost.jsonl').read_text().splitlines()]
    prior = [json.loads(line) for line in (state / 'cost-before.jsonl').read_text().splitlines()]
    settled = {e['id']:Decimal(e['usd']) for e in events if e['kind']=='settled'}
    reserved = {e['id']:Decimal(e['usd']) for e in events if e['kind']=='reserve'}
    if len(settled) != sum(e['kind']=='settled' for e in events) or len(reserved) != sum(e['kind']=='reserve' for e in events) or not settled.keys() <= reserved.keys():
        raise ValueError('Invalid ledger identities')
    total = sum(settled.values(),Decimal(0))
    prior_total = sum((Decimal(e['usd']) for e in prior if e['kind']=='settled'),Decimal(0))
    summary = dict(finished=complete['finished'], arms=arms, metrics=metrics, observed_pairs=pairs,
        discordant_counts=dict(record_only=sum(p['record']=='accepted' and p['rules']=='rejected' for p in pairs),
                               rules_only=sum(p['rules']=='accepted' and p['record']=='rejected' for p in pairs)),
        spending=dict(prior_usd=str(prior_total), cohort_usd=str(total-prior_total), total_usd=str(total),
                      outstanding_reserved_usd=str(sum((v for k,v in reserved.items() if k not in settled),Decimal(0)))),
        limitations=['Raw evaluator acceptance criteria and outcomes are unchanged; original process-only and corrected evidence-consistent accepted counts are reported separately.', 'Oracle result/closure consistency is checked; raw final oracle stdout/stderr is not persisted by this evaluator, so no independent receipt-level verification is claimed.', 'Pilot not pooled. Two selected tasks in one lineage are exploratory, not a population estimate.',
                     'Fixed-denominator acceptance is conservative observed success yield, not imputed success of missing arms. Wilson intervals ignore lineage dependence and are descriptive only.',
                     'Infrastructure, model/protocol failures and guard rejections remain in the arm records; no best-of selection.',
                     'Frozen engine can report zero wall_clock_ms on early rejection; no latency comparison is made.',
                     'M3 acceptance requires at least150 qualifying tasks and broader paired evidence; this cohort alone cannot satisfy it.'])
    (ROOT / 'docs/m3-smoke-result.json').write_text(json.dumps(summary,indent=2)+'\n')
    manifest = {str(p.relative_to(destination)): hashlib.sha256(p.read_bytes()).hexdigest() for p in destination.rglob('*') if p.is_file()}
    (ROOT / 'docs/m3-smoke-export-sha256.json').write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps(dict(metrics=metrics, observed_pairs=len(pairs), spending=summary['spending']),indent=2))


if __name__ == '__main__':
    main()
