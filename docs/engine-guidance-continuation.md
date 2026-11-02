# Guidance across observed returns

Normal `project acceptance feedback` now appends an operator-authored publication
and a scope/source binding in the same transaction. Normal proposal requests fold
that publication forward after observation or review, without another publication
of identical text. The fold preserves the original publication ID, requirement
revision and source snapshot and exposes the current source/decision separately.
Attention is recomputed by the existing source-bound projection, never reused.

Carry requires unchanged requirements and immutable project scope, nonwithdrawn
obligations and an open prospective resource admission with remaining tool/wall
capacity. It neither resets reservations nor adds model calls, paths or permissions.
The request's own final model reservation may settle without invalidating its
response; new calls and execution still pass their existing budget checks.
Unsettled execution/request reservations continue to block consumption. Reading
unchanged published intent is not an execution admission. A source change can
retain intent but makes old semantic review stale; it cannot establish that the
changed source was reviewed. Every committed action still freezes a new forecast.

Legacy publications without a scope/source binding stay in history but require
explicit republication. There is no inferred or backdated approval. Changed
requirements cannot resurrect an older publication. Withdrawn/out-of-scope wants
are excluded, including during source reopening and acceptance returns. A complete
accepted task still yields no eligible attention when no standing residual remains.

## Independent semantic review boundary

Current `Review` is typed and rejects unknown fields. It requires the exact source,
requirements revision, current test receipt, and bounded evidence for every
requirement. Accept additionally requires all criteria met and a passing receipt
for that source. This is an **operator attestation**, not proof of human authorship,
reviewer identity, reviewer effort or independence. Guidance carry grants none of
these. Stale review remains invalid and cannot be refreshed from its old text.

A future automated review adapter needs a separate admission and submission contract:

- Admission: immutable verifier identity/implementation digest, independent evidence
  sources and coverage policy, permitted task/scope, resource bounds, and whether
  operator adoption is required. Identity cannot be supplied as trusted generator text.
- Submission: schema version, admission ID, exact factual root/source/requirements/
  test receipt, reviewer kind (`automated` or explicit operator attestation), per-key
  result (`met`, `not_met`, `unknown`), evidence references, measured resource receipt
  or explicit unknown, and separate semantic/test coverage declarations.
- Application: reject unknown admission, generator self-review, stale/replayed
  bindings, missing coverage, unavailable evidence and unknown-as-met coercion.
  Preserve unresolved evidence; never infer active human minutes from elapsed time.
  Acceptance remains blocked unless the independently admitted policy establishes
  coverage. An accepted typed payload alone cannot establish independence.

This contract is a design boundary, **not an implemented automated reviewer**.
Adding a nominal reviewer-name field to the existing API would falsely imply an
independence check; this change deliberately retains the honest operator interface.

## Verification and next validation

Deterministic regressions use disposable local invoice state and a fake runner;
there are no model calls or real project executions. They cover publication carry
through return/rejection, source drift, stale review/response, restart, private
review-text exclusion, changed intent, withdrawn status, scope mismatch, legacy
unknown binding, exhaustion and expiry. Existing attention controls cover replay,
no residual and accepted/rejected closure. Historical evidence is unchanged.

Next meaningful bounded validation: a fresh admitted synthetic invoice episode,
freeze guidance once, let a genuine model propose, obtain an independent rejection,
then obtain the second proposal through the normal request path **without feedback
republication**. Compare publication ID and changed attention/source binding, run
only newly selected admitted actions, and require fresh independent acceptance.
No such live admission or execution is included in this turn. Previous successful
live evidence and its manual republication remain accurately described in
`engine-attention-live.md`; they are not rewritten as automatic continuation.

Final checks: `cargo +1.90.0 test -p tsr-cli --lib --offline`: **157 passed,
0 failed, 3 existing ignores**. The three new guidance regressions also passed
separately. CLI all-target clippy with `-D warnings`, changed-file rustfmt and
`git diff --check` passed. Logs and independent-review summary are preserved in
`target/guidance-continuation/`. Independent review found the legacy binding issue;
after correction it reported no remaining blocker. No protected files changed.

Fresh follow-up validation is complete: [live evidence](engine-guidance-live.md).
Exactly one publication survived two generated repairs and independent reviews;
accepted export and no-residual stop succeeded. Semantic review remains manual.
