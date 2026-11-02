# Next-step generation under persisted revised A1

This fresh admission continued from the actual preceding cycle's final root,
`c912599388805cec2fd5d6b425791fd1b3f7c1c18d5338a6da2190b36fc2db4b`.
It did not reset the archive or invent another return. Criteria were frozen before
calls in `engine-archive-next-step-preregistration.md`.

The new `prepare_followup` projection pins the prior final snapshot by SHA256,
requires equality with the current request, checks its event sequence and original
A1 lineage, preserves the frozen world/parents, and exports only allowlisted synthetic
fields and the exact current generated revision. It never serializes private review
or full history. The caller must explicitly admit that source snapshot; the helper
performs no discovery, credentials, transport or execution. Snapshot hash:
`861efc61c39d110a160b2fd896f816a2f0a8e0b24ed0991c080e48e71504e881`.

## Actual model result

The possibility request included the exact revised A1 text and revision identity.
It generated two distinct admissible next steps: route `access` to `night-reader`,
or route `meaning` to `translator`. Both add a missing reconsideration link and say
that the challenge can then be answered, not that a response already occurred.
No winning operation was supplied or selected by an operator.

The second, independently invoked assessment request exhausted all 8192 completion
tokens as reported reasoning tokens. It returned `finish_reason=length`, with no
complete typed assessment. The runner stopped; it did not retry, choose an operation,
substitute an assessment or claim a model inability response.

| Stage | Input | Completion | Reasoning | USD settled | Seconds |
|---|---:|---:|---:|---:|---:|
| Possibilities | 2648 | 1180 | 1020 | 0.000828 | 8.842 |
| Assessment | 3396 | 8192 | 8192 | 0.004068 | 28.118 |

Total **$0.004896**, two calls, one unused slot expired. Same
`deepseek-v4.1-flash` at CheaperInference, 240-second request limits; no account or
shared-key changes. No external actions, private sources or real people were involved.
No local runtime was started. Earlier attempts and charges remain preserved.

Independent review confirmed relevant proposals under the actual revised guidance,
followed by an assessment-output failure. No action was selected, so there is **no new
observed improvement or relational revision**. This is not evidence of task impossibility,
causal benefit from A1, or social fulfillment. The translator has acknowledgment-only
admitted evidence; routing there would not by itself establish substantive response.

A fresh-process check confirms complete request equality before/after the run. Both
challenges remain registered, unrouted and unresolved. No action-started marker or
third-call marker exists. All metered charges settled; no provider outcome is unknown.
The exact limitation is inability to obtain a completed assessment within this bounded
configuration. Its internal reasoning cause is unknown; no raw reasoning was logged.
Further calls require a fresh admission and a justified configuration/representation
change, not recycling this expired slot or blindly repeating the request.

Evidence: `target/synthetic-archive-next/` admission, exact payloads, started markers,
receipts/body hashes, typed possibilities, terminal closure and `invariants.json`.
The normal generated-action/return/revision cycle demonstrated by the prior run remains
valid evidence; this follow-up does not add a successful subsequent selected behavior.

## Separate cancellation-stub investigation

The earlier aggregate failure was traced to test synchronization. `DispatchIntent`
precedes `/api/chat` transmission. Production cancellation can correctly close a socket
after connection but before headers are written. The shared stub asserted that every
accepted socket must deliver bytes, causing its `n > 0` assertion and thread join to
fail. That record did not demonstrate a production cancellation failure.

The live-cancellation test now waits for the stub to receive the complete chat request.
The stub then holds the response until the local run has finished cancellation. It uses
a bounded release channel; the existing three-second cancellation synchronization bound,
two-second provider admission deadline, EOF assertions, lock exclusion, 8704-token
reservation assertion, adoption rejection and unknown-completion assertion are preserved.
The test also asserts exactly one complete chat request was received. Pre-dispatch and
post-response cancellation remain separate tests. Other stub users retain their behavior.

Independent review found this removes the demonstrated race without weakening coverage;
it does not establish absence of every possible load-related timeout.

## Checks

- Follow-up allowlist/binding and driver Python tests: **90 passed**. Pinned source drift,
  current-state drift, stale guidance and private-field exclusion are covered.
- Offline local verifier transport module, eight parallel test threads: **8 passed**.
- One post-fix aggregate CLI library run: **207 passed, 0 failed, 6 ignored**, 45.11s.
  The previous failed aggregate log remains preserved; this is a new coherent run after
  the synchronization change, not an isolated pass relabeled as an aggregate result.
- CLI all-target Clippy with warnings denied: passed.
- Independent review covered the code changes, preregistration, exact live context and
  actual result. No additional reviewer calls or project executions occurred.

Logs are `target/synthetic-archive-next/{python,cancellation-module,rust-lib,clippy}.log`.
No production cancellation behavior or protected fixture/assertion was changed.
