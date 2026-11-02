"""Offline only: pin exact-edit pair inputs and executable; never launch or contact providers."""
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[1]
TASK = '0c5a91a6aaa862c279aed1e1602a74eda7c42c4fc0f72247de513fa948f50b70'


def sha(path): return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    spec = importlib.util.spec_from_file_location('budget', ROOT / 'scripts/m3-exact-budget.py')
    budget = importlib.util.module_from_spec(spec); spec.loader.exec_module(budget)
    old = json.loads((ROOT / 'docs/m3-progress-validation-plan.json').read_text())
    job = sorted(old['jobs'], key=lambda j: j['task_id'])[0]
    if job['task_id'] != TASK or sha(ROOT / job['admission']) != job['admission_sha256']:
        raise ValueError('qualified deterministic selection changed')
    base = ROOT / 'target/m3-exact-live' / TASK
    profile = tomllib.loads((base / 'profile.toml').read_text())
    if profile['budgets'] != dict(generator_tokens=200000, tool_minutes=15, wall_clock_secs=1800, generator_calls=20, patch_bytes=131072):
        raise ValueError('comparable budget changed')
    if profile['read_scope']['paths'] != [job['file']]: raise ValueError('read scope changed')
    for name in ['tasks.jsonl', 'profile.toml', 'preparation.json', 'broker-contexts.json']:
        if (base/name).read_bytes() != (ROOT/'target/m3-continuation'/TASK/name).read_bytes():
            raise ValueError('qualified input changed')
    for mode in ['record','rules']:
        if (base/mode).exists() or list(base.glob('startup-'+mode+'.sqlite*')): raise ValueError('not fresh')
    if (ROOT/'target/m3-exact-live-state').exists(): raise ValueError('one-shot state exists')
    task = json.loads((base/'tasks.jsonl').read_text())
    subprocess.run(['git','-C',task['repo'],'cat-file','-e',task['base_snapshot']+'^{tree}'],check=True)
    baseline = json.loads((ROOT/'docs/m3-exact-live-baseline.json').read_text())
    if budget.history.freeze((ROOT/'target/m3-provider-cost.jsonl').read_bytes()) != baseline or baseline['settled_usd'] != '0.229582':
        raise ValueError('current ledger changed')
    old_manifest = json.loads((ROOT/'docs/m3-continuation-readiness.json').read_text())['sha256']
    changed = [p for p,h in old_manifest.items() if sha(ROOT/p) != h]
    if changed != ['crates/tsr-batch/src/broker_v2.rs']: raise ValueError('unexpected historical input change')
    exports = json.loads((ROOT/'docs/m3-continuation-export-sha256.json').read_text())
    folder = ROOT/'docs/m3-continuation-execution'
    if {str(p.relative_to(folder)) for p in folder.rglob('*') if p.is_file()} != set(exports): raise ValueError('prior export set changed')
    for p,h in exports.items():
        if sha(folder/p) != h: raise ValueError('prior evidence changed')
    tracked = subprocess.check_output(['git','ls-files'],cwd=ROOT,text=True).splitlines()
    files = set(old_manifest)
    files.update(p for p in tracked if p.startswith('crates/') and p.endswith(('.rs','.toml')))
    files.update(p for p in tracked if p.startswith('scripts/') and p.endswith(('.py','.rs')))
    files.update(['Cargo.toml','Cargo.lock',job['admission'], 'docs/m3-progress-validation-plan.json',
        'docs/m3-exact-live-baseline.json','scripts/m3-freeze-exact-live.py',
        'scripts/m3-exact-live-controller.py','scripts/m3-exact-live-broker.py','scripts/m3-exact-budget.py',
        'scripts/test_m3_exact_live.py','scripts/test_m3_edit_broker.py',
        'crates/tsr-batch/tests/m3_exact_edits.rs','target/m3-exact-build/debug/tsr'])
    files.update(str(p.relative_to(ROOT)) for p in base.rglob('*') if p.is_file())
    python = str(Path(sys.executable).resolve())
    files.add(python)
    preparation = json.loads((base/'preparation.json').read_text())
    engine = 'target/m3-exact-build/debug/tsr'
    manifest = dict(version=1,status='offline-frozen-pending-review-and-paid-approval',
        implementation_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
        selection='Lexicographically smallest task_id among the two previously qualified tasks in docs/m3-progress-validation-plan.json; fixed before new outcomes.',
        task=TASK,function=job['function'],source_path=job['file'],failing_test=job['test'],
        admission=job['admission'],admission_sha256=job['admission_sha256'],
        arms=[dict(task=TASK,mode=mode,input=str(base.relative_to(ROOT)),file=job['file'],function=job['function'],test=job['test'],lineage=job['lineage']) for mode in ['record','rules']],
        controller='scripts/m3-exact-live-controller.py',broker='scripts/m3-exact-live-broker.py',
        schema_adapter='scripts/m3-edit-broker.py',engine=engine,engine_sha256=sha(ROOT/engine),python=python,
        baseline='docs/m3-exact-live-baseline.json',ledger='target/m3-provider-cost.jsonl',state='target/m3-exact-live-state',
        baseline_settled_usd='0.229582',baseline_outstanding_usd='0',incremental_cap_usd='5',aggregate_cap_usd='100',
        reservation_usd='2',max_calls_per_arm=20,max_calls_total=40,max_wall_seconds_per_arm=1800,
        max_generator_tokens_per_arm=200000,max_tool_minutes_per_arm=15,max_patch_bytes_per_arm=131072,
        global_wall_seconds=4200,export_margin_seconds=300,max_concurrent_arms=1,automatic_retries=0,
        global_deadline_semantics='Monotonic dispatch/acceptance deadline including setup and export; receipt helper subprocesses capped at 60s and hash helpers at 15s. Synchronous filesystem export is not a hard-interruptible wall limit.',
        initial_minimum_free_gib=240,live_guard_gib=200,sandbox_image=preparation['image'],
        sandbox_user='65534:65534',sandbox_network=False,
        endpoint='https://api.cheaperinference.com/v1/chat/completions',model='deepseek-v4.1-flash',
        max_input_price_per_million_usd='0.30',max_output_price_per_million_usd='1.20',max_output_tokens=16384,
        max_wire_bytes=300000,price_guard='Fresh catalog check before every inference; validate finite nonnegative prices, worst-case request bound and exact settled USD receipt; never retry.',
        initial_store='Independent new empty stores and startup DBs per arm; actual start roots independently checked and required equal.',
        disclosure_scope='Only selected public task ID, named test and allowed source path; complete/bounded observations of crates/rts-daemon/src/error.rs (including its inline tests), bounded named-test stdout/stderr, action metadata, forecasts and controller progress. No hidden reversal/reference diff, credentials, unrelated files/sessions or private logs. Credential used only for API authentication, never as model content.',
        evaluation='Same frozen binary, evaluator sources, profile, input CAS, preparation, image and adapter in both modes. Only Mode differs. No pooling with historical failures.',
        stop_policy='Any protocol/provider/billing/process/precondition/integrity/evidence failure halts entire pair; valid final-oracle accepted or rejected may proceed. Explicit Stop required. No retries, deletion or resume.',
        terminal_requirements=['At-most-once durable intent, process PID/start/finish/exit and quiescence checks.',
            'Exact per-arm reservation/settlement/diagnostic reconciliation; preserve unknown reservations.',
            'Export task/profile/preparation/CAS/store/result, validate CAS and event-derived roots, final closure and explicit Stop.',
            'Independent final-oracle receipt verification for every completed outcome; never treat visible tests as acceptance.',
            'Manifest all exported evidence; retain failures and unattempted arms. Partial/missing evidence is unverified and requires salvage before another run.'],
        missing_validation=['No live provider compatibility/behavior validation; single-call setting remains unverified for Chat Completions.',
            'Fresh runtime/image/disk/idle checks and current price check required immediately before live dispatch.',
            'Separate paid approval bound to this freeze hash is required; no approval file created.'],
        sha256={p:sha(ROOT/p) for p in sorted(files)})
    dest=ROOT/'docs/m3-exact-live-freeze.json'
    dest.write_text(json.dumps(manifest,indent=2)+'\n')
    print(json.dumps(dict(freeze_sha256=sha(dest),engine_sha256=manifest['engine_sha256'],frozen_files=len(files),prior_export_files_verified=len(exports))))

if __name__=='__main__':main()
