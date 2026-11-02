# Independent review: narrow B9.1 503 recovery

Reviewer: `/root/core_gap_review`; implementation: `/root/b91_driver`;
accounting checks: `/root/b91_accounting`. No live calls or target execution.

Scope is a separate recovery admission for Trial 15's exact boundary-patch503,
not general automatic retry. Final independent source and runbook review found
no blocker after the following corrections:

- Verify copied complete history, old resource archive, projection manifest and
  imported working-journal prefix on restart; preserve STOP absence as well as
  presence. The failed request remains in full audit and campaign consumption.
- Reuse the existing campaign stopped-ledger and official-price checks before
  effects. New calls and executions use the same shared wrappers as CLI tests.
- Report three new calls/two groups, separate from four historical requests;
  preserve new admission identities and original failed-zero evidence.
- Enforce cumulative historical tool consumption plus the new120000ms ceiling
  within240000ms. Retain original immutable checks and independent reviews.

Two real-controller/test-only-worker recovery tests passed in87.680seconds;
28 affected tests passed and five were skipped; the normal full six-stage
native fixture passed in50.946seconds. The actual Trial15 proof and disposable
clone were checked with transport and worker callbacks forbidden. The parent
whole-tree inventory remained byte-identical. Fixture results are mechanics
verification, not live completion or semantic emergence.

The reviewer checked the proof descriptor's four paths/hashes and runnable
commands, token limits and cost estimate. Remaining empirical blocker is provider
availability and a future live linked recovery under the existing campaign.
