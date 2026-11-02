# B9.1 project bridges — offline implementation evidence

2026-10-23. Implements the two gaps recorded at `784b687`; does **not** assert that
B9.1 has passed. Cargo proposal context withholds protected file bodies (explicitly admitted supporting
spans remain separate), preserving the frozen no-test-answer input boundary.
The frozen preparation, original desires, failed seed, acceptance
criteria and protected source/tests are unchanged. See `engine-b91-preparation.md`
and PRD §B9.1 (the two-desires paragraph).

## Ordinary proposal path

`project request` now includes `coding.relational`: explicitly published parent
identities/meanings, current-source controller returns, allowlisted supplemental
observations, reviewed hypotheses and their immutable history. Private requirements,
review text and unpublished supplemental dimensions are excluded. Binding covers
source, requirement revision, published parents and current observations. Relational
meaning is qualitative guidance, not an independent weight, authority or fulfillment.

Normal `tsr-proposal@v1` responses support two additional suggestion kinds:

- `relational`: `binding` and the existing relational `Proposal` contract. At least two
  distinct published parents; bounded meaning and discriminator; optional revision of
  the latest active ancestor, with separated retained meaning, return interpretation,
  uncertainty and development. `development.proposed_development: null` preserves prior meaning fields; the
  development object itself is required for revisions.
- `experiment`: `experiment={binding,snapshot,requirements_revision,harness,cases,mapping}`.
  The content and requirement mapping are generated data, not an independent review.

`project run --response FILE` persists a response containing only these kinds and
returns `review_required` with `proposal_record`; it cannot execute it. Mixed responses
retain ordinary candidate selection for executable suggestions. Merely mentioning a
hypothesis in a mixed response does not admit it as guidance.

After independent review of the exact stored content:

```sh
target/debug/tsr engine --directory RUN project relational inspect
target/debug/tsr engine --directory RUN project relational admit \
  --state-root CURRENT_ROOT --proposal PROPOSAL_RECORD --index 0
target/debug/tsr engine --directory RUN project relational experiment-admit --file REVIEW.json
```

`REVIEW.json` contains `state_root`, `proposal`, `index`, `author`, `reviewer`,
`generator`, and `review`. All three role identities must differ; these remain operator
attestations, not proof of independent reasoning. `ExperimentOrigin` records the exact
proposal/index → supplemental admission link. Stale content requires regeneration or
reconsideration; operators cannot silently edit generated checks during admission.

`relational status --state-root ROOT --admission ID --withdrawn` withdraws guidance;
omitting the flag restores its status but does not restore stale source applicability.
A status no-op writes nothing. Source/return changes retain history but remove current
guidance pending source-bound revision/review. No-op narratives at unchanged evidence
are rejected. Original preferences and acceptance requirements are never rewritten.

The existing installed-local helper supports opt-in `project generate --possibilities`
(with the usual explicit model/output/endpoint arguments). It allows the model to choose
an edit, hypothesis or experiment; it supplies only current opaque bindings. Default
repair mode is unchanged. The helper saves review-only output without applying/admitting
it; Rust validates the content. **No helper/model call was made in this implementation.**
Existing 2,048 output-token and 240-second maxima remain; larger outputs are not silently
retried. Model competence or live schema reliability is not established by this change.

## Separately admitted Cargo experiment and return

`supplemental-cargo@v2` supports the B9.1 root Cargo package with automatic integration
tests. It adds one or two new protected `tests/tsr_supplemental_*.rs` files, preserves
every original byte/manifest/lock, and adds unique test IDs to the original inventory
(up to the adapter's existing four total cases). Workspace/named-package supplemental
experiments are deliberately unsupported. No argv, dependencies or build scripts can
be added by a proposal.

Use the existing `project verifier supplemental-spec --admission ID` export to initialize
a **different** disposable episode. That episode still requires explicit prospective
resource admission and fresh `project verify-admit --state-root ROOT --snapshot HASH`.
The source-bound verification action has zero edits; its normal committed execution
uses the existing sealed, offline Cargo build/full-test commands and resource accounting.
The parent experiment admission alone grants no execution or budget.

`project verifier supplemental-link --admission ID --from CHECK_RUN --receipt RECEIPT`
imports only a matching controller receipt from that separate episode. Original and
added source, complete configuration, commitment, execution event, resource roots and
receipt identity must match. Repeated import is idempotent and does not charge the
parent again. Revocation and source drift invalidate applicability.

Cargo summaries alone are not counterexamples. The source-bound Rules receipt must
carry the final strict completed-suite verdict, with matching full inventory, exits
and output bounds. Duplicate test-name inventory now fails closed. A completed named
failure can establish a finite counterexample for its independently reviewed mapping;
compile failure, missing/inconsistent evidence or timeout remains Unknown. A finite
pass also remains semantic Unknown, never universal proof or automatic acceptance.

The next ordinary project request exposes that bound return. A relational revision can
retain or develop meaning, and a subsequent generated patch must still use current
inspection handles and independent acceptance. No second action is automatic.

## Verification and scope

Six new Rust fixture regressions exercise deferred proposal → explicit guidance review
→ generated-format experiment review → separate verification admission → fake-Runner
return → imported evidence → fresh revision/context, plus restart, replay, no budget,
source drift, alias/unknown ancestor, withdrawal, namespace/collision checks, failed
build/finite-pass Unknown, forged event provenance and private-evidence exclusion.
These are authored protocol fixtures, **not generated outcomes, real Cargo executions,
or semantic evidence of culmination**. Existing expected-failure/dimensional-Contrast,
verification resource/restart and supplemental tests remain intact.

The local-helper test uses a fake provider and validates unchanged generated content,
controller bindings, inability to inject review/authority, and deferred/no-execution
output. A fresh-process actual CLI fixture verifies storage → separate admission →
proposal context and stale replay rejection without executing project code.

Independent reviewers challenged provenance, privacy and Cargo classification. The
first two issues were fixed with adversarial regressions; final review found no blocker.
The operator review boundary, finite-evidence limits and provider dependency remain.

## Commands and actual results

Evidence logs are retained locally in `target/b91-bridges-verification/`.

- `rustup run 1.90.0 cargo test --offline -p tsr-cli --lib`: **219 passed,
  zero failed, six ignored**, 47.54 seconds. The first restricted run had five
  socket-bind permission failures; the final run used supported per-command approval
  for offline loopback stubs. No assertions/deadlines were weakened.
- `rustup run 1.90.0 cargo test --offline -p tsr-cli --lib b91_`: **six passed**.
- `PYTHONPATH=scripts python3 -m unittest test_engine_local_model test_engine_b91_prepare`:
  **27 passed**, including six fresh-process preparation/CLI tests.
- `rustup run 1.90.0 cargo build --offline -p tsr-cli --bin tsr`: passed.
- `rustup run 1.90.0 cargo clippy --offline -p tsr-cli --lib --bin tsr -- -D warnings`:
  passed. `git diff --check`: passed. Changed Rust files were formatted with 1.90
  rustfmt and child traversal disabled; no protected formatting edits.

This is affected-library/CLI coverage, not a new whole-workspace sweep. All tests and
fake provider operations terminated; no model runtime was started. No paid requests,
new operational admissions, target executions, installs, protected edits or external
publication occurred. Existing unrelated untracked files remain untouched.

## Next live validation, separate admission required

Retain the previously frozen B9.1 scenario and preregistered maximum eight calls / three
execution groups (seed, independently reviewed generated experiment, boundary patch).
The expected-failure forecast must precede the actual seed return. Both original aims
must remain open with named Contrast. Then genuine generation, not these fixture
strings, must propose the second-aim experiment and use its observed return to propose
the boundary patch rather than retry the failed strategy. Review/admission remains
explicit between generated experiment and execution. No paid provider/source export or
new execution admission is granted by this document or implementation. Formal B9.1
research gates remain closed until that actual sequence is observed and independently
assessed; no operational calibration/maturity claim is made.
