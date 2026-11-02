# Offline whole-workspace verification — 2026-10-22

Starting revision: `b5a94ae`, branch `codex/m3-evidence-pipeline`.
No live inference, project-target execution, downloads, credentials, publication,
or paid comparison. All provider HTTP activity in these checks uses inert loopback
fixtures. Execution opt-ins and fixture regeneration are unset. Protected tests and
fixtures are unchanged.

## Commands and bounds

Rust 1.90: `cargo test --locked --offline --workspace --no-fail-fast -j 2 -- --test-threads=4`.
The test PATH includes the pinned Rust toolchain and system commands but excludes
installed Docker/Podman binaries, so the legacy self-test cannot launch a real runtime.
This tests the documented no-runtime M0 path, not daemon availability.
The initial workspace command has a 1,200-second outer bound.

Python: `python3 -m unittest discover -s scripts -p 'test_engine*.py'`, with
`TSR_ENGINE_TEST_IMAGE`, `TSR_ENGINE_PREPARED_SPEC`, and `TSR_REGEN_FIXTURES` unset.
The first run had a 300-second outer bound. Temporary loopback listeners required
explicit tool escalation because the restricted sandbox denies bind. No installed
model/provider was contacted by these fixtures.

## Diagnosed failures and corrections

1. `test_rejection_continuation_cannot_extend_original_step_budget` supplied two mock
   CLI replies, predating the `project-budget` read added in `6db31a6`. Added the
   inactive-budget reply without changing existing assertions. Added a separate test
   proving active budgets stop the legacy transport before provider dispatch or journal
   creation. No production behavior change.
2. `operator_drafts_and_episode_initialization_need_no_internal_state_edits` omitted
   the attention binding required since `13c0ce6`. Its fixture response now copies
   the binding from the actual request. Original assertions and source fixture remain
   unchanged; missing/stale binding validation is preserved.
3. Five Rust verifier tests and eleven Python tests could not bind localhost in the
   restricted sandbox. Preserve initial failures separately from permitted reruns.
4. `d3_session_copy` stalled before printing the test harness banner. A one-second
   process sample showed only `_dyld_start`, 96 KiB physical footprint, and no test
   frames. The owned child was terminated with SIGTERM after identity checking so
   `--no-fail-fast` could continue. This establishes startup interruption, not its root
   cause and not an engine assertion failure. No security settings were changed.

Independent code review found no blocker in either test-contract correction. No
protected assertion, deadline, production gate or desired criterion was weakened.

## Known baseline qualifications

- Protected formatter difference remains in `crates/tsr-cli/tests/m4_train.rs` around
  line 45 (single-line `else` block). It predates this pass and remains unapproved.
- Historical corpus-mutant failures have unresolved causes; see
  `engine-mutant-failure-review.md`. Passing this sweep cannot erase them.
- The prior two-second verifier stub deadline flake is documented in
  `engine-root-cause-review.md`. Its deadline and assertions remain unchanged.
- `lefthook` is absent. No installation attempted; explicit checks are reported.

## Evidence

Logs, process sample and bounded command receipts: `target/workspace-verification/`.
Results below are populated from terminal command records, not inferred from progress.

## Terminal results

| Check | Actual result |
| --- | --- |
| Initial complete Rust workspace sweep | Exit 101; 952 passed, 6 failed, 9 ignored; plus one target interrupted before harness startup; 813.35 seconds |
| Affected Rust targets (`tsr-cli --lib --test d3_session_copy --test engine_cli`) | Exit 0; 208 passed, 0 failed, 4 ignored; four test threads; 64.25 seconds |
| Final Python engine suite | 174 run: 164 passed, 10 skipped, 0 failures/errors; 12.28 seconds |
| Workspace all-target Clippy, Rust 1.90, offline, `-D warnings` | Exit 0; 14.08 seconds |
| Changed Rust file formatting and diff whitespace | Passed |
| Whole-workspace formatting | One known protected difference in `m4_train.rs`; not changed |

Across the initial sweep and affected-target rerun, **959 distinct Rust tests have
passing results**, with 9 ignored. This is combined coverage, **not a claim that one
full workspace invocation exited cleanly**. The five bind-denied tests all passed
with loopback permission, retaining four-way parallel execution and the original
two-second admission. The historical parallel deadline flake did not recur; its
cause and general parallel stability are not proved by this pass. The interrupted
session-copy target passed on permitted rerun; the startup-stall cause is unresolved.
The previous corpus-mutant failures did not recur (29 passed, 1 ignored in that target).
The two approved training-cutoff tests passed. No protected tests were modified here.

Read-only process reconciliation after terminal receipts found no remaining known
suite child, Ollama runtime or engine-directions process. No provider was started.
Unrelated pre-existing untracked files remain untouched. Commit only the two scoped
test-contract repairs and this report; no production change was necessary.

Paid comparison approval remains **pending**, and the exact-model hard-cap guarantee
remains **unresolved** as documented in `engine-relational-cap-and-lifecycle.md`.
These checks do not establish semantic emergence or mature operational evidence.
