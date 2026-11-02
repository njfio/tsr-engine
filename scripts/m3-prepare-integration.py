"""Generate a separate trivial synthetic task and freeze its production integration path."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT=Path(__file__).resolve().parents[1]
BASE=ROOT/'target/m3-integration-input'
ENGINE=ROOT/'target/m3-exact-build/debug/tsr'
IMAGE='sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6'
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def digest(b):return subprocess.check_output([str(ROOT/'target/m3-blake3-checksum')],input=b).decode().strip()
def save(p,v):
    with p.open('x') as f:json.dump(v,f,indent=2);f.write('\n')

def main():
    BASE.mkdir();repo=BASE/'repo';(repo/'src').mkdir(parents=True);(repo/'tests').mkdir()
    (repo/'Cargo.toml').write_text('[package]\nname = "m3-known-answer"\nversion = "0.1.0"\nedition = "2021"\n')
    (repo/'src/lib.rs').write_text('/// Return the integer 42. This is an isolated synthetic integration task.\npub fn answer() -> u32 { 0 }\n')
    (repo/'tests/answer.rs').write_text('#[test]\nfn answer_is_42() { assert_eq!(m3_known_answer::answer(), 42); }\n#[test]\nfn answer_is_stable() { assert_eq!(m3_known_answer::answer(), m3_known_answer::answer()); }\n')
    env=os.environ|dict(GIT_AUTHOR_DATE='2026-10-17T00:00:00Z',GIT_COMMITTER_DATE='2026-10-17T00:00:00Z')
    def git(*args):return subprocess.check_output(['git','-C',str(repo),'-c','core.hooksPath=/dev/null','-c','user.name=Synthetic Integration','-c','user.email=synthetic@example.invalid',*args],env=env).decode().strip()
    git('init','-q');git('add','.');git('commit','-qm','synthetic red fixture');red=git('rev-parse','HEAD');tree=git('rev-parse','HEAD^{tree}')
    (repo/'src/lib.rs').write_text((repo/'src/lib.rs').read_text().replace('{ 0 }','{ 42 }'))
    git('add','.');git('commit','-qm','synthetic known answer');green=git('rev-parse','HEAD')
    reference=subprocess.check_output(['git','-C',str(repo),'diff',red,green,'--','src/lib.rs'])
    lock=BASE/'synthetic.lock';lock.write_text('version = 4\n\n[[package]]\nname = "m3-known-answer"\nversion = "0.1.0"\n')
    req=dict(repo=str(repo),snapshot=tree,lock=str(lock),repository_identity='synthetic:m3-known-answer',image=IMAGE,generated_by='inert dependency-free fixture generator; no package resolver',rust_test_environment=True,build_workspace=False,test_argv=['cargo','test','--locked','--offline','--workspace','--no-fail-fast','-j','1'])
    save(BASE/'preparation-request.json',req)
    result=subprocess.check_output([str(ENGINE),'corpus','prepare','--request',str(BASE/'preparation-request.json'),'--out',str(BASE)],env=os.environ|{'TSR_DB':str(BASE/'prepare-startup.sqlite')}).decode()
    prep=json.loads((BASE/'preparation.json').read_text());prep_id=digest((BASE/'preparation.json').read_bytes())
    task_id=digest(json.dumps(['synthetic:m3-known-answer',red,green,prep_id],separators=(',',':')).encode())
    task=dict(id=task_id,preparation_id=prep_id,repo=str(repo),parent=red,commit=green,parent_snapshot=tree,flipped_tests=['answer_is_42'],hidden_diff=digest(reference),family='synthetic-known-answer',lineage='synthetic:m3-known-answer',timestamp='2026-10-17T00:00:00Z',oracle_test_files=['tests/answer.rs'],outcome=dict(id=digest(b'synthetic-known-answer-outcome-v1'),name='synthetic_answer_and_suite',version=1,horizon='task_close',resolves_true_when='the protected answer_is_42 test and full suite pass',censored_when='sandbox or evidence unavailable',intervention_rule='candidate sees red source and bounded visible feedback; reference diff remains hidden'))
    (BASE/'cas'/digest(reference)).write_bytes(reference)
    (BASE/'tasks.jsonl').write_text(json.dumps(task)+'\n')
    profile='''version = 1
name = "m3-integration"
[read_scope]
paths = ["src/lib.rs"]
rts_tools = []
[[tests]]
id = "named-failing-test"
argv = ["cargo", "test", "--", "answer_is_42", "--exact"]
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
    (BASE/'profile.toml').write_text(profile)
    save(BASE/'broker-contexts.json',{task_id:dict(required_reads=['src/lib.rs'],failing_test_id='named-failing-test')})
    spec=importlib.util.spec_from_file_location('budget',ROOT/'scripts/m3-exact-budget.py');budget=importlib.util.module_from_spec(spec);spec.loader.exec_module(budget)
    baseline=budget.history.freeze((ROOT/'target/m3-provider-cost.jsonl').read_bytes())
    if baseline['settled_usd']!='0.229744':raise ValueError('stage starting cost changed')
    save(ROOT/'docs/m3-integration-baseline.json',baseline)
    previous=json.loads((ROOT/'docs/m3-exact-live-freeze.json').read_text())
    files=set(previous['sha256'])
    files.update(str(p.relative_to(ROOT)) for p in BASE.rglob('*') if p.is_file())
    files.update(['scripts/m3-edit-v2-broker.py','scripts/m3-integration-broker.py','scripts/m3-integration-controller.py','scripts/m3-prepare-integration.py','docs/m3-integration-baseline.json'])
    m=dict(version=1,stage='synthetic-only-integration',implementation_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT).decode().strip(),arms=[dict(task=task_id,mode=mode,input=str(BASE.relative_to(ROOT)),file='src/lib.rs',test='answer_is_42',lineage='synthetic:m3-known-answer') for mode in ['record','rules']],engine=str(ENGINE.relative_to(ROOT)),broker='scripts/m3-integration-broker.py',python=str(Path(sys.executable).resolve()),baseline='docs/m3-integration-baseline.json',ledger='target/m3-provider-cost.jsonl',state='target/m3-integration-state',sandbox_image=IMAGE,initial_minimum_free_gib=240,live_guard_gib=200,max_calls_per_arm=20,max_calls_total=40,max_wall_seconds_per_arm=1800,incremental_cap_usd='5',aggregate_cap_usd='100',automatic_retries=0,disclosure_scope='Only synthetic task ID, source src/lib.rs, named test identity, bounded test feedback and controller progress; no qualified corpus content or reference diff.',sha256={p:sha(ROOT/p) for p in sorted(files)})
    save(ROOT/'docs/m3-integration-freeze.json',m)
    print(json.dumps(dict(task=task_id,freeze_sha256=sha(ROOT/'docs/m3-integration-freeze.json'),files=len(files),preparation=result.strip())))

if __name__=='__main__':main()
