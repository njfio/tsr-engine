"""Preserve native admission evidence, excluding disposable source/build copies."""
from pathlib import Path
import json, shutil, re, os
from m3_export_guard import terminal_attempt, retain_history
root=Path(__file__).resolve().parents[1]
records=[]
previous_summary = root/"docs/m3-native-admission/summary.json"
historical = {a["attempt"]: a for a in json.loads(previous_summary.read_text())["attempts"]} if previous_summary.exists() else {}
skipped_active = []
attempts = [('m3-qualified-pilot',False),('m3-qualified-core',True),('m3-qualified-core-v2',True),('m3-qualified-remaining',True),('m3-qualified-daemon-retry',True),('m3-qualified-cancel-tokio',True),('m3-qualified-agentid',True),('m3-qualified-agentid-v2',True)]
attempts += [(path.name, True) for pattern in ('m3-qualified-resume-*','m3-qualified-agentid-next-*','m3-qualified-agentid-following-*','m3-qualified-rts-broad-*','m3-qualified-agentid-db-*','m3-qualified-rts-core-*','m3-qualified-agentid-storage-*','m3-qualified-rts-structure-*','m3-qualified-rts-extraction-*','m3-qualified-rts-indexing-*','m3-qualified-agentid-records-*','m3-qualified-rts-behavior-*','m3-qualified-rts-language-*','m3-qualified-rts-signature-*','m3-qualified-rts-metrics-*','m3-qualified-agentid-policy-*','m3-qualified-rts-cache-key-*','m3-qualified-rts-fuzzy-*','m3-qualified-rts-syntax-*','m3-qualified-rts-typed-*','m3-qualified-rts-reference-*','m3-qualified-rts-context-*','m3-qualified-rts-filtering-*','m3-qualified-rts-supplement-daemon-continuation-*','m3-qualified-rts-supplement-wire-continuation-*','m3-qualified-rts-supplement-predicates-continuation-*','m3-qualified-rts-supplement-structural-continuation-*') for path in sorted((root/'target').glob(pattern))]
for name, strict in retain_history(attempts, historical):
    source=root/'target'/name
    if name in historical:
        records.append(historical[name])
        continue
    if not terminal_attempt(source, root/'target', root/'docs/m3-native-admission'):
        skipped_active.append(name)
        if name in historical:
            records.append(historical[name])
        continue
    dest=root/'docs/m3-native-admission'/name
    dest.mkdir(parents=True,exist_ok=True)
    for directory, subdirs, files in os.walk(source):
        subdirs[:] = [name for name in subdirs if name != 'copy']
        for name_in_dir in files:
            path = Path(directory)/name_in_dir
            rel=path.relative_to(source)
            target=dest/rel;target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(path,target)
    reports=[json.loads(line) for line in (dest/'admission.jsonl').read_text().splitlines()]
    phases=[]
    for phase in dest.glob('*/*.json'):
        data=json.loads(phase.read_text())
        if 'run' not in data: continue
        task_dir=phase.parent
        run=json.loads((task_dir/'cas'/data['run']).read_text())
        stdout=(task_dir/'cas'/run['result']['stdout_digest']).read_text()
        totals=[sum(int(row[i]) for row in re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',stdout)) for i in range(3)]
        phases.append({'task':task_dir.name,'phase':phase.stem,'exit':data['exit'],'duration_ms':run['result']['duration_ms'],'totals_passed_failed_ignored':totals,'run_digest':data['run']})
    phases.sort(key=lambda p: (p['task'], {'baseline':0,'red':1,'mutated':2,'green':3,'reversal':4}.get(p['phase'].split('-')[0],5), int(p['phase'].split('-')[1]) if p['phase'].split('-')[-1].isdigit() else 0))
    expected = len((dest/'tasks.jsonl').read_text().splitlines())
    records.append({'attempt':name,'strict_protocol':strict,'reports':reports,'phases':phases,'accepted_for_m3':sum(strict and r['accepted'] for r in reports),'complete':len(reports)==expected})
# A retry or same-function mutation must never inflate the corpus count.
unique_groups = set()
unique_tasks = set()
accepted_lineages = set()
accepted_families = set()
lineage_groups = {}
for attempt in records:
    manifest = root/'docs/m3-native-admission'/attempt['attempt']/'tasks.jsonl'
    accepted_rows = {row['id']:row for row in (json.loads(line) for line in manifest.read_text().splitlines())}
    for report in attempt['reports']:
        if attempt['strict_protocol'] and report['accepted']:
            unique_groups.add(report['function_group'])
            unique_tasks.add(report['task_id'])
            accepted_lineages.add(accepted_rows[report['task_id']]['lineage'])
            accepted_families.add(accepted_rows[report['task_id']]['family'])
            lineage_groups.setdefault(accepted_rows[report['task_id']]['lineage'],set()).add(report['function_group'])
summary={'attempts':records,'admitted_production_tasks':len(unique_groups),'accepted_unique_task_ids':sorted(unique_tasks),'paid_runs':0,'paired_system_runs':0,'repository_lineages':len(accepted_lineages),'accepted_lineages':sorted(accepted_lineages),'accepted_families':sorted(accepted_families),'admitted_by_lineage':{key:len(value) for key,value in sorted(lineage_groups.items())},'m3_acceptance_met':False}
probe_path = root/'docs/m3-provider-pilot.json'
if probe_path.exists():
    probe = json.loads(probe_path.read_text())
    summary['provider_compatibility_probes'] = 1
    summary['provider_accounted_usd'] = probe['accounted_usd']
    summary['paid_runs_definition'] = 'Paid corpus task runs; excludes harmless compatibility probe.'
# Operational evidence is separate from qualification; never reset its accounting.
from decimal import Decimal
ledger_path = root/'target/m3-provider-cost.jsonl'
if ledger_path.exists():
    events = [json.loads(line) for line in ledger_path.read_text().splitlines()]
    reserves = {e['id']:Decimal(e['usd']) for e in events if e['kind']=='reserve'}
    settled = {e['id']:Decimal(e['usd']) for e in events if e['kind']=='settled'}
    if len(reserves) != sum(e['kind']=='reserve' for e in events) or len(settled) != sum(e['kind']=='settled' for e in events) or not settled.keys() <= reserves.keys():
        raise ValueError('Invalid provider ledger identities')
    summary['provider_accounted_usd'] = str(sum(settled.values(), Decimal(0)))
    summary['provider_outstanding_reserved_usd'] = str(sum((v for k,v in reserves.items() if k not in settled), Decimal(0)))
    summary['provider_settled_calls'] = len(settled)
    summary['provider_halt_events'] = sum(e['kind']=='halt' for e in events)
for name in ('m3-live-pilot-result.json','m3-pilot-retry-v2-result.json','m3-paired-cohort-1-result.json','m3-paired-cohort-2-result.json'):
    path = root/'docs'/name
    if not path.exists():continue
    report = json.loads(path.read_text())
    summary['paid_runs'] += sum(len(a.get('results', [])) if 'results' in a else int(a.get('result') is not None) for a in report['arms'])
    summary['paired_system_runs'] += report.get('evaluable_paired_tasks', len(report.get('observed_pairs', [])))
summary['paid_runs_definition'] = 'Saved live-driver arm result rows across separate attempts/cohorts; excludes CLI preflight failures without results and the probe. Provider settled calls and dollars are reported separately.'
summary['paired_system_runs_definition'] = 'Sum of observed accepted/rejected task pairs in separately reported cohorts; not pooled performance evidence.'
(root/'docs/m3-native-admission/summary.json').write_text(json.dumps(summary,indent=2)+'\n')

progress_path=root/'docs/m3-corpus-progress.json'
progress=json.loads(progress_path.read_text())
progress['admitted_production_tasks']=summary['admitted_production_tasks']
for field in ('provider_compatibility_probes','provider_accounted_usd','provider_outstanding_reserved_usd','provider_settled_calls','provider_halt_events','paid_runs','paired_system_runs','paid_runs_definition','paired_system_runs_definition'):
    if field in summary: progress[field]=summary[field]
progress['repository_lineages']=summary['repository_lineages']
progress['accepted_families']=summary['accepted_families']
progress.pop('strict_admission_attempts',None)
progress['native_admission_attempts']=records
progress_path.write_text(json.dumps(progress,indent=2)+'\n')

if 'full_mutation_run' in progress:
    progress['surface_pilot_full_mutation_run']=progress.pop('full_mutation_run')
progress['independently_exercised_mutation_pilots']=1+len({phase['task'] for attempt in records for phase in attempt['phases']})
progress_path.write_text(json.dumps(progress,indent=2)+'\n')

# Human-readable index; historical attempts remain visible in the JSON and raw CAS.
lines = ['# Native M3 admission evidence', '',
         f"Qualified distinct function groups: **{summary['admitted_production_tasks']}**.",
         f"Repository lineages represented by qualified tasks: **{summary['repository_lineages']}**.",
         f"Observed paired task outcomes across separately reported cohorts: **{summary['paired_system_runs']}**. M3 acceptance remains incomplete.", '',
         'These are real-source test outcomes, not model-performance measurements. Admission requires a completed full green baseline, each named red check, completed full red, exact tracked-source reversal, each named green check, and completed full green. Interrupted or timed-out phases never count as admissions.', '',
         '| Task | Family | Lineage | Admission evidence |',
         '| --- | --- | --- | --- |']
seen = set()
for attempt in records:
    if not attempt['strict_protocol']:
        continue
    source = root/'docs/m3-native-admission'/attempt['attempt']
    rows = {row['id']:row for row in (json.loads(line) for line in (source/'tasks.jsonl').read_text().splitlines())}
    for report in attempt['reports']:
        if not report['accepted'] or report['function_group'] in seen:
            continue
        seen.add(report['function_group'])
        task = rows[report['task_id']]
        lines.append(f"| `{task['id'][:12]}` | {task['family']} | {task['lineage']} | [{attempt['attempt']}]({attempt['attempt']}/admission.jsonl) |")
lines += ['', 'All attempts, including preflight refusals and incomplete runs, are preserved in [summary.json](summary.json). Raw evidence stays outside disposable source/build copies.', '',
          'The first repository remains one lineage regardless of function, crate or revision. A second source baseline alone does not add an admitted-task lineage or supply paired model outcomes.']
(root/'docs/m3-native-admission/README.md').write_text('\n'.join(lines)+'\n')

print(json.dumps(dict(admitted=summary["admitted_production_tasks"], skipped_unverified_attempts=skipped_active)))
