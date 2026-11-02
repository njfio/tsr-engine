# Fixed12 terminal comparison — 18 October 2026

The frozen comparison halted on the 200 GiB live disk guard at 00:35:14 UTC. It produced 22 classified arms covering 11 complete pairs. The final pair's rules arm was interrupted during preparation before any paid request; its record arm was never launched. The original halt remains `halted-review-required` / `process/containment failure`, with the worker's explicit live-disk-guard error retained.

Independent oracle verification confirms **record 2/12 assigned tasks, rules 0/12**. This is an incomplete exploratory result, not evidence of superiority and not complete M3 acceptance.

## Outcomes

| Pair | Source file | Record | Rules |
|---|---|---|---|
| 1 | `crates/agentid-core/src/policy.rs` | accepted | rejected |
| 2 | `crates/rts-daemon/src/reconciler.rs` | rejected | stale-digest-or-oversize |
| 3 | `crates/agentid-core/src/validation.rs` | accepted | stale-digest-or-oversize |
| 4 | `crates/rts-daemon/src/symbol_pagerank.rs` | schema-or-response-failure | schema-or-response-failure |
| 5 | `crates/agentid-core/src/events.rs` | schema-or-response-failure | schema-or-response-failure |
| 6 | `crates/rts-mcp/src/cli.rs` | schema-or-response-failure | schema-or-response-failure |
| 7 | `crates/agentid-core/src/audit.rs` | schema-or-response-failure | schema-or-response-failure |
| 8 | `crates/rts-core/src/signature.rs` | budget-unavailable | budget-unavailable |
| 9 | `crates/agentid-core/src/vault.rs` | schema-or-response-failure | schema-or-response-failure |
| 10 | `crates/rts-daemon/src/protocol.rs` | stale-digest-or-oversize | stale-digest-or-oversize |
| 11 | `crates/agentid-core/src/identity.rs` | stale-digest-or-oversize | rejected |
| 12 | `crates/rts-mcp/src/roots.rs` | unattempted | infrastructure-interrupted |

The two accepted record arms are AgentID policy and validation. The reconciler record arm passed its protected named tests but regressed the full suite, so the independent oracle correctly rejected it. Passing a visible test never counted as acceptance. Ten arms had schema/response failures (nine truncated responses, one tool-shape failure), five had edit-precondition failures, two hit the conservative token-reservation bound, and two were policy-blocked. These failures were retained without retries or substitutions.

Across the 11 fully classified pairs: two record-only successes, zero rules-only successes, zero both-successes and nine neither-successes. The preregistered assigned-task operational difference, rules minus record, is -16.7 percentage points. Record's Wilson 95% interval is 4.7–44.8%; rules' is 0–24.2%. Missing-outcome bounds are record 16.7–25.0% and rules 0–8.3%. The interrupted and unlaunched outcomes are not assumed oracle failures. No McNemar calculation was performed because the preregistered all-24-terminal condition was unmet. These intervals do not establish independent-task generalization across related functions/tests and only two lineages.

## Accounting and limits

| Mode | Settled calls | Additional USD | Completed worker seconds | Broker seconds |
|---|---:|---:|---:|---:|
| record | 58 | 0.171281 | 4366.185 | 1133.812 |
| rules | 48 | 0.133330 | 3435.267 | 729.673 |

**106 calls, $0.304611 additional, $0.544194 cumulative, zero outstanding reservations.** Every reservation ID has one settlement, all completed-arm totals match the shared ledger, and the original $0.239583 baseline prefix is unchanged. The interrupted arm incurred no calls or charges. Worker duration above covers classified arms; the interrupted preparation took about 40.1 seconds. There were no wall-time timeouts; the largest classified arm used eight calls and 660.9 seconds, below the 20-call/1,800-second bounds.

The controller launched 23 of 24 assignments, with 22 classified decisions. Remaining work is the final pair: rules interrupted, record unlaunched, both without an oracle outcome. The independent audit measured about **199.949 GiB free**, approximately 0.051 GiB below the 200 GiB floor. This threshold deficit is not an estimate of capacity needed to finish two fresh builds. The full guard has not passed, and this one-shot run must not be resumed or silently retried. Any future attempt needs a separately reviewed plan and sufficient authorized capacity. No files were deleted to reclaim space.

The worker initially recorded an owned container still present during cleanup. Later read-only checks found the exact container absent and no experiment processes or M3 containers running. Both observations remain preserved. Unrelated containers were untouched. No manual stop/delete operation was performed.

## Evidence and validation

- [All 24 assignment records, paired statistics and accounting](m3-fixed12-result.json).
- [Independent outcome audit](m3-fixed12-independent-outcome-audit.md) and [detailed audit evidence](m3-fixed12-independent-outcome-audit.json).
- [Durable execution export](m3-fixed12-execution/terminal.json) and [export SHA256 manifest](m3-fixed12-export-sha256.json).
- [Frozen analysis plan](m3-fixed12-analysis-plan.md), [launch checks](m3-fixed12-launch-check.json) and [freeze](m3-fixed12-freeze.json).

The independent audit reread protected oracle receipts for all three final-oracle runs, verified all 22 completed evidence manifests, SQLite integrity and diagnostic/reservation bindings, and checked all 5,362 frozen file pins. The post-run exporter independently reconciled charge IDs, per-arm totals, baseline bytes, assignments and pins. No candidate or inference was rerun during the audit. Pre-launch validation is preserved in the final review: 36 native offline cases re-exported through v6, a native zero-response budget case, and nine final unit tests. Those fixture checks are separate from the live real-repository outcomes.

Frozen implementation commit: `4ee7c7489dab6b87223c1825ad3e8ff3d20839be`; manifest SHA256: `7443c9ee69b4f5d7f8a1276826bd59f8915c083d5b42355710f5f4c6fff75e18`. No harness changes were made during execution. Only terminal reporting/export code and new evidence were added afterward.

## M3 assessment

The ingestion, qualification, sealed evaluation, policy comparison and independent-oracle machinery now have real-repository operational evidence, including two verified generated repairs. The inventory contains 151 admissions with family/lineage, while the frozen complete-run sampling pool contained 149; two admissions from an incomplete run were excluded. This experiment selected 12 constructed mutations in two real repositories, not historical human bug fixes or an untouched holdout. The selected roots task's prior paid exposure was disclosed before launch.

**M3 is not marked accepted.** This bounded run stopped before completing the fixed comparison, most arms never reached a final oracle, and the small, correlated, exposed sample does not establish broad performance. The existing selection/tension implementation is outside this run's audit; no new claim of full end-to-end milestone validation is made. The missing pair and operational response/budget failures remain explicit. No private-session mining, new credentials, secret uploads, additional paid attempts, push, PR, merge or deployment occurred.
