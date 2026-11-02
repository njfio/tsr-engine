# Approved cleanup and qualification resumption

Completed 2026-10-15. The user explicitly approved the exact 63-directory manifest (SHA-256 `6dde275186b872e76bcb95188b2f5b843d5e9c5c3d16c54a395696e9cbe4d00d`) and local qualification resumption.

## Cleanup and preservation

All **63 approved `copy/target` directories** were removed, after fresh manifest, idle-state, accepted-outcome, evidence, no-symlink and device/inode checks. The executor opened ancestors without following symlinks and used FD-relative safe deletion; it retained per-path receipts and refused replay. No other deletion occurred.

All **5,725 exported evidence files** and **21,502 source-copy entries** matched their pre-cleanup contents afterward. CAS, reports, logs, failed/cancelled attempts, images and offline caches remain. The post-cleanup native CAS/split audit passed for **65 admissions**. Source revisions and frozen inputs remained unchanged.

Free space changed from **194.95 GiB** to **438.23 GiB**, an observed increase of **243.28 GiB** (261,217,845,248 bytes). This is the observed filesystem change, which can include unrelated host allocation changes; manifest allocation was 243.09 GiB.

## Confirmed resumption

Fresh controller **45475** uses `target/m3-resource-recovery-state`. Workers **45491** and **45492** were confirmed in separate sealed containers: network `none`, user `65534:65534`, no privileged mode, capabilities `ALL` dropped. The fresh 10-second resource guard was monitoring at 2026-10-15T09:33:04.301317+00:00; its 200 GiB launch threshold cancels pending work without pausing active timeout supervisors. The 100 GiB reserve remains operational, not an OS quota. Launches also require a live recent watcher heartbeat and the exact old outer coordinator to remain stopped.

This finite continuation covers the two previously prelaunch-cancelled structure candidates in new output directories, followed by at most ten extraction candidates. It stops afterward, or earlier on resource/controller failure. It does not automatically launch the rest of the older campaign. Original cancellation receipts and reserved logs are preserved. The original coordinator 29246 remains stopped.

## Verification

- Read-only preflight passed: 63 exact targets, 5,725 evidence files, 21,502 source entries.
- Cleanup completed with all per-path statuses `removed`; post-check confirmed all 63 targets absent and source/evidence equality.
- `python3 scripts/m3-audit-admissions.py --out target/m3-admission-audit-post-cleanup-v3`: passed, 65 admissions.
- Cleanup regression suites: 9 passed; export guards: 3 passed; new recovery guards: 4 passed (**16 total**).
- Recovery review findings received fail-first regressions: a running/wrong-identity old coordinator is refused; nonzero child exit cannot mark the campaign completed. All now pass.
- Python syntax compilation and `git diff --check` passed. No Rust source changed; Rust workspace tests were not rerun.
- Independent reviewer approved the cleanup executor and corrected recovery controller; all **73 frozen hashes** matched before launch.

Evidence: `m3-terminal-build-cleanup-v3-result.json`, `m3-admission-audit-post-cleanup-v3.json`, `m3-resource-recovery-freeze.json`, and `m3-resource-recovery-launch.json`.

No paid calls, push, PR, merge or deployment occurred. M3 acceptance remains incomplete; running candidates are not counted as admissions. Further qualification after this finite continuation needs another reconciled queue checkpoint and sufficient storage.
