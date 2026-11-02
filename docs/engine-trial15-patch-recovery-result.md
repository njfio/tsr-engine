# Trial 15 linked patch recovery: verified repair, reflection unavailable

The backed-off recovery at4a4e01b generated a correct boundary patch and verified
it with two isolated execution groups. It did not repeat Trial15's seed or first
experiment. Post-verification reflection received HTTP503; final proposal and
native acceptance were not reached. This is a linked continuation, not a claim
that the original trial ran uninterrupted.

## Actual results

The generated patch adds an exactly-empty-input return and otherwise restores the
original parsing expression. Its rationale distinguishes source predictions from
observed case outcomes and unknown assertion-level details. Independent scope
review approved only `src/lib.rs`; no protected check or experiment was changed.

The patch passed original checks. A fresh independent readmission then bound the
unchanged generated experiment to the patched source. That run passed both original
and both supplemental cases, with complete Cargo output and exit0. Independent
semantic review found the repair acceptable against both original requirements:
the empty case is explicit, and nonempty execution uses the byte-identical original
expression. Finite examples alone were not used as universal evidence.

Source snapshot: `40d0212f05fa8f05a4b08b8508cd712f4c8115ba165997f2902df6fd0d592aaf`.
Patch test receipt: `618a851b90d6ea62fd6cbd8270bacdc9d803a7df1f79acb5abf06dd8aea2fc10`.
The independent judgment is in
[the repair assessment](../specs/lanes/reviews/engine-trial15-patch-recovery-result.md).
It was not adopted into native state; both desires remain open pending the normal
semantic acceptance boundary. No invented reflection, next choice, fulfillment,
calibration or causal advantage is claimed.

## Failure and accounting

Recovery used two new requests and two execution groups. The successful patch
request used2992 prompt and1089 completion tokens (917 reasoning tokens included),
4081 total, and cost$0.000688. The second request, post-verification reflection,
returned HTTP503 after34.393seconds, request
`54f40ca8-d9d6-4b33-8121-dde186f978c6`. Exact provider billing subsequently confirmed
failed/zero tokens/zero charge. No uncertain request was retried.

A separately reviewed reconciliation helper adapts the existing exact failed-zero
joins to the recovery admission and archived origin; pinned campaign/recovery
implementations and original histories were unchanged. Actual dry-run and six
negative proof cases passed. Exactly one evidence row was appended; identical
replay did not change the ledger. The old unknown native receipt remains, linked
to positive campaign billing reconciliation rather than rewritten.

Shared campaign totals: **14 requests, 10 execution groups, $0.005679 settled**,
no unresolved campaign reservation. Remaining:86calls/30groups/$4.994321.
The earlier two-call pair is separate. Historical settled subtotal is$0.733954;
older GardenTrial1 unknown charge/reservation is still separate and unchanged.
The unused recovery call is not recycled as permission to bypass its terminal state.

Two fresh-process read-only inspections dispatched nothing, had no pending local
effects, and left all294 files in Trial15 and its recovery unchanged. Evidence:
`target/tsr-integrated-campaign-1/recovery1-readonly-replay.json`.
Complete current receipts are under `target/b91-trial15-patch-recovery-1`;
review requests/results are in the sibling `-reviews` directory. Reconciled ledger
status is `target/tsr-integrated-campaign-1/reconciled-status-recovery1.json`.

## Remaining step

Provider availability is still the immediate blocker. Preserve this verified prefix
and back off; do not rerun the repair or unchanged experiment. Post-verification
reflection and final proposal would need a separately bound, zero-execution
continuation under the remaining campaign allowance. Existing patch recovery is
intentionally limited to the earlier boundary-patch failure, so it must not be
reused by editing its admission or journal. Native acceptance is a separate,
evidence-backed decision; this report does not silently adopt it.
