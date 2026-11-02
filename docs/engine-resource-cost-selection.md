# Budget-normalized execution cost and accounted verification

This implements the next bounded PRD B8 cost step. It does not fit a calibrated
cost model, claim cost-per-completion superiority, or activate Tier2. No inference,
corpus collection, downloads or original-target modification occurred.

## Expected cost at selection

Prospectively instrumented project episodes now expose `OptionView.resource_cost`
with `budget-normalized-action-cost@v1`. The shared obligation kernel consumes:

```
estimated future tool milliseconds / admitted tool-millisecond ceiling
+ future model tokens / admitted token ceiling
```

These are explicit resource-budget fractions, not dollars, milliseconds added to
tokens, or a learned TSR strength. The resource contribution is named
`budget_normalized_obligation_value@v1`; historical records are not rewritten.
Uninstrumented episodes retain the old declared action-cost proxy. Notebook and
other action adapters do not silently acquire coding-specific cost semantics.

For current generated patch actions, generation has already happened before
selection. Execution dispatches no model, so future model tokens for this action
are structurally zero. **Already spent generation is not free:** charged tokens
and tool time remain visible as sunk quantities and in cumulative budget checks.
They are not charged again to every candidate. Later corrective generation belongs
to its later reservation; this is not an estimate of total completion cost or
human-review time. Unknown model measurements remain reserved/unknown.

Tool estimates use the mean of complete prior observed executions with matching
image, user, timeout, protected test digest, case inventory, command count and
edited path set. Only Rules execution and corresponding complete receipt/intent
records in the current source history count. Both complete passing and failing
runs count; timeouts, integrity failures, unavailable results and shortened runs
cannot look artificially cheap. A compatible incomplete attempt forces the
conservative timeout reservation bound rather than selecting only fast survivors.
The pre-action source-prefix hash and supporting receipt IDs are exposed. No
future return or mutable stored weight supplies an estimate.

This is deliberately a coarse same-path-set estimate, not semantic equivalence,
a guarantee about a novel patch or cross-project calibration. New/unrelated path
sets, absent history or unready preparation use the full command-timeout bound,
including preparation when needed. All alternative costs use original admitted
budget denominators, not shrinking remaining-budget denominators. Hard reservations
still use upper bounds, regardless of a cheaper learned estimate. Estimates never
create execution authority or refund spent budget.

## Preparation accounting

An actual `cargo fetch --locked --offline` prerequisite is now reserved before
execution under `project-resource-accounting@v1`, with request identity, dependency
binding and tool-time upper bound. The Rules dependency return binds that request
and records measured milliseconds. Known failure is charged; successful reuse
executes nothing and charges nothing again. Missing return retains the reservation,
blocks another preparation/model/execution dispatch, and blocks closure. A restart
with the recorded return reconstructs the charge without redispatch or a second
settlement record. Original records are append-only.

The eligibility reconstructor independently joins preparation reservation and
Rules return, verifies identity/binding/chronology, includes measured usage in
closure totals and rejects uninstrumented post-admission preparation. An unknown
model reservation remains distinct from unresolved tool execution: its conservative
terminal charge does not automatically prohibit every subsequent bounded action.

## Full-workspace oracle

An owner-authored named-package Cargo `ProjectSpec` can opt in with:

```json
{"full_workspace_oracle": true}
```

This is an additional field on the existing complete scope, not a complete spec.
The ordinary non-named Cargo adapter already runs the workspace suite and refuses
this redundant flag. Old scopes omit the field and retain their serialization and
behavior. Generator responses cannot set it or provide commands.

The opt-in appends a fourth command after scoped edit, preparation placeholder and
exact named test:

```
cargo test --locked --offline --workspace --no-fail-fast -j 1 \
  --target-dir /tsr-build -- --format=pretty --test-threads=1
```

It runs in the same sealed nonroot, networkless execution with protected source
read-only and private build scratch. `--pull=never` prevents image acquisition.
The command count feeds both precommit bounds and the single durable execution
reservation. Its observed duration is included once in the execution return.
Interrupted execution retains the whole reservation and cannot auto-repeat.
A named-test failure still runs the workspace check, but a later pass cannot erase
that failure. Timeout/integrity failure stops execution and prevents acceptance.

The workspace result has its own controller-owned criterion
`test:full_workspace_oracle@v1`, separate from named-test criteria. A named pass
with a workspace failure preserves partial progress and leaves this want unmet.
The full-suite parser validates completed harness groups and individual counts,
allows duplicate names **across** binaries, rejects duplicates within a group,
requires actual executed tests and fails closed on truncated/inconsistent output.
Its bounded output allowance is 1 MiB, versus the named-stage 4 KiB. A larger suite
that exceeds this bound remains unavailable rather than passing silently.

All desired workspace files/dependencies must already be included in the protected
scope and installed image. This opt-in does not discover missing files, download
dependencies, confer arbitrary test authority or replace independent semantic
review. The collection plan still needs those exact admissions.

## Verification

- Actual sealed installed-image fixture: named test plus full workspace oracle
  passed, **four receipts / 11,729 ms** charged once. Fixture source and tests were
  newly created in a disposable directory; no corpus target was modified.
- Actual installed-image offline preparation: **1,477 ms** charged; successful
  reuse left the total unchanged. No network or downloads were enabled.
- Deterministic integration matrix: named pass/full fail, named fail/full pass,
  full timeout, restart and duplicate-dispatch refusal. Group-collision, malformed
  counts, empty-suite and truncation controls exercise the full classifier.
- Actual adapter receipt cost control holds probabilities and obligation gains
  fixed. Compatible observed duration changes ranking against an unrelated missing-
  history bound. Removing only the cost difference restores equal value; restart
  and unchanged-state controls preserve ranking. Additional sunk model tokens
  remain charged without changing future-action ranking.
- Prerequisite failure/reuse/unknown tests verify cumulative accounting and refusal
  to retry uncertain work. Existing budget-exhaustion and unknown-usage tests pass.

Affected checks: 130 CLI library tests passed (three ignored, including the two
installed-image checks explicitly run above); 137 learner unit tests plus one
integration test passed. CLI/learn all-target clippy with warnings denied and CLI
build passed. Independent read-only review found no remaining hard blocker after
correcting group identity and full-suite criterion handling. Protected broad-suite
cutoff conflicts remain unchanged; this is not a full-workspace-clean claim.

Logs: `target/resource-{cli-tests,learn-tests,clippy,build}.log`,
`target/resource-causal-tests.log`, `target/oracle-test.log`,
`target/oracle-installed.log`, `target/preparation-installed.log`.
The installed checks require explicit local Docker access; ordinary library runs
leave them ignored. They are mechanics evidence, not operational training labels.

## Remaining boundaries

Prospective collection remains unstarted. The five-task manifest is development-only
and requires scoped mutated copies, protected windows, public semantic requirements,
installed-image readiness and independent reviewer admission before any permitted
local generation. Full-workspace verification is now available inside accounting;
out-of-band commands remain outside that coverage and must not be attested complete.
The original 151 qualifications remain distinct from mature independently reviewed
episodes. Provider capability, semantic review and real elapsed maturity remain
operational dependencies. The two protected cutoff-test changes are still pending
and unchanged. No Tier2 activation or whole-system completion is claimed.
