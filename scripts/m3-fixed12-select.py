"""Outcome-blind deterministic sampling from complete strict admissions; no execution."""
from pathlib import Path
import hashlib
import json
import re
import subprocess
from collections import defaultdict
ROOT=Path(__file__).resolve().parents[1]
SEED='tsr-m3-fixed12-real-corpus-v1'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def rank(s):return hashlib.sha256((SEED+'\n'+s).encode()).hexdigest()
def main():
    progress=json.loads((ROOT/'docs/m3-corpus-progress.json').read_text());pool={};excluded=[]
    for a in progress['native_admission_attempts']:
        if not a.get('accepted_for_m3'):continue
        if not(a.get('strict_protocol') and a.get('complete')):
            excluded.append(dict(attempt=a['attempt'],reason='qualification attempt not marked complete',accepted_task_ids=[r['task_id'] for r in a['reports'] if r.get('accepted')]));continue
        p=ROOT/'target'/a['attempt'];reports={r['task_id']:r for r in map(json.loads,(p/'admission.jsonl').read_text().splitlines()) if r.get('accepted')}
        for t in map(json.loads,(p/'tasks.jsonl').read_text().splitlines()):
            if t['id'] not in reports:continue
            diff=(p/'cas'/t['mutation_diff']).read_text();files=re.findall(r'^\+\+\+ b/(.+)$',diff,re.M)
            if len(files)!=1:raise ValueError('multi-file admission needs reviewed scope')
            row=dict(task=t['id'],lineage=t['lineage'],family=t['family'],function_group=reports[t['id']]['function_group'],file=files[0],failing_tests=t['failing_tests'],admission=str((p/'admission.jsonl').relative_to(ROOT)),admission_sha256=sha(p/'admission.jsonl'),tasks_sha256=sha(p/'tasks.jsonl'),rank=rank(t['id']))
            if t['id'] in pool and pool[t['id']]['function_group']!=row['function_group']:raise ValueError('conflicting admission')
            pool.setdefault(t['id'],row)
    selected=[];groups=set()
    for lineage in sorted({r['lineage'] for r in pool.values()}):
        families=defaultdict(list)
        for r in pool.values():
            if r['lineage']==lineage:families[r['family']].append(r)
        order=sorted(families,key=lambda f:(rank(lineage+'\n'+f),f));chosen=[];paths=set()
        while len(chosen)<6:
            for family in order:
                candidates=[r for r in families[family] if r['function_group'] not in groups]
                if not candidates:continue
                candidate=min(candidates,key=lambda r:(r['file'] in paths,r['rank'],r['task']))
                chosen.append(candidate);groups.add(candidate['function_group']);paths.add(candidate['file'])
                if len(chosen)==6:break
        selected.extend(chosen)
    # Alternate lineages; within-pair mode order alternates, independent of outcomes.
    selected=[selected[i+j] for i in range(6) for j in (0,6)]
    for i,r in enumerate(selected):
        p=ROOT/r['admission'];t=next(t for t in map(json.loads,p.with_name('tasks.jsonl').read_text().splitlines()) if t['id']==r['task'])
        source=subprocess.check_output(['git','-C',t['repo'],'show',t['base_snapshot']+':'+r['file']],timeout=20)
        r.update(pair_index=i,mode_order=['record','rules'] if i%2==0 else ['rules','record'],base_source_bytes=len(source),base_source_sha256=hashlib.sha256(source).hexdigest(),named_test=sorted(r['failing_tests'])[0])
    output=dict(seed=SEED,algorithm='Six per lineage; round-robin family buckets ordered by SHA256(seed+newline+lineage+newline+family). Within each family prefer an unseen source path, then smallest SHA256(seed+newline+task ID). Distinct function groups. Alternate lineages and counterbalance mode order by pair index. No model outcomes or previous performance used.',eligible_count=len(pool),inventory=sorted(pool.values(),key=lambda r:r['task']),excluded_attempts=excluded,selected=selected,source_progress_sha256=sha(ROOT/'docs/m3-corpus-progress.json'))
    with (ROOT/'docs/m3-fixed12-selection.json').open('x') as f:json.dump(output,f,indent=2);f.write('\n')
    for r in selected:print(r['pair_index'],r['task'][:12],r['family'],r['file'],r['base_source_bytes'],len(r['failing_tests']))
if __name__=='__main__':main()
