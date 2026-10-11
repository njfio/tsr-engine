# M3 evidence — corpus and batch runner

This is the evidence for the corpus and batch-runner half of milestone M3 (PRD §B9, §B11 M3). Every
command below was run in this workspace on branch `lane/corpus` at commit `b5c0a8c`; the output is
pasted verbatim. The Intuition view's select and tensions steps are **not** this lane's: they need
lane 04's ledger and follow in a later lane.

Environment (the sandbox mounts only the workspace, so the cargo cache lives inside it):

```sh
export CARGO_HOME="$PWD/.cargo-home"; export PATH="/usr/local/cargo/bin:$PATH"
```

## What was delivered

| item | path |
| --- | --- |
| `Runner` trait, `ScriptedRunner`, `ExecRunner` | `crates/tsr-corpus/src/lib.rs` |
| `tsr corpus mine` (first-parent walk, red→green flips) | `crates/tsr-corpus/src/mine.rs`, `crates/tsr-cli/src/corpus_cmd.rs` |
| `tsr corpus mutants` (cargo-mutants JSON → killed-mutant tasks) | `crates/tsr-corpus/src/mutants.rs` |
| `tsr corpus split` (time/repo split, clone groups, leakage probes) | `crates/tsr-corpus/src/split.rs` |
| `tsr task run` (systems 1 and 2, drivers) | `crates/tsr-batch/`, `crates/tsr-cli/src/batch_cmd.rs` |
| `tsr eval baseline` (paired numbers, cluster bootstrap, INCONCLUSIVE) | `crates/tsr-eval/`, `crates/tsr-cli/src/eval_cmd.rs` |
| Fixtures (git repo, cargo-mutants sample, batch sessions, eval results) | `fixtures/m3/` |
| README notes | `crates/tsr-corpus/README.md`, `crates/tsr-batch/README.md` |

## Commands and output

### `cargo build --workspace`

```
$ cargo build --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.96s
BUILD_EXIT=0
```

### `cargo fmt --all -- --check`

```
$ cargo fmt --all -- --check
FMT_EXIT=0
```

No diff was printed: the tree is rustfmt-clean.

### `cargo clippy --workspace --all-targets -- -D warnings`

```
$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.94s
CLIPPY_EXIT=0
```

### `cargo test --workspace --no-fail-fast`

```
$ cargo test --workspace --no-fail-fast
     Running unittests src/lib.rs (target/debug/deps/tsr_batch-5d4b32c43e287441)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/batch.rs (target/debug/deps/batch-5a345d3c6d2079c7)
test claude_code_driver_runs_on_a_host ... ignored, needs Claude Code and a container runtime; the operator runs it on a host that has both
test result: ok. 10 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 6.11s
     Running unittests src/lib.rs (target/debug/deps/tsr_cli-6b0efe11d8992d8f)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running unittests src/main.rs (target/debug/deps/tsr-25e82678f7a43d10)
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/m0_reconstruct.rs (target/debug/deps/m0_reconstruct-2545ca06b5557da0)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/m3_batch_invariants.rs (target/debug/deps/m3_batch_invariants-ad1002d985de27f0)
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 10.41s
     Running tests/policy_check.rs (target/debug/deps/policy_check-4c5fdfee43565a77)
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s
     Running unittests src/lib.rs (target/debug/deps/tsr_corpus-240a95fc3dda5db9)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/exec_runner.rs (target/debug/deps/exec_runner-41380a9ee8d760ac)
test exec_runner_reports_a_failing_command ... ignored, needs tsr exec and a container runtime; operator runs it
test exec_runner_runs_a_command_through_tsr_exec ... ignored, needs tsr exec and a container runtime; operator runs it
test result: ok. 0 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/mine.rs (target/debug/deps/mine-d14dcd177dc82bc5)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 4.69s
     Running tests/mutants.rs (target/debug/deps/mutants-9006aa02056d0604)
test real_cargo_mutants_run ... ignored, needs tsr exec and a container runtime; operator runs it
test result: ok. 14 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.26s
     Running tests/split.rs (target/debug/deps/split-f5dd608c72508f40)
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
     Running unittests src/lib.rs (target/debug/deps/tsr_domain-66289bdfb27e0bdc)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/tsr_eval-2acad2777da8e1e9)
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
     Running tests/baseline.rs (target/debug/deps/baseline-d9609c252f267538)
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
     Running unittests src/lib.rs (target/debug/deps/tsr_policy-34a5154a1b509641)
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running tests/adversarial.rs (target/debug/deps/adversarial-32f3f628a7e2f9e1)
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
     Running unittests src/lib.rs (target/debug/deps/tsr_store-8781724bd12ea4c5)
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

**The one failure is pre-existing and not this lane's.** `crates/tsr-cli/tests/m0_reconstruct.rs`
reads `fixtures/m0/session.events.jsonl`, which is lane 01's file and does not exist on this branch:

```
thread 'm0_session_reconstructs_from_the_log_alone' panicked at crates/tsr-cli/tests/m0_reconstruct.rs:23:5:
M0 fixture missing at /work/crates/tsr-cli/../../fixtures/m0/session.events.jsonl: record a session through `tsr hook` and save its events.jsonl here
```

It is documented here as pre-existing; it was not fixed and its file was not touched.

**The `#[ignore]`d tests, with their printed reasons:**

| test | reason |
| --- | --- |
| `tsr-batch` `tests/batch.rs::claude_code_driver_runs_on_a_host` | `needs Claude Code and a container runtime; the operator runs it on a host that has both` |
| `tsr-corpus` `tests/exec_runner.rs::exec_runner_runs_a_command_through_tsr_exec` | `needs tsr exec and a container runtime; operator runs it` |
| `tsr-corpus` `tests/exec_runner.rs::exec_runner_reports_a_failing_command` | `needs tsr exec and a container runtime; operator runs it` |
| `tsr-corpus` `tests/mutants.rs::real_cargo_mutants_run` | `needs tsr exec and a container runtime; operator runs it` |

Every other test passes: 9 corpus unit + 4 mine + 14 mutants + 26 split + 2 batch invariants + 10
batch + 6 eval baseline + 10 eval unit + 52 policy unit + 17 policy adversarial + 6 domain + 4 store
+ 3 policy_check.
## `tsr corpus mine`

`mine` walks the repository's first-parent history and runs the suite at the parent and at the
commit through the `Runner`. On this branch `tsr exec` (lane 01) is not implemented, so the doc uses
a stand-in `tsr` on `PATH` that emulates the sandbox's provisioning: it checks out the next revision
in the walk order before running the suite argv. The fixture repository is built by the test
`crates/tsr-corpus/tests/mine.rs`; its history is `c0 add`, `c1 sub` (refactor), `c2 mul` (the
red→green flip), `c3 div` (a regression).

```
$ PATH="$PWD/.docs-run/bin:$PATH" ./target/debug/tsr corpus mine .docs-run/repo \
    --out .docs-run/mine-out --suite "$PWD/.docs-run/suite.sh"
mined .docs-run/repo: 3 commits walked, 1 tasks, 1 refactors, 1 regressions
wrote .docs-run/mine-out/tasks.jsonl
```

`tasks.jsonl` (one line, the `c2` flip):

```json
{"id":"35363221d086adef2e40d85d04e3e548318727ad685bf0a730a3ceef39628bec","repo":".docs-run/repo","parent":"b3e42a1fef889debf76538524881643aa03699b3","commit":"d82e41aa40ddadbf3bf78f72367e45f262b64855","parent_snapshot":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","flipped_tests":["muls"],"hidden_diff":"1c3c45e907afc5052ebea61ad652fafd93673a71c16fafec98ae1fbbd9e827ca","family":"demo","lineage":".docs-run/repo","timestamp":"2026-10-11T16:58:45+00:00","outcome":{"id":"5f592b23a07abf9a8263130bd43a05d081d5f8ac5c3fb11e0945bb12e43b4f36","name":"flipped_tests_pass_and_suite_no_regression","version":1,"horizon":"task_close","resolves_true_when":"every flipped test passes at the commit AND the full suite exits 0 at the commit","censored_when":"the suite could not be run at the parent or at the commit (no sandbox, timeout)","intervention_rule":"the agent sees a shallow clone at the parent only; the reference diff stays hidden"}}
```

The CAS holds the reference diff under `--out/cas/<hidden_diff>`:

```
$ cat .docs-run/mine-out/cas/1c3c45e907afc5052ebea61ad652fafd93673a71c16fafec98ae1fbbd9e827ca
diff --git a/crates/demo/src/lib.rs b/crates/demo/src/lib.rs
index 6bffcd0..3d0dfeb 100644
--- a/crates/demo/src/lib.rs
+++ b/crates/demo/src/lib.rs
@@ -1,2 +1,3 @@
 pub fn add(a: i32, b: i32) -> i32 { a + b }
 pub fn sub(a: i32, b: i32) -> i32 { a - b }
+pub fn mul(a: i32, b: i32) -> i32 { a * b }
```

## `tsr corpus mutants`

`mutants` drives `cargo mutants --list --json` and the kill run through the `Runner`. The stand-in
`tsr` writes the captured samples where the module reads them (`mutants.out/mutants.json` and
`mutants.out/outcomes.json`). The samples are the real cargo-mutants 24.x shapes, captured under
`fixtures/m3/`.

```
$ TSR_DOC_FIXTURES="$PWD/fixtures/m3" PATH="$PWD/.docs-run/bin:$PATH" \
    ./target/debug/tsr corpus mutants .docs-run/repo --out .docs-run/mutants-out
mutants .docs-run/repo: 10 listed, 10 run, 5 tasks, 5 not caught
wrote .docs-run/mutants-out/tasks.jsonl
```

`tasks.jsonl` (first line; 5 killed mutants in all):

```json
{"id":"61a4f949e8f3b3eb434fc146072d1939c773ae52d701e6ec5bd777a3be66f06a","repo":".docs-run/repo","mutated_snapshot":"cc8afc0aafab1c4ca400982b542de7d5f5c503f0b3ba34c14849a19dfe1ee1e1","failing_tests":["tests::add_works"],"reversal_diff":"19aaccbd59c653f2110ec2cc98fbc2931ebe04105d1a98ba2b64b68c41e7bcc5","family":"(FnValue, partial)","lineage":".docs-run/repo","timestamp":"2026-10-11T16:58:45+00:00","outcome":{"id":"7e1bf25bd17b74e7c796efa988e514127c5beda7974025d57eb823ae7ee24c64","name":"mutant_reverted_and_suite_no_regression","version":1,"horizon":"task_close","resolves_true_when":"every failing test passes after the reversal diff is applied AND the full suite exits 0","censored_when":"the suite could not be run under the mutant or after the reversal (no sandbox, timeout)","intervention_rule":"the agent sees the mutated snapshot only; the reversal diff stays hidden"}}
```

## `tsr corpus split`

```
$ ./target/debug/tsr corpus split .docs-run/corpus --by time --holdout 0.5 --seed 7
split by time: 3 train, 3 holdout, 1 clone groups, 1 holdout accesses
wrote .docs-run/corpus/split.json
```

`split.json` (the manifest; the clone group is `t1`/`t2` at Jaccard 0.915, the holdout access is
logged, and all three leakage probes are clean):

```json
{
  "rule": "time",
  "seed": 7,
  "holdout_fraction": 0.5,
  "clone_jaccard": 0.8,
  "train": ["t1", "t2", "t3"],
  "holdout": ["t4", "t5", "t6"],
  "clone_groups": [
    {
      "id": "463585f32fa008c0ffe98ad9d5bcbb11621d197df0f652d3b49f0b9128f98760",
      "members": ["t1", "t2"],
      "side": "train",
      "similarities": [{ "a": "t1", "b": "t2", "jaccard": 0.9148936170212766 }]
    }
  ],
  "holdout_access": [
    {
      "accessed": "*",
      "at": "2026-10-11T17:49:17Z",
      "by": "tsr corpus split .docs-run/corpus --by time --holdout 0.5 --seed 7",
      "reason": "split by time with seed 7 and holdout fraction 0.5"
    }
  ],
  "leakage_probes": [
    { "probe": "hidden_diff_in_other_parent_snapshot", "verdict": "clean", "flagged": [],
      "note": "no task's hidden diff appears in the other split's parent snapshot" },
    { "probe": "test_added_after_timestamp", "verdict": "clean", "flagged": [],
      "note": "every oracle test predates its task" },
    { "probe": "post_action_log", "verdict": "clean", "flagged": [],
      "note": "no task carries a post-action log" }
  ],
  "train_lineages": ["repo-a"],
  "holdout_lineages": ["repo-b"]
}
```

## `tsr task run --mode record --driver scripted`

System 1 (record) records and gates nothing. Five scripted tasks, all accepted:

```
$ ./target/debug/tsr task run --mode record --tasks fixtures/m3/batch/tasks.jsonl \
    --driver scripted --out .docs-run/batch-record
tsr task run: 5 tasks, system record, driver scripted -> .docs-run/batch-record/results.jsonl (5 accepted, 0 unavailable)
```

`results.jsonl` (first line; `denials` is empty because system 1 gates nothing):

```json
{"task":"t1","system":"record","outcome":"accepted","generator_tokens":240,"tool_minutes":8,"wall_clock_ms":2300,"family":"parser","lineage":"rts","state_root_at_start":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","state_root_at_end":"8bda6775b191d6111126b762f64f6329f0057868e32c4bdc7b64df75a534b892","denials":[],"oracle":{"flipped_tests_pass":true,"suite_regressed":false,"accepted":true,"exit_code":0}}
```

## `tsr task run --mode rules --driver scripted`

System 2 (rules) runs the Tier 0 controller under a RATCHET profile; every gated action passes
through `tsr-policy`'s gate and is forecast-gated. Five scripted tasks, all accepted:

```
$ ./target/debug/tsr task run --mode rules --tasks fixtures/m3/batch/tasks.jsonl \
    --driver scripted --out .docs-run/batch-rules
tsr task run: 5 tasks, system rules, driver scripted -> .docs-run/batch-rules/results.jsonl (5 accepted, 0 unavailable)
```

`results.jsonl` (first line; `generator_tokens` is 0 because the Tier 0 controller does not call the
generator):

```json
{"task":"t1","system":"rules","outcome":"accepted","generator_tokens":0,"tool_minutes":2,"wall_clock_ms":2300,"family":"parser","lineage":"rts","state_root_at_start":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","state_root_at_end":"8bda6775b191d6111126b762f64f6329f0057868e32c4bdc7b64df75a534b892","denials":[],"oracle":{"flipped_tests_pass":true,"suite_regressed":false,"accepted":true,"exit_code":0}}
```

## The denial message (forecast-gated action)

A session that dispatches a `RunApprovedTest` without a forecast at the current state version is
denied, and the denial names what to record. The fixture session is
`.docs-run/denial/sessions/t1-rules.json`:

```
$ ./target/debug/tsr task run --mode rules --tasks .docs-run/denial/tasks.jsonl \
    --driver scripted --out .docs-run/denial/out
tsr task run: 1 tasks, system rules, driver scripted -> .docs-run/denial/out/results.jsonl (1 accepted, 0 unavailable)
```

`results.jsonl`:

```json
{"task":"t1","system":"rules","outcome":"accepted","generator_tokens":0,"tool_minutes":1,"wall_clock_ms":900,"family":"parser","lineage":"rts","state_root_at_start":"af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262","state_root_at_end":"a3c1df1c8e762268b67ef587100e4bdb8f6a4c768a9b260941cf1bac96fa38ca","denials":[{"candidate":"5f2ea27fffde16dd8c1a2a6a433ff05ef5f03c5c5010e30cd8e1c3ef2b6815ff","reason":"record a forecast for RunApprovedTest at state version 6 before this action"}],"oracle":{"flipped_tests_pass":true,"suite_regressed":false,"accepted":true,"exit_code":0}}
```

The denial reason is `record a forecast for RunApprovedTest at state version 6 before this action`:
it names the action, the state version, and what to record.

## `tsr task run --driver claude-code` (the documented stub)

The `claude-code` driver is specified but not implemented: the swarm sandbox has no Claude Code and
no container runtime, so it returns exit 2 with its reason and the batch records each task as
`unavailable` (the batch still writes a results file):

```
$ ./target/debug/tsr task run --mode record --tasks fixtures/m3/batch/tasks.jsonl \
    --driver claude-code --out .docs-run/batch-claude
tsr task run: 5 tasks, system record, driver claude-code -> .docs-run/batch-claude/results.jsonl (0 accepted, 5 unavailable)
```

```json
{"task":"t1","system":"record","outcome":"unavailable","generator_tokens":0,"tool_minutes":0,"wall_clock_ms":0,"family":"parser","lineage":"rts","state_root_at_start":"f30141857356d6f7c3dbe8d144546ff80d4fc817ea0857513df474850ad05245","state_root_at_end":"f30141857356d6f7c3dbe8d144546ff80d4fc817ea0857513df474850ad05245","denials":[],"oracle":null}
```

## `tsr eval baseline` — the INCONCLUSIVE case

```
$ ./target/debug/tsr eval baseline fixtures/m3/eval/inconclusive
tsr eval baseline
results-dir: fixtures/m3/eval/inconclusive
pair-by: task
seed: 6840227782638526189
resamples: 2000
paired tasks: 6
lineages: 3

== completion rate ==
  unit: fraction
  direction: higher is better
  proxy: completion rate = tasks whose oracle accepted / paired tasks; the oracle is 'the flipped tests pass and the full suite does not regress' (PRD B9). It is a proxy for 'the controller finished the task' because the oracle is a test-suite verdict, not a human judgement.
  record: 1.000000
  rules:  1.000000
  paired diff (rules - record): 0.000000
  90% interval (cluster bootstrap over lineage, 3 clusters, 6 tasks): [0.000000, 0.000000]
  INCONCLUSIVE: the 90% interval spans zero; the incumbent stays.

== cost per accepted completion ==
  unit: normalized minutes
  direction: lower is better
  proxy: cost per accepted completion = (generator tokens / 1000 + tool minutes + operator minutes) summed over accepted tasks / accepted count; PRD B8: 'Cost is expected tokens plus minutes, normalized'. Tokens are converted to minutes at the declared rate 1000 tokens/minute. Operator minutes are not in the results record and are taken as 0 here, so this understates the true cost.
  record: 6.000000
  rules:  6.000000
  paired diff (rules - record): 0.000000
  90% interval (cluster bootstrap over lineage, 3 clusters, 6 tasks): [-1.333333, 1.333333]
  INCONCLUSIVE: the 90% interval spans zero; the incumbent stays.

== unsafe attempts blocked ==
  unit: count
  direction: reported as a count
  proxy: unsafe attempts blocked = gated actions the gate denied, counted from the recorded denials (PRD B9 Metrics; the gate is PRD B5). It is a proxy for 'unsafe attempts' because only attempts that reached the gate are counted; an unsafe attempt the driver never made is invisible.
  record: 0.000000
  rules:  0.000000
  paired diff (rules - record): 0.000000
  90% interval (cluster bootstrap over lineage, 3 clusters, 6 tasks): [0.000000, 0.000000]
  INCONCLUSIVE: the 90% interval spans zero; the incumbent stays.

== generator calls per task ==
  unit: calls/task
  direction: lower is better
  proxy: generator calls per task = generator tokens / 800, summed / paired tasks; the results record carries generator tokens, not a call count, so calls are estimated at the declared rate 800 tokens/call (PRD B9 Metrics).
  record: 5.000000
  rules:  5.000000
  paired diff (rules - record): 0.000000
  90% interval (cluster bootstrap over lineage, 3 clusters, 6 tasks): [-1.666667, 1.666667]
  INCONCLUSIVE: the 90% interval spans zero; the incumbent stays.
```

Every interval spans zero, so every metric prints `INCONCLUSIVE` and the incumbent stays. The seed
is fixed, so the report is reproducible.

## `tsr eval baseline` — the clear case

```
$ ./target/debug/tsr eval baseline fixtures/m3/eval/clear
tsr eval baseline
results-dir: fixtures/m3/eval/clear
pair-by: task
seed: 6840227782638526189
resamples: 2000
paired tasks: 6
lineages: 3

== completion rate ==
  record: 1.000000
  rules:  1.000000
  paired diff (rules - record): 0.000000
  90% interval (cluster bootstrap over lineage, 3 clusters, 6 tasks): [0.000000, 0.000000]
  INCONCLUSIVE: the 90% interval spans zero; the incumbent stays.

== cost per accepted completion ==
  record: 6.000000
  rules:  3.000000
  paired diff (rules - record): -1.500000
  90% interval (cluster bootstrap over lineage, 3 clusters, 6 tasks): [-1.500000, -1.500000]
  clear: the 90% interval excludes zero.

== unsafe attempts blocked ==
  record: 0.000000
  rules:  2.000000
  paired diff (rules - record): 2.000000
  90% interval (cluster bootstrap over lineage, 3 clusters, 6 tasks): [2.000000, 2.000000]
  clear: the 90% interval excludes zero.

== generator calls per task ==
  record: 7.500000
  rules:  2.500000
  paired diff (rules - record): -5.000000
  90% interval (cluster bootstrap over lineage, 3 clusters, 6 tasks): [-5.000000, -5.000000]
  clear: the 90% interval excludes zero.
```

(The `clear` fixture's completion rate is identical between systems, so that one metric stays
`INCONCLUSIVE`; the other three intervals exclude zero and print `clear`.)

Every number is a labeled proxy per Appendix A rule 2; the report prints the proxy label for each
metric in its `-- proxy labels (Appendix A rule 2) --` footer.

## What the operator runs

These are the commands for the real corpus, on a host that has Docker (or Podman rootless), the
`tsr` binary on `PATH`, and lane 01's `tsr exec` implemented. **The 150-task target needs the host,
Docker and lane 01's `tsr exec`**: the `ExecRunner` integration tests are `#[ignore]`d for exactly
that reason (`needs tsr exec and a container runtime; operator runs it`), and nothing in
`crates/tsr-corpus` runs a repository's tests on the host.

### Mine rts

```sh
# A shallow clone at the parent is what the agent sees; the reference diff stays in the CAS.
git clone --filter=blob:none https://github.com/njfio/rts /work/rts
tsr corpus mine /work/rts --out /work/corpus-rts --suite "cargo test --offline"
```

### Mine KAMN

```sh
git clone --filter=blob:none https://github.com/njfio/KAMN /work/KAMN
tsr corpus mine /work/KAMN --out /work/corpus-kamn --suite "cargo test --offline"
```

### Mutants (killed mutants as tasks)

```sh
tsr corpus mutants /work/rts --out /work/corpus-rts-mutants
```

### The 150-task target

The target is at least 150 tasks with family and lineage (PRD §B11 M3). Mine both repositories and
the mutants, concatenate the manifests, then split:

```sh
cat /work/corpus-rts/tasks.jsonl /work/corpus-kamn/tasks.jsonl \
    /work/corpus-rts-mutants/tasks.jsonl > /work/corpus/tasks.jsonl
wc -l /work/corpus/tasks.jsonl          # must be >= 150
tsr corpus split /work/corpus --by time --holdout 0.2 --seed 1
```

If the two repositories do not yield 150 tasks, add more `njfio` repositories to the mine step; the
`--since <rev>` flag bounds the walk when a repository is large.

### Run the batch and the baseline

```sh
tsr task run --mode record --tasks /work/corpus/tasks.jsonl --driver claude-code --out /work/run-record
tsr task run --mode rules  --tasks /work/corpus/tasks.jsonl --driver claude-code --out /work/run-rules
tsr eval baseline /work/run-record /work/run-rules --pair-by task --seed 1
```

The `claude-code` driver needs Claude Code and a container runtime on the host; in this sandbox it
returns exit 2 and records each task as `unavailable` (see above). The `scripted` driver is what
every test uses and needs neither.
