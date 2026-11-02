# Independent held-out repair results

Reviewer: independent agent `/root/suite_review`. Source inspection and receipt review only; no provider requests, target executions, replacement code or native acceptance mutation by this reviewer. The preregistration remains `docs/engine-heldout-repair-plan.md`.

## singleton-clamp — repair verified, full cycle unsuccessful

Run: `target/heldout-clamp-1`. The normal controller reached all four real execution groups: expected seed failure, generated experiment failure, generated patch pass, and byte-identical experiment re-verification pass. It then stopped on a valid runtime semantic rejection of final reflection. It did **not** reach the separate FileReviewer gate or native acceptance. No manual override is appropriate. This is an unsuccessful full-cycle attempt in the fresh-task denominator, despite a technically correct repair.

### Actual repair and experiment

The model generated two substantive supplemental cases: singleton endpoint results, plus ordered/extreme inclusive clamp cases and a reversed `(lower=4, upper=3)` case expecting the exact error. Case declarations match Rust function names. This tests the second aim meaningfully, not a vacuous assertion or inverted expectation. The unchanged harness bytes were directly compared in `checks-before/coding-work/tests/tsr_supplemental_singleton_clamp.rs` and `checks-after/coding-work/tests/tsr_supplemental_singleton_clamp.rs` against the retained generated proposal.

The actual generated patch introduces `if lower > upper { return Err("invalid range".to_string()); }` and retains `Ok(value.max(lower).min(upper))`. Independent source reasoning supports both requirements: reversed bounds preserve the original exact error; ordered bounds keep the original expression; equal bounds produce that endpoint for any input. Min/max comparisons perform no arithmetic that could overflow. The protected `Cargo.toml`, `Cargo.lock`, and `tests/contract.rs` bytes were directly compared against the frozen source and were unchanged. No hand-authored repair was substituted.

Patch commitment: `1568e8e1c7f5ed119d88b93b0842875c86d2ed3d2fb9da5b8a572be3b92d3a3e`.
Repaired snapshot: `e7bb52777c10776ccc3d392ddca4ed46c4a8eb787e6a3ed829298c67fc9bc227`.
Protected passing receipt: `fb1728032e0967f5f30fe82bdfdc138cf692a4e451976de81efb10df7bd8ccee`.
Unchanged supplemental passing receipt: `a78418434c824f65ca53e7bae852ba0325ba7bcdda612daca65b6a7d5f6ad8b9`.

These finite receipts plus source reasoning support the scoped repair, not general all-program reliability. The native repair remains unaccepted because the controller stopped before formal assessment.

### Truthfulness defect missed by runtime review

The retained `generate_relation_after_experiment` development text says the case outcomes include **passed `existing_behavior_preserved`** and also failed `existing_behavior_preserved`. The bound `return_basis.resolved_observations` contains only a failed outcome for that case:

- Evidence ID `2`, supplemental `existing_behavior_preserved`, outcome `failed`, receipt `11fb5d4bb72afed4e32e3cf67a1f914cf1306f5849b8dfb38d87d8430c864e44`.
- Evidence ID `3`, supplemental `singleton_returns_endpoint`, outcome `passed`.
- Evidence ID `0`, protected `requested_behavior_handled`, outcome `passed`.
- Evidence ID `1`, protected `existing_behaviour_unchanged`, outcome `failed`.

Thus the extra passing claim is unsupported, even though the same prose also mentions the failure and properly qualifies assertion execution/returned values as unknown. The runtime review accepted that reflection. The later patch rationale accurately lists `0 passed, 1 failed, 2 failed, 3 passed`; the final reflection accurately lists the current four passing cases. Those later statements do not erase the earlier false observation claim. The preregistered truthful-cycle requirement is not satisfied.

### Terminal runtime rejection and independent disagreement

The final runtime review `373dd0f9a99fff76a276cb93ca7c9242e00b1ec74e631b0c3a9f570ab4b5cbe5` returned `reject`, with `parent_fidelity:not_met`, because the final discriminator and retained-meaning prose omit explicit repetition of the all-i64 scope and no-overflow qualifier. It explicitly states this is not a finding that the source violates the requirement. The other three findings are `met`.

Independently, I do **not** find evidence that the final reflection actually narrows or replaces the parent requirements. It retains their exact parent identities; the immutable requirements remain supplied separately; its unresolved field explicitly references all-i64 uncertainty; and the short relationship/discriminator is compatible with the full requirements. Requiring each summary field to repeat every qualifier is not an explicit frozen criterion. This looks like an overly strict runtime interpretation, not established requirement violation. That disagreement does not authorize overriding the runtime verdict or changing the measured outcome. The run remains stopped and unsuccessful end to end. Together with the earlier missed false-pass claim, this shows both semantic sensitivity and error in the same-model runtime review boundary.

### Assistance, correction and accounting

No intermediate operator steering or replacement code was used. One known-invalid **patch-scope reviewer** response had a `schema_enum` error with one unsupported evidence reference (`ca203747c81c9aaa38b3eb72bdf56561af938b544cb95fc6268b2ec1726b5ea0`). The existing bounded correction issued a full fresh review; the original diagnostics and correction marker remain. This was protocol correction, not semantic repair or an uncertain retry. The terminal valid semantic rejection was not retried.

Parent reconciled terminal accounting: **12 calls, four actual execution groups, $0.014341**, all settled; campaign145calls/44groups/$0.120512, no campaign unknowns. Exact token totals and no-effect terminal replay are to be supplied by the accounting/replay record, not guessed here. No final native acceptance file was created by this reviewer. The result supports repair generation and live bounded reviewer-format recovery, but not a successful truthful autonomous full cycle or non-additive relational culmination.

## unicode-prefix — useful experiment and interpretation, protocol exhaustion before repair

Run: `target/heldout-unicode-1`. Two actual execution groups completed: seed failure and generated experiment failure. The source remains byte-identical to the authored failed seed; protected Cargo files and contract tests remain byte-identical to the frozen files. No patch was committed/executed, no unchanged experiment re-verification ran, and no final FileReviewer/native acceptance was reached. This is unsuccessful in the fresh-task full-cycle denominator.

The valid generated experiment was substantive and semantically correct: it asserts `prefix("aé🦀",2)==Ok("aé")`, `prefix("é",2)==Err("invalid count")`, and zero-count success on empty and multibyte strings. These are true scalar-count expectations, not inversion, vacuity or wrong-aim checks. Actual supplemental receipt `36c60c8c19cdbd7ea9a28929a1c0e1a073aa0d03f5609701b58747789e4b32c8` records the positive/invalid-count case failed and zero-count case passed. Protected receipt `8b05e970cbf31ef6bc635cb289f6a029e7e2510ac1867f8b5ab06c376c95f3c7` likewise records requested behavior passed and preservation failed. Both parents remain present.

Initial and post-experiment reflections accurately distinguish these actual case outcomes from source-predicted byte/scalar behavior, and leave assertion execution, returned values and universal behavior unknown. The generated source prediction correctly identifies that byte-length validation may allow a count greater than the number of Unicode scalar values. No false observation statement was found in the retained valid reflections.

Two initial experiment responses violated the advertised shape. Retained structural quarantine identifies unsupported `relationship` in request `c23c64ee5e2f9cac3cc6fe9e7b104cbe6f43492cb533bf80f5089af268142657`, then unsupported `relational` in request `a70f95167a6a2154144e1854faed8e41096bc78495f6036f9856904ebdb17bd6`; each also has an additional hashed field whose original name is not retained. Do not guess that field's name or unretained prose. Two counted full-response corrections then yielded the valid experiment. The later boundary-patch response failed `invalid_rationale_length`: structural quarantine records a590-byte rationale against the512-byte limit. The shared two-correction allowance was already exhausted, so the controller correctly stopped without another call or patch execution. The rejected205-byte replacement body is not retained; neither patch correctness nor patch incorrectness can be inferred from its hash/length.

All six generator calls were HTTP200 terminal `stop`, including the three invalid responses; this is known protocol failure, not transport uncertainty or truncated generation. Accounting supplied by the parent: **nine calls (six generator/three reviewer),48,064 tokens, two execution groups,$0.010116**, all settled. Fresh-process terminal replay independently inspected at `target/heldout-unicode-1-replay.json` records zero provider calls/target executions, all130 run files unchanged and campaign ledger unchanged. No failure was erased or reclassified as an accepted repair.

## original parser regression — scoped repair independently accepted

Run: `target/heldout-parser-regression-1`. This is the original task, **not a third held-out task**. All four actual execution groups completed in the normal controller: seed failure, generated experiment failure, generated repair/protected pass, and byte-identical experiment re-verification pass. All six runtime reviews accepted their exact subjects. No protocol correction, intermediate operator steering, replacement code or linked continuation was needed.

The generated experiment substantively checks signed parsing, malformed internal/leading/trailing empty fields, whitespace and i64 overflow, plus exactly-empty success. It is correct and nonvacuous. The generated repair adds `input.is_empty()` returning `Ok(Vec::new())`, then preserves the original comma split/map/i64 parse/error mapping/collect expression modulo formatting. For every nonempty input the branch is false, so this source-equivalence argument supports preservation beyond the sampled tests. Protected Cargo files/tests were directly compared to frozen originals and unchanged; the generated supplemental harness was directly compared byte-for-byte before and after repair.

Protected passing receipt: `d768d198c8a33e382a3ae255c7db4935ebefce38a37241a5e1c6d830b354cf7b`.
Supplemental passing receipt: `bcdd47b89785f80b22b75151c75978206726707a22aae9d2853af6b4fd2bf3b8`.
Repaired snapshot: `a241bf455a544034864f74a1aa304abf22926b39f12ebe46a5b3980007537ffc`.
Requirements revision: `4d8bb056c186f86bddab6eec266d679e08702b17dbbec8aac1f8411feea148ab`.

The retained reflections correctly identify initial/supplemental pass-versus-fail case outcomes, then all four current passing outcomes after repair, while separating source predictions and unknown assertion execution/returned values. No observation inversion or unsupported universal test claim was identified. Relational content remains largely boundary-sensitive conjunction; this result does not independently establish non-additive culmination or causal advantage.

After the normal controller reached the independent final gate, this reviewer wrote only the exact source/root/subject-bound FileReviewer decision `target/heldout-parser-regression-1-reviews/c58c2831dc84e8d7d0da880a124544fb5b50bc3cc0e6782fcc29a26347aa23f7.review.json`, hash `0d10a70389a9d616147867535ca8081dbda5a34184e7df3bb937fe4a54a8265a`. The decision is **accept** for the two original repair criteria. Root `7151d6cb5c9d998b9eb719511d8f07d0773debfb077ef7db7e912036b5b5f72c`, snapshot and receipt were checked against current native read-only inspection before writing. This is explicit independent final assessment assistance, not fully automatic formal acceptance or a human-time measurement. No provider/target effect or acceptance adoption was performed by this reviewer. Final driver adoption/clean-stop/replay evidence is joined below.

Parent reconciled accounting: **11 calls,66,210 tokens,four execution groups,$0.012105**, settled; campaign165calls/50groups/$0.142733, no campaign unknowns. The three preregistered attempts were not replaced or retried after terminal failure.


## Final verification and denominators

Read-only inspection of `target/heldout-parser-regression-1-finalization.json` and `-replay.json` confirms native decision `b104956a2050a1476f776d1e7c902e505d52b1fe3cf46e08b51d1c5933f97da8`, scoped desire status fulfilled, semantic status accepted_by_operator, optional proposal skipped, and completed_operator_reviewed. Both finalization and fresh replay used zero provider calls and zero target executions; source/campaign ledger were unchanged, and replay preserved all200 run files. This is completed normal-path operation with separate final independent assessment, not automatic formal self-acceptance. The conservative `live_b91_passed:false` implementation flag was not rewritten; the result is supported by the actual evidence above.

The clamp replay separately preserves its terminal runtime refusal,200 run files and ledger, with zero effects. Unicode similarly preserves its terminal exhaustion and130 files. No terminal failure was reopened or omitted.

| Case class | Task | Generated experiment | Real repair and re-verification | Full preregistered cycle |
|---|---|---|---|---|
| Fresh | singleton-clamp | Meaningful; actual failure | Passed, source independently correct | Failed: runtime final rejection; earlier false case claim also missed |
| Fresh | unicode-prefix | Meaningful; actual failure | Not run | Failed: bounded protocol corrections exhausted |
| Regression | Original parser | Meaningful; actual failure | Passed, independently accepted | Passed, clean stop/replay; separate final assessment assistance |

**Fresh full-cycle success0/2; original regression1/1.** These denominators are fixed and separate. Two correct repair outputs were actually verified across all three attempts, but that is not the full-cycle metric. Total evaluation:32calls,10real execution groups,191,702tokens,$0.036562 settled. Final campaign165/200calls,50/60groups,$0.142733/$5, no campaign unknowns; older GardenTrial1 unknown reservation remains separate. Remaining capacity is not evidence of reliability or a reason to relabel these outcomes.

`docs/engine-heldout-repair-results.md` was checked against the retained records for the fresh-case result statements and resource counts. The meaningful finding is mixed: real repair generation and safe stopping/replay work, but protocol robustness and semantic runtime evaluation still block repeatable full-cycle success on these two fresh tasks. One successful regression plus two fresh attempts cannot support a broad reliability estimate or causal TSR advantage. The review did not modify criteria, invent new acceptance gates, or rewrite historical outcomes.
