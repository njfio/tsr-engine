# Expanded terminal build-output cleanup proposal

Plan only; no deletion approved or executed. This supersedes the narrower 14-directory proposal for review purposes; do not execute both manifests.

The inventory contains exactly 37 directories created after the completed 63-directory cleanup at 2026-10-15T09:32:06.964757+00:00: 12 recovery, 4 indexing, 1 AgentID records, 5 behavior, 9 language, 5 signature qualification build directories, plus the completed offline resolver probe build directory. The two rejected recovery outcomes are retained unchanged; only their rebuildable compiler output is proposed. The rejected AgentID records task has no build directory to reclaim.

Estimated allocated size: 180.70 GiB. Exact manifest SHA256: d79ed06404b4eaf9255ec2db695ed951be0768b515f386b51982350e06be4363. Each target records device/inode/parent identity and terminal evidence hashes. Birth times and completed receipts establish the post-cleanup scope. Inventory excluded 36 older directories and two active metrics build directories. Queued metrics work is excluded even if it later creates a directory; this manifest never expands automatically. Source, oracle evidence, CAS, logs, dependency caches, images and unrelated data remain outside the target list.

Protected snapshot: 17,636 source entries, rechecked with zero changes; 8,878 exported evidence files byte-equal to original evidence. Prepared RTS and AgentID repositories were clean. Snapshot SHA256: 89febb279ceb8ac777d3a3f079ff280ddb9d5bf09fd88ba84cc24d11cf6f9daf. The source snapshot preserves current bytes, including any rejected-attempt mutation; it does not relabel rejected outcomes as valid.

At inventory time free space was 250.84 GiB. Adding estimated allocation gives 431.55 GiB free, or 231.55 GiB above the 200 GiB launch guard. Reserving the full 18 GiB metrics allowance conservatively leaves 213.55 GiB for subsequent batches. Their original planning allowances total 240 GiB, leaving a shortfall of 26.45 GiB before additional overhead. Thus this proposal extends runway but cannot guarantee all remaining original qualification, much less supplementary proposals. APFS shared extents mean actual reclaim can be lower; fresh free-space preflight is mandatory.

Execution requires separate explicit approval, all workers/containers quiescent, terminal reconciliation, unchanged protected snapshots, fresh path/receipt identity checks and descriptor-relative exact-target deletion. No healthy metrics worker was paused. No paid calls, historical retries, push or deployment occurred.

Independent review found no plan-level blocker; review evidence is recorded separately without changing the immutable manifest.
