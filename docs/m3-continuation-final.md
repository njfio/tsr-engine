# Approved three-arm continuation: terminal result

The approved continuation launched **2026-10-16 23:32:20.672318 UTC** from reviewed commit `8366a8ac2e56ba43dc94e229d535b71bb0a33c39` and readiness SHA256 `6a20b6bb4e94ac2ec31e5ec46cd60c865e57326662f77ab6b4e5900b2fbb9241`. It ran two arms serially and **halted after a protected-test integrity rejection**. The third arm was never started. No retries, expanded cohort, deletion, push or publication occurred.

| Task / mode | Raw outcome | Paid requests | Settled cost | Final oracle |
|---|---|---:|---:|---|
| ProtocolError::to_wire / rules | unavailable: multiple tool calls | 7 | $0.006518 | not invoked |
| Route::to_wire / record | rejected: changed protected test evidence | 16 | $0.008595 | not invoked |
| Route::to_wire / rules | not attempted | 0 | $0 | not invoked |

**23 reservations and 23 settlements. Incremental: $0.015113. Aggregate: $0.229582. Outstanding: $0.** The $0.214469 historical ledger prefix is unchanged. Costs are the provider's settled billing receipts; no ambiguous charges remain.

## What happened

ProtocolError/rules ran from **23:32:21.174733 to 23:38:41.667820 UTC**. Six committed actions were inspect, visible test, inspect, patch, patch, inspect. The visible test exited 101 after 192587 ms. Its seventh paid response contained multiple tool calls with `finish_reason=tool_calls`, no refusal indicator, and no legacy/top-level alternate envelope. The strict adapter requires one call and returned `tool_shape`. Content-free metadata establishes the multiple-call category, not the precise count or contents. The arm remained unavailable; all charges settled and the controller verified/exported its evidence before advancing. No Stop or final oracle occurred for this arm. This new evidence does not reconstruct the earlier run's unknown sixth response.

Route/record ran from **23:38:42.226400 to 23:45:22.718213 UTC**. It made 16 valid proposals, including three patches and a final **Stop** action. Its two visible tests exited 101 after 177120 ms and 2276 ms. On submission, the evaluator closed the task with:

`tamper:changed full-suite test evidence crates/rts-mcp/src/roots.rs`

The candidate rewrote the allowed source file and removed protected tests; the [preserved source diff](m3-continuation-route-source.diff) corroborates the guard. No claim is made about the model's intent. The evaluator returned raw `rejected`, with `oracle:null` and no final-oracle receipt events. This is an integrity rejection before final-oracle execution, not a negative oracle test result.

The controller then raised `non-oracle rejection or closure mismatch` under the reviewed global halt rule. It did not reinterpret this rejection as an eligible format failure. Route/rules has no intent or process receipt and no output state. The exact remaining unperformed action is that single provider-backed arm; it is withheld by the approved halt policy, not by an approval-tool denial or billing uncertainty.

## Evidence, receipts and paired roots

The normal controller export for ProtocolError/rules is verified. Route/record's export was intentionally interrupted by its guard, so its partial export and raw outcome remain explicitly labelled partial in [the original terminal summary](m3-continuation-result.json). No decision or manifest was retroactively inserted into that controller state.

A separate [forensic audit](m3-continuation-forensic-audit.json) checks Route/record's copied result, SQLite integrity, event-ID-derived roots, CAS, closure, Stop, lack of oracle receipt events, and exact reservation/diagnostic sequence within its process interval. It reconciles all 16 charges. The initial summary's `unbound_charges` means not bound to a completed controller-export decision; it does **not** mean unsettled billing. The supplemental audit resolves their arm attribution without making the rejection eligible for continuation.

Both audits and independent terminal review matched **106 exported files**, **225 frozen inputs**, and the exact **68-file prior export**. The primary audit checked 29 CAS objects; the supplemental audit checked 35 additional/copied CAS objects, including duplicate copies, so this is not a unique-object total. Stored-root checks hash event IDs in order; they do not independently recompute every canonical event payload ID. No candidate code was executed during these audits.

All observed starts are the empty root:
`af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262`

ProtocolError's prior record arm and this rules arm have matching actual starting roots, but both are unavailable. Route/record's starting root is independently checked by the supplemental audit; Route/rules has no observed root. **There are zero completed pairs and no record-versus-rules performance result.** The initial summary intentionally omits partially exported Route roots from its verified-root comparisons; the supplemental audit supplies the checked record root.

Terminal roots:

- ProtocolError/rules: `803982e68a6ee310e28e9f58b28afe17dc7f300d016e35283b9a0a47090b3726`
- Route/record: `535769fb54060cec9875aa4adf1f8c9aa57ad8b8ac7a151c429567dc66c19497`

No final-oracle receipts were generated by either arm. There are zero verified acceptances. M3 operational acceptance remains unmet; the pilot does not supply representative paired outcomes.

## Verification and preservation

Fresh live preflight verified the reviewed commit, exact readiness hash, 225 frozen files, fresh arm state, fully settled baseline, pinned Docker image, idle runtime and disk reserve. A read-only catalog check confirmed $0.15/$0.60 per million input/output tokens within existing bounds. An initial preflight wrapper import-path error occurred before any inference; the corrected read-only wrapper passed. The frozen controller was not edited.

The reviewed implementation already passed **16 Python lifecycle/integration tests and 13 Rust evaluator tests**. During this run, **3 report regression tests** passed, covering process/closure/outcome validation, partial exports, intent-without-process status, and observed-root comparisons. Independent reporting review found and resolved reporting issues before terminal export. Independent terminal review found no material discrepancy.

At **23:48:33.489672 UTC**, runtime checks confirmed controller PID40792 and worker PIDs40802/49488 and their process groups were gone, no M3 containers/workers remained, and the historical coordinator retained its expected stopped identity. Free disk was 298.705 GiB. Existing artifacts and unrelated user files were preserved. Lefthook was unavailable; no hook pass is claimed.

Branch/workspace: `codex/m3-evidence-pipeline`, `/Users/n/Documents/Codex/2026-10-14/task/tsr-engine-m3`.

Key evidence: [launch](m3-continuation-launch.json), [authorization](m3-continuation-authorization-provenance.json), [export hashes](m3-continuation-export-sha256.json), [terminal summary](m3-continuation-result.json), [supplemental forensic audit](m3-continuation-forensic-audit.json), [runtime check](m3-continuation-terminal-runtime.json), and [independent terminal review](m3-continuation-terminal-review.json).
