# Tuned repair regression results

Implementation: `050d786`. This pass follows the [frozen tuned plan](engine-tuned-repair-plan.md). It does not replace the original held-out result: **0/2 fresh full cycles, 1/1 original regression**. Same programs, immutable acceptance checks and bounded controller; no hand-authored replacement patches or manual stage selection.

## Singleton clamp

Run `target/tuned-clamp-1` completed 11 calls and four isolated execution groups, with 67,782 reported tokens and $0.010490 settled. No correction calls or unknown charges. The generated experiment failed on the admitted mutant; the generated patch passed the protected checks and the unchanged experiment. Protected files remained intact.

The full cycle nevertheless failed final runtime semantic review. Its canonical observed-outcome summary was correct, and post-experiment interpretation was truthful. The final discriminator retained the earlier present-tense claim that the source returns `Ok(upper)` for reversed bounds, despite the applied source now explicitly returning `Err("invalid range")`. Independent review agrees this is an actual stale-source contradiction, unlike the original run's omission-only review dispute. No acceptance override was made.

The no-development branch retained prior text verbatim as instructed; the model needed to choose justified development to update the transient diagnosis. Mechanical case-fact validation does not establish truthfulness of other prose. This run demonstrates a correct repair and useful rejection, not a successful full cycle or a solved semantic-grounding problem.

Fresh-process replay preserved the rejection with zero dispatch, 196 run files and the campaign ledger unchanged. Evidence: `target/tuned-clamp-1-accounting.json`, `target/tuned-clamp-1-replay.json`, and [independent review](../specs/lanes/reviews/engine-tuned-results.md).

## Unicode prefix

Run `target/tuned-unicode-1` stopped at `relation_after_experiment` with `unknown_stale_or_withdrawn_revision`. It consumed five calls, two execution groups, 24,945 reported tokens and $0.004394 settled. No correction calls or unknown charges. Initial relational guidance and a substantive generated experiment passed their runtime reviews; the seed and experiment failed as expected. No boundary patch, final verification or acceptance ran.

The rejected response remains quarantined and unadmitted. Independent inspection found its `revises`, `original_revision` and `prior_revision` references each contained 65 UTF-8 bytes, with the same hashed value. The unchanged dispatched state advertised one eligible 64-character revision ID. The returned references therefore could not match it: this was model contract noncompliance, with no evidence of stale host state. The exact extra character and rejected prose were not retained and are not guessed. HTTP 200 with finish reason `stop` establishes a known invalid response, not truncation. Replay refused the same terminal invalid outcome through the correction guard (`ineligible schema correction`) without dispatch; 114 run files and the ledger were unchanged.

Both tuned full cycles failed: **0/2 tuned successes**, separately from the original held-out denominator. The numeric repair passed its tests; Unicode never reached a patch. No additional retry was made.

## Verification and limits

Before dispatch, focused evidence, response-format, native cross-language profile and legacy-compatibility checks passed. The new native profile tests include full fixture-only driver/restart flow, unsupported passing claims, profile tampering and bounded assembled payloads. These are deterministic integration evidence, not live model success. No new complete workspace-suite pass is claimed for this change.

This pass used 16 calls, six groups, 92,727 reported tokens and $0.014884 settled. Final campaign usage is **181/200 calls, 56/60 execution groups and $0.157617**, with no campaign unknowns. Remaining capacity is 19 calls/four groups, but this scoped pass is complete and does not consume it automatically. The older Garden Trial 1 unknown reservation remains separate. Two tuned samples with jointly changed contracts cannot establish causal improvement or general reliability.

The source-bound read-only audit is `target/tuned-evaluation/final-accounting.json`; per-run replay receipts are `target/tuned-clamp-1-replay.json` and `target/tuned-unicode-1-replay.json`. Historical trial trees, original held-out records and frozen tests were preserved.
