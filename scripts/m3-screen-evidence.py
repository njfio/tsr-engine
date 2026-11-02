"""Capture bounded native screening artifacts without executing candidate code."""
from pathlib import Path
import json, shutil, hashlib
root=Path(__file__).resolve().parents[1]
cohorts={'pilot':root/'target/m3-native-library-pilot', **{name:root/'target/m3-diverse-screen'/name for name in ('rts-daemon','rts-mcp','rts-core')}}
records=[]
for name,source in cohorts.items():
    dest=root/'docs/m3-native-screen'/name
    dest.mkdir(parents=True,exist_ok=True)
    for filename in ('tasks.jsonl','runs.jsonl'):
        shutil.copyfile(source/filename,dest/filename)
    shutil.copytree(source/'cas',dest/'cas',dirs_exist_ok=True)
    tasks=[json.loads(line) for line in (source/'tasks.jsonl').read_text().splitlines()]
    records.append({'cohort':name,'candidates':len(tasks),'families':sorted({t['family'] for t in tasks}),'lineages':sorted({t['lineage'] for t in tasks}),'manifest_sha256':hashlib.sha256((source/'tasks.jsonl').read_bytes()).hexdigest(),'admitted':False})
summary={'source_tree':'02cca9df15e72f6506275157b0198526950d6fcb','preparation_id':'9cdacb959d4999301043e7164c968b704844aa84e55cf0a7cbc660fbe33c7326','cohorts':records,'diverse_selected':11,'diverse_caught':sum(x['candidates'] for x in records if x['cohort']!='pilot'),'diverse_caught_families':sorted({f for x in records if x['cohort']!='pilot' for f in x['families']}),'repository_lineages':1,'screening_is_admission':False}
(root/'docs/m3-native-screen/summary.json').write_text(json.dumps(summary,indent=2)+'\n')

progress_path=root/'docs/m3-corpus-progress.json'
progress=json.loads(progress_path.read_text())
progress.update({'unique_screened_mutations':summary['diverse_selected']+1,'tool_reported_caught':sum(x['candidates'] for x in records),'diverse_selected':summary['diverse_selected'],'diverse_caught':summary['diverse_caught'],'diverse_caught_families':len(summary['diverse_caught_families']),'repository_lineages':summary['repository_lineages']})
progress.pop('mutation_candidates_executed',None)
progress_path.write_text(json.dumps(progress,indent=2)+'\n')
