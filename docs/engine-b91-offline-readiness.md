# B9.1 offline launch preparation — not a live acceptance result

2026-10-23. This work connects previously separate prerequisites. It does not consume
an inference or execution admission, and does not establish a live B9.1 pass. Original
frozen scenario `target/b91-frozen-v2` and its acceptance criteria are unchanged.

## What is verified

The native evaluator forecast is an explicitly attributed diagnostic. After committing
an exact action, and **before** attempting it, the operator may call:

```sh
target/debug/tsr engine --directory RUN project evaluator-forecast --file ADMISSION.json
target/debug/tsr engine --directory RUN project evaluator-forecast-status
```

The typed admission names `state_root`, `snapshot`, `commitment`, `action`, `probability`
and `evaluator`. For this scenario probability is `0.1`, evaluator-authored, with no
invented prior outcomes. Native history binds it to the action and actual returned
receipt, keeping its residual separate from dimensional Contrast and from learned
controller predictions, calibration, authority and semantic acceptance. Startup or
unknown execution cannot become an observed expected failure.

The unchanged generated experiment can be independently reviewed for a later source
snapshot without inventing a second Generator proposal:

```sh
target/debug/tsr engine --directory RUN project relational experiment-readmit --file REVIEW.json
```

That file names fresh `state_root`, current `snapshot`, original `previous_admission`,
and a nested `review` object with separately supplied `author`, `reviewer`, `generator`
and `review` text. The native readmission preserves original
check bytes, case identities, mapping and generator lineage. It grants no execution;
the separate check episode, budget, verification admission and receipt import remain
required. Native project relational observations expose only source-checked generated
origin identities, not private reviewer text.

`scripts/engine-b91-project-envelope.py` adds an **offline-only** origin, projection and
journal boundary. Its command line has no dispatch or execution mode:

```sh
python3 scripts/engine-b91-prepare.py validate --directory target/b91-frozen-v2
python3 scripts/engine-b91-project-envelope.py freeze-origin \
  --frozen target/b91-frozen-v2 --directory FRESH_SYNTHETIC_RUN --origin ORIGIN.json
python3 scripts/engine-b91-project-envelope.py inspect \
  --directory FRESH_SYNTHETIC_RUN --origin ORIGIN.json \
  --origin-sha256 PRINTED_ORIGIN_HASH --journal JOURNAL.jsonl
```

`FRESH_SYNTHETIC_RUN` must already be a fresh native project initialized from the exact
frozen source/scope with the two frozen requirements published as guidance. Origin
creation refuses different files, source bytes, cases, goal, image, limits, existing
outcomes or imported history. The origin grants zero calls and zero executions; it is
not a substitute for the separate future live admission. `inspect` can create the
empty origin-bound journal if absent; it never sends or applies anything.

The projection reads normal native `project request` and `project config`. It uses
fixed synthetic parent text; only the exact admitted source and generated descendants
from the same journal are eligible. Protected bodies, arbitrary logs, private review,
extra support files and unrelated history are not projected. Native typed outcomes
preserve Unknown. Supplemental returns must join the same journal's generated
experiment and native provenance, including fresh readmission after a patch.

The first-call gate also reads native `evaluator-forecast-status`: its unique resolved
`0.1` diagnostic must match the current seed action, commitment, receipt and original
source snapshot. Later gates require the exact independently admitted relational
proposal, current review where applicable, actual source-bound experiment return,
observed generated patch and freshly bound unchanged experimental verification. A
withdrawn or merely counted historical narrative cannot satisfy these gates.

Dispatch-intent reserves a slot durably before an injected test transport runs. An
interrupted dispatch, missing validation, unknown application or unsettled metering
blocks continuation after restart. Known truncated output remains invalid rather than
being called model inability or an unknown execution. No unknown charge is zero and
no consumed slot is refunded. Only normal native `propose` validation and review-only
`project run` persistence are exercised by this interface; it refuses to execute edits.

## Checks actually run

- Native B9.1 focused checks: **12 passed**.
- Native `tsr-cli` library: **225 passed, 6 ignored, 0 failed**.
- Binary build, affected Rust clippy (`-D warnings`) and changed-file rustfmt check passed.
- `PYTHONPATH=scripts python3 -m unittest test_engine_b91_project_envelope`:
  **15 passed**. This includes real native fresh project initialization, request,
  source binding, proposal validation and deferred persistence without target execution.
- Combined Python regression (`test_engine_b91_project_envelope`, `test_engine_b91_prepare`,
  `test_engine_local_model`, `test_engine_synthetic_transport`): **52 passed** in 11.74 seconds.
- Independent source review found and verified corrections for native verdict shape, private
  case labels, interrupted application, exact review/return phase gates and the native
  seed forecast join. No remaining defect was identified within the stated offline scope;
  the launch gaps below remain.
- The six-stage restart/ordering test uses an explicitly authored fake controller and
  fake transport with fabricated fixture returns. It demonstrates guards and protocol
  transitions only. It does **not** exercise four actual executions, independently
  establish semantic competence, or establish a complete native live sequence.

Negative checks cover different action/return/forecast identity, stale source/state,
forged origin/admission, changed private source/case/intent, missing/withdrawn review,
foreign supplemental origin, stale prepatch checks, no readmission, unknown dispatch,
completed-before-validation interruption, uncertain store, repeated terminal receipt,
truncation and real transport-shaped usage/accounting. No credentials, model calls,
Cargo target execution, downloads, security changes or frozen test edits occurred.

## Historical launch blockers at 994d632

**The six-call/four-execution envelope is not yet launch-ready.** These are specific
integration gaps, not a need to redefine B9.1 or an assertion that approval already exists:

1. Native prospective resource reservation/settlement is not connected to this new
   project envelope. The native API requires finite `external_cost_upper` and episode
   `external_cost_microusd`; the proposed comparison currently specifies call/token/time
   bounds but no finite monetary accounting reserve. Supply and document an explicit
   accounting reserve through the future admission, distinguishing it from a guaranteed
   provider-enforced spend cap, or separately implement/review native unknown-cost
   reservation semantics. Do not use obsolete pricing, zero, or fabricate a hard cap.
2. The ordinary live driver has not been wired to the four native execution groups,
   separate check-episode lifecycle/resource admissions, independent experiment and
   relational reviews, fresh post-patch readmission, receipt import, semantic review
   and honest closure. The new fixture journal is not that orchestrator. A no-dispatch
   real-controller/fake-runner integration must verify this whole sequence and crash
   boundaries before enabling a live CLI.
3. Only after those offline checks, the exact proposed six-call/four-execution scope
   still needs its own live admission. Previous Garden allowances are consumed. No
   request was dispatched and no allowance is recycled here.

The next implementation should join these existing native boundaries, retaining
operator-assisted independent review. No new inference, target execution or broader
architecture is necessary to test that wiring offline. It must not add a success
shortcut, fabricate reviewed evidence, or reinterpret the authored fixture as a model
finding.


## Follow-up: native accounting and complete driver

The follow-up implementation adds `scripts/engine-b91-driver.py`. Its normal path
joins native reservation/settlement, six model stages, four separately admitted
execution groups, experiment return import and unchanged-check re-admission.
It retains the original parents, dimensional Contrast, separate evaluator forecast,
source versions and generated proposal identities. It cannot close an edited episode
using the verification-only inconclusive path: final closure requires a separately
supplied, complete native acceptance/rejection review. Missing or uncertain review
pauses the trial; the model never supplies its own approval.

Native unpriced accounting is explicit rather than a disguised zero-dollar budget:
`project resource-unpriced-admit --state-root ROOT` selects that mode for a bounded,
non-training episode, and `project resource-reserve-unpriced --file REQUEST.json`
records each prospective call/token reservation. Existing settlement records real
usage and known charges. `external_cost_total_microusd: null` means cost is unknown;
`measured_external_cost_microusd` exposes only known reported charges. Unknowns retain
reservations and block continuation; overruns stop the trial. Existing finite-cost
admissions remain unchanged. No provider cap or new shared-key setting is asserted.

The test worker exists only in the Rust test binary (`#[cfg(test)]`). It applies the
exact committed edit to disposable fixture files and supplies explicitly authored
Cargo-shaped outcomes to the native executor; it never runs Cargo, shell commands,
project source or a container. Production has no fake-worker command or environment
switch. Full-driver fixtures also inject authored model responses and independent
review fixtures. Such results are protocol evidence, not model discovery, genuine
semantic independence, a live B9.1 pass, or mature training evidence.

### Runnable interface (preparation is not approval)

After building the intended binary, create a fresh draft outside the future trial
root. This command performs no provider or target execution:

```sh
python3 scripts/engine-b91-driver.py prepare \
  --trial target/b91-live-trial1 \
  --admission target/b91-live-trial1.admission.json
```

Review its exact hash, provider/model, helper and binary hashes, synthetic scenario,
six-call/four-group limits and one-hour expiry. A future **separately approved** live
run additionally requires the exact admission hash, independent review directory,
and both explicit opt-ins:

```sh
# Only after approval of this exact fresh trial; not run during offline verification.
python3 scripts/engine-b91-driver.py run \
  --admission target/b91-live-trial1.admission.json \
  --admission-sha256 APPROVED_HASH \
  --reviews target/b91-live-trial1-reviews \
  --allow-paid-inference --allow-target-execution
```

Use a sibling review directory, not a prepopulated trial root. Review requests are
written locally with state/source/subject bindings. The operator supplies the
corresponding independently assessed `.review.json`, including the complete native
review at final semantic coverage, and resumes the same command within its original
expiry. Role names alone do not prove independence. No instruction, reference repair,
private review or fixture answer is transmitted as generation context.

Read-only inspection needs no opt-ins and never retries or applies a pending request:

```sh
python3 scripts/engine-b91-driver.py inspect \
  --admission target/b91-live-trial1.admission.json
```

A `STOP` file in the trial root withdraws further work. Changed implementation,
expired admission, missing review, invalid output, unknown transmission/application,
source drift or resource overrun blocks continuation. A pending intent is preserved
for reconciliation, not automatically replayed. Re-running a completed trial returns
its recorded result without extra calls or actions. The final proposal is never
executed. The consumed Garden grant remains unrelated.

### Verification record for the follow-up

The complete real-controller/fake-worker flow passed twice (47.50 and 47.89 seconds).
The final driver-owned native suite then passed **5 tests in 376.458 seconds**:
complete six-response/four-group flow and no-effect replay; 22 interruption trials
covering 11 operation categories before/after effect; known invalid output with
metering; review pause/withdrawal; and unknown charge with measured tokens retained.
The interruption matrix covers initialization, execution, reservation, transport,
settlement, experiment admission/import, patch commit, readmission and closure; it is
not a claim to test every possible instruction-level crash point.

The envelope plus admission checks passed **17 tests in 10.383 seconds**. Six new
native unpriced-accounting tests and 18 affected budget tests passed. Independent
source review found no remaining blocking defect after corrections for implementation
hashes, lock handling, exact return identity, unknown-cost settlement, review/closure
and resource-stop behavior. No denied action or uncertain live operation occurred.

Two additional focused native-driver regressions passed in **10.968 seconds**:
known execution overrun persists its receipt and stops before further generation,
and unknown cost preserves measured tokens without applying a proposal or retrying.
These fixtures remain separate from operational model evidence.

To reproduce the full cross-language path without provider or target execution:

```sh
rustup run 1.90.0 cargo build --offline -p tsr-cli --bin tsr
rustup run 1.90.0 cargo test --offline -p tsr-cli --lib \
  b91_fixture::native_driver_sequence -- --ignored
```

The named integration test is ignored in the ordinary library sweep because it needs
the built normal CLI and Python. It was explicitly run and passed; the ignore is not
a suppressed failure. The larger interruption suite uses the same test-only worker
binary via `TSR_B91_TEST_WORKER_BIN` and `test_engine_b91_driver.NativeSequence`.
No production command accepts that worker override.


Final affected Rust sweep: `rustup run 1.90.0 cargo test --offline -p tsr-cli --lib`
passed **232 tests, 0 failed, 7 ignored** in 53.43 seconds. The explicit cross-language
test above accounts for one of those default ignores and passed separately. Library
and binary Clippy (`-D warnings`), changed-file rustfmt, Python compilation and
`git diff --check` passed. This is affected-library verification, not a new whole-workspace
claim; no actual project/container test execution was included.

**Current readiness:** the no-dispatch engineering integration is verified for a fresh
six-call/four-execution B9.1 trial. No known implementation blocker remains in the
reviewed path. The remaining operational requirements are the exact fresh live
admission/opt-ins, available installed sealed runtime and existing provider access,
and separately supplied source-bound independent reviews during the run. These are
not assumed satisfied by fixture tests. This turn did not test live transport or
runtime availability and did not authorize a trial. No paid calls, account changes,
credential reads, target execution, protected edits or publication occurred.

Semantic questions remain for the live trial: whether actual generated relational
meaning and the second-aim experiment use observed return to guide a boundary patch
while preserving both desires. Authored fixtures cannot answer that question. No
calibration, mature-evidence or B9.1 live-success claim follows from these checks.
