# Identified model forecasts

The local direction provider uses `identified-obligations@v1`. `direction-request`
adds the existing ledger `obligation_ids`; no preference, observation, or receipt is
rewritten. IDs survive inspection, composition, observation revisions and appended
refinements. The state root binds each proposal to its exact request version.

The model supplies `forecasts`, an object keyed by every current obligation ID, and
`supports`, a list of selected IDs. A prerequisite continuation has its own forecast
object. Original/refinement relationships and historical first/terminal forecasts
are identified in the model context. The model must echo `state_root`.

The Python adapter saves the decoded raw offer as `directions-N.json`, validates it,
and saves its lossless positional conversion as `normalized-directions-N.json`.
Only the latter is submitted to the unchanged Rust offer interface. Rust checks the
state root again before mutation. Old positional offers/receipts remain readable;
new model responses cannot silently use that legacy format. Normalization does not
change predictions, desires, scoring or authority.

Validation rejects missing, unknown or duplicate identities, stale versions,
malformed structures, nonfinite/out-of-range values, and contradictory coverage
expectations. Duplicate JSON keys are rejected by the strict decoder. For a coverage
refinement A contained in original B, expected covered counts must satisfy:

```
E[A] <= E[B] <= E[A] + |B - A|
```

Equal sets therefore require equal forecasts. These are consistency bounds for the
existing count predicates, not a demand to predict actual success. Fractional and
uncertain forecasts remain allowed. Rejection names the offending identities and
relationship; it does not supply an action, rewrite a forecast, or force acceptance.

The journal records the contract and a hash of complete system/user messages, in
addition to the existing context hash and frozen call budget. Historical journal
entries are not backfilled. An uncertain dispatch remains nonretryable until
reconciled. The existing `composition-contract` allowance permits only three more
calls after cumulative attempt 17; this change does not grant additional calls.

Checks: 24 Python driver tests, 25 Rust deliberation tests (one existing ignored),
and CLI all-target Clippy passed. Independent review corrected structural-error
handling, complementary count bounds, positional remnants and huge-integer handling.
Logs: `target/identity-python.log`, `target/identity-rust.log`,
`target/identity-clippy.log`. This is provider-contract verification, not evidence of
truth, general understanding, or real-corpus statistical superiority.


Live attempt 18 reached the 240-second timeout before a response was received.
The local provider logged a terminal HTTP 500, the client exited, no proposal file
was written, and its temporary service was stopped. A terminal reconciliation entry
was appended without rewriting the started record or refunding the call. To bound
response cost, subsequent identified offers permit at most two alternatives (the
normal minimum remains two; a sole executable inspection may still use one).

Final live verification: attempt 19 returned two genuinely generated identified
alternatives (998 output tokens). Their original-account forecasts were consistent
with the same-set refinement, unlike attempts 14–17. The chosen composition raised
independently measured coverage from 1/3 to 2/3 while preserving brevity at 1.
The chosen one-note refinement was forecast at 1/3 but observed at 1: uncertainty
and prediction error remain visible; forecasts were not replaced with actual values.
Generated purpose prose still said “inspect” despite typed composition, so this
result does not establish reliable natural-language explanations.

Attempt 20 returned the ordinary post-return refinement review without admitting
additional desires. Final observation is `[2/3,1,2/3,1]`, with unchanged standing
wants and IDs. Restart root:
`f964569ff36e3eec036023037bb0814f45d9d0701753d59c2f358f5a86019508`.
Raw identified forecasts exactly match the normalized internal values. Evidence:
`target/prerequisite-live/identity-summary.json`, `identity-before.json`,
`identity-after.json`, `run/directions-19.json`, `run/normalized-directions-19.json`,
`run/turn-19.json`, and the append-only `run/local-directions.jsonl`.

All three remaining calls are consumed (20 cumulative), including the reconciled
timeout; zero paid calls. Temporary provider PID 74499 was stopped and no listener
remained. Full original account coverage is not complete. The exact next verification
need is a fresh bounded local direction continuation from this preserved root; no
new semantic user decision is needed, but this pass's inference allowance is exhausted.
No benchmark or whole-engine reliability claim follows from one corrected offer.
