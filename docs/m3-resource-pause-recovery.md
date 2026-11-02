# Resource pause and recovery

Final checkpoint, 2026-10-15 08:07 UTC: **65 distinct function groups admitted** (25 AgentID, 40 RTS). Structure finished with eight admissions, three preserved qualification rejections, and two operator cancellations. No qualification processes or M3 task containers remain; outer coordinator 29246 is stopped. Free space is approximately **195.13 GiB**.

All **5,725 exported files** match original evidence; the 65-task native CAS/split audit passed (`m3-admission-audit-resource-pause-final.json`). Previous attempt records are unchanged; 40 frozen expansion artifacts and context-evaluator hashes match. This integrity audit is not a reserved holdout or paired model-performance result. No Rust source changed and no Rust regression suite was rerun in this monitoring turn.

The refreshed cleanup proposal lists **63 directories**, **261,014,994,944 allocated bytes (243.09 GiB)**. The mixed/incomplete historical `m3-qualified-remaining` attempt is excluded conservatively. No deletion, paid calls, push, PR, merge or deployment occurred. Independent read-only review found no blocker in the pause mechanism or dry-run inventory. Exact receipts are in `m3-resource-pause-receipt.json`.

The 200 GiB watcher threshold stops only the outer coordinator, preventing another cohort. The cohort scripts also check 100 GiB before each job. Neither is an OS-enforced hard disk quota: active builds can continue allocating after a check. The 100 GiB value is an operational reserve, not a guaranteed minimum under unrelated host disk use.

The watcher stopped coordinator 29246 at 2026-10-15 07:45:29 UTC. The operator reasserted that stop after verifying its command and start time. To satisfy the instruction to let only already active jobs finish, the operator exclusively reserved the two pending job log paths. `queue.run` opens these paths exclusively before `Popen`, so it cannot launch them. One produces a preserved `FileExistsError` controller receipt and sets HALT; the other is skipped. These are operator resource cancellations, not evaluated task failures or negatives. No frozen controller, candidate input, or oracle was altered.

## Cleanup scope and approval

The exact proposed paths, inode/device identities, allocated sizes, and evidence counts are in `m3-terminal-build-cleanup-v3-plan.json`. Only terminal accepted attempts' rebuildable `copy/target` directories are included. Mixed/incomplete attempts are excluded conservatively. CAS, admission reports, run logs, source copies, prepared images and offline dependency caches are retained. APFS shared extents mean actual recovered space may be lower than the sum of allocated counts.

Suggested approval wording:

> I approve deleting only the rebuildable `copy/target` directories enumerated in `docs/m3-terminal-build-cleanup-v3-plan.json` (SHA-256: 6dde275186b872e76bcb95188b2f5b843d5e9c5c3d16c54a395696e9cbe4d00d, 63 targets), after fresh terminal-state, evidence, symlink and inode/device checks. Preserve all source copies, CAS, reports, logs, failed/cancelled attempts, images and offline caches. No paid calls or unrelated deletion.

No deletion has occurred. The old cleanup executor is hard-coded to the previous 30-target inventory and broad-cohort boundary; it must not be run against this new proposal. Any future executor needs separate review, an exact manifest hash, fail-closed identity checks, and a per-path receipt, with no launch running concurrently.

## Recovery after storage approval

1. Keep the outer coordinator stopped. Confirm the current cohort is terminal and no `tsr-m3-*` task containers or qualification processes remain; unrelated containers must remain untouched.
2. Reverify preserved exports, native CAS/split audit, frozen artifact hashes, source revisions and source cleanliness. Keep the cancellation receipts and reserved logs. Do not remove them to simulate an uninterrupted campaign.
3. If cleanup is approved, reverify the exact approved inventory and evidence immediately before deletion, retain per-path results, and measure actual free space afterward. Additional usable storage is an alternative.
4. Rebuild removed artifacts only inside the original prepared, network-disabled container using retained source copies and offline dependencies (`cargo build --locked --offline --workspace -j 1`). Rebuilding is new work and consumes disk again; original qualification evidence must not be overwritten.
5. Reconcile the two operator cancellations separately from three genuine structure qualification rejections. A fresh continuation must use new state/output paths, explicit pending candidates and frozen inputs, two-worker concurrency, active timeout supervision, and a launch pause that can cancel pending futures cleanly. Never retry into an existing evidence directory.
6. The old outer coordinator will stop after collecting the cancelled cohort's terminal errors; merely sending SIGCONT does not continue the campaign. Retire or let it reconcile only after exact process-identity checks at an idle boundary. Do not blindly resume it or reuse its exclusive state directory.
7. Before any new cohort, budget its full bounded allocation above the 100 GiB operational reserve, restore a fresh identity-checked watcher, and verify it is live. A cleanup approval alone does not guarantee enough disk for the entire remaining campaign.

No provider calls are needed for these local operations. M3 remains incomplete until corpus and real paired evaluation acceptance criteria are satisfied.
