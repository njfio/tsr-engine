# State-first possibility comparison and inconclusive verification closure

## Actual local comparison, 2026-10-22

The separately admitted comparison used one of two allowed local calls and stopped
on explicit generation inability. The second slot expired; there was no assessment,
selection, notebook/project execution or replacement response. The temporary owned
Ollama runtime stopped after the terminal result. External charge was zero. Token
usage and provider duration are unknown because the existing validation-only receipt
does not retain those counters; unknown resource use is not zero resource use.

Request 33 used installed `qwen2.5-coder:32b`, digest
`4bd6cbf2d094264457a17aab6bd6acd1ed7a72fb8f8be3cfb193f63c78dd56df`,
temperature 0, context 32,768, output ceiling 2,048 and 240-second request timeout.
The preserved state root was
`3d736f20dfa9683eb7b1d6b0c7d1cf2589fdb4ccc16bf4950e01188f1bf51c4f`.
The complete direction request compared equal before and after the experiment.

The actual request was compared with preserved request 32: model, options, system,
schema, state, authority, obligations, notes and action contract were identical.
Only the possibility-stage context projection changed. Assessment code remained
unchanged but was not reached. Full original context remains bound by digest.

This result does **not** show improved generation. Under this exact installed model
and configuration, reducing and reordering context did not overcome inability.
It does not establish that useful partial actions are impossible. Both standing
wants and their real incompatibility remain intact. Further unchanged retries are
not justified by this result. A materially different generation method or provider
would need its own explicit scope; no such call is implied or dispatched here.

`--assess-only --two-stage` now supports a comparison that cannot refine or call
`deliberate`. It stops on inability or the first invalid offer, including when a
bounded search is configured. A valid assessment reports schema/action/forecast
validation only, never controller selection or observed success.

Evidence: `target/context-comparison-live/{manifest,payload-controls,terminal}.json`,
`state-before.json`, `state-after.json`, `expired.json`, and the append-only journal
`target/support-live/run/local-directions.jsonl`. Request envelope:
`a212067c1eb695f72b1948a32a8bc184a4bb4bc5cb32a83b0ea41276a6fb620b`.
Raw provider text is not retained, in accordance with repository rules; validation
metadata, inability classification and content hashes are preserved.

## Verification-only lifecycle boundary

`project-lifecycle-close-inconclusive --reason TEXT` is a distinct terminal reason,
`verification_inconclusive@v1`. It requires actual source-bound verification returns,
unchanged source, zero edits and settled resource reservations. Pending execution,
source drift, missing verification provenance and reclassification of an already
closed episode fail closed. Settled unknown usage remains charged and unknown.

It records no acceptance, rejection, review or training label. Dataset eligibility
reconstructs and validates its evidence, then explicitly withholds training eligibility.
The existing no-execution advisory-only closure is unchanged. Prior executions,
receipts and old closed episodes are never rewritten.

The bounded decision engine's principal unresolved functional limit remains useful
genuine generation in this conflicting-wants case. This closure repair makes the
existing supplemental evidence honest and terminal; it does not solve Intuition,
prove semantic understanding, or produce mature calibration data. The 151 qualified
corpus tasks remain distinct from an unknown count of eligible mature episodes.

## Applied closure and validation

After implementation tests and independent review, the prior check episode
`target/supplemental-live/check-run` closed at **2026-10-22T12:46:14.305Z**.
Closure event `aa164d9719b38dd0238b94c0fb1f0cac49d740c6fda1f81c384b5a2aa7f8a98e`
records the explicit verification-inconclusive reason. The actual execution was
not repeated. Tool charge remains **12,109 ms**, source snapshot remains
`98733d039d7991a574cf9ea32b87255df941aa17d394a93456d180bfe9cfa4c5`,
and acceptance/review/label are all absent. Eligibility reports
`verification_inconclusive_no_acceptance_label`. Restart reproduces the projection;
the original partial candidate's advisory closure compares exactly equal. Old
pre-closure evidence remains in place. New evidence is in
`target/supplemental-live/inconclusive-{closure,eligibility,budget,restarted,terminal}.json`.

Independent review caught and corrected two issues before completion: assess-only
search must stop on inability, and learning-side reconstruction must compare the
exact original configuration hash rather than only check its presence. The latter
uses a strict compatibility serializer with cross-crate equivalence tests, including
optional scope fields; unknown schema fields fail closed.

Checks: 71 Python direction-driver tests, three focused Rust closure/equivalence
tests, 138 learn library tests, affected all-target Clippy and build passed.
The parallel CLI library run reported 196 passed, one failed, three ignored. Its
sole failure was `local_transport_isolated_metered_quarantined_and_not_replayable`:
the completed stub response elapsed 2,685 ms against its 2,000 ms admission and was
correctly quarantined as exceeding resources. The same test's timing failure was
observed before this change; this run does not by itself establish a general cause
or prove concurrency stability. No assertions or admission deadlines were changed.

The unchanged full CLI library suite then passed serially: **197 passed, 3 ignored**
in 146.17 seconds, including the deadline-sensitive stub test. Both run logs are
retained in `target/context-comparison-live/cli-tests{,-serial}.log`; the parallel
failure remains disclosed rather than represented as an unqualified clean run.
