# Independent prospective tuned-repair review

Reviewer: `/root/suite_review`. Date: 2026-10-27. Reviewed the working diff after `d30a8c0`, including `engine-b91-{driver,evidence,project-envelope}.py`, `engine-project-runtime-review.py`, new response-format helper and tests, grounded-profile tests and the frozen tuned plan. No provider inference or target execution was performed by this reviewer.

## Judgment

No blocking implementation finding in the reviewed change. This is a prospective, bounded contract repair, not a retrospective improvement to the original held-out result. The original fresh-task denominator remains **0/2**, with **1/1 original regression**. Both future cases must be reported as tuned retries, including all failures.

The factual field now requires the exact current full-catalog case summary. It retains failed and unknown cases, excludes assertion/value telemetry that does not exist, and rejects changes rather than repairing model output. Source/state/receipt binding remains in the independently validated return basis and catalog digest. Selected support references are a separate subset, explicitly not a way to hide catalog failures. Rich relational interpretation remains in the meaning/development/unresolved and relational fields; those fields still require semantic review. This restriction supplies observed facts, not a winning aspiration, test or patch. Retention without novel development remains possible.

New admission v6 and origin v3 explicitly select this contract and pin the helper implementation. Legacy origins do not receive the new fact-field validator or response-format payload. Historical proposals are not rewritten. Added schema guidance is applied before the existing serialized payload bound, only to experiment and patch generation. Native validators, source checks, correction limits, protected tests and immutable original requirements are unchanged.

The revised review criteria correctly distinguish authoritative parent requirements from nonexhaustive summaries. Omission alone is not a contradiction; actual narrowing, waiver and false observational claims remain grounds for rejection. A reviewer can still err, including in free prose. The canonical summary must not be advertised as validation of arbitrary interpretation.

## Structured-output evidence and limit

Official [CheaperInference parameters documentation](https://www.cheaperinference.com/docs#parameters), independently read on 2026-10-27, states that `response_format` is forwarded and support is model-specific; parameters may be ignored or rejected. Its public Luna metadata snapshot identifies `structured-output` and Chat Completions support (`target/luna-structured-readonly/model.json`, SHA-256 `fd16b79d468b602c26b4e710fb63c9cf7eb64e4b09728f6950fdf0a0100db103`). The [OpenAI structured-output guide](https://developers.openai.com/api/docs/guides/structured-outputs) documents strict schema behavior and refusal/incomplete-output exceptions. This change deliberately uses `strict:false` for optional fields and dynamic harness paths: it is best-effort formatting guidance, not demonstrated constrained decoding or a provider guarantee. The provider may reject the prospective request; such a result must consume its counted attempt and remain visible.

The helper supplies field shapes and existing path/length constraints, never an assertion or reference repair. JSON Schema character lengths cannot enforce UTF-8 byte limits; native byte validation remains mandatory and tested. Optional probability is not forced to null. No security, provider, model or data-scope expansion was found.

## Verification

Independent reviewer command:

`cd scripts && python3 -m unittest test_engine_reflection_facts test_engine_repair_response_format -v`

**19 tests passed in 9.698 seconds**, including five imported existing native evidence tests. Coverage includes false passing/duplicate/invented cases, mixed pass/fail/unknown, changed epoch, source-bound inference, support-subset distinction, unchanged input, free-prose non-guarantee, native extra-field rejection, and Unicode character-versus-byte rejection. These are deterministic checks, not model-performance evidence.

Inspected grounded-profile tests exercise actual assembled six-stage context for both task identities and legacy profile, implementation/origin tamper, wire-boundary limits, full native controller flow with explicitly fake worker/provider outcomes, restart, and rejected false summary before generated storage. Final terminal results for that concurrently running suite must be recorded before launch; this reviewer did not duplicate it.

## Launch boundary

The proposed two-run reservation of 28 calls/eight execution groups fits the reported remaining 35 calls/ten groups under the unchanged campaign total 200/60/$5. Exact fresh admission hashes, implementation pins, current rates, expiry headroom and live ledger must still be checked by the parent immediately before dispatch. This review is not a substitute for those existing launch checks. No new acceptance criteria are introduced. Two jointly tuned samples cannot establish causal improvement or general reliability.

### Final source review update

Parent reports terminal success for all five new grounded-profile checks, including the full native fake-provider/fake-worker sequence (104.947 seconds), plus 17 legacy checks. The final runtime-review pin adjustment was independently inspected: `pins(evidence_profile)` conditionally includes the new response-format helper for `canonical-case-facts@v1`; the admission passes the exact driver profile, and runtime callbacks compare the full admission against its current regenerated value. Legacy helper sets remain unchanged. Regenerate stale prospective admissions after this source change; do not dispatch the earlier draft. Runtime-module verification and exact fresh artifact checks are recorded separately when available.

### Final artifact checks

The parent reports runtime-module terminal results: seven passing checks, four explicitly opted-out native checks; the separately completed five-test profile suite covers the native flow. Independently validated the fresh clamp v6 admission through `validate_admission(..., fixture=False)` without dispatch, compared all current driver/runtime implementation pins and criteria binding, and checked canonical artifact hashes:

- `target/tuned-clamp-1.admission.v2.json`: `a6672eb2c32a6f7f4245d536e362d017d69e8e0f2a6110e8be0ece5831d4bf44`.
- `target/tuned-clamp-1.runtime-review.v2.json`: `43f76f9389818cfca2c78b6933f79a66f842d2a8907fb5d96c97b587a321d6e9`.

These bind singleton-clamp, unchanged suite, Luna, seven generator/seven runtime-review calls, four groups, existing 16/32 KiB bounds, two shared corrections, separate final FileReviewer and expiry 2026-10-27T01:15:37.498762Z. Old drafts are preserved, not dispatched. The whole-plan preflight records baseline 165 calls/50 groups/$0.142733, no unknowns, and 28/eight prospective effects with $1.40 conservative allowance; it is explicitly nonexclusive, with durable per-effect checks still required. No blocker found for this exact fresh first-run admission. The second run needs its own exact fresh admission and reconciliation of the first outcome as already planned.
