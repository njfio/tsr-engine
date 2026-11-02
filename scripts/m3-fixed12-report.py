"""Read-only terminal analysis/export; never launches evaluation or inference."""
import collections
import datetime
from decimal import Decimal as D
import hashlib
import json
import math
from pathlib import Path
import shutil

ROOT = Path(__file__).resolve().parents[1]


def read(path):
    return json.loads(path.read_text())


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def save(path, value):
    with path.open('x') as f:
        json.dump(value, f, indent=2)
        f.write('\n')


def wilson(k, n):
    z = 1.959963984540054
    p = k / n
    center = (p + z*z/(2*n))/(1+z*z/n)
    half = z*math.sqrt(p*(1-p)/n+z*z/(4*n*n))/(1+z*z/n)
    return [max(0, center-half), min(1, center+half)]


def main():
    m = read(ROOT/'docs/m3-fixed12-freeze.json')
    s = ROOT/m['state']
    terminal = read(s/'terminal.json')
    assert terminal['status'] in ('completed', 'halted-review-required')
    decisions = {r['index']: r for r in terminal['results']}
    assert all(read(s/str(i)/'decision.json') == r for i, r in decisions.items())
    baseline = read(ROOT/m['baseline'])
    raw = (ROOT/m['ledger']).read_bytes()
    assert hashlib.sha256(raw[:baseline['bytes']]).hexdigest() == baseline['sha256']
    ledger = [json.loads(line) for line in raw[baseline['bytes']:].splitlines()]
    reserves = {r['id']: r for r in ledger if r['kind'] == 'reserve'}
    settled = {r['id']: r for r in ledger if r['kind'] == 'settled'}
    assert len(reserves) == sum(r['kind'] == 'reserve' for r in ledger)
    assert len(settled) == sum(r['kind'] == 'settled' for r in ledger)
    assert set(reserves) == set(settled), 'unsettled billing: do not export clean accounting'
    assert {r['kind'] for r in ledger} <= {'reserve', 'settled'}
    additional = sum((D(r['usd']) for r in settled.values()), D(0))
    assert sum(r['requests'] for r in decisions.values()) == len(settled)
    assert sum((D(r['settled_usd']) for r in decisions.values()), D(0)) == additional
    pins = {p: sha(ROOT/p) == digest for p, digest in m['sha256'].items()}
    assert all(pins.values()), 'frozen pin mismatch'
    rows = []
    for arm in m['arms']:
        i = arm['index']
        worker_paths = list((s/str(i)).glob(arm['task'][:12]+'-'+arm['mode']+'.json'))
        worker = read(worker_paths[0]) if worker_paths else None
        r = dict(arm)
        r.update(decisions.get(i, dict(outcome='infrastructure-interrupted' if worker else 'unattempted',
                                      final_oracle_completed=False, final_oracle_success=False,
                                      requests=0, settled_usd='0')))
        r['worker'] = worker
        r['missing_or_unclassified'] = i not in decisions
        r['broker_elapsed_ms'] = sum(d.get('elapsed_ms', 0) for d in
            read(s/str(i)/'evidence/diagnostics.json')) if i in decisions else 0
        rows.append(r)
    by_mode = {}
    for mode in ('record', 'rules'):
        selected = [r for r in rows if r['mode'] == mode]
        successes = sum(r['final_oracle_success'] for r in selected)
        missing = sum(r['missing_or_unclassified'] for r in selected)
        by_mode[mode] = dict(assigned=12, completed=12-missing, verified_successes=successes,
            missing=missing, operational_success_fraction=successes/12,
            missingness_bounds=[successes/12, (successes+missing)/12],
            wilson_95=wilson(successes, 12), outcomes=dict(collections.Counter(r['outcome'] for r in selected)),
            calls=sum(r['requests'] for r in selected),
            additional_usd=str(sum((D(r['settled_usd']) for r in selected), D(0))),
            worker_seconds=sum(r.get('latency_seconds', 0) for r in selected),
            broker_elapsed_ms=sum(r['broker_elapsed_ms'] for r in selected))
    pairs = []
    for i in range(12):
        pair = {r['mode']: r for r in rows if r['pair_index'] == i}
        pairs.append(dict(pair_index=i, task=pair['record']['task'], file=pair['record']['file'],
            record=pair['record']['outcome'], rules=pair['rules']['outcome'],
            record_success=pair['record']['final_oracle_success'], rules_success=pair['rules']['final_oracle_success'],
            complete=not any(r['missing_or_unclassified'] for r in pair.values())))
    complete = [p for p in pairs if p['complete']]
    result = dict(timestamp=datetime.datetime.now(datetime.timezone.utc).isoformat(),
        terminal_status=terminal['status'], terminal_reason=terminal.get('reason'),
        underlying_halt=rows[22]['worker'], arms=rows, pairs=pairs, by_mode=by_mode,
        completed_arms=len(decisions), complete_pairs=len(complete),
        launched_arms=sum(r['worker'] is not None for r in rows),
        paid_calls=len(settled), additional_usd=str(additional),
        cumulative_usd=str(D(baseline['settled_usd'])+additional), outstanding_reservations=0,
        free_bytes=shutil.disk_usage(ROOT).free, live_floor_bytes=200*1024**3,
        frozen_pins_verified=len(pins), freeze_sha256=sha(ROOT/'docs/m3-fixed12-freeze.json'),
        paired_operational_difference_rules_minus_record=(by_mode['rules']['verified_successes']-by_mode['record']['verified_successes'])/12,
        complete_pair_categories=dict(collections.Counter(
            'both' if p['record_success'] and p['rules_success'] else 'record_only' if p['record_success'] else
            'rules_only' if p['rules_success'] else 'neither' for p in complete)),
        exact_mcnemar=None, exact_mcnemar_reason='Not all 24 arms are terminal/classified; preregistered condition unmet.',
        data_scope='Constructed mutations in two real repositories; not historical bug fixes or an untouched holdout.')
    dest = ROOT/'docs/m3-fixed12-execution'
    if dest.exists():
        raise ValueError('export already exists')
    for p in s.rglob('*'):
        assert not p.is_symlink() and (p.is_file() or p.is_dir())
    shutil.copytree(s, dest)
    for name in ('m3-fixed12-freeze.json', 'm3-fixed12-baseline.json', 'm3-fixed12-launch-check.json', 'm3-fixed12-analysis-plan.md'):
        shutil.copyfile(ROOT/'docs'/name, dest/name)
    (dest/'ledger.jsonl').write_bytes(raw)
    shutil.copyfile(ROOT/'target/m3-fixed12-live.log', dest/'controller.log')
    # Interrupted preparation had no paid request or verified outcome. Preserve its bounded metadata/logs.
    partial = ROOT/m['arms'][22]['input']/m['arms'][22]['mode']
    for p in partial.glob('*.json*'):
        if p.is_file() and not p.is_symlink():
            shutil.copyfile(p, dest/'22'/('partial-'+p.name))
    save(ROOT/'docs/m3-fixed12-result.json', result)
    save(ROOT/'docs/m3-fixed12-export-sha256.json', {str(p.relative_to(dest)): sha(p) for p in dest.rglob('*') if p.is_file()})
    print(json.dumps({k:v for k,v in result.items() if k not in ('arms','pairs','underlying_halt')}))


if __name__ == '__main__':
    main()
