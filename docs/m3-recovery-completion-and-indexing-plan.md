# Recovery complete; indexing prepared, not launched

The finite recovery cohort finished **2026-10-15 12:06:45 UTC**: **12 completed, 0 active, 0 queued**. Ten candidates passed strict qualification, and two were rejected because the full mutated suite timed out (exit 124). The two rejected IDs start `ab2ad4ce6da6` and `f288451065cf`. Their evidence and all earlier operator cancellation records remain intact; neither timeout was retried.

Total verified corpus: **75 distinct function groups** (25 AgentID, 50 RTS). The completed cohort contributed the two previously cancelled structure cases and eight extraction cases. These are local real-source qualification outcomes, not model-performance results. M3 acceptance is still incomplete.

## Final verification

- Final native CAS/split audit passed at `target/m3-recovery-monitor-audit-17`; report preserved as `m3-resource-recovery-final-audit.json`.
- **6,888 exported files** were independently byte-compared with source evidence after terminal completion. The audit summary hash matches the current admission summary.
- Prior attempt records match the pre-recovery checkpoint; **73 recovery frozen hashes** still match.
- At 12:07 UTC no qualification worker or `tsr-m3-*` container remained. Controller 45475 and observer 64101 had exited. The original outer coordinator 29246 remains stopped.
- Free space was approximately **375.18 GiB**. The cohort ended normally, without a resource pause or orchestration failure. The guard's final saved monitoring heartbeat is historical after normal controller exit, not a live watcher for future cohorts.

No paid calls, further cleanup, push, merge or deployment occurred.

## Next eligible local cohort

RTS indexing is prepared in `m3-indexing-continuation-freeze.json` and `../scripts/m3-indexing-continuation.py`, using the existing frozen `m3-rts-indexing-cohort-plan.json`. It is **not launched**; its state/output directories do not exist. Explicit `--execute-approved` is required and must not be supplied without launch authorization.

The two screening groups propose at most four candidates: `ImpactBounds::clamp`, `collect_defs`, `rescan_and_reconcile`, and `Reconciler::should_throttle`. Capacity is **zero to four additional strict admissions**, at most **79 total** if all pass. Eligibility and outcomes are not guaranteed before screening and qualification.

The new controller resolves the old predecessor mismatch by requiring the actual recovery terminal record and verified 75-admission audit. It uses fresh state and output paths, the unchanged sealed qualification engine, at most two workers, unchanged 600-second command/2700-second worker limits, and a live 200 GiB launch guard. Pending launches stop cleanly while active workers retain timeout supervision.

The prior indexing plan projected 24 GiB for build copies. The prepared continuation reserves **40 GiB** including screening and overhead, requiring **240 GiB free before launch** (200 GiB threshold plus 40 GiB). With roughly 375 GiB free, projected headroom is about 135 GiB above that prelaunch minimum and about 335 GiB after the planning allowance. The 100 GiB operational reserve is not an OS-enforced quota; fresh checks remain mandatory at any later launch.

Preparation checks: Python syntax compilation, `git diff --check`, and **five inert guard regressions passed**, including refusing default invocation without the explicit launch flag, wrong/running old coordinator identity, insufficient disk/dead watcher, and correct failure status. All next-cohort frozen hashes were verified. No Rust source changed and no Rust workspace suite was rerun.

Independent read-only review found no blocker: all **46 next-cohort frozen hashes** match, the predecessor has 17 clean process receipts, and no next-cohort state or qualification outputs exist.
