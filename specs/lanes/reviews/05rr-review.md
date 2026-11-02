**Lane 05rr (M3, second repair): not merge-ready.** The oracle still accepts a session that deletes or edits the flipped test, and the session still never touches the copy (D3); mutant tasks still cannot be provisioned and none of their reversals apply (D4, D9); the bootstrap still prints a false "clear" (R4); the evidence is still the 067c57b capture (D7); and one of the lane's own tests is red.

At 94775da, with a stand-in `tsr exec` that runs and logs its argv in its cwd.

## Items

**(1) D4, R1 provisioning: PARTLY.**
- Mined tasks are FIXED (run.rs:305-349). A mined 4-commit repo gives `3 commits walked, 1 tasks, 1 refactors, 1 regressions` with `parent_snapshot` = `c0^{tree}`. `task run --mode record` and `--mode rules`, without `--scripted-runner`, run `tsr corpus materialize … (cwd=…/copies)`, and the copy holds the parent tree.
- R1 is FIXED. `"parent_snapshot":"/etc"` exits 2 (`unsafe snapshot "/etc"`), and the exec log is empty.
- Mutants are NOT FIXED. `tsr corpus mutants` still writes `mutated_snapshot` = blake3 of the descriptor (mutants.rs:755) and never writes `base_snapshot` or `mutation_diff`; `CorpusTask::snapshot()` falls back to that digest (task.rs:101-107), and it passes the hex check. So `task run` over real `mutants` output fails with `provisioning failed … --snapshot cc8afc0a… exited 1`, and the whole batch exits 2.
- Even when the field is present, run.rs:363 reads the mutation diff from the batch's `--out` CAS, not the corpus's, and run.rs:362 skips applying it whenever `dest` is missing.

**(2) D3, R3 session in the copy, oracle judged: D3 NOT FIXED; R3 FIXED.**
- `Driver::drive` (driver.rs:133) still receives no copy (run.rs:375); the D3 tests still script the runner (batch.rs:429-435).
- No code restores the oracle files from the snapshot.
- `tsr corpus oracle` on variants of the mined copy (`--tests tests::flipped_add` / `--full-suite`):

  | Variant | Flipped test | Full suite |
  |---|---|---|
  | bug present | fail 101 | fail 101 |
  | bug fixed | pass | pass |
  | test deleted, bug present | pass | pass |
  | assertion edited, bug present | pass | pass |
  | test renamed to `zz_moved` | pass (filter matches nothing) | fail |
- The `missing-forecast` session under `--scripted-runner` is still `accepted` (5/5).
- R3 is fixed. `--tests tests::other,tests::flipped_add` gives `pass`. A two-test task through `task run` (no scripted runner) is `1 accepted`, and the oracle argv is `cargo test -- tests::other tests::flipped_add`.

**(3) D6 Tier 0 in shipped rules mode: FIXED.**
- batch_cmd.rs:102-108 attaches the controller; driver.rs:526-546 and run.rs:403-428 end only that task. A rules batch where t1 overspends (2×50 tool-minutes vs 30) ends t1 `rejected` with `stop{reason:"budget"}` then `task_closed{reason:"stop:budget"}`; t2, t3, m1, m2 run and are accepted. See N4.

**(4) R2, D1 residual: FIXED.**
- mutants.rs:428-443 guards `log_path`. A planted absolute path, `../../host-secret.log`, and a symlink `log/link.log → host file` each make the command fail with `unsafe log_path … must resolve inside mutants.out`. `HOST_SECRET` never reaches tasks.jsonl.
- `run_one` re-checks the task id (run.rs:270).

**(5) D9: list FIXED; reversal NOT FIXED.**
- The list is fixed. The LIST_ARGV (mutants.rs:74-78), run in a fresh dir with a fake `cargo mutants`, exits 0 and writes mutants.out/mutants.json.
- The reversal is still broken.
  - The pipeline builds reversals from `outcome.mutant()` (mutants.rs:732, 747); outcomes.json descriptors carry no `diff`, so every task falls back to the span reconstruction (mutants.rs:476-495).
  - End to end through `tsr corpus mutants` on a real repo, all 5 reversals in the CAS fail `git apply --check` against every mutated file. The `replace + with -` reversal is still `-` + `-` / `+    a + b`.
  - The d9 test passes only because it uses the list descriptor, which has a `diff`.
  - The diff-based path copies the `@@` header unswapped (mutants.rs:566-569). A throwaway test with a body-replacing FnValue mutation (`@@ -1,6 +1,4 @@`) gives `git apply`: `corrupt patch at line 12`.

**(6) D8: FIXED.**
- Record then rules into one `--out` writes results-record.jsonl and results-rules.jsonl, and `tsr eval baseline <out>` reports `paired tasks: 5`.
- `--driver claude-code` gets past provisioning: `1 unavailable`, exit 0.

**(7) D10: FIXED.**
- With no `--profile` and no `profiles/` above the cwd it exits 2 naming where it looked; the binary embeds no `tsr-cli/../..` path.
- Every metric value, diff and bound prints as `name=value` (`cost_per_accepted=-6`), via tsr-eval's own copy of `tsr_domain::Proxy` (proxy.rs), not the domain type.

**(8) R4 NOT FIXED; R5 FIXED.**
- R4: bootstrap.rs:181-183 still skips undefined resamples silently, and `resamples` is the requested count (bootstrap.rs:191, 204). Record accepting only in lineage B (cost 10), rules only in A (cost 5), prints `resamples: 2000`, `[cost_per_accepted=-5, cost_per_accepted=-5]` and `clear`. There is no r4 test.
- R5: `materialize` now uses `git archive | tar` (mine.rs:436-466), and `git worktree list` is unchanged after a mine. There is no r5 test, and the change broke mine_d4 (N1).

**(9) D7 evidence: NOT FIXED.**
- docs/m3-evidence.md:3 still says it was captured at `067c57b`. It quotes 247 tests and `mine_d4 (1 passed)`.
- It has no `task run`, no mine of a real repo, no oracle refusal, and no acceptance commands; §7 says those are "not runnable".
- Its §6 eval output differs from the binary's on every number line (`0.600000` vs `completion_rate=0.6`).

## New defects

**N1 (medium): a lane test is red.** `mine_d4::d4_mine_materialises_parent_and_commit_separately` fails at crates/tsr-corpus/tests/mine_d4.rs:197 (`left: ""`, right is the root sha). Its `GitRunner` asks `git rev-parse HEAD` inside the snapshot, and an archive extract has no `.git`. It fails on any machine.

**N2 (medium): relative paths resolve against the copies root.** Provisioning runs with cwd `out/copies` (run.rs:322-337), but `dest` and `task.repo` are passed through unresolved.
- With `--out brel`, the copy lands in `brel/copies/brel/copies/<id>/record`. The oracle cannot start in the missing `dest` (exit 127), and the task is silently recorded `rejected`. The mutant diff is skipped too (run.rs:362).
- `tsr corpus mine repo4` records `"repo":"repo4"`, and `task run` then fails with `materialize --repo repo4 … exited 1`.

**N3 (medium; Docker-dependent, not run): real `tsr exec` provisioning cannot read the source repo.**
- `container_argv` mounts only the cwd (exec.rs:125-128), here `out/copies`, while `materialize --repo` names a host path outside it.
- ExecRunner sets no `TSR_SESSION`, which `tsr exec` requires (evidence §7: exit 2).
- The provisioning's writable mount is the whole copies root, holding every task's and both systems' copies, not one disposable copy (B10).

**N4 (low): the Tier 0 read rule is self-referential.** batch_cmd.rs:196-213 takes the "failing assertion" and "call site" from the session's own first two inspects. Any session that inspects any file before its first test therefore passes read-before-test.

**N5 (medium, pre-existing): `tsr corpus mutants` uses the operator's repository as the sandbox's writable mount.** mutants.rs:700 and 712 call `runner.run(repo, …)`. The kill run executes the repository's code there and leaves `mutants.out/` behind (`git status`: `?? mutants.out/`). This breaks B10.

## Definition of done

- build: **MET**. fmt: **MET** (1.97 and 1.90).
- clippy: **MET** on 1.90. 1.97 fails on tsr-policy gate.rs:411 `filter_next` (not the lane's) and driver.rs:424 `collapsible_match`; both are 1.97-only (1.90 clippy is clean).
- `cargo test --workspace --no-fail-fast` green: **NOT MET**. 275 passed, 3 failed, 4 ignored. The failures are mine_d4 (the lane's) and the two Docker-only m0 tests (m0_forecast_default exit-code, m0_harness fixture).
- hook/exec/store_path/task.rs and lane 04/06 files untouched: **MET** per lane.diff (lib.rs adds only `pub mod corpus_cmd;`; main.rs hunks touch only Corpus, Eval, Task Run).
- fixtures/m0 byte-identical: **CAN'T TELL**; fixtures are excluded from the diff.
- Every item has an id-named test that failed before its fix: **NOT MET**. There are no r4, r5 or d7 tests.
- Real copies from a mined repo in record and rules mode: **MET**.
- No unchecked path from tasks.jsonl or outcomes.json: **MET**, but see N2.
- Session in the copy, oracle restores the tests and rejects delete, rename or edit: **NOT MET**.
- A two-test task can be accepted: **MET**.
- Tier 0 in rules mode; budget stop closes the task and the batch continues: **MET**.
- The mutants list is produced by running the argv: **MET**.
- Each reversal passes `git apply --check`: **NOT MET**.
- Record and rules runs sharing an `--out` pair all their tasks: **MET**.
- No compile-time profile path, and Proxy on every printed number: **MET**.
- The bootstrap reports resamples used and goes INCONCLUSIVE above 5% dropped: **NOT MET**.
- Mine leaves no worktree: **MET**.
- Evidence verbatim, naming its commit: **NOT MET**.
- Original brief's DoD: **NOT MET** (suite red, evidence stale). mine, mutants, split and eval clauses hold in tests; the rules-denial clause holds in m3_batch_invariants, though a CLI rules run over fixtures/m3/batch records 0 denials (per-task sessions win).

## What I ran

- `cargo build --workspace --offline`: exit 0.
- `cargo fmt --all -- --check`, on 1.97 and 1.90: exit 0.
- clippy on 1.97: exit 101 (gate.rs:411). On 1.97 with `-A clippy::filter_next`: exit 101 (driver.rs:424). On 1.90 (`cargo +1.90.0 clippy …`): exit 0.
- `cargo test --workspace --no-fail-fast`: exit 101; 275 passed, 3 failed, 4 ignored.
- `tsr corpus mine <4-commit repo> --suite suite.sh`: 1 task, 1 refactor, 1 regression; worktree list unchanged.
- `tsr task run --mode record` then `--mode rules`, mined task, no scripted runner: real copies; both `rejected`, which is correct with the bug present.
- `tsr corpus mutants` (fake cargo-mutants): 5 tasks, 0/5 reversals apply; `task run` over them: exit 2.
- Evidence §5 split: matches; §6 eval: differs.
- Every other reproduction is quoted with its item above.
- Cleanup: removed the throwaway test, plus `tree/target/` (test scratch) and `crates/tsr-cli/.tsr/`, which the test run created.
