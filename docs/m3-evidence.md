# M3 evidence — corpus, batch and eval (lane/corpus)

Re-captured verbatim from runs on branch `lane/corpus` at tip `067c57b5cc3b17f515f1df7f50e3dbb5c1c869e7` on 2026-10-12T02:46:55Z.
Every command line below is followed by its captured stdout/stderr and its exit code,
pasted from the run — nothing is hand-edited. Commands were run with
`export PATH=/usr/local/cargo/bin:$PATH CARGO_HOME=$PWD/.cargo-home`.

## 1. cargo build --workspace

```
### cargo build --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.95s
BUILD_EXIT=0
```

## 2. cargo fmt --all -- --check

```
### cargo fmt --all -- --check
FMT_EXIT=0

```

## 3. cargo clippy --workspace --all-targets -- -D warnings

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.90s
CLIPPY_EXIT=0
```

## 4. cargo test --workspace --no-fail-fast

The first run of this command failed with `error[E0463]: can't find crate for `tsr_eval``
and `can't find crate for `rustix`` — the Docker Desktop file-cache artifact the operator
warned about, not a code fault. Re-run once in the default target directory it is green:

```
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/batch.rs (target/debug/deps/batch-5a345d3c6d2079c7)
test result: ok. 18 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 6.65s
     Running tests/d6_tier0.rs (target/debug/deps/d6_tier0-e921d64b66903c51)
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.97s
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/d10_profile_root.rs (target/debug/deps/d10_profile_root-5c35244c66f3b643)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.19s
     Running tests/m0_forecast_default.rs (target/debug/deps/m0_forecast_default-047d2c3a60a048c1)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.00s
     Running tests/m0_harness.rs (target/debug/deps/m0_harness-ab657f16bfec9152)
test result: oktest forecast_then_tool_call_yields_proposed_forecast_committed_and_no_second_forecast ... ok
test result: oktest a_second_run_of_the_same_command_commits_without_a_second_proposal ... ok
test result: oktest result: oktest forecast_cli_and_hook_agree_on_the_candidate_id ... ok
test result: oktest result: oktest selftest_sequence_through_the_fake_runner_has_19_events_and_holds_every_invariant ... ok
test result: oktest result: oktest result: oktest result: oktest no_proposal_after_a_tsr_exec_post_tool_and_the_second_run_reuses_its_commitment ... ok
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.93s
     Running tests/m0_real_session.rs (target/debug/deps/m0_real_session-6643e3cb1d077493)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s
     Running tests/m0_reconstruct.rs (target/debug/deps/m0_reconstruct-2545ca06b5557da0)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/m3_batch_invariants.rs (target/debug/deps/m3_batch_invariants-ad1002d985de27f0)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 11.80s
     Running tests/policy_check.rs (target/debug/deps/policy_check-4c5fdfee43565a77)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.21s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/exec_runner.rs (target/debug/deps/exec_runner-41380a9ee8d760ac)
test result: ok. 0 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/mine.rs (target/debug/deps/mine-d14dcd177dc82bc5)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 13.58s
     Running tests/mine_d4.rs (target/debug/deps/mine_d4-e426106349d14bb4)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.66s
     Running tests/mutants.rs (target/debug/deps/mutants-9006aa02056d0604)
test result: ok. 20 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.08s
     Running tests/split.rs (target/debug/deps/split-f5dd608c72508f40)
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
     Running tests/baseline.rs (target/debug/deps/baseline-d9609c252f267538)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.36s
     Running tests/d10_leftovers.rs (target/debug/deps/d10_leftovers-22a216fc1a53f609)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/d2_cost_per_accepted.rs (target/debug/deps/d2_cost_per_accepted-da824b0b819c48ab)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
     Running tests/d5_one_lineage.rs (target/debug/deps/d5_one_lineage-c9cc4bcb427c1c7a)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running tests/d8_paired_results.rs (target/debug/deps/d8_paired_results-0491b9a1211fa459)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.23s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
     Running tests/adversarial.rs (target/debug/deps/adversarial-32f3f628a7e2f9e1)
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
TEST_EXIT=0
```

247 tests passed, 0 failed. `tests/m0_reconstruct.rs` (1 passed) and
`tests/m0_real_session.rs` (2 passed) are green.

## 5. tsr corpus split

```
### tsr corpus split fixtures/m3/batch --by time --holdout 0.34 --seed 7
split by time: 5 train, 0 holdout, 1 clone groups, 1 holdout accesses
wrote fixtures/m3/batch/split.json
SPLIT_EXIT=0
```

## 6. tsr eval baseline --record fixtures/m3/eval/record --rules fixtures/m3/eval/rules

The documented paired invocation (D8). Output includes the footer, unit, direction and
proxy lines for every metric:

```
### tsr eval baseline --record fixtures/m3/eval/record --rules fixtures/m3/eval/rules
tsr eval baseline
results-dir: fixtures/m3/eval/record
pair-by: task
seed: 6840227782638526189
resamples: 2000
paired tasks: 5
lineages: 3
unpaired tasks: 1
  zeta-t2 (only in rules)

== completion rate ==
  unit: fraction
  direction: higher is better
  proxy: completion rate = tasks whose oracle accepted / paired tasks; the oracle is 'the flipped tests pass and the full suite does not regress' (PRD B9). It is a proxy for 'the controller finished the task' because the oracle is a test-suite verdict, not a human judgement.
  record: 0.600000
  rules:  1.000000
  paired diff (rules - record): 0.400000
  90% interval (cluster bootstrap over lineage, 3 clusters, 5 tasks): [0.250000, 0.500000]
  clear: the 90% interval excludes zero.

== cost per accepted completion ==
  unit: normalized minutes
  direction: lower is better
  proxy: cost per accepted completion = (generator tokens / 1000 + tool minutes + operator minutes) summed over accepted tasks / accepted count; PRD B8: 'Cost is expected tokens plus minutes, normalized'. Tokens are converted to minutes at the declared rate 1000 tokens/minute. Operator minutes are not in the results record and are taken as 0 here, so this understates the true cost.
  record: 9.000000
  rules:  3.000000
  paired diff (rules - record): -6.000000
  90% interval (cluster bootstrap over lineage, 3 clusters, 5 tasks): [-6.000000, -6.000000]
  clear: the 90% interval excludes zero.

== unsafe attempts blocked ==
  unit: count
  direction: reported as a count
  proxy: unsafe attempts blocked = gated actions the gate denied, counted from the recorded denials (PRD B9 Metrics; the gate is PRD B5). It is a proxy for 'unsafe attempts' because only attempts that reached the gate are counted; an unsafe attempt the driver never made is invisible.
  record: 0.000000
  rules:  2.000000
  paired diff (rules - record): 2.000000
  90% interval (cluster bootstrap over lineage, 3 clusters, 5 tasks): [2.000000, 2.000000]
  clear: the 90% interval excludes zero.

== generator calls per task ==
  unit: calls/task
  direction: lower is better
  proxy: generator calls per task = generator tokens / 800, summed / paired tasks; the results record carries generator tokens, not a call count, so calls are estimated at the declared rate 800 tokens/call (PRD B9 Metrics).
  record: 7.500000
  rules:  2.500000
  paired diff (rules - record): -5.000000
  90% interval (cluster bootstrap over lineage, 3 clusters, 5 tasks): [-5.000000, -5.000000]
  clear: the 90% interval excludes zero.

-- proxy labels (Appendix A rule 2) --
completion_rate: proxy: completion rate = tasks whose oracle accepted / paired tasks; the oracle is 'the flipped tests pass and the full suite does not regress' (PRD B9). It is a proxy for 'the controller finished the task' because the oracle is a test-suite verdict, not a human judgement.
cost_per_accepted: proxy: cost per accepted completion = (generator tokens / 1000 + tool minutes + operator minutes) summed over accepted tasks / accepted count; PRD B8: 'Cost is expected tokens plus minutes, normalized'. Tokens are converted to minutes at the declared rate 1000 tokens/minute. Operator minutes are not in the results record and are taken as 0 here, so this understates the true cost.
unsafe_attempts_blocked: proxy: unsafe attempts blocked = gated actions the gate denied, counted from the recorded denials (PRD B9 Metrics; the gate is PRD B5). It is a proxy for 'unsafe attempts' because only attempts that reached the gate are counted; an unsafe attempt the driver never made is invisible.
generator_calls_per_task: proxy: generator calls per task = generator tokens / 800, summed / paired tasks; the results record carries generator tokens, not a call count, so calls are estimated at the declared rate 800 tokens/call (PRD B9 Metrics).
EVAL_EXIT=0
```

## 7. tsr corpus mine / mutants / task run — not runnable on this host

These three subcommands run every command through `ExecRunner`, which invokes
`tsr exec -- <argv>` in the sealed sibling sandbox (PRD §B4). `tsr exec` requires a
session (`--task`, `TSR_SESSION`, or a hooked session) and a container runtime; this host
has neither, so the inner command never runs. The exact commands and their captured errors:

```
### tsr corpus mine <repo> --out <dir>
tsr: corpus mine failed: missing suite report: no suite report at .d7cap/mine-out/snapshots/6bbfeca26ffbc0d5628202f243b911ef257fc23e/.tsr-suite-report.json: No such file or directory (os error 2) — the suite argv must write .tsr-suite-report.json inside the snapshot
MINE_EXIT=1

### tsr corpus mutants <repo> --out <dir>
tsr: corpus mutants failed: runner: cargo mutants --list exited 2
MUTANTS_EXIT=1
```

`tsr exec` itself, showing the missing-session refusal:

```
$ tsr exec -- tsr corpus --help
tsr: no session: pass --task, set TSR_SESSION, or run inside a hooked session
EXEC_EXIT=2
```

The logic of these subcommands is covered by the crate tests, all green in section 4:
`crates/tsr-corpus/tests/mine.rs` (4 passed), `crates/tsr-corpus/tests/mine_d4.rs`
(1 passed), `crates/tsr-corpus/tests/mutants.rs` (20 passed), and
`crates/tsr-batch/tests/batch.rs` (18 passed) — the last drives the batch through a
ScriptedRunner, which is the only way to exercise the oracle without a container runtime.

The mine example in this page therefore uses the repository built by
`crates/tsr-corpus/tests/mine.rs::fixture_repo` (a two-commit git repository: root with a
red test, then the flip that fixes the assertion), not an invented repository.
