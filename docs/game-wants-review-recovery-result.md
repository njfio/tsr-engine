# Ordinary-arm review recovery result

The pair remains incomplete. The TSR arm was not rerun. The ordinary arm's original generated proposal has not executed or received a review verdict.

The reviewed append-only recovery reconciled the first HTTP400 using its exact authenticated failed-zero provider record. A Unicode canonicalization mismatch first stopped reconciliation before mutation; the campaign-specific hash comparison was corrected and independently reviewed. All 15 recovery checks then passed in 157.897 seconds. Commits: `91f20faf`, `5b2bba4b`.

The exact-wire preview passed with zero calls. One fresh review used the same retained proposal, evidence, model and settings. It returned another HTTP400, no candidate, in 1.942 seconds. Its exact authenticated provider lookup confirms zero prompt/output/total tokens and zero charge. The one-shot ticket is exhausted; no further request was sent. No worker remains active and no ordinary game execution occurred.

## Concrete request defect

Both review messages contain no JSON substring while `response_format` is `json_object`. OpenAI documents that JSON mode requires JSON in the message context and rejects its absence: https://developers.openai.com/api/docs/guides/structured-outputs#json-mode . This is a concrete compatibility defect and likely explains both failures, but the retained transport contains only the error-body hash, so the exact CheaperInference error is not proven.

The shared builder now appends an explicit JSON-object instruction when its system prompt lacks JSON, before capacity validation. It does not alter proposal, evidence, verdict schema, model, settings or review standards. Four offline checks passed; independent review approved the captured-defect fix. This is not a universal semantic classifier for prompts mentioning JSON incidentally. No patched request has been dispatched.

## Accounting and next boundary

Campaign: 523 calls, 180 execution groups; known settled cost remains $1.046236. Committed ledger exposure is $1.146236: the historical $0.05 uncertain estimate remains, and the second review's $0.05 reservation has not yet been reconciled in the ledger despite its authenticated zero-charge lookup. There are no unknown execution effects. The historical estimate is not a provider-enforced maximum.

Pair: TSR 11 calls / 19 groups / $0.061578; ordinary 3 calls / 0 groups / $0.002857 known. Pair total 14 calls / 19 groups / $0.064435 known. Both failed reviews are independently confirmed zero-charge; the latter still needs append-only native/campaign reconciliation.

The next step is a separately reviewed source-bound continuation for the corrected-format review of the same proposal, plus exact second-failure accounting reconciliation. The consumed identical-wire ticket cannot authorize changed bytes or another retry, and old admissions must not be silently rebound. This is an unresolved continuation boundary within the existing pair, not a claim that the comparison finished or a request to rerun TSR.

Operator assistance includes the recovery implementation, Unicode boundary correction, accounting reconciliation and one technical retry. No operator supplied game code or a review verdict. The new result cannot establish a TSR advantage.

Retained evidence: `target/game-wants-live-v3/ordinary/rebound-initial/` contains both original and v2 tickets, original proof, both exact provider lookups, all test logs, failed reconciliation record, successful reconciliation and exact-wire check, final failure log, terminal accounting and terminal status. Original incomplete reports and all historical journals remain preserved.
