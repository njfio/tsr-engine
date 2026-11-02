# Real repair completed through bounded correction and linked recovery

The engine generated a Rust repair, applied it in the sealed disposable workspace, and passed the immutable original checks and the unchanged generated experiment. An independent agent reviewed the actual source and execution evidence, then acceptance was recorded in a linked review copy. The historical incomplete runs remain unchanged. This is genuine execution evidence, not a simulator or an assistant-authored replacement patch.

## Actual sequence and limits

Trial 19 began with the evaluator-authored flawed seed. Luna generated guidance and an experiment; after one counted structural correction, the experiment executed and exposed the nonempty-field regression. A separate runtime review admitted the resulting evidence-informed revision. Its first patch response was rejected before application: retained structure proves a 592-byte rationale exceeded the published 512-byte limit, but the generic native rejection does not establish that this was its only defect.

A separately admitted continuation archived the complete Trial 19 history and retained its reviewed successful prefix. Luna received exact structural feedback and generated a wholly new patch. Three separate Luna runtime reviews admitted patch scope, unchanged-experiment readmission, and post-result relational guidance. The patch preserved the original nonempty split/parse/error/collect behavior and added an exactly-empty branch returning an empty vector. No host-written solution was substituted.

Both actual execution groups passed: original protected checks 2/2 and unchanged supplemental checks 4/4, all command exits 0. Finite test passes alone are not universal proof; independent source-equivalence reasoning supports nonempty preservation. Both original desires remained traceable throughout. Post-result reflection was grounded, with a minor broad phrase about earlier failures noted by the reviewer. It is not evidence of general reasoning reliability or causal benefit from relational guidance.

The driver stopped after reflection, before dispatching its optional final proposal, because native `project request` exceeded 32768 bytes. The native request contains duplicated current guidance and retained relational history; Python's external-payload bound cannot solve a failure that occurs earlier. No unknown effect or lost call was inferred. This is a demonstrated orchestration limitation: optional next-proposal generation remains coupled to formal acceptance. The limit was not raised or bypassed.

## Independent acceptance without more inference or execution

The already-tested project was copied to `target/b91-trial19-independent-acceptance/project`. Exact inventories and lineage are in its sibling `provenance.json`. The independent review was adopted through existing native `project acceptance review`, not through a model-generated approval. Decision:

`6455431cada2698b9b0a613093368089032d2d006f334d970df9a13f6ddaf9a0`

Snapshot:

`2ea6dab53fc0fb7c521856f42b50034fdfbb25824abb0009e73585166fa350cb`

Native state is `accepted`, both scoped desires `fulfilled`, with no unmet or unknown original requirements and next step `complete`. The review evidence updates the existing belief fold; its 2/3 acceptance-rate proxy is not calibrated confidence. Fresh-process acceptance inspection was identical, and the linked lifecycle was closed. Native `accepted_by_operator` denotes this independent agent judgment through the operator API; it does not claim a human review, measured human effort or mature training evidence.

All 206 files of the source recovery tree remained byte-identical, with no additions. Original Trial 19 remained unchanged. Zero provider calls and zero target executions were needed for the linked acceptance. The original driver still records its request-size stop; it is not relabeled as an uninterrupted successful run.

## Accounting

- Trial 19: 8 calls (5 generation, 3 review), 2 real execution groups, 41,880 tokens, $0.007628.
- Linked recovery: 5 calls (2 generation, 3 review), 2 real execution groups, 32,542 tokens, $0.005058.
- Combined useful repair chain:13 calls,4 groups, 74,422 tokens, $0.012686, including both failed generator responses. No refund or reset.
- Shared campaign: **94/100 calls, 27/40 groups, $0.066883/$5**, no campaign unknowns. Remaining 6 calls,13 groups, $4.933117. Older Garden Trial 1 unknown reservation remains separate.

## Evidence and verification

Implementation commits: `65c432d` bounded protocol correction and `2a64874` source-bound recovery. [Trial 19 failure and correction](engine-real-repair-trial19.md), [recovery mechanics and tests](engine-b91-schema-patch-recovery.md), [independent result judgment](../specs/lanes/reviews/engine-trial19-schema-recovery-result.md).

Local evidence:

- `target/b91-trial19-schema-recovery-1`: generated proposals, native receipts, runtime reviews, archived failures, accounting and read-only inspection audits.
- `target/b91-trial19-independent-acceptance`: review, adoption, closure, original inventories, equal fresh-process inspections and `verification.json`.
- `request.stderr.txt` in the acceptance copy independently reproduces `tsr engine: proposal request exceeds 32768 bytes`; no final-proposal dispatch occurred.

Read-only acceptance replay:

```sh
target/debug/tsr engine --directory target/b91-trial19-independent-acceptance/project project acceptance inspect
```

Offline native/fake-worker, negative, restart and affected-envelope checks passed as recorded in the linked implementation documents. No full-workspace pass is claimed for these changes. Missing local lefthook remains disclosed; no software was installed. No production edits, private exports, credentials/security changes, push or deployment occurred.

## Remaining boundary

The actual repair goal is met. Automatic intermediate review and protocol correction worked, but this was a linked recovery with an independent final acceptance step, not uninterrupted general autonomy. The subsequent offline correction below decouples acceptance from optional next-proposal generation. Native history projection remains a separate limitation when an unresolved aim actually needs another generated proposal; no evidence was removed and no limit was raised.

## Offline clean-stop correction

The normal repair driver now obtains the existing independent final judgment before considering any further proposal. Final-review binding uses the existing native attention state root and acceptance source snapshot, with freshness checks before adoption; it does not build a generation request. Accepted repair closes cleanly and reports the optional proposal as skipped. A rejection is recorded as unresolved and requiring fresh action admission, with no extra generation or claim that the desires were fulfilled. Missing review and stale evidence still block. A previously generated final proposal remains in history.

This is an offline workflow correction, not a relabeling or rerun of the live chain above. The shared campaign remains 94 calls / 27 groups / $0.066883, no new provider calls or target executions. The native 32768-byte request bound remains unchanged. For genuine unresolved follow-on work, its relational-history size issue still needs a bounded projection design rather than dropping evidence or raising limits blindly.

## Fresh-run recovery readiness (offline)

The normal v3 driver already classifies an out-of-range patch rationale as `invalid_rationale_length` before native validation. Its existing two-correction allowance can request a complete fresh response at the same source-bound stage, retaining the failed response's structural quarantine, usage and cost. It does not alter identifiers or patch content. Generic native rejection, semantic rejection, stale evidence and uncertain dispatch remain ineligible. This covers the demonstrated 592-byte rationale defect prospectively; it does not reclassify Trial 19 or prove its discarded candidate had no other defect.

After clean stop, a successful fresh path uses five generator calls and six separate runtime-review calls, with four isolated execution groups. Two shared format corrections permit up to thirteen actual calls; the existing conservative admission reserves fourteen (eight generator plus six reviewer), retaining one unused optional-stage slot. Final acceptance remains a separate independent assessment. The remaining six campaign calls cannot cover this envelope, so no fresh live run is admitted here. A call-limit extension, unchanged total spend ceiling and fresh source-bound admission are required before dispatch.

## Offline verification of clean stop and normal correction

Independent review: [clean-stop and correction review](../specs/lanes/reviews/engine-b91-clean-stop.md). No production validator broadening was needed for the known rationale defect.

With `PYTHONPATH=scripts` and `TSR_B91_TEST_WORKER_BIN=target/debug/deps/tsr_cli-62904bb7a7cdafee`, the affected six-module unittest batch (`test_engine_b91_driver`, `test_engine_project_runtime_review`, `test_engine_b91_schema_correction`, `test_engine_b91_patch_recovery`, `test_engine_b91_schema_patch_recovery`, `test_engine_b91_clean_stop`) ran 43 tests in 867.298 seconds: 42 passed, one stale test expected six actual generation reservations. The only subsequent change to that tested path corrected the expectation to five and explicitly forbids final-proposal dispatch; its admission-capacity assertion remains unchanged.

A focused rerun of `test_engine_b91_driver.NativeSequence.test_luna_expanded_final_stages_full_native_sequence` plus `test_engine_b91_patch_rationale_correction` passed both tests in 103.947 seconds. The new normal-driver test verifies a 592-byte rationale refusal followed by a fresh valid patch without linked recovery, unchanged source/state before correction, six separate runtime reviews, 12 total fixture calls, four fake-worker groups, independent acceptance, clean stop and replay with no new ledger entries. The six pure clean-stop controls also passed. Logs and terminal results are in `target/b91-clean-stop-validation/`.

These are authored deterministic provider/worker fixtures through the actual native controller, not paid model behavior or real target-command execution. No full-workspace rerun or single wholly green aggregate after the assertion correction is claimed. All observed affected failures are resolved by the targeted rerun; historical live outcomes and campaign accounting are unchanged.
