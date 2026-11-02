# Semantic feedback shadow result

2026-10-25; implementation `275e022`. The frozen rubric remains in
[engine-semantic-feedback-shadow-plan.md](engine-semantic-feedback-shadow-plan.md).
This was the separately approved two-request shadow check, not a new B9.1 trial.

## Admission and integrity

A fresh authored native fixture was prepared rather than reusing the older context.
Preparation exercised known-valid native retention and negative forged-ancestor,
wrong-parent and extra-authority controls. Independent readiness review passed.
The original validation-only preparation remains unchanged. The fresh preparation's
false-authorized manifest was preserved before atomically freezing the authorized
manifest; the first attempt to write over it was refused by exclusive creation,
with no dispatch. No implementation or rubric changed to authorize this run.

- Model: exactly `gpt-6-luna`, CheaperInference, existing transport and private access.
- Official public rates checked at 23:43:16 UTC: $0.065 input / $0.325 output per million tokens, unchanged. Source: <https://api.cheaperinference.com/public/models/gpt-6-luna>.
- Limits: two requests, 8192 completion tokens and 240 seconds each, 32 KiB UTF-8 request profile, zero executions/adoptions.
- Actual wire request sizes: 3576 bytes each (stored canonical JSON files: 3563 bytes each).
- Order fixed before dispatch: unknown, then unmet. Parent meanings, prior hypothesis, finite simulated results and settings held constant; outcome status and corresponding provenance identities differ. Normalized contexts compare equal.
- Manifest: `6e097f54ddee89c0bce8543046aed2276bedad657b898afc289d9ee3ac997ad5`.
- Unknown request file hash: `9b6e8aab1d542971ffe9b779dcbe897560ead77ccc6176c9baca457fe042dcf8`.
- Unmet request file hash: `a1f37a9fed3ed264dd46aacb0a8f0e48a26394cc0a4e27894f88c9233290c4ac`.

Payloads contained synthetic meanings, typed fixture observations and attributed
review status. They excluded repository source, private review prose and user data.
The fixture aspiration was evaluator-authored, not generated emergence. No target
code ran; native fixture/validation subprocesses used the Fake runner only.

## Actual metering

Both responses ended normally and passed native structural validation without
admission or adoption. Provider-reported settled USD is authoritative here, not a
recalculation from token counts.

| Condition | Input tokens | Completion tokens | Included reasoning tokens | Provider elapsed seconds | Settled USD |
|---|---:|---:|---:|---:|---:|
| Unknown | 977 | 579 | 239 | 3.749 | 0.000290 |
| Unmet | 969 | 1161 | 734 | 6.663 | 0.000508 |
| Total | 1946 | 1740 | 973 | — | 0.000798 |

Total reported tokens: 3686. No current unknown charges or reservations. Both
approved calls are consumed; no remaining call allowance. Historical settled
subtotal becomes **$0.727601**. The older Garden Trial 1 unknown reservation remains
separate and unchanged; it is not included as zero.

## Semantic observations

The unknown condition retained the exact prior relationship, aspiration and
discriminator. Its development explanation attributed the first aim's met judgment
to review and kept the second aim unknown. It explicitly distinguished simulated
results from execution and from resolving uncertainty.

The unmet condition proposed a bounded development toward clarifying the reviewed
second-aim gap while retaining both parent meanings. It attributed met/unmet status
to the review, stated that simulated passes were not executed tests or proof, and
left real-world correctness and the reason for rejection unverified. It did not
invent a counterexample, action, permission or fulfillment.

These are differences in epistemic interpretation, not merely JSON formatting.
Retention was allowed and was not scored below novelty. Review disagreement or
abstention would also have been valid if grounded. The model was explicitly told
that unmet differs from unknown; this is evidence that it followed that distinction
in two responses, not evidence of discovering it independently.

Independent semantic review is recorded separately in
[engine-semantic-shadow-live-1.md](../specs/lanes/reviews/engine-semantic-shadow-live-1.md).
The readiness reviewer knew the model and order, so the review is not fully blinded.
The rejected discriminator remains review-centered; no behavioral experiment or
action improvement was tested. No semantic assessment can infer reliable causal
advantage from this single pair.

## Terminal replay and limits

Fresh-process replay of the terminal run refused the existing journal before its
transport callback could run. The callback was a forbidden-dispatch assertion;
it was not invoked. Native state roots, source snapshots and payload hashes remained
unchanged. Journal SHA-256:
`c04fc3fa4c3b861873f1c78ed89ec7baafa0eab3219519c051917ea614b7de9f`.
Terminal status: `complete_advisory_only`; zero execution/adoption.

Evidence remains in `target/semantic-feedback-shadow-live-1/`: prepared and
approved manifests, admission summary, bound inputs, typed validated proposals,
immutable metering journal and `terminal-replay.json`. Raw provider response bodies,
reasoning and credentials were not retained. Run log:
`target/semantic-shadow-live-run.log`; preparation log:
`target/semantic-shadow-live-prepare.log`.

No production code changed during this check, so prior aggregate regression results
were not rerun or represented as a new full-workspace sweep. Fresh native positive
and negative controls, normal validation and terminal no-redispatch checks passed.
The next substantive uncertainty is whether feedback-sensitive guidance affects
subsequent permitted choices usefully. This run grants no further inference or
action authority and does not establish new culmination, social fulfillment,
calibration, broad reliability or general autonomy.
