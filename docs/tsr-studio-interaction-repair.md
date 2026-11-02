# TSR Studio interaction repair — 2026-10-30

## Result

Conversation is the primary view. Send preserves an idempotent message, prepares a source-bound session asynchronously when allowed, and displays actual recorded model responses with their frozen input IDs. Work/files and TSR context are secondary. Questions may receive a justified no-action answer; proposals are not labeled executions. Drafts and pending submission IDs survive reload and network failure.

Preparation no longer holds the global/project input lock throughout expensive native work. Final metadata and native feedback receipts commit under the project lock. A durable scheduled attempt precedes worker launch. Stop is checked before preparation and atomically around launch, so cancellation does not race into a new process. Stop retains desires and unresolved effects. Messages arriving during preparation are queued and excluded from that response's input attribution.

## Verification

- 21 authored HTTP/native integration tests passed in 81.598 seconds (`target/ui-interaction-race-tests.log`). These cover create/select, blocked Send, refusal, forged approval rejection, authored approved state, one native response, duplicate submission/reload, later-input separation, preparation concurrency, Stop before worker and during preparation, stale/history/budget behavior.
- Independent source review passed; two discovered launch/commit races were fixed before the conclusive run.
- Python compilation, JavaScript syntax checks and `git diff --check` passed.
- Running service restarted through its existing current-user job. Bootstrap and both actual project API views load; Jungle Wing has one message and Pond has two. Neither is running. No browser refresh was performed.
- New rendered interaction/screenshots are **not verified**: the Mac was locked and automatic unlock failed. The existing Docker daemon was unavailable. Browser fixture selectors were updated and syntax-checked, not executed.
- Fixtures are explicitly labeled and isolated. No real provider call or game execution occurred.

## Financial blocker

The UI shows one recovery disclosure, not an inactive-contributor error masquerading as a failed send. No acceptance endpoint exists: Host/Origin/CSRF is insufficient to authenticate the human against arbitrary local clients. Approval must arrive through the existing authenticated conversation; the UI can record refusal without discarding input.

Required disclosure: permanently abandon the interrupted tool-free output, retain its uncertain historical charge ($0.05 is an estimate, not a verified maximum), and permit fresh calls only within approved remaining budget. Unknown tool/execution effects remain blocked. No approval, quarantine, financial waiver or fresh paid admission was applied. A reviewed financial disposition/fresh admission remains required after authentic approval; this report does not claim that merely accepting text already unlocks paid recovery.

Accounting verified unchanged: 347 calls, 101 operations, $0.403253 known, $0.453253 committed, one unknown call and zero unknown executions. Pond's original messages and games remain preserved. Its gorgeous redesign is queued.
