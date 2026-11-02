# Offline startup reliability follow-up

2026-10-31, source `1a736f6a`. No paid calls, live rebinding, game edits,
provider changes, or frozen-history mutations. This investigation did not change
production code or increase timeouts.

## What was reproduced

The exact previously stalled `d6_tier0-524ba8f7fa638572` binary now lists tests
successfully in 4.885s and passes all **nine tests** in 0.198s wall time (0.08s
reported test-body time), unchanged. Process creation itself took 0.003s.
A supplemental `cargo test --offline -p tsr-batch --test d6_tier0` rebuilt
dependencies and progressed to compiling `tsr-batch`, but had not launched tests
after over 317 seconds (over 187 seconds without compiler output). The owned
command was interrupted, exit 130, preserving `d6-cargo.log`. This is an incomplete
build diagnostic, not a test failure or pass; no timeout setting was changed.
The direct unchanged-binary test result above is the completed targeted check.

The original six-minute pre-output stall and subsequent 30-second failed probe
remain preserved. This targeted pass does not retrospectively pass the interrupted
workspace suite.

Five alternating baseline/optimized initialization pairs used fresh unique native
directories and the exact original authored scope/requirements. All ten passed;
all editable and protected fixture files matched the original text exactly.
Baseline median was **0.194s**, optimized median **0.147s**; range across all ten
was 0.139–0.244s. Process creation took 0.002–0.003s. The original 73.041s
initialization anomaly did not recur. Raw responses retain distinct per-run event
identities; these initialization outputs are not claimed byte-equivalent. The
prior fixed-state read comparison separately established output equivalence.

The binaries are the same preserved baseline and optimized SHA-256s recorded in
`criterion-return-performance.md`. No cache flush was attempted, so these warm
measurements cannot exclude cold executable startup delays. One diagnostic used
an incorrect nonexistent cached-binary path and received FileNotFoundError; that
failure is retained and is not a product failure. The subsequent paired probe used
the verified `target/debug/tsr` path.

## Classification and limits

No persistent test-body, fixture-interference or initialization regression was
reproduced. The tests already use unique temporary stores and scripted repositories;
they do not execute a real project. `project_initialize` writes/syncs fixture files
and records native configuration; it contains no provider call or intentional
73-second wait. The test stall occurred before libtest printed its test count.
Together these observations are consistent with a transient startup/environment
issue; they do not establish its cause. **The OS-level cause is not proven.**
No historical process stack or syscall trace was captured. Antivirus, OS scanning,
filesystem latency and resource contention are possibilities, not findings.
There is no evidence-backed application fix to apply here, and no user decision is
needed for the completed bounded diagnostic. A recurrence needs an owned-process
startup/phase trace while it is stalled, not a shorter timeout or weaker checks.

Raw probe commands, stdout/stderr, spawn timings and unique fixture directories:
`target/criterion-reliability/`. Prior failed probes remain under
`target/criterion-performance/`.

## Repeated reconstruction and safe next boundary

The 556 calls are subprocess invocations, not 556 independent pieces of required
work. The largest groups were 122 cycle requests, 12 previews and 47 full project
requests. Within one cycle request the source independently computes:

- `project/cycle.rs:322`: status, attention, budget/resource-root, adaptive state,
  reconsideration, continuation and objective context.
- `project.rs:350`: status already reconstructs source, adaptive records and
  acceptance.
- `project/attention.rs:6`: repeats adaptive state, acceptance, records and
  criterion inputs.
- `project/cycle.rs:205`: reconsideration rebuilds relational context, adaptive
  state, events and considered records.
- `project/cycle.rs:49`: continuation anchor repeats relational/acceptance views.
- `project/relational.rs:46,350`: relational inputs reconstruct acceptance,
  guidance and records; admitted relationships perform further withdrawal lookups.

These are source-confirmed repetitions, not newly measured per-function CPU
percentages. Independent read-only review recommends starting with explicit
request-local sharing of the relational projection between reconsideration and
continuation-anchor calculation, retaining standalone wrappers. A general Engine
validity cache would be a larger, riskier step and was not implemented.

Any broader verified-state object must live only within one top-level read-only
operation: coherent SQLite read transaction; validated event identities/chain and
CAS bytes; immutable derived views; source/protected-file and CAS rechecks at the
return boundary; freshly evaluated deadline, Stop, reservations and unknown effects.
It cannot survive a write, provider request, execution, restart or next decision.
Changed evidence must fail closed. SQLite alone does not snapshot external files.
`adaptive.rs:352` currently owns a transaction for one adaptive-state call, so a
shared lifetime must preserve transaction ownership and not let nested helpers end
an outer transaction. No such redesign was launched in this investigation.

Independent read-only review checked the measured JSON and safe-lifetime proposal;
no blocking report issue remained. `git diff --check` passed. All owned probes
and the supplemental build are terminal. Ledger and frozen manifest SHA-256s
remain those recorded in the preceding performance report.
