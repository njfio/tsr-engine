# B9.1 Trial 9 — test names used instead of evidence references

Approved trial on 2026-10-25 at `5928609`, admission
`2eba883975e19048d3aabb47d8349bd6e74c5e2f20b2d37b1df23334f663058c`.
The initial environment check could not execute because the Mac was unavailable;
access returned before admission/dispatch. Independent readiness review then verified
all helper/binary hashes, unchanged scenario, six-call/four-group bounds and one-hour
expiry (03:20:58 UTC). Fresh Python verified certificate/hostname checks enabled and
G3 absent; the pinned installed image was available. No downloads or security changes.

## Actual outcome

The expected seed failure left both original desires open. Exactly-empty input passed
and existing behavior failed. The native evaluator-authored p=0.1 forecast remains
bound to the seed action/return; this is not learned calibration.

The first generated aspiration preserved both parents as a joint parsing contract.
Its optional evidence basis correctly separated observed case results, source
predictions and unknown assertion details. Independent review approved provisional
guidance, with a qualification: dropping empty fields only predicts success when the
remaining fields parse. It did not establish richer emergence or fulfillment.

The second response generated a finite, dependency-free supplemental experiment.
Independent review admitted it within the allowed harness scope. Actual execution
passed the exactly-empty case. In the nonempty case, the signed-value assertion
completed before `parse("1,,2").is_err()` failed. Later malformed-input, whitespace
and overflow assertions did not execute. This sequence is established by independent
inspection of the frozen harness and bounded log; the typed model catalog remains
case-level evidence, not assertion telemetry.

The third response included `return_basis`, unlike Trial 8, but its observed list
contained the test names `empty_input_handled`, `existing_behaviour_unchanged`,
`tsr_empty_input_ok` and `tsr_nonempty_semantics`, rather than catalog IDs `0`–`3`.
This is established by exact hashes/lengths of retained structure matched against
already-published names, not reconstruction of discarded prose. The gate rejected
these references before review-only storage or admission. The revision's natural
language was not retained, so its semantic quality remains unknown.

**Incomplete:** no boundary patch, original tests on a patch, unchanged experiment
re-verification, admitted return revision or final proposal. This does not demonstrate
a complete grounded learning cycle, causal benefit, general autonomy or richer desire
culmination. Earlier trials and the failed response were not rewritten.

## Accounting and replay

- **3 calls, 2 execution groups; $0.003351 settled**, no unknown provider charges.
- **8546 prompt + 5312 completion = 13858 tokens**.
- **14195 aggregate tool ms**, 191 evaluator seed patch bytes; no generated patch.
- STOP expires three remaining calls and two groups. Native parent remains open and
  unaccepted; no invented closure, maturity or human review time.
- Two fresh read-only processes agree: **67 events, 33 completed effects, zero pending**,
  dispatch disabled. Driver/provider/quarantine/review hashes did not change.
- All **24 captured historical trial hashes** match. All three run commands terminated;
  no persistent provider service was started.

Evidence lives under `target/b91-live-trial9/`: driver/provider records, native project,
checks-before, frozen scenario, quarantine, STOP, terminal request/status/budget/summary/
forecast and replay check. Sibling admission, stage logs, prior hashes and separately
bound review records preserve the operational evidence.

## Offline improvement and limits

The original prose explicitly required `return_evidence.cases[].id`. Trial 9 violated
that instruction; this was **not** a contradictory instruction or faulty acceptance
gate. Independent review judged it useful to also enumerate the exact current IDs in
the advertised JSON schema. The assembled request now derives that enum from the same
catalog it displays. Empty catalogs permit only an empty observed list. No name-to-ID
normalization, new authority or validator relaxation was introduced.

Tests cover the schema's exact membership, valid IDs, rejected names, empty catalogs,
and existing stale-binding/duplicate checks. The full Rust/Python driver fixture uses
authored outputs and zero target commands, not live evidence. Independent review is
required for prose truth even when references are structurally correct.

This representation improvement is untested live. Any further live trial requires
fresh approval; spent/expired allowances cannot be recycled. Verification follows.

Offline verification so far:
- The six-stage real Rust/Python fixture passed, retaining the 16-KiB limit and zero
  target commands. Its ten-test batch had one stale test-schema expectation (old
  string-only shape); this failure remains in `target/b91-trial9-offline-native.log`
  (64.257s). No production failure was suppressed.
- That assertion now checks the actual catalog's exact enum or empty-catalog rule,
  retaining all conditional/field assertions. All **9 focused tests passed** in
  5.583s (`target/b91-trial9-final-focused.log`).
- Independent implementation and report review found no remaining issue. These are
  offline fixture checks, not a successful live Trial 9 or proof of model improvement.
- Full offline Python discovery: **608 total, 591 passed, 17 opt-in skips**, zero
  failures/errors in 133.020s (`target/b91-trial9-offline-python.log`). Existing
  resource-cleanup warnings remained visible. `git diff --check` passed.
- No Rust production code changed; no new whole-workspace Rust sweep is claimed.
  The corrected request representation remains unverified against live generation.
