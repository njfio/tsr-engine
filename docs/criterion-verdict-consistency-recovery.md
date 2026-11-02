# Criterion verdict/findings consistency: prevention and recovery boundary

The two stopped trials are preserved. No provider call, live admission, historical mutation or repeated tool execution occurred in this offline repair.

## Cause and existing mechanism

The criterion response schema enumerated `accept`, `reject`, `unknown`, and `abstain`, but did not explain their cross-field constraints. Python `engine-pursuit-reflection.py::_validate` and native `criterion_return.rs::project_criterion_apply` nevertheless enforce: accept requires every finding met; reject requires at least one not_met; unknown requires at least one unknown. Abstention remains valid. A response with unknown verdict and all-met findings is rejected, not automatically interpreted as acceptance.

The browser adapter's `checked()` retains rejected bytes and the counted charge, then asks `engine-bounded-correction.py::Budget.feedback` whether correction is allowed. That component explicitly permits JSON/structural defects only and excludes semantic decisions, identities, source/provenance failures and unknown effects. Consistency errors are ordinary semantic validation errors, not eligible shape diagnostics. The terminal `not a known settled content-format failure` therefore reflects the current policy; it is not a lost format-correction allowance. The native rules independently enforce the same rejection.

The ordinary stage preserves a completed valid assessment for replay. An unresolved started request is not dispatched again. A closed semantic stop cannot be reopened by reusing a correction slot. Existing structural correction can preserve the original request and append validator feedback, with shared counted limits and journal binding, but that machinery does **not** authorize reinterpretation of contradictory judgment fields.

## Changes

New `criterion-evidence@v4` captures expose those existing constraints in the prompt, a `verdict_consistency` payload, and the verdict schema description. The ordinary path explicitly selects the criterion validator. No findings or verdicts are transformed. The new typed `CriterionConsistencyFailure` reports the precise contradictory field states and applicable rule, and states that intended meaning is unknown. Its diagnostic is deliberately outside the shape-correction allowlist.

The shared counted-stage function accepts an explicit validator; other stages retain their prior validator. Historical v1/v2/v3 captures retain their exact original prompts and validation path. Source, evidence, scope, frozen criteria, native CAS/Stop, budgets and independent review remain unchanged. This is prevention and diagnosis, not a claim that models can no longer contradict themselves.

## Verification

`python3 -m unittest scripts.test_engine_criterion_consistency scripts.test_engine_criterion_return.CriterionShape scripts.test_engine_pursuit_reflection scripts.test_engine_bounded_correction -v` — **31 passed** (`target/criterion-consistency-tests.log`).

The authored checks include all 36 combinations of two finding states and four verdicts, exact response preservation, consistent accept/reject/unknown/abstain without reassessment, binding/evidence errors excluded from coaching, no correction reservation for inconsistency, bounded structural exhaustion, restart without duplicate dispatch, Stop and unknown effects, stale payload refusal, legacy capture compatibility, and the actual new capture-to-stage validator selection. No tool execution callback is invoked by the stage fixtures; these are not a new real-browser or live-provider test. Native validation was not changed; no new full workspace/browser run was warranted for this prompt/validator-path change.

Python compilation and `git diff --check` passed. Independent review by `contrast_bridge_review` accepted the patch and confirmed the recovery-policy boundary. Repository hooks remain unavailable; no implied hook or full-suite pass.

## Remaining decision

Automatic semantic-consistency reassessment is **not enabled**. It cannot safely be called a format repair: either the overall verdict or one or more findings might be intended. Blindly converting unknown to accept, or forcing a finding to unknown, would invent the assessor's judgment.

The minimum prospective decision is:

> After a settled criterion response contradicts its own findings, may the engine request one fresh assessment of the identical source and evidence, counting its cost and preserving the failed response, while never retrying a valid rejection or unresolved effect?

If approved, a separately admitted policy should share the existing finite correction allowance, permit at most one consistency regeneration per criterion stage, retain exact original request/evidence plus validator feedback, and stop on repeated inconsistency, source drift, Stop, exhausted resources or unresolved effects. The fresh response must still pass all existing review/native rules. No existing closed trial would be reopened. Whether the model changes its judgment would be recovery behavior, not new experiential learning or proof of culmination.
