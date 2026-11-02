# B9.1 Trial 5 — generated experiment and passing patch; review-boundary stop

2026-10-24, approved fresh trial on `0dd264e`, admission
`54e15b7a07e759de5a2ef974c01523dd19be365f2662d8290663afc6988cff14`.
Independent manifest review confirmed six calls/four groups, 8192 completion tokens
and 240 seconds per call, one hour ending 22:36:22 UTC, 240000 aggregate tool ms.
Fresh Python verified active CA/hostname checks and absence of the affected G3 root;
pinned installed image matched. A brief executor disconnect occurred before launch;
reconciliation showed no trial journal or dispatch. Official authenticated model
metadata confirmed availability. No download or settings changes occurred.

## Actual sequence

1. The evaluator-authored seed failed as expected: empty-input check passed, existing
   behavior failed. Both parent desires remained open and unaccepted. The exact native
   seed forecast was evaluator-authored p=0.1, observed false, residual 0.1; no learned
   calibration claim. Contrast names the unmet criterion and `wrong_behaviour`, separately
   from that residual (`terminal-request.json` and `terminal-evaluator-forecast.json`).
2. A genuine model-generated initial relational aspiration passed native validation and
   independent review. It preserved the empty/nonempty distinction. Review characterized
   it as boundary reconciliation, not proof of richer emergence or fulfillment.
3. A genuine generated experiment added one supplemental Rust file and two named tests.
   It maps both aims and tests the second aim with ordinary signed numbers, an empty
   field (`1,,2`), whitespace and overflow. It was separately reviewed/admitted, then
   executed in an isolated disposable project. Empty-input passed; the nonempty test
   failed at the empty-field assertion. Later assertions in that failing test did not run.
4. The generated revision used that actual counterexample, retained both parents and
   identified the filter as the source of regression. Its stated exact returned vector
   follows from source inspection, not a separately printed measurement. Finite coverage
   limits remained explicit. Independent review admitted provisional guidance.
5. A genuine generated patch changed only disposable `src/lib.rs`: exactly empty input
   returns an empty vector, otherwise the original split/map/parse/collect expression
   executes without filtering. Independent static review found that this preserves the
   original nonempty semantics beyond the finite sample. The patch execution passed
   the immutable original checks and full Cargo suite. Semantic acceptance was still
   unresolved; the unchanged supplemental experiment had not yet been rerun.
6. Fresh-source experiment readmission failed **before the fourth execution**. The
   independent reviewer supplied a 529-byte review note, exceeding the native 512-byte
   UTF-8 bound. This was a reviewer/driver boundary defect, not a model-output failure.

No fabricated completion: unchanged-experiment re-verification, post-verification
revision, final proposal and final acceptance did not occur. **Trial 5 remains incomplete.**
It demonstrates the core expected-failure → second-aim experiment → grounded revision →
boundary patch sequence, but not the full approved end-to-end acceptance. No ablation
establishes that relational guidance caused improvement. This is operator-assisted
independent review, not unattended autonomy.

## Reconciliation and accounting

The native state root after refusal equals the readmission review's pre-action root
`8801fd5973bff4a7473c5a2c02aee503bde26f5fee85e895d8a14c65ef814a22`.
Read-only native history ends at the patch return and has no readmission record.
Code checks the review bounds before either supplemental admission or readmission
append. This establishes a pre-mutation refusal. The original driver intent remains
unresolved in its immutable journal; no retrospective result was inserted and no retry
was made. STOP expires all unused allowances.

- **4 calls**, all known and settled; **$0.002319**.
- **10,595 prompt + 2,573 completion = 13,168 tokens**.
- **3 execution groups**, **30,676 tool ms**; **438 patch bytes** including seed.
- No unknown provider cost/usage; local driver unresolved intent is reported separately.
- Two unused calls and one execution group expired.
- Two fresh-process inspect runs agree: 90 events, 44 completed effects, only
  `readmit_experiment` unresolved, dispatch disabled. No journals/reviews changed.
- All 14 prior-trial journal/STOP/quarantine hashes remain unchanged.

Evidence: `target/b91-live-trial5/` driver/provider journals, frozen scenario, native
project/checks-before records, terminal status/request/budget/forecast/summary, STOP,
replay-check; sibling admission, launch/stage logs, prior-hash inventory and independent
review files. Raw wire/reasoning bodies are not logged. No provider service was started.

## Offline repair, without continuing the trial

`Driver.review` now publishes the native review contract and checks exact four fields,
nonblank strings, <=512 UTF-8 bytes each, and distinct role identities before recording
an approval or attempting a mutation. It does not truncate review evidence. Prior
529-byte review and trial history remain unchanged. This changes future admissions'
implementation hashes, not this trial's authority. Independent code review found no
blocking issue. No native validator or protected file changed.

The smallest further live step requires fresh approval; expired Trial 5 slots cannot
be reused. A fresh six-call/four-group trial with these reviewer bounds would test the
full sequence. A separately scoped verification-only continuation would need its own
explicit admission and provenance design; it is not silently substituted here.

## Checks

Final offline results follow; authored fake-provider/worker checks are not live evidence.

- Focused review/contract/lineage suites: **13 passed**, 9.395s (`target/b91-trial5-offline-focused.log`).
- Explicit cfg(test) native six-stage lineage/driver fixture: **1 passed**, 61.518s (`target/b91-trial5-offline-native.log`), fake provider and worker receipts, zero target commands; terminal replay no redispatch.
- `git diff --check` passed. Full Rust suite not rerun: no Rust production changes.
- Native parent episode remains open/unaccepted; STOP is a terminal driver stop, not an invented native closure, mature label or human-review effort measurement.
- Full offline Python discovery: **600 total, 583 passed, 17 opt-in skips**, zero failures/errors, 132.422s (`target/b91-trial5-offline-python.log`). Local socket/subprocess permission used; no provider or target commands in this suite. Existing cleanup ResourceWarnings were visible, not suppressed.
