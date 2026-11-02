# Trial 11: independent candidate acceptance and continued core work

2026-10-25, linked local review after `47a91fb`. Zero provider calls and zero
target executions; no final experiment adoption. The two original runs remain
historically stopped/advisory, not relabelled as an uninterrupted accepted episode.

## Independent judgment and native adoption

A separate reviewer inspected the frozen requirements, original/current source,
protected receipt and unchanged supplemental experiment. Both requirements are met:

- Exactly empty input returns `Ok(Vec::new())` in the new explicit branch.
- Every nonempty input bypasses that branch and runs the original
  split/parse/map-error/collect expression, unchanged except formatting. This source
  equivalence, rather than finite example counts, supports preservation.

The protected receipt records both checks passed on snapshot
`cf21151fc9de0c2f452474c9bca71b38a41c2414383a65652ec7162c8cc3f1e1`.
Supplemental examples passed; their universal requirement findings remain unknown.
The optional final experiment was neither needed nor executed.

An exact copy of the continuation project was created at
`target/b91-trial11-independent-acceptance/project`. Its sibling `provenance.json`
contains original inventories. The reviewed decision was recorded through existing
`project acceptance review --file` machinery, after confirming source bindings.
Decision ID: `8f054fd07c2fd57896a1bf0dd718d6fccf9a92d01440efa2d7d8c2079bc53ddf`.
Native terminology `accepted_by_operator` denotes the explicitly authorized
independent agent judgment adopted through that API; it does not assert a human
performed the review or measure human effort.

## Observed effects and restart

- Native candidate status: `accepted`; both original semantic desires: `fulfilled`.
- One reviewed candidate is retained. The acceptance-rate proxy changes from 0.5
  to 2/3; this is a history fold, not calibrated confidence.
- The matching frozen acceptance forecast resolves once, residual -0.5; dimensional
  Contrast has no unmet original criteria. Surprise and Contrast remain separate.
- Fresh-process inspection returns exactly the same acceptance/cycle state, also
  after rebuilding the native binary with the subsequent core correction.
- Closed inherited admission still hides active per-goal belief/guidance, while
  preserving reviewed evidence. No new execution authority was created.
- Next-attention reports no eligible residual goal, with closed/stopped admission
  blockers intact. Test-derived desires remain in history; satisfaction is scoped.
- Historical source/database/receipt/STOP/closure files retain their original hashes.

Read-only replay (no network or target execution):

```sh
target/debug/tsr engine --directory target/b91-trial11-independent-acceptance/project project acceptance inspect
target/debug/tsr engine --directory target/b91-trial11-independent-acceptance/project project attention
```

Evidence: `target/b91-trial11-independent-acceptance/{provenance,review,adoption,
verification,after-inspect,restart-inspect,attention,source-reconciliation}.json`.
No new costs; recorded settled subtotal remains $0.726803, older Garden Trial 1
unknown reservation remains separate. No maturity, causal superiority or general
reliability claim follows from this accepted synthetic repair.

## Execution caveats

The executor briefly disconnected after copy/draft creation; it recovered before
adoption. The independent reviewer used SQLite `mode=ro` on the original shadow
store, which created a zero-byte WAL and 32 KiB SHM. These read sidecars are recorded
in `source-reconciliation.json`; original files and database hashes did not change.
They were not silently removed. Further original database reads use immutable mode
or copies. A strict inventory precheck stopped on these additions, leaving the blank
draft unchanged; a subsequent native submission rejected that blank draft without
recording a decision. Only the reviewed, fully populated submission was accepted.

## Next core investigation

A source audit reproduced a relational-update defect: native semantic reviews
could change desire status/Contrast while leaving the relational input hash unchanged
when source and tool receipts stay fixed. That would prevent return-driven hypothesis
revision despite new independent evidence. The regression failed before the fix and passed afterward. The narrow correction
binds independent semantic returns without exporting private review prose. See
[mechanism and verification](engine-semantic-return-relational-binding.md). This
is return-driven qualitative guidance, not a live generation-quality claim.
