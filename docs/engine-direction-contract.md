# Prerequisite proposal diagnosis and correction

The preserved support-live state at root
`3d736f20dfa9683eb7b1d6b0c7d1cf2589fdb4ccc16bf4950e01188f1bf51c4f`
was resumed without resetting wants or providing a candidate answer.

## Diagnosis before inference

Calls 5 and 9 each offered a `compose` direction with a future `compose`
continuation. The old JSON schema permitted that combination; its prose described
an inspection prerequisite, and Rust correctly rejected a composition as a
knowledge-acquiring prerequisite. This was a schema/controller mismatch, not a
stale state identity: all calls 5–9 named the same current root. Calls 6–8 instead
repeated a composition whose measured effect was unchanged and a direct inspection
without a useful plan. That is a separate unsupported-progress failure; correcting
the schema alone does not demonstrate its recovery.

Old journal hashes covered message content, not the complete schema/options.
Decoded raw offers, normalized offers and rejection traces were preserved, but
full historical wire requests cannot be recovered from a hash. The diagnosis
compares those offers against the then-current schema construction and Rust
validation. New calls save the complete local request envelope and its digest.

The first corrected request has 12,252 message bytes and a 5,670-byte JSON schema.
Its largest context fields are recent returns, obligations, tensions and selection.
No context-exhaustion or stale-identity error was observed in the preserved five
failures. Model capability remains an open explanation for repeated unsupported
effects; prompt inclusion is not evidence of understanding or successful recovery.

## Interface correction

`direction_request_schema` emits a union of disjoint complete objects: direct
inspection, direct composition, or inspection with a dependent future composition.
Composition cannot syntactically carry a continuation. Unknown evidence still
requires inspection; the model generates directions, grouping, title, supports and
forecasts. There is no reference answer, forced winner, altered utility or relaxed
standing desire.

`validate_action_contract` checks scope and dependency before submission to Rust:
new knowledge must actually be acquired by that inspection; future composition
must use at least one acquired claim and no other unavailable claims. Duplicate,
unknown, already inspected prerequisite IDs and unsupported action fields are
rejected. It never rewrites a proposal or forecast. Rust remains the final gate
and verifies mechanical outcomes and authority.

The Python validator rejects the whole offer on a contract error; Rust can
quarantine individual alternatives. With the current model boundary's two-offer
maximum, one invalid member normally leaves too few distinct alternatives anyway.
This conservative behavior can cost retries and is not represented as semantic
review. Existing uncertainty, action costs, standing loss and fallback rules remain.

## Verification and live evidence

`python3 -m unittest discover -s scripts -p test_engine_directions.py` passed 29
tests, including disjoint variants, absent/all-known evidence, invalid dependency,
duplicated/unadmitted IDs, unchanged raw input/forecasts, stop control, journal
reconciliation, frozen budgets and forecast identity. Independent review found no
blocking defect before live execution.

The new batch is `action-contract-v1`, at most ten installed-local-model calls,
each using the existing 240-second timeout. It starts after nine terminal historic
calls; overall journal numbering starts at 10. The provider is loopback-only
`qwen2.5-coder:32b`, with cloud disabled, no downloads and no paid calls. Full
requests and decoded offers remain local under `target/support-live/run/`.
Live results must be reported separately from these deterministic contract checks.

Corpus accounting is unchanged: 151 qualified tasks; eligible mature independent
common-event episodes remain unknown. Notebook coverage/brevity are mechanical
predicates, not semantic truth or general understanding.

The first corrected-schema continuation used two calls (journal 10/11). Both
returned schema-valid direct actions but repeated the same unsupported-progress
pair, so the existing repeated-error stop ended execution. State was unchanged.
This establishes backend compatibility and removes the invalid shape; it does not
establish useful recovery.

A second representation correction pairs rejected proposals with their feedback
and exposes a structured action-effect contract. Only already-inspected original
text supplies word counts; unknown lengths remain null. Direct versus terminal
supported progress is stated separately. New rejected proposals carry a receipt
hash checked before reuse; legacy feedback is explicitly marked as lacking that
original-response digest. This supplies measured constraints and the model's own
failed attempt, not a replacement candidate or reference answer. The same batch
continues with eight remaining slots. Offline controls now total 31 passing tests.

The second correction's call 12 again repeated the same ineffective pair. The
third correction (`disjoint-action-plans@v3`) excludes direct inspection only
when no selected active canonical predicate can benefit from new knowledge.
Typed `understand`/`inspect_notes` predicates and admitted unread scope determine
this; generated prose does not. Planned inspection remains available, and no
future composition is supplied by the controller. Structured guidance states
when that plan is required.

Call 13 was dispatched with this schema, then the execution transport disconnected.
Reconciliation found no driver, no provider PID 9681, no listener on port 11435,
and no saved response or terminal receipt. Its journal remains `started`: the
provider result is **unknown**, not a successful or rejected proposal. No retry
was made. Four of the ten batch slots are consumed/reserved; six are undispatched.
The request envelope is `request-13-10a07443419c.json`. Post-disconnect state in
`target/support-live/post-disconnect-state.json` retains the original root and
observation `[0.5, 0.8, 0.0]`; no observed action occurred.

Final affected checks: 32 Python unit tests pass; Python compilation and diff
whitespace checks pass. The changes correct the interface and preserve failure
evidence. They do **not** establish useful live recovery: repeated no-progress
proposals remain a reliability limitation, and the latest schema has no returned
live result. Full coverage and the original four-word limit can conflict; neither
criterion was weakened and internal predicate scores are not semantic truth.

Final independent read-only review found no blocking defect in v3. The added
selected-child/withdrawn-canonical-parent control also passes.

A final guidance change initially exposed a missing-metadata KeyError in the
existing context-only test. The helper now treats absent selection metadata
conservatively; the unchanged test and all 32 affected tests pass on the final
code. Git's lefthook executable is unavailable, so the checks above were run
explicitly rather than attributed to commit hooks.

## Audited interruption and residual-context continuation

`--reconcile-unknown REASON` performs no provider call and never reapplies a saved
proposal. Under the exclusive driver journal lock it verifies the request envelope,
requires the unchanged controller root and absence of application receipts, and
appends `generation_abandoned`. A saved response is hashed and marked deliberately
not applied; otherwise provider outcome remains unknown, including interruption
between durable intent and dispatch. Changed state or application receipts refuse
this path rather than calling an already-applied response unapplied. Every started
request still consumes its slot. A new request is new work, not exactly-once retry.

Request 13 was reconciled this way, preserving its envelope and unknown provider
outcome. The driver and temporary provider had been confirmed absent. No controller
application was detected. The original state root remained unchanged.

The full envelope showed repeated generated desire prose across tensions,
refinement summaries and typed obligations. In particular, prose requesting
inspection accompanied an `include_notes` predicate that measures account claims.
The selection settings also named action kinds unavailable in this adapter.
`direction_context` removes those repeated summaries/settings, retains typed wants,
lineage, active/selected status, original history and uncertainty, and describes
current missing measured IDs or observed word counts. It supplies no next action,
new title, reference answer or optimistic forecast. Unknown note lengths stay
unknown. All forecast dimensions and controller validation remain unchanged.
This is an evidence-backed ambiguity reduction, not proof that model failures
were caused solely by prompt redundancy.

The new request 14 uses `typed-residuals@v1`. Its user context is 9,554 bytes versus
11,296 before reduction. It consumes the fifth slot in the existing ten-call batch;
request 13 is not refunded. `--stop-file` exposes the already-tested dispatch/apply
stop control to the CLI. The local driver retains its 240-second request timeout.
Offline tests now cover unknown pre/post-send, recorded-before-apply, changed or
already-applied controller, tampered envelope, consumed budget and residual-context
lineage/unchanged controls. Live outcome is reported separately below.

Requests 14 and 15 both returned and were rejected, with identical saved-proposal
SHA-256 `4d9687f1cde757decbf2921abec6d75b5e9dd192df9df6251c427ed7d88dfd6c`.
The planned inspection shape was corrected, but the direct account was unchanged
and its forecast still overstated brevity; the terminal plan sacrificed brevity.
No controller state changed. The repeated-error stop prevented further dispatch.
This is failed useful recovery, not evidence that context compression solved it.

`typed-residuals@v2` additionally contrasts proposed forecasts against measurements
that the proposed action cannot change: inspection preserves account measurements;
composition of the existing title and claim set preserves the observed state.
This feedback only identifies contradictions on unchanged components. It does not
compute a winning candidate, author a replacement title, change forecasts, or
relax controller acceptance. Malformed rejected input is ignored by this diagnostic
rather than treated as a valid candidate. The complete suite now has 39 passing
checks. Four batch slots remained before testing this second projection.

Request 16 retained the same actions but changed its direct composition brevity
forecast from 1.0 to the measured 0.8. Rust then rejected the offer for lacking two
valid distinct progress-bearing alternatives. This is prediction correction on an
unchanged candidate, not useful action or demonstrated adaptation. Its proposal
hash differs from requests 14/15; first-step refinement prediction remains wrong.
Request 17 was dispatched with that rejection feedback and is pending at this
checkpoint. Eight batch slots are dispatched (including abandoned unknown 13),
two undispatched. No further allowance is implied. Controller state remains at
the original root through request 16. The temporary provider is retained only
while request 17 remains active; reconcile its journal before further action.

Independent read-only review found no blocker in reconciliation, the compact
projection, or unchanged-component feedback. All 39 affected Python tests,
compilation and whitespace checks pass. Live reliability remains unresolved:
correcting a prediction did not make the model generate a useful alternative.

## State-bound proposal reconsideration

Request 17 finished rejected with the same response hash as request 16. The driver
stopped; temporary provider PID 35666 was terminated. No controller state change
occurred. Two slots remain in the existing batch. Saved sampling settings are
`temperature=0`, `num_ctx=32768`, `num_predict=2048`; changing feedback altered a
forecast but not the repeated action identities. Sampling settings are not changed.

The current-state verified rejection history contains three distinct notebook
possibility identities: direct inspection, the same inspection with a composition
plan, and the unchanged composition. Forecast changes, purpose wording, note order
and same-length title wording do not make a new measured possibility in this
adapter. Title content has no semantic predicate here; this equivalence is not a
claim about general prose or other adapters. Continuation effects are part of the
identity, so a meaningfully different future plan is distinguishable.

`notebook-effects@v1` records that history in the request context and asks for
materially different possibilities or honest inability. Runtime enforcement is
narrow: an entire offer may not consist of previously rejected compositions that
mechanically reproduce the current claim set and title word count. Other old
members can remain; an unchosen member is not labeled a failure. Independent
review caught both an all-rejections exclusion and a prefix-based exclusion: even
weak-support abstention may depend on estimates. Neither rule remains. Corrected
forecasts/support for potentially productive effects still reach ordinary Rust
validation. Broader diversity is guidance, not an assertion that old possibilities
can never become useful.

The requirement is bound to the exact controller root. Changed observations,
wants, learned state or commitments permit reconsidering previous actions; nothing
is blacklisted globally. Only digest-verified local receipts supply the mandatory
set. The full append-only journal retains every receipt; context bounds history to
twelve offers and excerpts reasons to 256 characters.

The response contract now also accepts `state_root` plus a bounded `inability`
explanation. It records `generation_inability`, preserves the response and unmet
wants, consumes the request slot and stops without applying an action. This is the
generator's reported inability, not proof of objective impossibility or success.
Offline controls verify corrected same-action submissions, changed-evidence and
unchanged-state behavior, wording/order equivalence, substantive changes, integrity,
and inability without controller application.

### Live reconsideration result

Request 18 (`request-18-e1e8afe4bea5.json`) returned `generation_inability` and the
driver stopped. Its preserved response is `inability-18.json`, SHA-256
`fc1d561c4a0aceea2cfc73856d32e69c28c775ca7a85a651964a66b7659a2eab`.
This verifies the genuine local-model inability route rather than a fabricated
successful candidate. It does not establish objective infeasibility: the model
failed to identify a useful alternative under the supplied contract.

`post-reconsideration-state.json` confirms the unchanged root
`3d736f20dfa9683eb7b1d6b0c7d1cf2589fdb4ccc16bf4950e01188f1bf51c4f`, status
`deliberate`, and observation `[0.5, 0.8, 0.0]`. Desires were not weakened, satisfied
or abandoned. Nine of the ten batch slots are dispatched, including unknown
request 13; one remains unused. No further request was dispatched after inability.
The temporary provider PID 56795 was terminated and its listener checked absent.

Final validation: 43 Python tests pass, compilation and diff checks pass, and
independent review found no remaining blocker after narrowing runtime exclusions
to mechanically unchanged accounts. The mechanism now distinguishes revision of
forecasts from action diversity, preserves useful revisits after evidence changes,
and stops honestly on inability. Useful live proposal generation remains unresolved;
these checks do not establish full functioning-engine completion or calibrated
mature-episode support. The remaining issue is generation of different useful
possibilities, not acceptance of unsupported success claims. A next bounded
investigation should separate proposing possibilities from simultaneous forecast
and support form completion, preserving the same controller and original wants;
repeating this deterministic request has no demonstrated value.
