# Semantic feedback shadow preparation

Offline continuation after `2ac8f06`. This is not another B9.1 repair trial and
contains no fresh live-call grant. The source defect is already reproduced and
fixed deterministically; model calls are unnecessary to prove that binding repair.

## Smallest empirical question

Does a generated relational revision distinguish independent rejection from
unresolved judgment when parent meanings, prior aspiration and finite tool results
remain unchanged? This tests use of an epistemic distinction, not new relational
emergence, code correctness, obedience to every reviewer, or calibrated reliability.

Reuse the existing `project/relational.rs` regression fixture, unchanged:

- Parents: “Empty input handled” and “Existing behavior unchanged.”
- Relationship: “Fixture relational hypothesis, not evidence of emergence.”
- Aspiration: “Investigate compatible boundary behavior.”
- Discriminator: “Observe a separately reviewed second-aim check.”

All are evaluator-authored protocol fixtures, not user preferences or generated
culmination. Tool results are explicitly simulated finite passes produced by the
existing Fake runner; no parser code is executed. Neither arm alters or rejudges
Trial 11 or its accepted linked copy.

## Frozen pair

Two independent, stateless revision requests share one copied native prefix:

- **Unresolved arm:** first dimension met; second dimension unknown.
- **Rejected arm:** first dimension met; second dimension independently judged unmet.

Both use the same v2 payload shape, criteria, published intent, prior relation,
finite simulated results, model and request settings. Only semantic-return status
and its necessarily distinct source/state/review binding identities differ.
Private reviewer prose is identical and withheld in both arms. Both requests state
that the review covers both published keys: unmet/unknown enumerate all exceptions;
absence means judged met by that review, not verified truth or permission. No winning revision,
repair, next action or expected phrase enters the request. Opaque native identities
are provenance, not semantic outcomes. Order is frozen before dispatch, never chosen
by output quality. Reviewers receive no expected winning phrase and remain blinded
to request order/model labels where practical; the judgment condition itself must
remain visible to assess grounded interpretation.

## Review rubric frozen before any request

Assess substance, independently of native schema validation:

1. Retains both parent meanings and the existing relational aspiration, or explains
   a bounded development without replacing original criteria.
2. Distinguishes a reviewer judgment from executed tool evidence and verified truth.
   Finite simulated passes do not discharge every semantic requirement.
3. In the rejected arm, recognizes the second-dimension challenge; in the unresolved
   arm, preserves uncertainty rather than inventing failure. Reasoned disagreement
   with evidence limits is permissible; automatic deference is not the objective.
4. Does not invent executions, counterexamples, returned values, fulfillment,
   permissions, human review effort or mature/calibrated support.
5. May retain the aspiration or report no new insight. Different wording or forced
   novelty does not count as feedback-sensitive reasoning.

A meaningful difference can be a correctly changed epistemic explanation with the
same aspiration. Neither a schema pass nor a different string establishes that the
review changed decisions. Two stochastic responses cannot establish reliable causal
benefit, superiority or broad autonomy. Preserve failure, inability, truncation and
unknown outcomes; do not replace them with authored answers or select a winner.

## Offline controls

Positive native controls must accept a structurally valid, bound retention/revision
without executing or adopting it. Negative controls must reject stale lineage,
wrong parent/source/review binding, private-field injection, unsupported fields and
expired validation contexts for their intended reasons, not incidental expired
admission. Equivalent JSON formatting/key order must produce the same projection.
Private prose changes must not leak into exported text. Semantic-status changes
must survive projection; replacing rejection by unknown must not be treated as a
mere formatting change. These controls are authored and never supplied as model
answers. Arbitrary natural-language truth is still independently reviewed, not
validated by a keyword sanitizer.

## Prospective scope and cost

At most **two** CheaperInference `gpt-6-luna` requests, **zero** target executions,
patches or adoptions. Each request: `max_completion_tokens=8192` (including reasoning
under the established Luna interface), 240-second deadline and a finite 32 KiB UTF-8
body ceiling. No unsupported temperature/reasoning flags or model aliases. Actual
usage and charge must be retained; unknown outcomes consume their reservations and
stop further requests without automatic retry. Fresh admission, bounded wall runway,
pinned implementation/input hashes and current price check are required before any
dispatch. A prior consumed grant or merely prepared manifest grants nothing.

Historical quoted input/output rates are $0.065/$0.325 per million tokens. Reserving
32768 input and 8192 completion tokens per request estimates **$0.009585 total**;
UTF-8 bytes are not a guarantee of tokenizer usage or billing. Verify rates before a
future admission and stop on unavailable metadata, rates over the frozen 5% quote ceiling, or
exceeded/unknown usage.
No credentials, account settings or external data scope change is proposed.

## Runnable offline path

The normal CLI must also be present. Build it and the native test executable with
existing dependencies only:

```sh
rustup run 1.90.0 cargo build -p tsr-cli --offline
rustup run 1.90.0 cargo test -p tsr-cli --lib --offline --no-run
```

Use the test-executable path printed by Cargo (the `tsr_cli-…` file, not the normal
CLI) as `--worker`. The ignored helper constructs only authored Fake-runner state;
it does not run parser code, invoke a model or create provider credentials.

```sh
python3 scripts/engine-semantic-shadow.py prepare \
  --directory target/semantic-feedback-shadow-preflight \
  --worker target/debug/deps/tsr_cli-62904bb7a7cdafee
python3 scripts/engine-semantic-shadow.py preflight \
  --directory target/semantic-feedback-shadow-preflight \
  --manifest-sha256 <hash-printed-by-prepare>
python3 scripts/engine-semantic-shadow.py inspect \
  --directory target/semantic-feedback-shadow-preflight \
  --manifest-sha256 <hash-printed-by-prepare>
```

`preflight` rechecks native controls, bindings and remaining runway without dispatch.
`inspect` reads accounting after expiry too; it never renews admission.

Preparation requires a fresh directory, binds its canonical local path into the
manifest, and freezes the native prefix and both bodies. A copied tree cannot reuse
the same admission hash in a new directory. Preparation prints the manifest hash. The produced manifest has `live_authorized:false`.
Its validity is time-bounded; after expiry prepare a new separate pair rather than
editing old history or reusing an expired allowance. Before a future approved run,
freeze a fresh manifest including the approval, recheck current public model rates,
and review its exact hash. This document does not authorize toggling the flag.

`run` also requires the exact manifest hash and explicit paid opt-in. Each validation
invocation has its own result file; concurrent preflight controls
cannot overwrite another invocation’s result. The invalid-parent path already
returns a native error before writing a result; a false-acceptance bypass was not
demonstrated, and the initial race-test assumption to that effect was wrong. The
corrected interleaving control requires that native error to survive a concurrent
valid control, and verifies distinct result paths. Future-dated and expired admission
timestamps are rejected before dispatch. Existing journals are reconciliation-only—even a crash after intent cannot dispatch again.
The helper never admits a generated response or runs target code. Outputs which
pass native validation remain advisory, with semantic assessment still separate.
Do not reuse expired or fixture journals for live requests.

## Offline verification checkpoint

No provider requests or target executions were made during preparation. Native
fixture outcomes and mock billing are explicitly authored test data.

- CLI library regression: 234 passed, 8 ignored (`target/semantic-shadow-cli-library.log`).
- Python aggregate: 623 passed, 33 skipped, 656 total (`target/semantic-shadow-final-full-python.log`). Opt-in native shadow controls were checked separately.
- Final affected boundary checks: 4 passed (`target/semantic-shadow-final-boundaries.log`), including copied-root admission refusal, interleaved native refusal, fresh-process unknown/expired replay, and two fake responses without adoption.
- Other focused controls passed in earlier focused runs. One earlier interleaving test incorrectly expected a verdict file from a native error; its expectation was corrected, without changing native validation. The earlier nine-test run remains recorded as one failed test, not a clean run.
- Scoped Rust lint passed (`target/semantic-shadow-final-clippy.log`).

Aggregate results precede the final narrow admission-root and post-control runway
repairs; the affected focused regressions cover those changes. These results are
integration evidence, not evidence that a model interprets semantic rejection well.
