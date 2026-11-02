# Native qualitative context, without proxy predicates

This closes the local bootstrap/admission gap identified in `engine-complementarity-native-boundary.md`. It adds a general native qualitative adapter, not a new scenario route, model service, executor or numerical desire score. It reuses Engine's transactional append-only event/CAS store, native Preference identities, the provider Response/Suggestion vocabulary, public-desire proposal type and relational proposal type. The coding and notebook observed-return validators are unchanged.

## What works

An isolated native store can hold attributed original public meanings and visible records without attaching measurable acceptance predicates. It distinguishes `initial_context` from empirical returns. Visible assertions and model interpretations are not verified world truth. Acceptance references are immutable opaque references, separately exposed and never evaluated or changed here. No observation receipt or action commitment is fabricated.

The same general request method chooses `propose`, `review` or `stop` from persistent state. It exposes original/current meanings, active status, revision ancestry, applicable relational guidance and unknown evidence limits. Generated candidates are quarantined as Generator-origin records; a distinct source-bound operator review may admit, reject or leave unknown. A public meaning update changes that identity's current view while preserving its original meaning. Multi-parent relational guidance can be admitted without a prior action return, under an explicit initial-context basis only.

Subsequent requests consume admitted state. A parent meaning/revision change invalidates older guidance until explicitly re-reviewed. Withdrawal propagates transitively through participating public parents and suppresses dependent relational guidance without erasing history; explicit restoration preserves history. Parent order alone is not development. Exact duplicates and stale references are rejected. Semantic paraphrase is a reviewer judgment, not something schema proves.

Only `PublicDesire` and `Relational` can be submitted for admission. `NewCandidate` is an ordinary native proposal vocabulary item but remains preview-only here. All execution, task-acceptance changes and automatic admission are absent. Empty proposals are a valid previewed abstention and do not create history. The adapter stores no weights and does not choose an executable candidate. Reviewer labels establish attribution, not cryptographic independence or truth.

## General runnable entry point

Build with the installed Rust 1.90 toolchain and existing offline dependencies:

```sh
cargo +1.90.0 build -p tsr-cli --bin tsr-qualitative --offline
```

All commands are local; none invokes a provider or worker:

```sh
target/debug/tsr-qualitative RUN init BOOTSTRAP.json
target/debug/tsr-qualitative RUN request
target/debug/tsr-qualitative RUN preview SUBMISSION.json
target/debug/tsr-qualitative RUN submit SUBMISSION.json
target/debug/tsr-qualitative RUN review REVIEW.json
target/debug/tsr-qualitative RUN status STATUS.json
```

`init` requires an empty isolated store. `BOOTSTRAP.json` has `parents` (label, meaning, attribution), `records` (label, content, attribution, uncertainty), `constraints`, and `acceptance_references`. Strings are bounded to 512 UTF-8 bytes; at most 16 parents and 64 records. A local user may supply this schema, but doing so grants no external export permission.

`request` provides the exact current basis/root/binding/identities and response contract. A submission has `basis`, `generator`, and the native `response`. Initial-context submission accepts no observation receipt or return-development claim. `submit` returns a proposal event ID and moves the current state to review. It does not admit the candidate. Review fields are `state_root`, `proposal`, `reviewer`, `decision` (`admit`, `reject`, `unknown`), and a bounded `reason`. Reviewer must differ from generator. Review binds to the exact current full native event head; changes between proposal and review invalidate it. Reasons remain local and are excluded from the request projection.

Status fields: `state_root`, `identity`, `revision` (current revision or null), `withdrawn`. A no-op status preserves the root. Replaying a submission or review fails as stale/already reviewed, rather than applying it again. Maximum 128 quarantined proposals; the last admitted slot remains reviewable.

## Frozen fictional preparation

No preferred aspiration or model response is supplied:

```sh
python3 scripts/engine-complementarity-native-preflight.py --directory target/complementarity-native-new
target/debug/tsr-qualitative target/complementarity-native-state-new init target/complementarity-native-new/native-bootstrap.json
target/debug/tsr-qualitative target/complementarity-native-state-new request
```

The converter preserves the exact two original meanings, all six fictional assertions/assumptions/uncertainties/attribution and two fixture observations. Empty acceptance references deliberately avoid inventing tests of the rich desires. The native input is `native-bootstrap.json`; the accompanying source packet is provenance, not provider authorization. The old preparation instance remains untouched. No generated content is adopted by these commands.

## Verification and remaining limits

Eight native authored-fixture tests passed, covering initial admission without returns, restart/next-candidate preview, forged empirical provenance/authority, distinct acceptance, stale context, transitive withdrawal/restoration, unchanged inputs/no-op replay, unknown/reject reviews, parent-order aliases, stale guidance/full event head, and capacity-bound review. Four Python source-mapping checks passed. Existing observed-return relational tests: 10 passed, 1 deliberately ignored fixture builder. Existing coding public-desire tests: 8 passed. No full workspace run is claimed.

Independent review caught and resolved stale parent guidance, full-head review checking, order-dependent aliases and last-slot reviewability. These checks use authored proposals. They prove wiring and boundaries, not model emergence, semantic fidelity or causal benefit. The adapter is not wired to existing adaptive ranking or real execution, and does not autonomously generate new parents. Integration with empirical return learning remains through existing adapters; this initial-context profile does not impersonate it.

## Future live envelope — still pending

At most six CheaperInference `gpt-6-luna` generator/reviewer requests, 8,192 completion tokens, 32 KiB body and 240 seconds per request; zero executions. Prospective reservation $0.30 within the existing $5 / 200-call / 60-group campaign, not a new budget. Baseline remains 191 calls / 57 groups / $0.160913; no calls or live-output adoption occurred here.

A fresh allowlisted export/admission must pin this native state and the unchanged fictional source before dispatch; the full local request must not automatically be exported. First generation can abstain and stop. A justified generated initial relationship may receive independent review/admission; only then may a native next proposal use it. Later reconsideration is optional and must identify attributed review as fallible judgment, not observed return. No forced six-step trajectory or resampling for novelty. This local adapter does not yet admit semantic feedback as an empirical basis; any future extension must remain explicit. Do not claim the six-call transport path is launch-ready merely because bootstrap now works.

Fresh-process executable check also passed: `target/complementarity-native-preparation-2/check.json` records identical init/request state across processes, 2 original parents, 8 attributed visible records, zero returns/calls/executions/generated adoptions. Bootstrap file SHA-256: `fddef6c5000ad181147bd50c76077444c88ffdaab0459a019cfa60541f088df8`. Native root: `1ae87f0360162dc28f88a1ef59fdebff3fcb9edaaf2a1fe3c351ad60dd9f4137`. The local request remains local-only, not a frozen provider payload.
