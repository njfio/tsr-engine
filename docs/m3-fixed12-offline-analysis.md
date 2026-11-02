# Fixed12 bounded offline interpretation

**Practical verdict: the oracle produced trustworthy outcomes, but this comparison is inconclusive about gate quality.** Record achieved two verified repairs and Rules zero. Only three of 22 classified arms reached the final oracle (13.6%; 3/24 assigned, 12.5%). Most attempts stopped at the model/adapter boundary. This measures the operational yield of this frozen integration; it does not establish that the gate harms repair quality. No new inference, candidate execution, source/oracle edits, cleanup, or live-run preparation was performed for this analysis.

## Every complete pair

One-based pair numbers; the original artifact arm indices are zero-based. A passing visible test alone is not final-oracle success.

| Pair / file | Record | Rules |
|---|---|---|
| 1 policy.rs | Final oracle accepted | Driver prerequisite block before first patch |
| 2 reconciler.rs | Protected named tests passed; full suite regressed; oracle rejected | Edit-precondition failure after an earlier edit |
| 3 validation.rs | Final oracle accepted | Edit-precondition failure after an earlier edit |
| 4 symbol_pagerank.rs | Truncated response after visible test passed | Multiple tool calls violated the single-proposal schema |
| 5 events.rs | Truncated response | Truncated response |
| 6 cli.rs | Truncated response after visible test passed | Truncated response after visible test passed |
| 7 audit.rs | Truncated response | Truncated response after visible test passed |
| 8 signature.rs | Per-arm token-reservation bound after visible test passed | Same token-reservation bound after visible test passed |
| 9 vault.rs | Truncated response | Truncated response |
| 10 protocol.rs | Edit-precondition failure | Edit-precondition failure |
| 11 identity.rs | Edit-precondition failure | Driver prerequisite block before first patch |

Pair 12 (roots.rs) is missing: Rules was interrupted by the disk guard during preparation with zero provider calls; record was unlaunched. Its outcomes are not imputed as oracle failures.

| Observed terminal cause | Record | Rules | Total |
|---|---:|---:|---:|
| Final oracle accepted | 2 | 0 | 2 |
| Final oracle rejected | 1 | 0 | 1 |
| Rules driver prerequisite block | 0 | 2 | 2 |
| Edit precondition | 2 | 3 | 5 |
| Truncation (`finish_reason=length`) | 5 | 4 | 9 |
| Tool shape (multiple tool calls) | 0 | 1 | 1 |
| Token reservation bound | 1 | 1 | 2 |
| Infrastructure-interrupted | 0 | 1 | 1 |
| Unlaunched | 1 | 0 | 1 |

There were no call-limit, wall-time, or dollar-cap stops. Nine truncated responses had no tool call and ended with `finish_reason=length`; the one shape failure returned multiple tool calls. The five edit errors are preserved as the native composite `stale or oversized edit source`, not a guessed model intention or a formal Rules denial. Source-size filtering does not establish which check failed in each generated edit. Truncation/shape failures are observed interface failures; their precise upstream cause is not proven.

## What can be attributed to the gate?

Both record-only successes followed **inspect → edit → visible test (exit 0) → inspect → Stop → independent accepted oracle**.

- **Pair 1:** Rules inspected, then proposed an edit before the required failing test. The Rules-specific driver check stopped it before proposing/committing that patch to the native ledger. This is a directly observed gate-induced stop. The successful record trajectory demonstrates that a repair was possible on this task, but it is not the same sampled patch/trajectory; the blocked Rules response's ungated outcome is unknown.
- **Pair 3:** Rules inspected, ran the failing test (exit 101), edited, then attempted a further edit that failed the exact-edit precondition. It did not stop at the Rules prerequisite gate. Record's success versus this failure cannot be assigned to a demonstrated gate denial.
- **Pair 11:** the second observed Rules prerequisite stop occurred, but its record partner also failed an edit precondition. It creates no success discordance.

Thus there are **two observed mode-specific driver gate stops among 11 Rules arms**, but **only one of the two record-only success pairs contains such a stop**. Zero pair-level changes in eventual success are causally identified: there is no observed same-proposal ungated counterfactual, and each arm sampled a separate model trajectory. The driver blocks have empty native `denials` lists; they are not formal commit-denial events. Independent review found empty denial lists in all 17 available native results and zero denial events across all 22 SQLite stores (five edit-precondition exits had no full native result). These counts must not be conflated with proof that either blocked patch would have succeeded or caused an unauthorized effect.

Across the 20 non-success classified arms, the accounting is one failed oracle, two driver blocks, five edit-precondition failures, ten schema/truncation failures and two token stops (20 failures total across 22 classified arms, alongside two successes). Seventeen of 22 classified arms stopped on edit/interface/token issues, before final oracle. These are competing stopping causes, not an estimated causal effect of stochasticity. The observed outcome categories differ in pairs 1,2,3,4,11; only pairs 1 and11 include a driver block, and their counterfactual success remains unknown.

Operational success is record 2/12 versus Rules 0/12, a Rules-minus-record difference of -16.7 percentage points. Missing-outcome bounds remain record 16.7–25.0%, Rules 0–8.3%. The descriptive difference favors record, but low oracle completion, two lineages, related tests, disclosed prior exposure, single sampled trajectories and the missing final pair make gate-quality inference inconclusive. No preregistered all 24-terminal McNemar test is applicable, and no superiority claim is made.

## Pair 8: a token guard, not a financial limit

The frozen profile allowed **200,000 total generator tokens per arm**. Before dispatch the transport required:

`serialized request bytes + 8,192 framing-token allowance + 16,384 maximum output tokens <= remaining_tokens`

This deliberately conservative bound treats bytes as input tokens; it is not the provider's tokenizer or a proof that the next actual response would exhaust the budget. Settled provider usage matches native generator-token counters:

| Mode | Tokens already used | Remaining | Next conservative bound | Excess |
|---|---:|---:|---:|---:|
| Rules | 104,447 | 95,553 | 74,523 + 8,192 + 16,384 = 99,099 | 3,546 |
| Record | 104,011 | 95,989 | 74,543 + 8,192 + 16,384 = 99,119 | 3,130 |

Both checks ran at encode stage before a sixth reservation or inference dispatch, after five paid calls and a passing visible test. `settled:false` on these diagnostic rows means no billed request was made, not an unresolved charge. The separate financial checks were not the stopping cause. Final accounting remains 106 settled calls, $0.304611 incremental, $0.544194 cumulative, zero outstanding.

## M3 acceptance and next action

The PRD M3 row requires at least 150 tasks with family/lineage, baseline numbers with intervals, and logged selection/tension behavior without composition. The inventory has 151 admissions, but only 149 were in the complete-run sampling pool; two admissions came from an incomplete qualification run. A final acceptance review must establish the ≥150-task evidence rollup and selection/tension criteria explicitly, not infer them from this pilot. This report does not rerun qualification or audit the selection/tension implementation.

Real-repository baseline outcomes and intervals now exist, but the fixed comparison is incomplete and dominated by failures before the oracle. The remaining pair, the integration limitations, and the halted resource state need explicit resolution in any later acceptance decision. The PRD's minimum 60 held-out tasks concerns the first learned release; it should not be silently recast as a new M3 requirement. These constructed mutation tasks are not historical human bug fixes or an untouched holdout. M3 remains unaccepted; this is not a claim that the pipeline implementation is wholly missing.

**Recommended next action: a bounded offline adapter-reliability diagnosis using the saved failure envelopes and harmless fixtures.** Focus on single-proposal/Stop formatting, truncated output and conservative token/context accounting while retaining the oracle and budget safeguards. It is the largest observed bottleneck (17/22 classified arms), and should be understood before another paid comparison. Do not tune gate policy from this sample or silently enlarge the frozen budget. This recommendation is not an implementation or live-run authorization.

## Separate read-only reclaimable inventory

[Exact paths and allocated-byte measurements](m3-fixed12-reclaimable-inventory.json) cover only the completed fixed12 candidate copies. Twenty build `target/` directories total **70,793,314,304 allocated bytes (65.931 GiB)**. All 22 completed workspaces total **66.020 GiB**, including those builds; these totals overlap and must not be added. Whole-workspace sizes are contextual, not deletion recommendations, because their sources must remain immutable.

| Pair | Record build GiB | Rules build GiB |
|---|---:|---:|
| 1 | 1.965 | 0.000 |
| 2 | 4.628 | 4.628 |
| 3 | 1.965 | 1.965 |
| 4 | 4.628 | 4.628 |
| 5 | 1.965 | 1.965 |
| 6 | 4.628 | 4.628 |
| 7 | 1.965 | 1.965 |
| 8 | 4.628 | 4.628 |
| 9 | 1.964 | 1.964 |
| 10 | 4.628 | 4.628 |
| 11 | 1.965 | 0.000 |

Pairs 1 and11 Rules never built and have no build directory. These reconstructible build outputs are candidates for separately authorized cleanup, not already reclaimed space. APFS sharing/clones and concurrent filesystem activity can make actual recovered space differ from `du` allocation estimates. The interrupted arm 22, older runs, trusted oracle material, source copies, CAS, receipts, logs, snapshots, shared engine and images are excluded. Nothing was deleted or changed and no next live run was prepared.

Sources: [terminal results](m3-fixed12-result.json), [execution evidence](m3-fixed12-execution/terminal.json), [independent outcome audit](m3-fixed12-independent-outcome-audit.md), frozen broker/profile and PRD B9/M3. Numerical review is recorded separately in `m3-fixed12-offline-analysis-review.md`.
