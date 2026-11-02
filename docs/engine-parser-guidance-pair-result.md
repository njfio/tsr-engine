# Parser guidance pair — stopped on provider uncertainty

The approved two-request output-only comparison attempted its first request and stopped. CheaperInference returned HTTP **503** after **0.612 seconds**, with no typed proposal, usage counts or charge. Request identity: `5a06f778-6291-4d4d-8e00-ebdfe4a7dfaf`. This establishes an HTTP failure response, not zero billing or proof of no provider-side work. The second request was not dispatched. No retry, target execution, adoption or acceptance occurred.

There are no paired semantic results. In particular, this attempt says nothing about useful guidance elaboration, new non-additive emergence, model competence or causal superiority. Blinded comparison cannot be performed without generated outputs; the independent review instead checks failure/accounting/replay evidence.

## Scope and accounting

The launcher checked frozen manifest `020dd9102b3ed5562fe265da3176e475a63fb45c39b6546b206e4abd3962e9f8`, the 5,976/7,354-byte exact payloads and the existing campaign baseline before dispatch. A local admission records the prospective **$0.10 two-call allowance**, launcher hash, zero execution/adoption and fresh official price quote. The campaign ledger separately reserved **$0.05 before the actual first dispatch**, then retained that reservation when billing remained unknown. The unused second-call allowance expires at terminal outcome; it was never a consumed ledger call.

Fresh official model metadata listed input/output rates of **$0.083523/$0.417614 per million tokens**, cache read/write **$0.0167045648/$0.20880706**, with no input-token price threshold. These differ from older recorded quotes but pass the existing campaign's conservative per-call allowance check. They are pricing metadata, not a measured bill. No account or security settings were changed.

- Calls: **1 attempted**, 1 not dispatched; maximum per request remained 8192 completion tokens/240 seconds/32 KiB.
- Tokens: **unknown**, not zero.
- New settled charge: **unknown**, not zero; **$0.05 reserved**.
- Campaign: **184/200 calls, 57/60 groups**; settled known subtotal remains **$0.159002**, committed including this reservation **$0.209002**.
- Campaign is stopped for reconciliation of that exact request. Older Garden Trial 1 unknown accounting remains separate.

## Evidence and review

`target/guidance-pair-live-1/` contains immutable admission, first dispatch intent, bounded receipt/diagnostic and terminal result. The exact launcher is `target/guidance-pair-live-1.py`; its SHA-256 is recorded in the admission. It uses the existing campaign ledger and synthetic transport, with no target execution path. Pre-dispatch independent review corrected a stale exact-price comparison and added measured token/time overrun stops before any live request.

`target/guidance-pair-live-1-replay.json` records a fresh-process replay: all five tracked run/ledger files remained byte-identical, no second intent exists, and the saved terminal result was returned before any network or dispatch. The frozen pair and historical trials were not changed.

Independent result/accounting review is in `specs/lanes/reviews/engine-parser-guidance-pair.md`. It confirms one reservation and unknown bill, no second dispatch, and no semantic conclusion. The stored `schema_valid: false` means transport produced no candidate; it must not be scored as model schema noncompliance.

No permission denial occurred. The remaining operational blocker is unknown usage/billing on the captured 503 request. Reconciliation may later establish its actual terminal charge; absence of a bill must not be treated as zero. A fresh trial must not reuse this request identity or expired allowance.

The next genuine fidelity question remains whether developed relational guidance changes useful possibilities across a return. This modest elaboration pair could supply narrow evidence, but even success would not establish desires culminating into a qualitatively new aspiration. No additional test or result is invented to replace the missing pair.

## Subsequent exact billing lookup

At 2026-10-27 07:47:38 UTC, a single read-only lookup of the captured provider request ID returned exactly one matching `gpt-6-luna` record, `status: failed`, zero prompt/completion/total tokens, and `billing: {status: failed, billed_cost_usd: "0.000000"}`. The top-level cost is null; the explicit nested billing result supplies the zero charge under the existing billing contract. This is positive matching evidence, not an inference from absence or HTTP 503 alone. Evidence: `target/guidance-pair-live-1-lookup.json`.

The exact lookup was joined to the frozen admission and payload, captured provider identity, dispatch chronology, sole campaign reservation and original unknown bill. Independent accounting preparation plus separate root source/provenance review approved the append. The existing `bill_reconciled` mechanism records an explicit `standalone_shadow_pair` source and artifact map; it does not fabricate native/provider journals to fit legacy digest field names. No pinned campaign implementation was changed.

The appended reconciliation establishes **$0 for this failed request** and releases its $0.05 reservation. Campaign state is now **184/200 calls, 57/60 execution groups, $0.159002 settled and committed, no campaign unknowns**. The original call, null bill and terminal failure are retained. The older Garden Trial 1 reservation remains separate and unchanged. No call allowance is refunded, and the expired second-arm allowance is not reopened.

Evidence includes `target/guidance-pair-failed-zero.py`, the exact reviewed dry run, approved review and applied record under `target/guidance-pair-live-1/failed-zero-*.json`. Wrong identity, positive charge, stale chronology and wrong binding controls reject. Actual repeat application is idempotent. `target/guidance-pair-live-1-reconciled-replay.json` confirms fresh-process terminal replay and repeat application leave all five tracked run/ledger files byte-identical, with no second intent. The replay intentionally returns the historical terminal snapshot; current reconciled accounting is recorded separately rather than rewriting history.

No additional model request or second-arm dispatch occurred. The accounting blocker is resolved; the semantic comparison remains incomplete and unscored.

Independent post-append audit: `target/guidance-pair-live-1/failed-zero-independent-audit.json`. It confirms exactly one reconciliation row (`052c676697d7046a0d201c4955e79be961556dac22b97302240cc1c26962a857`), preserved original records and unchanged counts.
