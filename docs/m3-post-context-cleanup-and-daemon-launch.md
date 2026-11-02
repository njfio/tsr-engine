# Approved cleanup and supplementary daemon launch

The user approved only the frozen 35-directory post-context cleanup manifest d04839063db8d829e28ad27fda1418ae83275322638cabaa932bdaa7415643b4. Fresh dry run verified exact paths, directory identities, terminal receipts, no workers/containers, stopped historical coordinator, 11,950 exports and 20,133 protected source/evidence entries. No additional paths were included.

The first dry run refused a sub-microsecond timestamp equality mismatch before deletion. M3-CLEANUP-01 compares the same UTC ISO serialization used in the manifest, retaining the birth-after-cutoff test. The fail-first regression and fix are recorded; four tests passed and independent review approved the fix. The repeated dry run passed.

Cleanup completed 2026-10-16T13:11:20.872773Z: exactly35 targets removed. Observed reclaim191,963,922,432bytes (178.78GiB). All source/evidence preservation checks passed, including16,765 selected source-copy entries and the broader20,133-entry protected snapshot;11,950exports remain byte-identical. Post-cleanup native integrity audit passed142admissions,exit0. This diagnostic does not establish paired model-performance evidence.

The reviewed supplementary daemon controller launched13:11:58UTC,PID76023; observerPID76478 at13:12:15UTC. Preflight391.10GiB exceeded256GiB minimum. All69frozen inputs matched. Maximum6candidates,2workers,module-v2 hash7ee73bc691b8d90d2de8fc9c7615ebaf4a5a6afee5b368303ba387603efaadc5; unchanged200GiB liveguard and qualification limits. Container inspection confirmed pinnedRTSimage,user65534:65534,networknone,privilegedfalse,CapDropALL. Eight controller tests passed. No new daemon admission is claimed at launch.

Only this approved cleanup executed. No paid calls, historical retries, pushes or weakened criteria. Future cohorts require their own terminal reconciliation and reviewed activation freeze. Corpus remains142until new strict outcomes arrive; at least8additional admissions are required, separately from baseline/paired evaluation evidence. Lefthook was unavailable for commits.
