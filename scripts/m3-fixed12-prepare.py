"""Create fresh paired corpus inputs and reviewable plan; no provider calls."""
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
ROOT=Path(__file__).resolve().parents[1]
def save(p,v):
    with p.open('x') as f:json.dump(v,f,indent=2);f.write('\n')
def main():
    selected=json.loads((ROOT/'docs/m3-fixed12-v2-selection.json').read_text())['selected'];base=ROOT/'target/m3-fixed12-inputs';base.mkdir();arms=[]
    for row in selected:
        source=(ROOT/row['admission']).parent
        task=next(t for t in map(json.loads,(source/'tasks.jsonl').read_text().splitlines()) if t['id']==row['task'])
        prep=json.loads((source/'preparation.json').read_text())
        for mode in row['mode_order']:
            index=len(arms);out=base/str(index);out.mkdir()
            (out/'tasks.jsonl').write_text(json.dumps(task)+'\n')
            shutil.copyfile(source/'preparation.json',out/'preparation.json');shutil.copytree(source/'cas',out/'cas')
            save(out/'broker-contexts.json',{row['task']:dict(required_reads=[row['file']],failing_test_id='named-failing-test')})
            profile='''version = 1
name = "m3-fixed12-paired-v1"
[read_scope]
paths = [%s]
rts_tools = []
[[tests]]
id = "named-failing-test"
argv = ["cargo", "test", "--", %s, "--exact"]
[budgets]
generator_tokens = 200000
tool_minutes = 15
wall_clock_secs = 1800
generator_calls = 20
patch_bytes = 131072
[approvals]
promote_diff = true
non_allowlisted_command = true
network = true
'''%(json.dumps(row['file']),json.dumps(row['named_test']))
            (out/'profile.toml').write_text(profile)
            arms.append(row|dict(index=index,mode=mode,model='deepseek-v4.1-flash',input=str(out.relative_to(ROOT)),test=row['named_test'],broker='scripts/m3-fixed12-broker.py',sandbox_image=prep['image'],preparation_id=task['preparation_id']))
    spec=importlib.util.spec_from_file_location('b',ROOT/'scripts/m3-fixed12-budget.py');b=importlib.util.module_from_spec(spec);spec.loader.exec_module(b)
    baseline=b.history.freeze((ROOT/'target/m3-provider-cost.jsonl').read_bytes());save(ROOT/'docs/m3-fixed12-baseline.json',baseline)
    prior=json.loads((ROOT/'docs/m3-two-model-v2-freeze.json').read_text())
    plan={k:v for k,v in prior.items() if k not in ('arms','sha256','review_corrections','prior_failed_revision')}
    plan.update(stage='fixed12-real-corpus-paired',status='preparation-only-no-live-authorization-this-turn',implementation_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),model='deepseek-v4.1-flash',controller='scripts/m3-fixed12-controller.py',broker='scripts/m3-fixed12-broker.py',baseline='docs/m3-fixed12-baseline.json',launch_baseline='docs/m3-fixed12-baseline.json',state='target/m3-fixed12-state',arms=arms,max_calls_total=480,incremental_cap_usd='60',aggregate_cap_usd='100',global_wall_seconds=46800,minimum_arm_headroom_seconds=1900,selection='docs/m3-fixed12-v2-selection.json',analysis='docs/m3-fixed12-analysis-plan.md',oracle='Every independently protected failing test must truly execute and pass, full suite exit0, independent receipt reread verified_success=true, event-root/CAS/closure valid. No reversal diff is sent to generator or applied as model repair.',disclosure_scope='Only selected allowlisted source files, complete failing-test identities and bounded native observations plus unchanged v3 instructions; no hidden mutation/reversal diff, unrelated private logs or secrets.',lineage_count=2,qualified_corpus_tasks=12,paired_arms=24,model_defaults='Same omitted temperature/reasoning and unchanged provider routing defaults in both modes; catalog/model ID is not proof of upstream served-model identity.')
    save(ROOT/'docs/m3-fixed12-plan.json',plan)
    print(json.dumps(dict(arms=len(arms),tasks=len(selected),baseline=baseline)))
if __name__=='__main__':main()
