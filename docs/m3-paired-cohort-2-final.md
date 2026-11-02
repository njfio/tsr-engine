# Repaired four-task comparison: final result

The evaluator repair is validated on the affected real repository task; M3 remains incomplete.
The comparison ended 2026-10-15T03:56:58Z and retains all eight planned arms, including the unstarted one.

| Task prefix | Record | Rules | Recorded reason |
| --- | --- | --- | --- |
|29b4ceb35e05|unavailable|rejected|Record token-reservation guard; rules changed protected test evidence.|
|1604d52d9b76|unavailable|rejected|Record token-reservation guard; rules proposed patching before required inspections/test.|
|223a50b42faf|rejected|rejected|Both stop:budget; no accepted repair.|
|0d00a69acdfe|unavailable|unstarted|Record provider transport timeout; ledger guard prevented rules launch.|

Seven executed arm processes exited 0 with no cleanup errors. The orchestration error reflects
uncertain provider billing, not an eighth executed process failure. One fully observed pair
has both outcomes rejected; no difference or repair-performance claim is supported. Fixed-denominator
observed accepted yield is 0/4 in each mode, including missing/unavailable arms. The predeclared
Wilson interval [0,0.48989] is descriptive and ignores source correlation. Prior pilots and
attempt1 remain separate. No timing comparison is made: known early-result timing limitations persist.

[Raw results and initial accounting](m3-paired-cohort-2-result.json),
[execution export](m3-paired-cohort-2-execution/), and
[frozen plan](m3-paired-cohort-2-plan.json) retain the original records.

## Billing and recovery

The final record request exceeded the 55-second client transport timeout. The sole pending
reservation was mapped to the unique matching model/endpoint request created 0.621 seconds later
in a three-second usage-query window; no concurrent paid worker existed. A read-only query first
returned forwarding, then settled with latency 206279ms and charge $0.003789. No model response was
retrieved, no inference was resent, and the unavailable outcome remains unchanged.

An append-only settled event replaces the $2 reservation with that confirmed charge. Final totals:
**$0.119525 cohort; $0.165094 aggregate; $0 outstanding**,78 settled calls overall. The initial
execution export still shows the then-pending reservation; [final accounting](m3-paired-cohort-2-final-accounting.json)
supersedes dollars only and references the provider evidence digest. The$100 aggregate cap was
never reset. `inference_sent:false` in query receipts describes those read-only billing queries,
not the earlier dispatched inference. Query scope excludes unrelated history and response bodies.

Provider documentation describes bounded request-history queries and notes that server requests
can outlast client timeouts: [usage and billing reference](https://www.cheaperinference.com/docs#usage).
No timeout, prompt, model, budget or experimental policy was changed based on these results.

Original local-only qualification resumed 03:58:52Z after identity/cleanup checks. Workers 66598 and66599
started the next pair; free disk was 287.8GiB, above the 100 GiB floor. The unresolved bill did not need
to block independent local work, and is now fully settled. No paid follow-up was launched.

## Evidence and remaining acceptance gap

The repaired outline admission passed all strict phases under engine 7ee73bc691b8d90d2de8fc9c7615ebaf4a5a6afee5b368303ba387603efaadc5:
baseline 999/0/15; three named failures; full mutated 996/3/15; three restored named passes;
full reversal 999/0/15. [Phase summary](m3-module-2-revalidation-phases.json) and
[receipt](m3-module-2-revalidation.json). Independent review confirmed source/task identity,
478 restored tracked files, and resolving CAS relationships; it did not independently recompute BLAKE3.

Two additional distinct admissions raise the exported count from 42 to 44 (18 AgentID, 26 RTS).
The [split-input audit](m3-split-audit-expansion-44/report.json) passed and preserves existing
partition labels; prior inspection prevents calling this an untouched holdout. Revalidating
outline adds no new task. M3 still requires at least 150 qualified tasks and broader real baseline
evidence; neither local regression tests nor these incomplete exploratory pairs satisfy that criterion.

Independent final accounting review verified exactly one appended settlement, the unchanged original ledger prefix, evidence hash/request/usage/charge, no duplicate entries, and the final totals. The result-export regression passed (1 test).
