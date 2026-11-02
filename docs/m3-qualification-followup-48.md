# Qualification checkpoint: 48 admissions

The core cohort completed all 10 candidates with clean controller exits and no cleanup errors. Four newly reconciled strict admissions increase the corpus from 44 to **48 distinct function groups**: 30 RTS and 18 AgentID. The four cases each restore 478 tracked files. Their full baseline/reversal suites both report 999 passed, zero failed, 15 ignored; mutated full suites report five or six failures, and every named red/green phase has the expected result. Exact phase receipts are in `m3-core-final-reconciliation.json`.

All 4,260 exported evidence files were byte-compared with source artifacts. The 48-task split-input audit passed using native CAS verification (`m3-admission-audit-48.json`). Its 30/18 repository split is a diagnostic, not a reserved evaluation holdout or performance result.

The guarded transition occurred at 04:53 UTC: the core controller ended, and the filtering extension began. Both candidates screened successfully and entered strict qualification at 04:59:36 UTC, using two sealed workers. Neither is admitted yet. The original outer coordinator remains intentionally paused while this extension owns the queue. The boundary controller will resume it only after clean completion and its exact identity, idle-worker, disk and fresh-watcher checks. No orchestration fault was observed. The evidence exporter now includes the filtering qualification prefix for subsequent terminal exports; do not export incomplete attempts.

The active qualification engine and the separately frozen context evaluator retain their recorded hashes. No context-v3 evaluation, paid call, deletion or retry of a paid arm was performed. Reconciled spend remains $0.165094 with no outstanding reservation.

## Storage projection

At the calculation checkpoint, free space was 264.17 GiB. The observed gross sizes were about 5.112 GiB per RTS attempt and 1.968 GiB per AgentID attempt. Assuming all candidates qualify and build allocations remain comparable, filtering adds roughly 10.22 GiB, AgentID storage adds 15.74 GiB, and structure adds 66.46 GiB. The 200-GiB outer guard is projected to pause during the structure cohort, which can finish under its own supervision at approximately **171.75 GiB free** and at most **71 admissions**. This preserves headroom above the 100-GiB reserve under these assumptions.

These are planning estimates: screening allocations, APFS sharing, rejection and unrelated disk use can change the stopping point. The guard is not an OS quota. No cleanup is authorized or performed; more storage or approval of a separately verified cleanup inventory will be needed before the full retained campaign fits.

## Candidate headroom

With 48 admitted and 102 remaining candidates including filtering, the old ceiling is 150 and tolerates no rejection. For illustration, a future yield of 90% requires 114 remaining candidates (12 additional), 80% requires 128 (26 additional), and 70% requires 146 (44 additional). These are sensitivity scenarios, not estimates from the purposively selected observed corpus. Additional candidate review is kept separate from admitted task counts and the running frozen queue.

Source review produced **17 additional proposals** in `m3-additional-headroom-shortlist.json`, with exact descriptors and selectors in `m3-additional-headroom-plan.json`. All descriptors match the captured inventory. The conditional ceiling becomes **167**, tolerating up to 17 rejections among 119 remaining candidates if all are eligible. Reaching 150 then requires at least 85.7% future yield; the illustrative 80% scenario still needs nine more candidates. Fifteen proposals need a separately validated compatible qualifier (nested modules and/or ancestor attributes); two direct MCP routing functions still need ordinary sealed screening. They are not added to the active queue, and are not admissions. Weak-test and correlated-case exclusions are documented rather than padding the count.
