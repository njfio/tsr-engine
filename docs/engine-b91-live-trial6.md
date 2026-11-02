# B9.1 Trial 6 — rejected no-development revision

2026-10-24, approved trial on `3b975b9`, independently reviewed admission
`72f75bfb16153d094a430e2de234a96585f86118c06c136b6b00cbdd5c2f4730`.
Limits: six calls, four isolated execution groups, 8192 completion tokens/240 seconds
per call, one hour ending 23:57:46 UTC, 240000 aggregate tool milliseconds.
Fresh Python confirmed active certificate/hostname verification and absent affected
G3 root; installed image digest matched and official metadata confirmed the approved
model. No downloads, settings changes or original-target changes occurred.

## Observed sequence and limitations

- The expected evaluator-authored seed failed. Exactly-empty input passed; existing
  behavior failed. Both original desires remained open. Native forecast/return and
  dimensional Contrast are retained separately; p=0.1 is evaluator-authored, not
  learned calibration.
- The first model call generated valid initial relational guidance distinguishing
  exactly-empty input from empty fields in nonempty input. Independent review approved
  provisional guidance, interpreting its “unsatisfiable under current source” wording
  as diagnosis of the current failing implementation, not incompatibility of desires.
  This is boundary reconciliation, not proof of richer emergence.
- The second call generated two supplemental Rust files with one test each: exactly
  empty succeeds; `1,,2`, `1,`, and `,1` must error. Independent review admitted these
  finite checks. Actual isolated execution passed the empty-input test and failed at
  the `1,,2` assertion. Later trailing/leading-comma assertions were not reached.
  These supplemental checks do not test malformed numbers or overflow.
- The third call returned a revision with `proposed_development:null`, while changing
  its `relationship` string. Python rejected it as `no_insight_meaning_drift`, consistently
  with the unchanged native validator. No rejected revision was admitted.
- No boundary patch was generated or executed; there was no unchanged-experiment
  re-verification, post-verification revision, final proposal or acceptance.

**Trial 6 is incomplete.** It repeats actual second-aim experiment generation and an
observed counterexample, but does not establish successful return-informed continuation.
It provides neither causal guidance benefit nor general autonomy evidence.

## Exact diagnostic evidence

Independent review compared retained canonical string hashes against the prior admitted
proposal: only `relationship` differs (347 → 350 UTF-8 bytes); aspiration and
discriminator are identical. The rejected wording is not retained, so three extra
bytes do not establish substantive semantic drift, hallucination or loss of meaning.
Quarantine retains types, lengths, hashes and the exact rejection, not raw provider
wire/reasoning text. The supplied instructions and native/Python equality checks agree.
They used “retain” without explicitly saying verbatim; this is a possible usability
ambiguity, not evidence that it caused this response.

## Accounting, stop and replay

- **3 calls**, **$0.001309 settled**; no unknown charge or usage.
- **7002 prompt + 1163 completion = 8165 tokens**.
- **2 execution groups**, **12646 tool ms**; **191 evaluator seed patch bytes**.
- Three unused calls and two execution groups expired at STOP. No retry.
- Native parent episode remains open/unaccepted; no invented maturity or review effort.
- Two fresh-process read-only inspections agree: 67 events, 33 completed effects,
  no unresolved effects, dispatch disabled. Journals, quarantine and review files
  unchanged. All 17 saved prior-trial file hashes match.

Evidence: `target/b91-live-trial6/` journals, quarantine, frozen/native project and
checks-before records, terminal request/status/budget/forecast/summary, STOP and
replay-check; sibling admission, execution logs, prior-hash inventory and independently
written reviews. All owned run commands terminated; no persistent provider service.

## Narrow offline clarification

For revisions declaring no development, messages now explicitly say to copy prior
relationship/aspiration/discriminator **verbatim as JSON string values**, without
paraphrase or normalization. Initial proposals are explicitly outside this rule.
Existing validators, lineage, no-new-insight option, authority and history are unchanged.
The model can still propose a genuine development through the existing nonnull branch;
novelty is not forced. This clarification is not a proven provider-compliance fix.

Independent review caught an initially unscoped sentence; it was corrected to “For
revisions” and tested in complete assembled messages. No further live call is authorized
by this change. A fresh bounded trial would require approval; an immediate unchanged
retry is not justified by this record.

## Offline checks

Results recorded below. Authored fixtures are mechanics tests, not a live B9.1 pass.

- Final focused lineage/review/contract checks: **13 passed**, 9.770s (`target/b91-trial6-offline-checks-final.log`). Includes actual assembled instructions and whitespace-only relationship drift rejection.
- Native six-stage fixture on frozen final helpers: **1 passed**, 58.856s (`target/b91-trial6-offline-native-final.log`), fake provider/worker receipts, zero target commands, no-redispatch terminal replay.
- The first offline native fixture stopped with `admitted implementation changed` while reviewer-requested wording was being edited. This was the intended guard, not a target failure; its log is preserved in `target/b91-trial6-offline-native.log`. Only that affected fixture was restarted after code was frozen.
- Independent review passed after explicitly restricting the new sentence to revisions. Native validators/acceptance rules remain unchanged. `git diff --check` passed; no full Rust rerun because Rust production code did not change.
- Full offline Python discovery: **600 total, 583 passed, 17 opt-in skips**, zero failures/errors, 132.881s (`target/b91-trial6-offline-python.log`). Final revision-scope wording was also checked by the focused and native runs above. Cleanup ResourceWarnings remain visible, not suppressed.
