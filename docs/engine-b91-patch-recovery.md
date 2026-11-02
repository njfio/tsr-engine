# B9.1 recovery after a confirmed uncharged 503

This narrow continuation resumes Trial 15 at `boundary_patch`. It does not restart
the seed or the independently reviewed experiment. No live recovery has run yet.

The parent must have exactly three validated and reviewed generative stages,
completed seed and experiment returns, and a fourth request at `boundary_patch`
that received HTTP 503. The exact provider request must already have an independently
reviewed failed-zero reconciliation in the existing campaign ledger. Unknown
transport, another HTTP status, an applied patch, or mismatched evidence is rejected.

## Preserved evidence and fresh authority

The original trial, journals, source, criteria and STOP presence or absence remain
unchanged. Trial 15 has no STOP file: the campaign wrapper recorded its terminal
failure in journals instead. Recovery pins that actual absence; it does not invent
a historical STOP.

A separate directory holds an exact native project copy. Historical resources are
archived as `parent-shadow`; full original journals and the first experiment remain
in `parent-evidence`. A declared working-journal projection imports the three
successful stages, while the full four-attempt journal, including the failed call,
remains inspectable. The original origin record is retained. Explicit local
relocation and the new admission namespace produce a fresh request identity.

The new admission permits only:

- Three Luna requests: boundary patch, return revision, final proposal.
- Two isolated execution groups: the patch's original checks, then the unchanged
  generated experiment. The final proposal cannot execute.
- 8,192 completion tokens and 240 seconds per request; 16 KiB patch input and
  32 KiB for each final-stage request, reserving 106,496 total tokens.
- 120,000 tool milliseconds across the new groups, with inherited cumulative tool
  accounting checked; the same source path and immutable acceptance checks.

The normal patch-scope review, experiment readmission, semantic review and native
accept/reject machinery remain in place. A model proposal grants no authority.
The same shared campaign ledger counts every new request and execution; old calls
and costs are neither refunded nor charged again. Unknown new outcomes stop the
campaign. Price and ledger-state checks happen before new effects.

At the verified checkpoint the campaign has consumed 12 calls and eight groups,
with $0.004991 settled and no unresolved campaign charges. The older Garden Trial 1
reservation is separate. The remaining approved campaign bounds are 88 calls,
32 groups and $4.995009. This recovery uses at most three and two respectively.
At the previously verified Luna rates, the conservative token allocation costs
about $0.013312; current rates are checked again before launch. Three $0.05 ledger
reservations are conservative accounting, not a provider-enforced billing promise.

## Prepare and run after provider backoff

Prepare a new admission immediately before the backed-off run, so its one-hour
window and implementation hashes are fresh:

```sh
python3 scripts/engine-b91-patch-recovery.py prepare \
  --parent-admission target/b91-integrated-trial15.admission.json \
  --proof target/b91-patch-recovery-proof-trial15.json \
  --trial target/b91-trial15-patch-recovery-1 \
  --admission target/b91-trial15-patch-recovery-1.admission.json
```

Independently check the exact printed admission hash and unchanged scope. The
existing campaign approval covers this continuation; this is not a request for a
new spending allowance. Then use that exact hash:

```sh
python3 scripts/engine-b91-patch-recovery.py run \
  --admission target/b91-trial15-patch-recovery-1.admission.json \
  --admission-sha256 REVIEWED_HASH \
  --reviews target/b91-trial15-patch-recovery-1-reviews \
  --allow-paid-inference --allow-target-execution
```

Normal independent-review pauses resume with the identical command and admission.
Do not edit model responses or historical records. Uncertain or invalid outcomes
must not be retried automatically. `inspect` with the same admission is read-only.
Preparation and inspection perform no model calls or target execution.

## Offline verification

- Two real-controller tests with fake provider and test-only worker receipts passed
  in 87.680 seconds. They exercise the same campaign wrappers as the CLI, the full
  remaining three-stage/two-group path, distinct request IDs, retained consumption,
  terminal replay, archive/source/STOP drift, wrong failure classes, limits, and
  interrupted dispatch with global accounting stop.
- Affected envelope, continuation, admission and accounting checks: 33 tests run,
  28 passed and five existing environment-dependent native checks skipped, in
  14.295 seconds. The new native recovery checks above ran separately.
- The normal six-stage expanded-profile native fixture also passed in 50.946
  seconds, including replay, after extracting the shared patch tail.
- Actual Trial 15 proof validation and disposable cloning passed with forbidden
  provider/worker callbacks never invoked. The full original tree remained
  byte-identical; the disposable check copy was removed. No active live recovery
  admission was left behind.
- Independent source review found and resolved missing archive/projection checks
  and the launch price/ledger-state preflight. Final review found no blocker.

Fixture-generated answers and receipts establish mechanics only. This is not a
live B9.1 pass, a new accepted repair, richer semantic culmination, calibration,
or evidence of causal advantage. A successful later recovery is a linked
continuation, not a claim that the original failed trial ran uninterrupted.

## Live linked result

The subsequently approved recovery generated and verified a correct repair, then
stopped on providerHTTP503 during reflection. See
[the result report](engine-trial15-patch-recovery-result.md). No code execution
needs repeating; this is not a complete reflective cycle or native acceptance.
