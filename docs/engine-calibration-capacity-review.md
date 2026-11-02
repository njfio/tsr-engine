# Calibration capacity review

[Fresh post-fix live verification](engine-capacity-live-evidence.md) subsequently
completed all supplied operational wants in five local calls without authored answers.

The counterproductive live choice exposed an implementation defect, not merely bad
luck under uncertainty. The notebook adapter knew the mechanical action's coverage
capacity but applied its cap only when capacity was **less than** current coverage.
Equal coverage and partial improvements bypassed the bound. A transferred negative
forecast residual could therefore manufacture impossible progress.

In preserved call 7, a two-note composition could cover only 2/3 of the account, yet
calibration projected 1. In call 9, another two-note composition projected about
0.7386 against current 2/3. Both received positive value without possible original
coverage progress. Refinement conservation and standing-loss accounting were intact;
the input projections supplied to them were wrong.

## Correction

`engine/deliberation.rs::assess` now bounds every calibrated predicate by the capacity
computed from the validated, sealed notebook transition. The shared kernel receives
the same upper bounds. Pessimistic predictions below capacity remain possible; a bound
does not record success or replace the frozen generator forecast. Raw forecasts,
historical choices, measured returns and contrary evidence are unchanged.

The adapter assessment version is now `shared-obligation-kernel@v2`; the shared kernel
is unchanged. Historical v1 turns are not silently presented as v2 same-state learning
audits. No change to authority, refinement weights, protected tests or project review.

This is a generic capacity constraint, not a preference for three notes or a particular
answer. Reinspection, partial inspection, same-coverage replacement and complete
composition all obey their own independently computed bounds. Other adapters must
supply bounds justified in their domains; the notebook's deterministic capacity is
not evidence that code execution or semantic judgments are predictable in advance.

## Evidence

- New `calibrated_progress_cannot_exceed_mechanical_action_capacity` failed before the
  fix with “calibration exceeded independently known capacity.” It now passes for
  unchanged/replaced subsets and actual growth under transferred underestimation,
  with unchanged-state controls. Partial/repeated inspection has a separate test.
- Two previous tests depended on the defect to execute impossible progress. They now
  require abstention without fabricated outcomes or selection of feasible progress.
  Existing failed/contradictory-return learning tests remain; replay also includes the
  actual failed-progress receipts rather than discarding them.
- `cargo test --offline -p tsr-cli -p tsr-domain --lib`: **79 + 12 passed**;
  one explicit local-receipt diagnostic is ignored in ordinary runs.
- Workspace/all-target Clippy with `-D warnings`, offline CLI build, changed-file
  formatting and diff checks passed. Independent review cleared the fix and tests.
  Rebuilt CLI reads preserve the historical root/observations; journal and saved
  state hashes still match the prior verification manifest. The old audit is not
  relabeled as a current-version causal comparison.
- Explicit offline diagnostic: **1 passed**, original receipt bytes unchanged.
  Calls 1 and 3 retain choice 3. Call 7 changes from choice 1 to choice 2; incomplete
  alternatives score `-0.003333` each, complete alternative `0.196228`. Call 9
  abstains: all four scores are negative. Replay uses each recorded historical state;
  it does not fabricate a new trajectory after the counterfactual call 7.

Logs: `target/shared-kernel-live/capacity-before.log`, `capacity-after.log`,
`capacity-replay-absolute.log`, and `capacity-clippy.log`. The first diagnostic attempt
used a relative receipt path, failed before reading evidence, and was rerun with an
absolute path. Reproduce from the repository root:

This diagnostic is specifically for the bundled standing-wants example. It validates
the original wants and fully inspected note corpus against that example before replay;
it is not a generic importer for arbitrary domain receipts.

```sh
TSR_DIRECTION_RECEIPTS="$PWD/target/shared-kernel-live/run" \
  /Users/n/.cargo/bin/rustup run 1.90.0 cargo test --offline -p tsr-cli --lib \
  replay_direction_receipts_against_capacity_bounds -- --ignored --nocapture
```

## Completion recommendation

The demonstrated capacity defect is corrected. Together with the prior live cycle,
offline replay verifies the bounded reusable decision-engine mechanism; another live
batch is not a necessary completion gate for this correction. The old run remains
incomplete, and no post-fix live success is claimed.

Reliability remains bounded: generated forecasts may be poor; residual transfer uses
coarse context similarity; probabilities are engineering proxies, not established
statistical calibration; and pessimism can cause abstention. Notebook predicates
measure coverage and brevity, not truth or understanding. Project semantic review
remains independent and manual. These are explicit limitations, not an undisclosed
user decision or authorization blocker. No inference was performed for this review.
