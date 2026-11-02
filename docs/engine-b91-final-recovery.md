# B9.1 final-stage recovery

`engine-b91-final-recovery.py` continues a verified synthetic patch after a positively reconciled, uncharged HTTP 503 during `relation_after_verification` or `final_proposal_only`. It is a **linked continuation**, not a new uninterrupted six-stage result. The implementation is committed in `c27a095`; this document does not establish live success.

## Prepare and run

Use a fresh directory and the independently reviewed proof descriptor for the exact failed parent request. Preparation validates local evidence and does not dispatch.

```bash
python3 scripts/engine-b91-final-recovery.py prepare \
  --parent-admission target/b91-trial15-patch-recovery-1.admission.json \
  --proof target/b91-final-recovery-proof-recovery1.json \
  --trial target/FRESH_FINAL \
  --admission target/FRESH_FINAL.admission.json \
  --not-before APPROVED_UNIX_TIME
```

Review the generated admission and retain its printed SHA-256. `--not-before` is an absolute Unix timestamp; use the approved backoff boundary. The admission expires one hour after that boundary. Launch only within the existing approved campaign scope:

```bash
python3 scripts/engine-b91-final-recovery.py run \
  --admission target/FRESH_FINAL.admission.json \
  --admission-sha256 REVIEWED_SHA256 \
  --reviews target/FRESH_FINAL-reviews \
  --allow-paid-inference
```

The run checks campaign availability and current official pricing before dispatch. Independent review requests appear in the review directory. Supply separately reviewed, source-bound responses; generation cannot approve itself. Repeating the same run command resumes a review pause or returns a recorded terminal result. It does not retry an uncertain request.

Read-only journal inspection:

```bash
python3 scripts/engine-b91-final-recovery.py inspect \
  --admission target/FRESH_FINAL.admission.json
```

If the final proposal also receives HTTP 503, preserve that run, reconcile its exact provider identity and charge, and prepare a separately admitted child using its admission and new proof. The same adapter selects one remaining call automatically. Do not reuse the failed identity or edit a journal.

## Boundaries and evidence

- At most two new calls: return revision, then final proposal. A final-only child gets one call. Each permits 32 KiB input payload, 8,192 completion tokens and 240 seconds.
- **Zero execution groups and zero patch bytes.** Execution and commitment methods refuse; the final proposal is never executed.
- All calls count against the same campaign ledger. Unknown outcomes retain reservations and stop further dispatch; prior calls are never refunded or erased.
- The original native history, failed provider receipt and accounting remain intact. The child archives them and explicitly imports the successful prefix into its working journal. Its admission namespace creates a fresh request identity.
- Passed patch and unchanged supplemental-check receipts are bound to source, requirements and historical evidence anchors. Drift, unsupported failure status, missing failed-zero reconciliation, changed archives, or uncertain application blocks continuation.
- Final independent review may record native acceptance or rejection of the unchanged verified source. The zero-execution episode closes advisory-only: it is not a new executed outcome or mature training example. A passing finite test is not universal semantic proof.

## Verification completed before live launch

The native-controller fixture exercised the two-call path, a separate final-only one-call path, campaign accounting, replay without additional calls, zero-execution guards, backoff, projection tampering and uncertain-call restart behavior. It passed in **113.991 seconds**. Responses were evaluator-authored and the test worker did not execute target code; this is mechanics evidence, not generated semantic success.

Affected continuation, envelope and billing checks: **24 passed, 5 skipped** in 17.906 seconds. The skipped cases required a separately supplied native test-worker binary. Python compilation and whitespace checks passed.

An additional read-only check validated Recovery1's actual failed-zero proof, constructed a disposable native clone, and confirmed the expected next stage and two-call/zero-patch budget. No provider or execution callback ran, and the original parent file tree remained byte-identical. These checks do not claim exhaustive crash-point coverage or a completed live B9.1 result.
