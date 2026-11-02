# Independent clean-stop review

Reviewer: `/root/b91_full_review`. Offline source/test review; no live inference or target execution.

No material source blocker found. Final semantic review now precedes optional proposal generation and uses the bounded `project attention` root plus matching acceptance snapshot, rather than the oversized generator request. Snapshot/root are rechecked after the independent callback and immediately before native adoption; cached stale reviews cannot be adopted. The existing native acceptance API still enforces requirement/source/receipt evidence. Runtime review delegates final acceptance to the separate FileReviewer.

Only acceptance closes the project and skips optional generation. Rejection returns `independently_rejected_fresh_admission_required`, preserves unresolved desires and does not close or generate another action. No reviewer/unknown decision blocks. Historical optional proposals are retained and labeled; prior run records are not rewritten. The native 32768-byte limit is unchanged. Legacy patch-recovery accounting now reports actual call count instead of the former fixed three.

Six focused controls reportedly pass: accepted no generator request/call, rejected open replay, missing reviewer, callback drift, cached stale preadoption review and retained historical proposal. Source diff check passed independently. Native affected suites and aggregate verification remain in progress at this checkpoint; this is source signoff pending those results, not a clean-suite claim.

## Existing normal schema recovery covers the demonstrated rationale failure

`validate_stage_envelope` now emits `invalid_rationale_length` before native validation when edit rationale is not a string or is outside 1..512 UTF-8 bytes. This exact category is in the bounded correction allowlist. A fresh v3 normal trial can therefore request a complete fresh response for the observed 592-byte defect without special clone recovery. All existing source, authority, native validation and runtime-review gates still apply. Generic `native_validation_rejected`, stale bindings, `return_basis_rejected`, semantic rejections and uncertain outcomes are not eligible. Historical Trial19 is not reclassified, and the oversize rationale is not proof that its discarded candidate had no other defect.

No broader native-rejection retry mechanism is justified by this evidence. A focused normal-driver patch-stage rationale correction fixture can verify this existing route and shared-budget/restart behavior without new provider calls.

## Specific normal patch-correction fixture source review

Reviewed `test_engine_b91_patch_rationale_correction.py` before execution. It runs the normal v3 Driver with the actual native controller, authored provider/reviewer fixtures, cfg(test)-only worker and shared campaign ledger. A 592-byte rationale is injected into the first boundary-patch response; the next callback asserts identical state root, source snapshot and file bytes, with no patch execution or patch-scope review yet. It checks exactly two patch requests, one correction marker, one eventual patch review, twelve aggregate calls/four worker groups, accepted clean stop without final proposal, and byte-identical ledger replay. These assertions meaningfully target the demonstrated integration failure; no material test-design issue found. Execution was pending at this entry, and even a pass would remain deterministic fixture evidence rather than live semantic performance.

## Aggregate result and reservation assertion repair

Inspected `/tmp/b91-clean-stop-complete.log`: the complete affected run ended **43 tests, 42 passed, one failed, 867.298 seconds**. Sole failure was `test_luna_expanded_final_stages_full_native_sequence`, expecting four 16384-token and two 32768-token actual reservations; accepted clean stop correctly emitted four plus one, because optional final generation is skipped.

Independently reviewed the narrow assertion repair: actual reservation expectation becomes four 16384 plus one 32768, with explicit absence of `final_proposal_only`. The unchanged admission-capacity assertion remains 180224. This distinguishes authorized capacity from consumed calls and does not relax a resource ceiling. No production change accompanied it. Targeted repaired-case and normal592-byte correction verification is still pending at this entry; the aggregate failure remains documented rather than relabeled green.

## Final signoff

Independently read `target/b91-clean-stop-validation/targeted-final.log`: both the repaired expanded-profile reservation case and the normal592-byte rationale correction/native-runtime-review/clean-stop case passed, **2 tests in 103.947 seconds**. Final diff check is clean. Combined evidence is the prior 42/43 aggregate (one stale actual-use assertion), its targeted corrected pass, the new correction pass and six focused clean-stop controls—not a rerun whole-workspace green claim.

The new native fixture demonstrates unchanged state/source before correction, no premature patch review or execution, twelve shared calls/four fixture-worker groups/six runtime reviews, independent final acceptance without optional provider generation, and byte-identical ledger replay. It remains authored fixture evidence, not live semantic validation.

No remaining material source or demonstrated test blocker for this scoped clean-stop and already-supported rationale-correction path. Final formal acceptance remains separately supplied; unknown or semantic rejection is not automatically retried. Reviewer made no provider calls or target executions and did not duplicate the test suites.
