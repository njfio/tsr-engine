# Prospective evidence collection preparation

No episode, inference, test execution against corpus targets, new model, operational
activation or external request was started in this pass. The five-task manifest is
a development pilot plan, not a performance sample or a mature corpus.

## Reproduce the plan

```sh
python3 scripts/engine-collection-plan.py build > target/collection-plan.json
python3 scripts/engine-collection-plan.py validate --manifest target/collection-plan.json
python3 -m unittest scripts/test_engine_collection_plan.py
```

The checked-in output is `engine-prospective-collection.json`. Validation recomputes
strict admission receipts and task identities using `engine-evidence-inventory.py`,
pins task/admission/preparation files by SHA256, and compares the complete plan.
Changing budgets, identities or criteria requires rebuilding and reviewing the plan.
It does not assert that a container image is currently available or that source,
review or model execution has been admitted. Inputs and outputs are operator-only;
do not send the manifest or source manifests to a generator.

The 151 tasks cover 11 mutation/package families in two repository lineages. Five
strata are selected, choosing the smallest task hash with a preserved preparation
manifest in each stratum, without using candidate outcomes. This tests plumbing
across packages/operators; it is neither random nor sufficient for the 150/30 support
floors. Missing preparation manifests are excluded, not reconstructed from guesses.

| Task prefix | Family | Independently executable check |
| --- | --- | --- |
| a552be6a9355 | BinaryOperator / rust_tree_sitter | Four frozen TypeScript signature unit tests |
| 39385564a640 | UnaryOperator / rts-mcp | Exact named-list and unknown-name behavior test |
| 0c5a91a6aaa8 | FnValue / rts-daemon | Protocol error round-trip unit test |
| 27230af7de9e | BinaryOperator / agentid-core | Custom spending-limit policy test |
| 06488423176d | FnValue / agentid-core | Three policy persistence/validation tests |

Exact names, frozen test paths, preparation image pins, full-suite argv, task identity
and clone group are in the manifest. All have strict prior baseline-green,
named-red, full-red, reversal-green evidence. That is qualification evidence only;
none of those results is copied into a future episode outcome.

## Provenance and independence

Task source and episode execution are different axes. These tasks are mutation-derived;
PRD B9 explicitly calls for mutants for volume. A future genuinely generated,
measured, independently reviewed repair can be an operational execution of a
mutation-derived task. A scripted repair, authored reference answer, fixture runner
or replay remains synthetic/replay and cannot become operational merely by waiting.
Existing qualification or historical final-oracle records are not retroactive episodes.
The manifest keeps `task_source=mutation_derived`, `execution_origin=not_started` and
prior exposure explicit. All five are development-only, never a fresh holdout.
Repeated turns, attempts and variants retain original task/clone identity.

Before a future live run, bind the existing project scope to an isolated **mutated**
source snapshot. Admit implementation windows separately; freeze every test/build
asset and bytes outside those windows. Include public behavior requirements grounded
in interfaces/docs. Never expose base source, reversal diff, mutation diff, original
Git history, evaluator CAS, private review notes or a reference-expression file to
the generator. The older `engine-rts-task-replay.py` intentionally reads a reference
answer and **must not be used** as the collection driver.

The corpus oracle (`corpus_cmd::oracle_cmd`, `tsr-corpus::sealed::SealedRunner`) uses
sealed execution and rejects missing exact-test evidence. The project adapter freezes
protected files and records command outcomes outside candidate work. Test success can
mechanize named-test/no-regression evidence, not semantic acceptance, review independence,
source integrity beyond the admitted scope or budget coverage. The full workspace
oracle is distinct from the project's named-package suite: do not claim full-workspace
acceptance from a package-only pass. Any separately run oracle must be included in
instrumented tool budgets before an independent reviewer can attest complete coverage.
The current project driver does not automatically account an out-of-band corpus oracle;
that integration remains required if full workspace checks are used for the event.

An independent operator must inspect the frozen task contract, current candidate,
protected hashes, exact test receipts and all resource records, then record acceptance
or rejection. A generator's rationale/self-review is not that review. Closure-level
eligibility review also checks coverage, identity/clone grouping and timely review.
Content hashes prove consistency, not that a human assertion is honest. No review or
`complete_instrumented_coverage=true` draft is manufactured by this planner.

## Runnable episode workflow after explicit scope admission

Required operator inputs: new `RUN`, isolated mutated `SOURCE`, protected `SCOPE.json`,
public `REQUIREMENTS.json`, selected task ID, independent reviewer identity, and an
already installed local model/endpoint when local generation is separately permitted.
The scope must use the pinned image, exact tests, source windows and bounded execution;
validate current image availability without downloading. No current campaign is authorized
by this document. Budget proposal per task: three calls, 55,296 tokens, one-hour wall,
30-minute tool ceiling, 32 KiB replacement bytes, zero external charge. These are ceilings,
not claims of actual cost or sufficient runtime. Calls remain individually at most 240s.

```sh
# After source/scope/requirements admission; no internal state editing:
target/debug/tsr engine --directory "$RUN" project init --spec "$SCOPE" --source "$SOURCE" --requirements "$REQUIREMENTS"
target/debug/tsr engine --directory "$RUN" shadow-view > "$RUN/state-view.json"
# Read state_root from that view. Draft only: origin must describe actual execution.
python3 scripts/engine-collection-plan.py admission-draft \
  --manifest docs/engine-prospective-collection.json --task "$TASK" \
  --state-root "$STATE_ROOT" --reviewer "$REVIEWER" --origin operational > "$RUN/admission.json"
target/debug/tsr engine --directory "$RUN" project-lifecycle-open --admission "$RUN/admission.json"
target/debug/tsr engine --directory "$RUN" project prepare
target/debug/tsr engine --directory "$RUN" project status
```

An explicit training admission now captures evidence-only pre-action rows automatically;
no fake model pin or missing-bundle configuration is necessary. Such receipts have null
model/probability and `model_evaluation_requested=false`. They never change selection.
Ordinary unadmitted/legacy/non-project paths remain unchanged. Configured shadow
predictions still use their original path. Tier2 remains disabled.

For separately authorized local generation, use `scripts/engine-local-model.py --help`
and the [instrumented transport](engine-project-telemetry.md), not the unaccounted
older broker. It captures the exact envelope/model, reserves resources before sending,
and retains failures/unknown charges. A supplied common-event probability is frozen
before action; if omitted, retain missingness and exclude the episode from a paired
baseline rather than inventing a forecast. Reconcile uncertain journals before any
new attempt. Execute through ordinary project run/resume; stop at admitted limits.
Do not feed evaluator reference answers into correction.

```sh
# Future explicitly authorized call; not executed during preparation:
python3 scripts/engine-local-model.py --directory "$RUN" --endpoint "$LOCAL_ENDPOINT" \
  --model "$INSTALLED_MODEL" --out "$RUN/proposal-1.json" \
  --max-output-tokens 2048 --timeout-seconds 240
# Only after a terminal validated proposal; execution remains independently gated:
target/debug/tsr engine --directory "$RUN" project run --response "$RUN/proposal-1.json"
```

```sh
target/debug/tsr engine --directory "$RUN" project acceptance inspect
# Independent reviewer authors a bound review using the normal acceptance commands.
# Only after current reviewed outcome and all executions/resources are reconciled:
target/debug/tsr engine --directory "$RUN" project-lifecycle-close > "$RUN/closure.json"
python3 scripts/engine-collection-plan.py maturity --closure "$RUN/closure.json"
target/debug/tsr engine --directory "$RUN" project-eligibility-view
# Independent eligibility reviewer authors COVERAGE_REVIEW.json after inspecting evidence:
target/debug/tsr engine --directory "$RUN" project-eligibility-review --review "$COVERAGE_REVIEW"
# At an actual current cutoff; do not use the future maturity date as today's --as-of:
target/debug/tsr engine --directory "$RUN" project-training-evidence --as-of "$ACTUAL_AS_OF" --out "$NEW_EVIDENCE"
```

The maturity helper calculates the earliest date from the actual saved closure plus
seven days, normalized to UTC; it refuses missing/multiple/future closures. It is a
calendar helper, not proof validation. No closure exists yet, so the prepared manifest
has no maturity date. Engine reconstruction remains authoritative and keeps immature,
unknown, interrupted, rejected and ineligible episodes distinct. Obtain timely review
within the engine's horizon; do not backdate it. Preserve original receipts on restart.
Build datasets only from eligible evidence with explicit as-of, clone-aware splits,
leakage checks, and genuinely unexposed holdout; five exposed tasks cannot satisfy Tier2.

## Remaining implementation and decisions

The bootstrap gap is fixed and tested through two alternatives, one selected action,
fixture execution, separate review, closure, export and restart. This is synthetic
mechanics validation; labels remain excluded. The manifest consistency tests do not
execute any corpus code. Checks passed: CLI library 125 tests (one ignored), four
planner tests, CLI all-target clippy with warnings denied, CLI build and diff check.
Independent read-only review found no blocker for this preparation-only change. See `target/collection-{bootstrap-test,plan-tests,cli-tests,clippy,build}.log`.

The next concrete PRD implementation gap is cost ranking: B8 specifies normalized
expected tokens plus minutes. `adaptive/shared_decision.rs` still uses `Action.cost`,
and generated patches carry a fixed 0.1 proxy. Resource budgets constrain affordability
but do not learn expected cost for selection. A coherent next change is a versioned,
receipt-derived comparable-action cost estimate, explicit operator normalization,
missingness/fallback and pre-action provenance, with unchanged-state and cost-only
selection contrasts. This is implementable without mature labels; it is **not** a
missing-data or vague user-direction blocker. A second collection integration need is
instrumenting any full-workspace oracle used outside the current package executor.

Two protected tests still require a specific user decision: permit changing only
`m4_corpus::b7_train_apply_prints_raw_and_calibrated_p_and_support` and
`m4_corpus::by_repo_seeds_1_to_8_give_three_non_empty_sides` invocations to pass an
explicit simulated cutoff, preserving assertions/fixtures. Their current fit-without-
cutoff expectation conflicts with reproducible maturity. Neither has been edited.

Actual collection needs an admitted isolated scope/reviewer and a bounded local-call
allowance; mature calibration then additionally needs enough independent measured
live episodes, real elapsed time, matched forecasts and held-out validation. Existing
provider inability and manual semantic-review limits remain. None authorizes external
inference, source exports, downloads, publication or activation.

## Follow-up implementation

The [resource-cost and oracle integration](engine-resource-cost-selection.md) now implements budget-normalized execution cost selection, prerequisite accounting and an owner-admitted full-workspace oracle within the existing reservation. Use that in-scope oracle rather than an unaccounted external command. The earlier implementation-gap paragraphs are retained as preparation history; scope/window/reviewer and local-call readiness still remain required, and no episode has started.

## Concrete one-task readiness

The earlier cost/full-oracle integration gaps are now closed by447b247. See [one-task admission](engine-one-task-readiness.md) for the verified isolated RTS scope, installed dependencies, exact bounds and next operational step. No inference or operational episode has started.
