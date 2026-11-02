"""Summarize observed accepted-run cost; projections are conditional, not corpus yield."""
import json
import re
import statistics
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]
summary = json.loads((ROOT / 'docs/m3-native-admission/summary.json').read_text())
seen = set()
rows = []
for attempt in summary['attempts']:
    base = ROOT / 'docs/m3-native-admission' / attempt['attempt']
    tasks = {t['id']: t for t in map(json.loads, (base / 'tasks.jsonl').read_text().splitlines())}
    for report in attempt['reports']:
        if not attempt['strict_protocol'] or not report['accepted'] or report['function_group'] in seen:
            continue
        seen.add(report['function_group'])
        task = report['task_id']
        directory = base / task
        runs = list(map(json.loads, (directory / 'runs.jsonl').read_text().splitlines()))
        phases = [p for p in attempt['phases'] if p['task'] == task]
        row = dict(task_id=task, attempt=attempt['attempt'], lineage=tasks[task]['lineage'],
            named_tests=len(tasks[task]['failing_tests']), command_minutes=sum(r['result']['duration_ms'] for r in runs)/60000,
            phases_minutes={p['phase']: p['duration_ms']/60000 for p in phases})
        row['prerequisite_build_minutes'] = sum(r['result']['duration_ms'] for r in runs if r['argv'][:2] == ['cargo','build'])/60000
        row['full_suite_harness_seconds'] = {}
        for phase in phases:
            if phase['phase'] not in ('baseline','mutated-full','reversal-full'):
                continue
            run = json.loads((directory / 'cas' / phase['run_digest']).read_text())
            stdout = (directory / 'cas' / run['result']['stdout_digest']).read_text()
            row['full_suite_harness_seconds'][phase['phase']] = sum(float(x) for x in re.findall(r'test result:.*?finished in ([0-9.]+)s', stdout))
        rows.append(row)
lineages = {}
for lineage in sorted({r['lineage'] for r in rows}):
    values = [r['command_minutes'] for r in rows if r['lineage'] == lineage]
    lineages[lineage] = dict(cases=len(values), minimum_minutes=min(values), median_minutes=statistics.median(values), maximum_minutes=max(values))
remaining = 150 - summary['admitted_production_tasks']
result = dict(admitted=summary['admitted_production_tasks'], remaining_to_150=remaining, observations=rows, by_lineage=lineages,
    measurement='Sum of sealed command durations per accepted case; excludes copying, hashing, screening, failed/interrupted attempts, setup and model runs.',
    conditional_projection=dict(workers=2, minutes_per_attempt_range=[8,28], qualification_only_hours_if_every_attempt_admits=[remaining*8/120,remaining*28/120], qualification_only_hours_if_half_attempts_admit=[remaining*8/60,remaining*28/60]),
    caveats=['Purposive tiny sample; no unbiased admission-yield estimate.', 'At least this many additional distinct eligible functions must first be found and caught.', 'Steady two-worker utilization assumed; not a wall-clock completion promise.', 'Paid paired model runs and their test costs are excluded.'])
(ROOT / 'docs/m3-throughput.json').write_text(json.dumps(result, indent=2)+'\n')
print(json.dumps(dict(admitted=result['admitted'],by_lineage=lineages,conditional_projection=result['conditional_projection']),indent=2))
