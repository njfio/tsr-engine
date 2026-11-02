# B9.1 Trial 3 — initial-development contract rejection

2026-10-23, unchanged verified implementation `791befd`. User-approved fresh admission
`dafc28fb3c0961f468aa7a4b390cf55fc7e13c91f1786975f63f4e2486b42860`,
16:50:08–17:50:08 UTC: at most six synthetic-only deepseek-v4.1-flash calls and four
isolated disposable execution groups. Independent review confirmed helper/binary
hashes, frozen scenario and bounds before launch.

Read-only preflight confirmed the pinned installed Rust image. Model metadata exceeded
the broker's 1MiB catalog limit; the same official endpoint read with a separate bounded
4MiB preflight returned 1,765,283 bytes and confirmed the exact model. No broker or
transport limit was modified, no model probe occurred, no settings or credentials changed.
The normal driver launched once through supported command permission.

## Actual sequence and result

One evaluator-authored seed group ran in the existing networkless sealed worker. The
expected tests failed: the empty-input dimension passed, but existing behavior did not.
Both original desires remain traceable/open, acceptance unresolved. The evaluator-authored
0.1 forecast was bound before this exact action and joined to its failed native return;
its residual is 0.1. Dimensional Contrast remains separate. This is not a learned
calibration result or a claim that unmet desires disappeared.

The first call at `relation_after_seed` completed with typed JSON and finish `stop`.
Envelope binding succeeded; Rust native validation rejected it. The new bounded
structural quarantine establishes:

- Singleton relational envelope with two 64-byte parent strings.
- `revises` is null, but `development` is a non-null object.
- `development.original_revision` is a 190-byte string rather than a revision hash.
- `development.prior_revision` is null rather than a prior revision identity.

The native contract requires no development without an ancestor
(`crates/tsr-cli/src/engine/adaptive/coding/project/relational.rs`, initial branch),
and exact original/prior revision identities for a revision. `Development` uses typed
hashes (`engine/deliberation/relational.rs`). These are demonstrated structural/provenance
violations; the generic native rejection does not identify which check ran first.
No raw response is reconstructed from hashes. The relationship, aspiration and
intended wording cannot be semantically assessed from this privacy-preserving evidence.
Independent review therefore found a contract failure, **not proven semantic inability**.

No generated candidate was admitted or stored as guidance. No second-aim experiment,
generated boundary patch, return revision or final continuation occurred.
**B9.1 did not pass.** The run stopped; five calls and three groups expired.

## Accounting and preservation

- Provider request: `3de82fdd-fa15-4dd4-b86d-a48f8fb4eb95`.
- One call: 895 prompt + 2,662 completion = **3,557 tokens**, including 2,191
  reasoning tokens within completion; 64.759 seconds transport elapsed.
- **$0.001298 settled**; zero unknown usage/cost requests or unresolved effects.
- One seed execution group: **7,888 ms** tool charge, 191 evaluator-authored patch bytes.
- STOP records terminal failure and expired allowance; native episode remains
  open/unaccepted, without fabricated closure, reviewer effort or maturity.
- Two fresh-process read-only inspections were identical: 27 events, 13 completed
  effects, no unresolved effects, dispatch disabled. Journal/quarantine hashes unchanged.
- Trial 1 and Trial 2 driver/provider/STOP hashes match their preserved values.
- The owned driver process terminated. No persistent runtime/service was started,
  no production target/protected source was changed, and no publication occurred.

Evidence: `target/b91-live-trial3/` contains driver/provider journals, structural
quarantine, terminal status/request/budget/evaluator-forecast/summary, STOP and
`replay-check.json`. Admission and launch log are sibling files. Quarantine is separate
from generation and admission and retains no raw arbitrary model text.

## Next concrete step

Offline, make the model-facing contract explicit: initial proposals have null/absent
`development`; revisions identify the original and latest admitted lineage by their
published hashes and include the separated meaning/return fields. Validate those exact
conditions before native submission, with initial/revision/stale/forged-identity tests.
Do not fill missing identities, reshape rejected answers, relax native authority or
retrofit Trial 3. Current prose names revision fields without spelling out their hash
semantics or initial nullness; no further unchanged paid retry is warranted.

This turn records the actual run and independent review only; it does not implement
that correction or authorize another trial. The previously verified 568-pass/16-skip
Python baseline is unchanged; no aggregate suite was rerun for this evidence-only commit.
