# Prospective inquiry contract repair

The frozen `565de0d` run remains a failed one-call run. Its response supplied a
scalar `considered_records` where native Rust required `Vec<String>`. The request
listed that field but failed to specify an array. This is a producer contract
omission; the native rejection was correct. No historical output is corrected,
adopted or relabeled successful.

## Shared contract and complete stage audit

`engine/qualitative/contract.rs` owns the versioned
`qualitative-model-contract@v1` bounded producer profile. `tsr-qualitative
model-contract` emits it from current state; `check-model-response` checks the
same definition and deserializes into the native types. The exploration exporter
uses this emitted profile rather than a second Python description of field types.
The runner checks every response before native submission, review or inspection.
It retains the original response and a bounded local rejection diagnostic; no
coercion, citation repair, response rewriting or retry is introduced.

This is a manually assembled native profile, **not automatic Serde derivation**.
Enums use native serialization, text bounds share the native constant, identities
come from the current request, and cross-language tests guard the remaining field
mapping. `valid:true` means profile validation, not permission or semantic truth.
Full native state/admission checks and independent reviews remain mandatory.

| Stage | Wrapper and native types | Required distinctions |
|---|---|---|
| Inquiry | `inquiry: null` or `Inquiry` | Exact root/attention/attribution; active parent array 1–16, distinct. `model_proposed` may omit/null answer source and exploration. `engine_exploration` requires enabled exact policy, corpus assertion source and exploration object. `user_supplied` remains operator-only. |
| Inquiry review | `Review` | Current root and sole pending proposal; distinct fixed reviewer; exact `admit`, `reject`, `unknown`; bounded nonblank reason. |
| Selection | `InformationChoice` containing `retrieval::Request` | Exact inquiry/root/corpus; IDs are an array, zero for abstention or one allowed considered ID; bounded pre-read relevance hypothesis. No access or support claim before reading. |
| Reflection | `Submission` → `Response` → zero/one `Suggestion` | Exact basis/protocol/root/attention/generator. Permitted variants: useful candidate, public desire, relational proposal; no executable variants. |
| Reflection review | `Review` | Separate reviewer and exact current quarantined proposal; same decision enum; no rewrite or authority grant. |
| Next preview | Same `Submission` profile | Reviewed state and receipt basis; preview only, no additional adoption/read/review. |

All objects reject extra fields. All text fields are nonblank and at most 512
UTF-8 bytes. Arrays remain arrays with one element. Optional means omission is
allowed; null is valid only where explicitly listed. The profile supplies exact
active IDs and their current revision identities, not placeholders.

Answer-source variants are native `corpus_assertions` (exact corpus and bounded
rationale), or kind-only `user_preference`, `user_authority`, `external_evidence`,
`mixed`, `unknown`. Unsupported sources still wait rather than being coerced into
corpus questions. Exploration policy is context, not a model-editable grant.

Candidate `serves` is an active-ID array (prospective profile bound 1–16).
Public desire revisions preserve identity, original parent ancestry and exact
current revision: null/omission only before any revision, otherwise the current
ID is required. `change` is clarification/development, basis is unknown and
nested generator matches the enclosing submission. Native checks reject unchanged
meaning and invalid lineage. Clarification stops the current runner.

Relational proposals require two or more active distinct parents, bounded
relationship/wanted condition/discriminator, and only known revision identities.
`revises` may be omitted/null for a new proposal; `catalysts` may be omitted/empty;
`development` may be omitted/null. Null catalysts and invented empirical development
are invalid. Native duplicate, supersession and ancestry checks remain unchanged.
A changed relationship may retain its wanted condition. Common event forecasts
may be omitted or `{}`, never null or fabricated passing evidence.

Initial context, retrieved fictional assertions and interpretations stay separate.
Receipt identity proves what was read, not truth or fulfillment. Changed public
meaning can invalidate the current receipt and stop; no fabricated replacement
return is introduced. Unknown review, rejection, abstention and stale state stop.

## Transport and privacy

The actual Luna request uses `max_completion_tokens:8192`, `stream:false` and
JSON-only prompt instructions. It does **not** set `response_format` or a provider
JSON-schema enforcement flag. Provider-side strict schema compatibility/enforcement
has not been verified; this repair makes no guarantee of compliant model output.
Existing strict JSON parsing and native rejection remain the safety boundary.
No network discovery was performed in this offline task.

The emitted contract contains structural instructions and current allowlisted
identities, not private/full native history. Existing explicit context projection,
source pins, body size bound (32 KiB), exact model lock, review separation,
accounting, interruption stop and no-repeat behavior remain in place. Old prepared
manifests are not launchable against modified source pins; fresh preparation and
admission are required for any later run. The runner intentionally still pins the
old 196-call/$0.162874 baseline; it is not a prepared fresh live admission at
current usage. That baseline must be explicitly reconciled in a later authorized
preparation, never bypassed or reset.

## Verification

- Offline `cargo check -p tsr-cli --lib`: passed.
- Offline `cargo build -p tsr-cli --bin tsr-qualitative`: passed.
- Offline `cargo test -p tsr-cli --lib qualitative`: 24 passed.
- `python3 -m unittest discover -s scripts -p 'test_engine_exploration*.py' -v`:
  7 tests, including a six-stage fake-provider flow through the real CLI and more
  than 70 profile/native positive and negative checks. Temporary fixture reads
  only; these are wiring evidence, not generated behavior or campaign inspections.
- Legacy inquiry tests: 5 passed, preserving the previous independent path.
- Independent source review found no blocking mismatch; confirmed profile checks
  are not full admission and provider schema enforcement is absent.

The new ancestry test initially reused an append-only temporary filename and
failed in test setup. It now uses a separate review file; no production validation
was weakened. Historical failure records and campaign records were not edited.

## Next live boundary (not authorized by this repair)

The complete flow still needs **up to six calls and one inspection**: inquiry,
review, selection, reflection, review, next preview. Stop early remains legitimate.
Same Luna/provider, fictional corpus, 8192 tokens/240 seconds/32 KiB per request,
prospective conservative $0.30 envelope. There is no extra correction-call budget.

Campaign remains **197/202 calls, 57/60 groups, $0.163331/$5**, with five calls
remaining. A whole fresh six-call flow cannot fit: minimum total call cap would be
203, with fresh explicit run admission and current pricing/source/ledger preflight.
No cap change is requested or applied here, and no live round was dispatched.
