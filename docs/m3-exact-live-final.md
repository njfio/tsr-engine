# Exact-edit live pilot: terminal halt

The approved one-shot controller launched **2026-10-17 04:56:08.477305 UTC**, PID **50904**, from commit `c0e302a651c371b5092dced53e786d82a8ec875d`, using freeze SHA256 `bdd85d2a556b3154da57aeb1f5c92af70ee5fd9a5a877de457f8a88edc10545d`. It halted after the first record-mode response failed proposal validation. No retry or rules-mode attempt occurred.

| Arm | Terminal outcome | Settled requests | Cost | Final oracle |
| --- | --- | ---: | ---: | --- |
| ProtocolError / record | unavailable: `proposal_type` | 1 | $0.000162 | not invoked |
| ProtocolError / rules | not attempted | 0 | $0 | not invoked |

**Incremental cost: $0.000162. Aggregate: $0.229744. Outstanding: $0.** The $0.229582 historical ledger prefix is byte-identical. The single $2 reservation has one exact settled billing receipt. The receipt reports 981 prompt tokens and 74 completion tokens (1,055 total). The engine result's `generator_tokens: 0` reflects failure before a validated completion reached the driver; it is not evidence of zero provider usage or zero cost.

## Failure and scope

Record worker PID **50915** ran from **04:56:09.114922 to 04:58:29.325574 UTC** and exited 0 with no process errors. Most of that interval preceded the first inference during workspace preparation. The model reservation was recorded at approximately **04:58:22.600 UTC**, settlement at **04:58:25.847 UTC**.

Content-free diagnostics establish one choice, one function call named `propose`, valid JSON arguments with object type, `finish_reason=tool_calls`, and no refusal indicator. The strict decoder returned `proposal_type`. This code can cover invalid field types, an unsupported action, or populated irrelevant fields. The response body was not retained, so the exact offending field cannot be determined. This was not a multiple-call failure, and no more specific cause or model intent is inferred.

No proposal was accepted into the engine log: **zero actions, zero edits, zero visible tests, no Stop, and zero final-oracle receipt events**. The task closed as:

`unavailable:broker failed; see controller cost ledger for reserved charges`

The controller preserved its base export, then raised `explicit submission missing` under the reviewed policy. It wrote `halted-review-required` and no completed arm decision. This is a protocol-availability failure before grading, not a failed final test or evidence that the repair succeeded. The exact-edit operation was never exercised by this live attempt.

Rules mode has neither an intent nor a process receipt and remains **not attempted**. Its authorized action was withheld by the reviewed fail-stop policy, not by an approval denial. There is no uncertain billing to reconcile and no further calls are permitted under this halted run.

## Paired roots and audit

Only record mode has an observed start root:

`af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262`

Its terminal root is:

`238c2582eedfcb287ac869b66caa5b5c21af4c69bd5fe04e7564a0c3fb2a11bb`

Rules has no observed root. **There is no eligible complete pair, no acceptance, and no record-versus-rules performance result. M3 operational acceptance remains unmet.**

The terminal-only audit:

- Reverified all **535 frozen inputs** and all **106 prior exported files**.
- Preserved **91 files** in the new evidence export, including the halted controller state and a separate forensic snapshot. No controller decision was retroactively created.
- Verified SQLite integrity, task/result identity, unavailable closure, event-derived roots, **29 CAS objects**, and the absence of final-oracle evidence.
- Bound the single reservation, settlement and diagnostic to the worker's exact process interval and retained the original settled cost.
- Confirmed controller PID50904, worker PID50915 and their process groups were gone, with no active M3 workers/containers, at **04:59:49.165405 UTC**. Free disk was **293.797 GiB**.

Root checks recompute the ordered event-ID root; they do not independently recompute every canonical payload ID. The forensic audit validates the unavailable outcome but does not turn the missing controller decision into a completed experimental arm. Partial exports remain labelled as such.

Three offline report regression tests passed, covering false-acceptance checks, process/closure/denial consistency, raw-to-export binding, explicit Stop, proposal/call counts and decision identity. Independent review found reporting gaps during development; these were fixed before export. No frozen execution code was edited during the run.

## Authorization and preservation

The exact freeze and commit were checked before launch. Fresh preflight confirmed isolated empty stores, the pinned interpreter/image, idle runtime, disk reserve, fully settled baseline and current model pricing of **$0.15 input / $0.60 output per million tokens**, within the frozen guard. Approval provenance records the user's **2026-10-17 04:54:44 UTC** approval of the two-attempt scope, 20 requests/1,800 seconds per arm, $5 additional within $100, and zero retries.

No further cohort, deletion, push, merge or publication occurred. Existing user files remain untouched. Runtime and catalog preflight and the actual launch used the authorized host execution path; no approval review rejected an action.

Branch/workspace: `codex/m3-evidence-pipeline`, `/Users/n/Documents/Codex/2026-10-14/task/tsr-engine-m3`.

Evidence: [launch](m3-exact-live-launch.json), [approval provenance](m3-exact-live-approval-provenance.json), [fresh preflight](m3-exact-live-launch-preflight.json), [audited result](m3-exact-live-result.json), [export hashes](m3-exact-live-export-sha256.json), [runtime receipt](m3-exact-live-terminal-runtime.json), and [independent terminal review](m3-exact-live-terminal-review.json).
