# Native attention to reviewed information inquiry

This prospective local path connects qualitative information choice to the existing
native attention kernel. It does not change prior experiments or authorize a new
provider run. Local authored tests establish wiring, not generated insight.

## One architecture map

```
Attributed original/current public desires + active/withdrawn lineage
  -> tsr_ledger::selection::select (existing lineage-half-retain kernel)
  -> question-request: current parent meanings, native attention, root binding
  -> propose-inquiry (model) OR user-inquiry (explicit trusted operator input)
  -> existing NewCandidate quarantine + qualitative_review
  -> existing reviewed possibility memory (question, not changed wanting)
  -> retrieve-for-question: admitted inquiry ID + bounded IDs + relevance hypothesis
  -> existing native bounded reader and content-bound append-only receipt
  -> request-v2 / question-request: question + actual bytes + relevance provenance
  -> existing reflection proposal, independent review, persistent meaning/memory
  -> existing root-bound public attachment -> generic proposal_request_inner
```

The selector needs no qualitative scores or acceptance predicates. It exposes all
eligible active parent identities; its deterministic order is **not** semantic topic
priority. It does not generate a question or determine which missing information
matters. That remains a generative proposal reviewed against the attributed desires
and stated uncertainty. No exploration policy is enabled by this change.

The new typed inquiry is small metadata around the existing candidate/review path,
not a new selector/controller. It pins the complete active-parent context and keeps
question, uncertainty, origin and attribution. Model input cannot claim user origin;
only the separate trusted local operator entry accepts that origin. Review uses the
existing trusted-caller boundary; different name strings alone do not prove reviewer
independence.

An admitted question is not an action grant. Reads still require explicit local
invocation against the configured allowlist. Once an inquiry exists in a state,
legacy unlinked retrieval is rejected. Old stores and frozen legacy projections
remain compatible. No changes to acceptance, budgets or provider export admission.

Parent development or status changes invalidate focus. Withdrawal/reactivation
cannot silently revive it. History and prior receipts remain available; stale
inquiries are excluded from current possibility memory. Relevance is recorded
before retrieval and never treated as post-read support. The receipt proves actual
access and content identity, not truth of fictional assertions or fulfillment.

## Local workflow

Build offline with the installed toolchain:

```sh
PATH=/Users/n/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:$PATH cargo build -p tsr-cli --bin tsr-qualitative --offline
```

On an already initialized, allowlisted local test store:

```sh
target/debug/tsr-qualitative STATE question-request
target/debug/tsr-qualitative STATE propose-inquiry inquiry.json
target/debug/tsr-qualitative STATE review review.json
target/debug/tsr-qualitative STATE question-request
target/debug/tsr-qualitative STATE retrieve-for-question choice.json
target/debug/tsr-qualitative STATE request-v2
```

`inquiry.json` contains current `state_root`, `attention` copied from
`inquiry_binding`, active `parents`, `question`, `uncertainty`, `attribution`, and
`origin: "model_proposed"`. No suggested question is supplied by this workflow.
For genuinely user-supplied focus only, use `user-inquiry` and `user_supplied`.
`review.json` uses the existing current-root/proposal/reviewer/decision/reason schema;
reject and unknown remain valid and do not authorize retrieval.
`choice.json` contains `inquiry` (the proposal event ID), `request` with current
`state_root`, exact `corpus`, selected allowlisted `ids`, and `relevance` (a pre-read
hypothesis). Reflection uses the existing exact returned basis and submit/review
path. Every state-changing command requires fresh current identity; replay does
not repeat effects. Merely viewing a request performs no action.

## Evidence and remaining boundary

Tests cover unchanged views, common selector identity, unreviewed/unknown rejection,
model/user origin separation, changed parent meaning, withdrawal/reactivation,
wrong inquiry context, legacy-read bypass rejection, actual temporary-file receipt,
reflection admission, common attachment context and fresh-process reconstruction.
Existing path/symlink/content/provenance tests remain applicable.

The provider workflow is deliberately **not** updated or run. A future reviewed
export would need to include this question proposal/review phase and carry its
binding through selection and reflection. No existing expired grant supplies that
permission. The next useful live observation is whether an independently reviewed,
model-proposed inquiry yields relevant retrieved information and an honestly
supported reflection or abstention. No preferred topic, forced change, semantic
priority score or emergence verdict follows from these deterministic tests.

Campaign unchanged: 193 calls, 57 groups, $0.161719 settled; no paid calls or campaign
inspections in this patch. Prior abstention and all earlier negative results stand.

Verification for this patch: 19 qualitative-filter Rust tests passed, followed by
all 3 inquiry tests (including the added changed-meaning control); 5 attachment
Rust tests passed. The CLI built offline. Six Python tests passed: one new
fresh-process inquiry/read/no-repeat test and five legacy export/driver controls.
Independent source review found origin spoofing, stale-memory projection and legacy
projection issues in the draft; all were repaired and re-review found no blocking
defect. These are affected-path checks, not a new full-workspace regression claim.

Reproduce the local checks:

```sh
PATH=/Users/n/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:$PATH cargo test -p tsr-cli --lib qualitative --offline
PATH=/Users/n/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:$PATH cargo test -p tsr-cli --lib public_attachment --offline
python3 -m unittest discover -s scripts -p 'test_engine_inquiry.py' -v
python3 -m unittest discover -s scripts -p 'test_engine_retrieval_export.py' -v
```
