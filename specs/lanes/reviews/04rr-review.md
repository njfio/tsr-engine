**Lane 04rr (M2, second repair): not merge-ready** — the workspace is green and nine of the fifteen open findings are fixed, but the residual sign in every fixture episode and in the Mirror over them is still observed minus forecast (D11), reconcile still cannot settle anything from the CAS and nothing records the operator message the brief requires in that case (D1), and the evidence doc's test run is not the tip's run.

## Items

**(1) D1: PARTLY.** Evidence is now per dispatch and retries are counted per candidate. The CAS still never decides anything.
- Repro (a): RunApprovedTest Proposed → Committed → Dispatched, with `cas/<blake3(log)>` written and no Executed. Reconcile prints `unresolved retryable` and exits 0.
- Main's code has no link from a dispatch to its tool log: `exec.rs:374` hashes the log but never writes `cas/`. So the brief's fallback applies. The tree holds no trace of the operator message that fallback requires (not in the evidence doc, the code or the tests).
- The new CAS path is dead: `PendingDispatch.evidence_digest` is set only on an Executed, which also settles the candidate (`reconcile_cmd.rs:296-304`), and `from_log` never populates keys/digests for a pending dispatch (`:150-190`), so `:378-385` never fires.
- Comments (`:142-149`, `m2_reconcile.rs:1214-1215`) claim "the CAS decides"; `d1_cas_digest_without_executed_stays_unresolved_retryable` asserts the old defect.

**(1) R4: FIXED.**
- Repro (b′): ProposePatch Dispatched → Executed (digest in CAS) → Dispatched. Prints `unresolved blocked`, exit 1; the log ends `executed, dispatched, unresolved`.
- Repro (c), three rounds of reconcile then re-dispatch. Round 1: `retryable`, exit 0. Rounds 2 and 3: `refused`, exit 2.
- Repro (d): Executed, then re-Dispatched, with no `cas/`. Prints `unresolved retryable`, not `executed`.

**R1: FIXED.** `m2_admission` passes 3/3 in the full workspace run.

**(2) D4: FIXED.**
- `tsr review --family alpha --stats --store <fixture>` prints `labels=10 … unweighted=0.2857`, exit 0.
- `README.md:136-137` shows the no-task form.
- Cosmetic: `tsr review --family alpha` without `--stats` now says `invalid task id ""` (`main.rs:250`).

**(3) D5: PARTLY.** Every loop row now calls `reconcile_one` with `ToolCheck::from_log` (`m2_reconcile.rs:619-765`). But the store each row reconciles is the same three events (`dispatched()`, `:177-209`): the fake executor's effect never reaches the log.
- Intent, Send, Acknowledgement and None therefore get identical input, and every row asserts `UnresolvedIdempotent`. That includes the no-crash row.
- The per-row tests still feed the fake's keys or a hand-built ToolCheck and assert `Executed` (`:333`, `:400-405`, `:465`), a result the CLI can never produce; the suite contradicts itself.

**(4) D9 propensity: FIXED.** With two commits of 0.25 then 0.75, the Mirror prints `propensity 0.7500` (`reflect_cmd.rs:318-331`). Minor: a later `Abstain` commit keeps the stale propensity (`:329`).

**(4) D9 intent: PARTLY.**
- A rejection on a task with no criteria now prints `intent: (none recorded)`.
- Intent is still "the first task-scoped preference", now with a filter on the string prefix `"the dimension this contrast clarified"` (`:262-264`, `:289`). The fixture Mirror shows the criterion text as the intent.
- `TaskOpened` carries no intent (`tsr-domain lib.rs:712`, `task.rs:28` stores only a digest). Nobody reported that the brief's requirement cannot be met without a contract change.

**(5) D10: FIXED.** `intuition_executed` resolves True after Proposed(Intuition) → Committed → Executed → closed (`resolve.rs:113-115`).

**(5) R2: FIXED.**
- TaskClosed{operator} plus an accepted label recorded in the log gives Censored. Stop{operator} plus accepted gives Censored.
- Fixture episodes close `done` (`m2_episodes.rs:484-486`). Regenerated fixtures are byte-identical.
- `reflect --family alpha|beta|gamma` shows bins with `observed 1.0000`.

**(5) R3: FIXED.**
- Repro: a copy of `claude-session.events.jsonl`, plus Stop{done}, plus `tsr review <task> accepted` through the CLI.
- The session's own two common-event forecasts go from Censored (as recorded, no label) to True.
- The limitation is documented, and `reflect --family alpha` over that store prints `no scored forecasts yet`, as the doc quotes.

**(6) D11: NOT FIXED.**
- The library is fixed: `filter.rs:56` and `score.rs:50,310`, and `m2_residual.rs` asserts that they agree.
- The fixture generator still writes `residual: truth - f.raw.get()` (`m2_episodes.rs:444`), under a comment saying "forecast minus observed". All 90 Surprise records in the 30 committed episodes are observed minus forecast.
- The Mirror over `alpha-00` prints `by generator residual -0.4670` for p=0.4670 with outcome 0.
- `m2-evidence.md:181-182` explains this as "forecast minus observed … so its residual is −0.4670", which is arithmetically wrong.
- The lane changed the domain contract's comment to "raw probability minus outcome"; the fixtures contradict it.

**(7) R5: FIXED.**
- With `TSR_DB` set, `reflect`, `task reconcile` and `review` read it from an empty working directory and create nothing; with no store (or `TSR_DB` at a missing file) every M2 command exits 2 `no store at …` and creates nothing.

**(8) R6: FIXED.** `TSR_REGEN_FIXTURES=1 cargo test -p tsr-cli --test m2_episodes` passes 7/7, exit 0, and leaves the episodes byte-identical.

**(8) R7: PARTLY.**
- Fixed: `m2_reflect.rs:567` now uses the temp dir, and the two test files are renamed to `m2_review_stats.rs` and `m2_mirror_d9.rs`.
- New writes into the workspace:
  - `m2_store.rs:25-26` creates `<root>/target-r5/m2-store/…`, a new directory that is not ignored (its files are `*.db` and `.tsr/`).
  - `m2_reconcile.rs:979-980` writes `<root>/target/d1-reconcile` whatever `CARGO_TARGET_DIR` is.
- That breaks the item's rule that tests write only to temp dirs.

**(8) R8: FIXED.**
- A forged id passed to `record_contrast` returns `ContrastIdMismatch` (`write.rs:127-136`).
- Withdrawn or OutOfScope appended directly `by: Rules` leaves the preference Open (`active.rs:170-178`).
- Residual gap (low): `expand` accepts a forged Contrast that was appended directly as an Observed event (`write.rs:152-166`), because the content-address check sits only on the write helper.

**(8) D12: FIXED.**
- `tsr note` on an unknown task exits 2. With `--store` pointing at a missing file, no file is created. With an existing store, the store is unchanged.

**(9) D6: PARTLY.**
- These blocks re-run byte-for-byte: `reflect <task>` (now with `by <forecaster>`), `--json` and `--family alpha/beta/gamma`.
- `review rejected`, `note` and `reconcile` match except for the timestamp-derived ids.
- The `cargo test --workspace` block is elided ("…") and claims **256** passed. The tip has **260** tests, which suggests the run predates the last commits; the four `m2_resolution` tests account exactly for the gap. So the run quoted is not the run of this branch's final state.

## New defects

**N1 (low): an early hook Stop{operator} censors a session for good.**
- `operator_stop` is sticky and is checked before anything else (`resolve.rs:143-145`, `:327-329`).
- The hook writes a Stop on every turn, with `operator` whenever `stop_hook_active` is set (`hook.rs:331-335`).
- Repro: Stop{operator} → Executed → Stop{done} → accepted resolves Censored, not True.

**N2 (low): unrequested change to M0 commands.**
- `tsr task status` and `tsr task export` now refuse a missing store with exit 2 instead of creating it (`main.rs:314-345`, `:358-371`).
- Outside the lane's owned arms; no m0 test covers it.

**N3 (low): misleading reconcile surface.** The dead `evidence_digest`/`from_log` paths (see D1) present a CAS check that cannot fire.

## Definition of done

- `cargo build --workspace`: **MET**.
- `cargo fmt --all -- --check`: **MET** (exit 0).
- clippy `-D warnings`: **MET for lane code.** On 1.97 it fails only on `filter_next` (`tsr-policy/gate.rs:411`) and `collapsible_match` (`exec.rs:482`), main's files, likely 1.97-only; clean with those allowed. 1.90 not available.
- `cargo test --workspace --no-fail-fast`: **MET.** 260 tests, 258 passed; the 2 failures are Docker-only (`m0_forecast_default::exec_resolves…`, `m0_harness::selftest_is_reproducible…`; docker daemon unreachable). `m0_reconstruct`, `m0_real_session`, every m2 and tsr-ledger test pass.
- `fixtures/m0`, `hook.rs`, `exec.rs`, `store_path.rs`, `task.rs` byte-identical to d2b5f96: **MET.** Absent from `lane.diff`; sha256 equal to `m3rr/tree` and `m4r/tree-*` (also from d2b5f96, untouched there). `lib.rs` gains only `pub mod` lines; but see N2.
- A test naming its ids per item that failed before its fix: **PARTLY.** Tests are named d1, r4, d4, d5, d9, d10, r2, r3, d11, r5 and d12 (that last covers R8). R6 and R7 have no named test. "Failed before" is **CAN'T TELL**: there is no git history.
- CAS log without Executed resolves through reconcile, or the operator is messaged: **NOT MET / CAN'T TELL.** The CAS never decides, and there is no record of the message.
- Re-dispatch never settled by an earlier Executed; a second retry refuses: **MET.**
- `review --family --stats` with no task: **MET.**
- Every crash-matrix row calls reconcile: **MET literally.** The rows are indistinguishable (D5).
- Propensity is the shown candidate's: **MET.** Intent is TaskOpened's: **NOT MET** (heuristic; TaskOpened has no intent).
- `intuition_executed`, TaskClosed{operator} and Stop{operator} censoring, hook Stop{done} resolution, non-zero fixture bins: **MET.**
- One residual sign: **NOT MET** (fixtures and Mirror).
- M2 commands find the M0 store and never create one they only read: **MET.**
- Evidence doc quotes every command verbatim, including the workspace runs: **NOT MET.** The test run is not the tip's, and the sign explanation is wrong.
- Original brief's DoD:
  - **MET:** admission, Surprise/Contrast, `review rejected` + supersede, `--json` five sections, `--family` bins and Brier, no TSR term with a number, "synthetic" stated.
  - **NOT MET:** "forecast minus observed" for the fixture Surprise.

## What I ran

- `cargo fmt --all -- --check`: exit 0.
- `cargo build --workspace`: exit 0.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 101, on `filter_next` at `gate.rs:411`.
- The same with `-A clippy::filter_next -A clippy::collapsible_match`: exit 0.
- `cargo test --workspace --no-fail-fast`: 258 passed, 2 failed (both Docker-only).
- `TSR_REGEN_FIXTURES=1 cargo test -p tsr-cli --test m2_episodes`: 7/7, exit 0; episode sha256 unchanged.
- Script re-running every `tsr reflect` block of `m2-evidence.md`: 5 of 5 identical; the 6th (`.ev-d6/claude-session.db`), rebuilt from the m0 copy, prints the quoted output.
- `review rejected`, `note` and `task reconcile` on a scratch copy of the fixture store: same lines as the doc, ids aside.
- `tsr review --family alpha --stats`, `review --stats`, and `review --family alpha` (no task).
- Python check of the 90 Surprise residuals in `fixtures/m2/episodes/*.json`: all equal observed minus forecast.
- R5/D12 shell checks: `TSR_DB` set or missing, empty working directory, `note` on an unknown task with a missing or existing store.
- Throwaway `crates/tsr-cli/tests/zz_rr.rs` (removed): D1/R4 CLI scenarios a, b′, c, d on real stores; D9 propensity/intent via the CLI; D10/R2 and N1 resolution; R3 over the m0 session copy with `tsr review accepted`; R8 forged contrast and fold.
- Clean-up: removed `zz_rr.rs`, `fixtures/m2/store` and the `target/`, `target-r5/`, `crates/tsr-cli/.tsr` the tests created; the tree's sha256 listing equals the pre-review one.
