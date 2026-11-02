"""Prepare fresh matched comparison inputs offline; never starts an arm."""
from pathlib import Path
import hashlib
import json
import shutil

ROOT = Path(__file__).resolve().parents[1]

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    assessment = json.loads((ROOT / 'docs/m3-next-comparison-assessment.json').read_text())
    output = ROOT / 'target/m3-smoke-comparison'
    output.mkdir(exist_ok=False)
    jobs = []
    for selected in assessment['jobs']:
        source = ROOT / selected['admission']
        if digest(source) != selected['admission_sha256']:
            raise ValueError('admission changed')
        task_id = selected['task_id']
        tasks = [json.loads(line) for line in source.with_name('tasks.jsonl').read_text().splitlines()]
        if len(tasks) != 1 or tasks[0]['id'] != task_id:
            raise ValueError('task mismatch')
        base = output / task_id
        base.mkdir()
        for name in ['tasks.jsonl','preparation.json']:
            shutil.copyfile(source.with_name(name), base / name)
        shutil.copytree(source.with_name('cas'), base / 'cas')
        (base / 'broker-contexts.json').write_text(json.dumps({task_id:{'required_reads':[selected['file']], 'failing_test_id':'named-failing-test'}}, indent=2)+'\n')
        profile = f'''version = 1
name = "m3-smoke-comparison"
[read_scope]
paths = [{json.dumps(selected['file'])}]
rts_tools = []
[[tests]]
id = "named-failing-test"
argv = ["cargo", "test", "--", {json.dumps(selected['test'])}, "--exact"]
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
'''
        (base / 'profile.toml').write_text(profile)
        jobs.append(dict(selected, input=str(base.relative_to(ROOT)), modes=['record','rules'], source='rts', initial_store='empty-new-per-arm', mutated_snapshot=tasks[0]['mutated_snapshot']))
    plan = dict(assessment, status='offline-prepared-not-launched', jobs=jobs,
        broker='scripts/m3-smoke-broker.py', shared_ledger='target/m3-provider-cost.jsonl',
        baseline='docs/m3-smoke-budget-baseline.json', state='target/m3-smoke-comparison-state',
        controller='scripts/m3-smoke-comparison.py', minimum_free_gib=200,
        initial_store_match='Both modes consume identical tasks/CAS/profile/context and independent empty stores; real start-root equality must be verified from results.',
        readiness_gaps=['Independent review and immutable hash freeze required before launch; live price/runtime checks remain launch-time gates.'])
    (ROOT / 'docs/m3-smoke-comparison-plan.json').write_text(json.dumps(plan, indent=2)+'\n')

if __name__ == '__main__': main()
