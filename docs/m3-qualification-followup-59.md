# Qualification checkpoint: 59 admissions

The structure cohort added two strict admissions, bringing the verified corpus to **59** (25 AgentID, 34 RTS). Both accepted cases have full baseline/reversal results 999 passed, 0 failed, 15 ignored; their mutated full suites fail with 11 and 23 tests respectively, with all named red/green checks completed.

Three structure cases remain rejected and preserved: two frozen-qualifier trusted-test-identity failures (`53cb914cfdb7...`, `d903f38fc79c...`), and `16b38f491cf1...`, whose mutated full suite returned timeout exit 124 instead of completed exit 101. The timeout reached a terminal controller receipt at 07:03:02 UTC; the queue then launched `2b3bbff63ee9...`. It was not retried and no limit was extended. Qualification censoring and compatibility rejections are not evidence of model repair failure.

Two supervised sealed workers remain active. `f873bff48676...` has passed its named green checks and is undergoing full reversal validation; `2b3bbff63ee9...` began the next qualification. Neither counts as admitted. Outer coordinator 29246 and watcher 27230 retain their exact recorded commands and start times. No orchestration fault or resource pause was observed.

The terminal-only exporter excluded active attempts, retained all previous records, and preserved the new rejection. All exported files were byte-compared against source artifacts; the exact count and timestamp are in `m3-qualification-checkpoint-59.json`. The final 59-task CAS/split-input audit passed. All frozen qualification/extension/context-evaluator hashes remain unchanged. No code or acceptance criteria changed in this checkpoint.

About 221 GiB free remains above both the 200 GiB outer pause threshold and 100 GiB reserve. The resource guard remains active. Given current allocation and pending candidates, the likely pause remains during structure; remaining rejection outcomes and allocations determine whether/when it triggers. At most 67 admissions can be reached by the end of this cohort after the three structure rejections. More storage or explicit approval of a freshly verified terminal-build cleanup manifest is required before any resource-blocked resume; no cleanup is authorized or performed here.

The 17 additional proposals remain unlaunched and unadmitted. No paid calls, new evaluator runs, push or deployment occurred. M3 remains incomplete. Supporting evidence: `m3-structure-first-reconciliation.json`, `m3-qualification-checkpoint-59.json`, `m3-admission-audit-59.json`, and the immutable native admission exports.
