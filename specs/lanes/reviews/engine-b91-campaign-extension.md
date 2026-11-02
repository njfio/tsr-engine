# Independent campaign call-extension review

Reviewer: `/root/b91_full_review`. Read-only source review plus this review record; no campaign mutation, provider call or target execution by the reviewer.

The scoped change permits exactly one explicit append-only call grant from 100 to 140. The original campaign manifest remains unchanged. The grant binds its manifest hash and immediately preceding ledger row, records an authority-evidence hash and reviewer, and is checked both on attachment and every fold. Changed dollar cap, execution-group cap, reservation amount, duplicate grants, stale prefixes and retroactive excess calls are rejected. Existing consumed calls, costs, unknown reservations and execution counts are retained. The $5 accounting cap, 40 execution groups, per-call reservation and provider/source limits are unchanged.

The authority hash is an auditable pointer, not independent proof of user authorization. Attachment still requires the exact separately reviewed grant hash and actual authorization. An implementation upgrade must also match the current campaign code; a grant does not waive that check.

`preflight_run` checks current remaining slots and conservative whole-run cost while holding the ledger lock. It is explicitly a nonexclusive readiness snapshot, not a reservation and not automatically invoked by the run entrypoint. The operator must perform this check before launch; each actual effect independently rechecks current limits. Unknown outcomes or overrun still block effects after a grant.

No material source blocker found. Reviewed new fixture tests cover prefix/manifest preservation, restart through call 140, refusal at 141, changed caps/duplicate/tampered grants, unknown-cost retention, and whole-run slot/cost failure without effects. Test execution and actual proposed grant provenance are pending at this entry; this is not artifact-attachment signoff or a live result.

## Source signoff

Implementer reports all five focused extension tests passed (session 84338 terminal). No additional source changes followed the review. Source signoff is complete with no material blocker. Actual upgrade/grant/authority artifacts remain to be independently checked before attachment; the parent will perform the explicit 14-call/four-group/$0.70 preflight. No live grant or effect was performed during this review.

## Actual artifact signoff

Read-only validation of the prepared artifacts passed. Upgrade `cc6b55afbd8f10e5e7039df0713f8db064f10da571b56e77792b8aaeb1dd587a` binds current code `884ae36ff4398b910e9d9caf6d8fe69940a71fedb610c733bc3e8011ae1b2ea1`, original manifest and actual ledger head `272d61df889057eb3e360c1e2afd88967879cd343c8a8c4363a75ca838b720eb`. Grant `d94cc0c4d146ecb6ad9acfeb71396236f5750b1022554085298fdbe95f344d44` binds the correctly computed next upgrade row `15584952b3ec6e6f3a60d484d9fce5cc82e24c7a0ed67044444da7f06a0b7d33`. Its authority hash matches the parent-supplied approval message identifier, whose authorization is supplied by trusted parent context rather than inferred from the hash.

Before attachment the actual ledger remains 94 calls, 27 execution groups, 66,883 microUSD known/committed, no unknowns and no overrun. Ledger bytes were unchanged by review. Exact upgrade followed immediately by exact grant is ready for parent attachment; intervening ledger activity requires regenerating/reviewing the prefix-bound grant. This signoff does not dispatch a trial or replace its source-bound admission and explicit whole-run preflight.

## Fresh Trial20 admission

Actual generator admission `657441529c101dc765ca421b628fee7a1c29fe342f6aba1610faf6dfbdbe64ac` and runtime-review admission `ce06b0aaab1817951ae86bf7e56aa2c5d3f7400c91a7836228134a6d6ad39623` passed current native-driver admission validation and exact recomputation of the source/campaign-bound reviewer admission. All helper/binary pins validate; trial root is absent. Expiration is 2026-10-26T22:36:28.645257Z.

Scope remains the same frozen synthetic parser, Luna, 240 seconds per request, 8192 completion tokens per request, only existing admitted patch authority. Eight generator slots include two total structural corrections; six distinct runtime-review slots produce a conservative fourteen-call/four-execution-group trial. Final native acceptance remains separate FileReviewer, not model self-approval. After grant attachment, 46 calls/13 groups and 4,933,117 microUSD remain; the prospective fourteen-call accounting allowance is 700,000 microUSD. Actual root preflight is still required after attachment. No trial effects occurred during this check. Ready for the parent's reviewed attachment, preflight and normal launch within this exact admission.
