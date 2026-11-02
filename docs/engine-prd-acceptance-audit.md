# PRD acceptance audit after ac4d447

2026-10-22. Read-only source/evidence audit plus offline regressions. “Implemented”
means code exists; “demonstrated” names an actual bounded result; “unproven” is a
missing evidence claim; “blocked” names an actual scope/environment/data constraint.
Passing structural tests never certifies semantic fulfillment.

| PRD scope | Implemented / demonstrated | Unproven or blocked |
|---|---|---|
| B1–B2 corrected decision goal | Persistent desires/status/lineage; shared conserved obligation scoring; separate Surprise/Contrast; multi-parent qualitative aspirations and return review. Workshop and garden ran five stages without manual steering. | Reliable semantic emergence, causal guidance benefit and rich-desire fulfillment unproven. V2 evidence ontology is offline-verified, not yet live-validated. |
| B3–B5 authority/execution | Typed policies, forecast/version/scope budgets and independent project executor gates; bounded simulator only appends admitted events. | Original Claude Path A hook remains record-only; guarded engine runner is a different operational path. No whole-system zero-unauthorized-effects claim from two demos. |
| B6 history/state | Append-only SQLite/CAS contracts; frozen source/candidate identities, uncertainty and durable restart. Actual trial2 restart preserved journal/storage without dispatch. | Historical trial1 generation/charge remains unknown; local intent does not prove provider execution. Research-level prime-event-structure claims require separate gates. |
| B7 / M2 Mirror | `reflect_cmd::build_mirror` / `render_mirror`: intent/criteria, alternatives, commitment, return and separate residual/contrast; family calibration and operator provenance. | Rich direction/relational views exist separately; unified Mirror integration is optional usability work, not absent persistence. Thirty synthetic episodes are not mature operational data. |
| B8 / M4 calibration | Logistic/isotonic fitting, schema/provenance, leakage, support, explicit as-of and close-plus-seven-day maturity; unknown eligibility fails closed. | No held-out controller-Brier advantage established. Needs genuinely eligible mature population: 150/event and >=30/family, not repeated rows or synthetic demos. |
| B9 / M3 corpus | Mining/mutation/admission and batch/report infrastructure; 151 qualified tasks retained; selection/tension mechanisms. | Task inventory does not establish completed independent reviewed episodes or matched baseline intervals sufficient for M3 exit. No current broad campaign is authorized. |
| B9.1 exact two-desires gate | Mechanisms for predicted failure, preserved wants, contrast, prerequisites and generated alternatives exist; prior generation failures preserved. | Exact empty-input vs unchanged-behavior scenario, low-surprise failure, then second-aim test/boundary patch is not established by garden/workshop/invoice. Needs faithful prospective verification, not relabeling a demo. |
| M5 release | Near-tie abstention/exploration, propensities, policy counterfactuals, shadow and support-gated Tier2 activation/rollback plumbing. | No qualifying preregistered learned-policy benefit or actual evaluated INCONCLUSIVE; neither an operational activation/rollback claim nor learned release follows from tests. |
| M6 extension | Shared decision kernel reused by coding and synthetic archive adapters. | Controlled continual arms, systems4–6 ablations, intuition usefulness, retrieval/representation gains unproven. RAF/Hodge/encoder remain gated research, not default next implementation. |
| B10 privacy/security | Narrow synthetic projection, immutable provenance, rejection/uncertainty handling, no raw provider bodies/secrets in evidence. | Recorder Path A does not universally gate tool execution. Containment guarantees require actual isolated executor evidence; local simulator is not an escape test. |
| B12 decisions / B13 risks | Task-specific scope/provider/budgets came from explicit user admissions. Previously approved two cutoff-only test edits are complete. | Published evaluation, wider domains, activation and new paid runs need their specific authority; do not revive the completed cutoff approval as a blocker. Canonical metaphysical mappings are not engineering success metrics. |

## Source-backed gaps and priorities

1. **Evidence ontology validation:** the demonstrated ambiguity is fixed in shared
   `scripts/engine-archive-contract.py`; tests cover visibility, trust, recording, stale
   versions and actual recorded payload bounds. A fresh bounded shadow comparison would
   test semantic effect; no new code defect is established merely because that is pending.
2. **Exact B9.1 gate:** freeze an honest local task with both criteria and expected-failure
   evidence, then use existing pipeline to test the actual sequence. Preparation can be
   local; model/project executions require explicit bounded admission. This is distinct
   from repeatedly trying an impossible joint notebook predicate.
3. **Original Path A integration:** `crates/tsr-cli/src/hook.rs` explicitly records and
   never blocks; `main.rs::run_hook` returns success on recorder errors. `policy check`
   is separate. Enforcement is a real remaining PRD code gap, not a data problem.
   Repository lane rules protect these M0 files, so they were not changed in this
   preserve-protected-files turn. A scoped gate-adapter change needs an explicit
   exception/ownership decision and dedicated compatibility tests; do not silently
   redesign the current driver or weaken frozen tests.
4. **Operational learning evidence:** continue only prospectively admitted, independently
   reviewed collection with real clocks, measured effort and source exclusions.
   Maturity needs elapsed time; corpus size or a passing patch cannot substitute.

There is no demonstrated need for another generic supervisor, speculative value theory
or autonomous learned attention. Configurable selection is not evidence that attention
itself learned. Scope deviations from the original provider-free Path A were explicitly
requested during development (normal Python provider driver, synthetic adapters); they
do not retroactively satisfy the original hook milestone.

## B9 gate assessment

- Contract/trace: extensively tested and actual bounded replay demonstrated; historical
  unknown is explicit and not quietly reconciled.
- Authority: scoped driver/executor tests exist; blanket original-hook guarantee absent.
- Recovery/control: crash, cancellation, abstention, statuses and forget mechanisms exist;
  this run checks their offline regressions, not universal recovery under all OS failures.
- State isolation: fresh admissions and preserved history evidenced; cross-arm isolation
  must also hold in any future evaluated protocol, not inferred from a single trajectory.
- Learning/calibration promotion: unproven; defaults remain shadow/fallback, no activation.

Independent source review agreed with this scoped distinction and identified the
record-only hook as the clear original-path implementation gap. Full regression results
below remain separate from milestone acceptance and semantic claims.

## Offline regression evidence (2026-10-22)

The complete run uses Rust 1.90, offline dependencies, two build jobs and normal
Rust test parallelism. Supported per-command permission allowed test subprocesses
and loopback sockets; no network settings were changed and no model calls were made.

- Python: `python3 -m unittest discover -s scripts -p 'test_engine_*.py'` —
  **226 passed, 10 skipped, 0 errors** (236 total, 87.76 seconds, exit 0).
- Rust: `cargo +1.90.0 test --workspace --no-fail-fast --offline` —
  **960 passed, 4 failed, 11 ignored**, exit 101, 820.30 seconds within the
  1,200-second bound. Three targets failed: the two M0 targets and corpus mutants.
  This is one complete sweep, not counts stitched from successful reruns.
- Fresh-process workshop, original garden and Garden Trial 2 replay all returned
  zero dispatches/actions. Trial 2 retains $0.006318 settled cost separately from
  the original unknown reservation; combined total remains unknown. The five
  `test_engine_demo.py` tests also passed (included in the Python aggregate).

Logs and machine-readable terminal results are under
`target/post-ontology-regressions/{python,rust}.log` and
`{python,rust}-result.json`; fresh-process replay summaries are `*-replay.json` there.
These local artifacts are not bundled into the source commit.

The two known M0 failures occurred again:

1. `m0_forecast_default::exec_resolves_the_session_task_and_refuses_without_one`
   (`m0_forecast_default.rs:503`) expected exit 3, received **127**, with empty stderr.
   The test assumes no runtime is detected, but detection probes `--version`. This
   run does not establish the deeper cause of runtime exit 127.
2. `m0_harness::selftest_is_reproducible_structurally_excluding_duration_ms_by_name`
   failed reading the fixture at `m0_harness.rs:195` (NotFound). Its runtime branch
   calls `scratch("run1")` again before reading; `scratch` removes the directory.

Both test files are byte-identical to baseline
`8b74019f9073dbaa5f9d13ebf9f7df7c40694e79`; the detection code is also unchanged.
`AGENTS.md` already documents these runtime-dependent failures. They are preserved,
not ignored or repaired by changing protected tests. Missing `lefthook` messages
from fixture Git operations are reported, not fixed by installing software.

Two additional corpus failures recur in the aggregate:
`d9_each_mutant_carries_its_own_failing_tests` and
`d9_found_problems_exit_is_not_a_failure`. Both report mutation reversal failure
against fixture base `116c5cea7ba5c6536ee596cf095a13721427a5c5`. Independent
source review found no changes since `ff30187` in the mutant implementation, its
tests, mining code or M3 fixtures. The earlier
[baseline/concurrency investigation](engine-mutant-failure-review.md) did not
establish a cause, and neither does this recurrence.

`reversal_diff()` collapses materialization, subprocess, patch and diff failures
to `None`; patch stderr is discarded and scratch removed. Thus the displayed
“diff does not apply” is not proof of a patch mismatch. These are **unresolved**,
not classified as harmless or environmental. No new regression is demonstrated.
The narrow next implementation step is stage-specific bounded failure diagnostics
(exit status, stderr, fixture-diff digest) before cleanup, followed by aggregate
reproduction. No protected assertions or runtime configuration should change.

The approved wrapper and Cargo command reached terminal completion; the owned exec
session closed. No model runtime was started. No global process-list bypass was used,
so this is not a claim that unrelated system processes were exhaustively audited.
Socket-related failures from the earlier restricted sweep did not recur under the
supported permission. No tests were rerun to replace these aggregate failures.
Independent reviewers checked the PRD classification, safe runbook and corpus failure
qualification. This turn changes documentation only; the tested code remains ac4d447.

Captured log SHA-256 values:

- Python: `5f89634a46fa7774ecdf8164b6cd104bdef2403c80b951ef36cdd0d5f9a2d11f`
- Rust: `256f0ff9b2413d369d148768f540e8ebf712f00079bfad907f5d69c85530ba74`

## Subsequent allocator investigation

The original aggregate result above remains unchanged. A subsequent investigation
reproduced timestamp scratch collisions, repaired exclusive allocation, and completed
a fresh workspace sweep with **966 passed, 2 baseline M0 failures, 11 ignored**. See
[diagnosis, causal limits and hook boundary](engine-reversal-diagnostics.md). The
new diagnostics preserve future failure stages; old missing diagnostics cannot be
recovered retrospectively.
