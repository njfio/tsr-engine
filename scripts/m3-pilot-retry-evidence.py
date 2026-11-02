"""Export terminal metadata only; retain originals and never export response bodies."""
from pathlib import Path
from datetime import datetime, timezone
from decimal import Decimal
import hashlib
import json
import shutil
import sqlite3

ROOT = Path(__file__).resolve().parents[1]


def main():
    state = ROOT / 'target/m3-live-pilot-retry-v2-state'
    complete = json.loads((state / 'complete.json').read_text())
    destination = ROOT / 'docs/m3-pilot-retry-v2-execution'
    destination.mkdir()
    for path in state.iterdir():
        if path.suffix in ('.json', '.log'):
            shutil.copyfile(path, destination / ('state-' + path.name))
    for relative in ('target/m3-provider-cost.jsonl',
                     'target/m3-provider-cost-diagnostics.jsonl',
                     'target/m3-pilot-retry-v2-broker.py',
                     'scripts/m3-retry-pilot.py'):
        shutil.copyfile(ROOT / relative, destination / Path(relative).name)
    arms = []
    for source, mode in [('agentid', 'record'), ('rts', 'record'), ('rts', 'rules')]:
        base = ROOT / 'target/m3-live-pilot-retry-v2' / source / mode
        result = base / ('results-' + mode + '.jsonl')
        record = dict(source=source, mode=mode, original_directory=str(base.relative_to(ROOT)))
        process = state / (source + '-' + mode + '.json')
        record['process'] = json.loads(process.read_text()) if process.exists() else None
        record['results'] = [json.loads(line) for line in result.read_text().splitlines()] if result.exists() else []
        if result.exists():
            record['result_sha256'] = hashlib.sha256(result.read_bytes()).hexdigest()
            shutil.copyfile(result, destination / (source + '-' + mode + '-results.jsonl'))
        stores = list((base / 'state').glob('*/*.sqlite'))
        if len(stores) == 1:
            with sqlite3.connect(f'file:{stores[0]}?mode=ro', uri=True) as connection:
                record['event_counts'] = dict(connection.execute('select kind,count(*) from events group by kind'))
                record['closure_reasons'] = [json.loads(row[0])['kind']['reason'] for row in connection.execute("select payload from events where kind='task_closed'")]
        arms.append(record)
    events = [json.loads(line) for line in (ROOT / 'target/m3-provider-cost.jsonl').read_text().splitlines()]
    reserved = {row['id']: Decimal(row['usd']) for row in events if row['kind'] == 'reserve'}
    settled = {row['id']: Decimal(row['usd']) for row in events if row['kind'] == 'settled'}
    if len(reserved) != sum(row['kind'] == 'reserve' for row in events) or len(settled) != sum(row['kind'] == 'settled' for row in events) or not settled.keys() <= reserved.keys():
        raise ValueError('Invalid ledger identities')
    total = sum(settled.values(), Decimal(0))
    old = json.loads((ROOT / 'docs/m3-live-pilot-result.json').read_text())
    selected = arms + [arm for arm in old['arms'] if arm['source'] == 'agentid' and arm['mode'] == 'rules']
    plan = json.loads((ROOT / 'docs/m3-pilot-retry-v2-plan.json').read_text())
    eligible = []
    for source in ('agentid', 'rts'):
        pair = [arm for arm in selected if arm['source'] == source]
        if len(pair) == 2 and {arm['mode'] for arm in pair} == {'record', 'rules'} and all(
            arm['process'] and arm['process']['status'] == 'completed'
            and arm['process']['exit'] == 0 and not arm['process']['errors']
            and len(arm['results']) == 1
            and arm['results'][0]['task'] == plan['task_ids'][source]
            and arm['results'][0]['system'] == arm['mode']
            and arm['results'][0]['outcome'] in ('accepted', 'rejected') for arm in pair):
            eligible.append(source)
    summary = dict(recorded_at=datetime.now(timezone.utc).isoformat(), finished=complete['finished'],
                   original_attempts='m3-live-pilot-result.json', plan='m3-pilot-retry-v2-plan.json',
                   arms=arms, comparison_selection='Prospective retry arms plus original AgentID rules; all prior failures retained.',
                   evaluable_paired_sources=eligible, evaluable_paired_tasks=len(eligible),
                   spending=dict(aggregate_limit_usd='100', original_probe_usd='0.000112', original_pilot_usd='0.016441',
                                 retry_usd=str(total-Decimal('0.016553')), all_settled_usd=str(total),
                                 outstanding_reserved_usd=str(sum((amount for key,amount in reserved.items() if key not in settled),Decimal(0))),
                                 settled_calls=len(settled), halt_events=sum(row['kind']=='halt' for row in events)),
                   corpus_admissions=32, corpus_qualification='stopped; not restarted', m3_acceptance_met=False,
                   limitations=['No best-of selection or extra retries for terminal model responses.',
                                'Original RTS failure subtype is unknown; prospective diagnostics cannot retroactively establish it.',
                                'Batch events use fixed clock; controller process records contain actual times.',
                                'This two-task pilot cannot establish general performance or M3 acceptance.',
                                'RTS rules was rejected by the frozen test-evidence guard after changing surface.rs; it did not receive a full oracle verdict.',
                                'RTS rules reports wall_clock_ms=0 on its tamper-rejection path. This is not actual runtime; process elapsed time is about391 seconds. Do not use that field for latency comparisons.',
                                'One paired task, both rejected, provides no evidence of a performance improvement. AgentID record is unavailable from terminal tool_shape and has no paired comparison.'])
    (ROOT / 'docs/m3-pilot-retry-v2-result.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({key: summary[key] for key in ('evaluable_paired_sources', 'spending')}, indent=2))


if __name__ == '__main__':
    main()
