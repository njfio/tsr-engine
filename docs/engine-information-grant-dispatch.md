# Grant dispatch correction

During the approved corrected batch preflight, the v2 grant was rejected before
any model or target effect. Its strict validator supported v2, but `grant_calls`
did not route that version to it. The prior upgrade and failed preparation remain
in place. No budget was consumed or reset.

The additive dispatcher fix leaves every validator and limit unchanged. Four
boundary tests and 17 campaign extension tests passed. The new authored ledger
test appends the full grant chain, rejects replay and cap drift, preserves counters,
and reopens the ledger. Independent review: `/root/grant_review` approved.

Fresh source pins and separate preparation use the same approved payload and
16-call/three-operation scope. The repaired release is pinned in
`experiments/information-pursuit-release-v2-dispatch/manifest.json`.
This is a local integration repair, not evidence of TSR behavior.
