# State-bound rejection feedback

Request 23 repeated five rejected offers spanning three action-effect identities.
It included a 4,115-byte reconsideration block, the raw last rejection (146 bytes),
the last rejected proposal (1,498 bytes), and measured-effect feedback (544 bytes).
The model's subsequent inability generalized previous account failures into no
alternatives. This establishes a context ambiguity/redundancy and a model error;
it does not prove that repetition alone caused the error.

## Abstraction rules

The controller still validates against the original hash-verified rejection
history. Only the generator-facing projection changes:

- Group repeated notebook effect identities: inspection note set, composition
  claim set plus title word count, and both actions of a proposed continuation.
  Purpose text, forecast edits and same-length title rewording do not create new
  effects. These are existing adapter semantics, not an action recommendation.
- Preserve receipt request numbers and proposal hashes for every grouped offer
  in the existing latest-12 same-state window. An explicit older-offer count makes
  that fixed window visible. No contrary observation is selected away because it
  is inconvenient. Complete journal, responses and envelopes stay on disk.
- Classify only the offer-level failure: selection abstention, failed validation,
  or other rejection with its excerpt. Do not attribute a whole-offer failure to
  each member, assert an unexecuted return, or assume historical controller
  behavior proves rejection under the current implementation.
- Separately identify exactly unchanged current accounts as mechanically known
  no-progress effects. No action-family exclusion follows. Changed state clears
  this same-state rule; other effects remain assessable under current contracts.
- Remove the duplicated raw last offer/rejection from model context after deriving
  its measured mismatch feedback. Keep that feedback, original wants, observations,
  uncertainties, authority and raw evidence. Joint conflict does not prohibit
  partial progress and does not permit changing any desire.

The new grouped reconsideration block alone is slightly larger (4,302 bytes),
but eliminating the two duplicated raw blocks reduces combined rejection material
from about 6,303 to 4,846 bytes. The point is explicit evidence scope, not merely
shorter context. No feasibility witness, title, claim choice or authored direction
is injected. The separate operator conflict certificate remains excluded from
model context. The visibility-only omitted-offer count was added after request24
was dispatched; this preserved state has no omitted older offers.

## Verification and bounded attempt

57 driver tests pass (`target/feedback-summary-tests.log`), including deduplication
with provenance, unchanged controls, changed state/effect handling, partial-progress
permission and removal of duplicated raw feedback while retaining measurements.
Existing tests retain stale binding, no-op, immutable assessment, uncertainty,
restart, and budget checks. Independent read-only review found no blocker.
Python compilation and diff checks pass; no Rust decision semantics changed.

The single correction experiment uses search `feedback-summary-v1`, one sample,
temperature 0.7, seed 7304, within the two remaining global slots of
`exact-effects-v1`. A valid possibility set may consume the second slot for fixed
assessment. This named search is explicitly authorized by the parent continuation,
not a reset of `diverse-possibilities-v1` or a new call grant. Each call retains its
240-second bound, envelope, context hash and provenance.

## Already-installed alternatives

Read-only `/api/tags` and `/api/show` metadata identified two local candidates;
no download, external request or alternative-model generation occurred:

- `hhao/qwen2.5-coder-tools:latest`: 7.6B Q4_K_M, chat template, completion/tool/insert
  capabilities, 32,768 context; digest
  `b7fade0651b448e0e57af67d9edcbdcb686cd8a9647c2cb1f123858438af0a33`.
- `mistral:latest`: 7B Q4_0, chat template, completion capability, 32,768 context;
  digest `61e88e884507ba5e06c49b40e6226884b2a16e872382c2b44a42f2d119d804a5`.

Metadata supports protocol compatibility, not demonstrated schema adherence or
useful generation. `target/installed-alternative-metadata.json` records the check.
The current run freezes model identity; any later comparison must preserve that
provenance rather than silently replace the provider within the same journal.

## Terminal result and provider limit

Request 24 returned explicit possibility-stage inability; no possibility set was
frozen, no assessment was dispatched and no action was applied. The search ended
`possibility_search_exhausted` with the original root
`3d736f20dfa9683eb7b1d6b0c7d1cf2589fdb4ccc16bf4950e01188f1bf51c4f`,
observation [0.5, 0.8, 0.0], and `proven_joint_conflict` still present.
Receipt: `target/support-live/run/inability-24.json`, SHA-256
`2753c6c60468a4b3a28173164b45f4d80d1014e60df47807d162d886079f2677`.
Envelope: `request-24-0954c35ddc06.json`. Output:
`target/support-live/feedback-summary-live.log`.

One of the two available slots was consumed; one remains unused. The authorized
single-sample experiment is exhausted; no repeated sample or provider switch was
performed. Temporary provider PID 10002 terminated with session exit zero and no
remaining listener. Unknown request 13, inability 19 and all subsequent receipts
are intact. There were no paid/external calls, downloads, target changes or
publication.

This is an observed limit of qwen2.5-coder:32b Q4_K_M with these preserved contexts,
schemas and sampling settings, not proof of universal model incapacity. Across the
latest attempts it did not produce a feasible useful alternative despite a valid
mechanical partial-progress witness. The context correction has deterministic
contract/provenance benefits but did not demonstrate live recovery. Further
unchanged retries are not justified. No inability assertion is treated as proof
of infeasibility or permission to weaken desires.

Search exhaustion now includes a `generation_limit` report binding empirical
attempts to state, model digest, envelope and sampling configuration, with terminal
outcomes and explicit scope limitations. Its offline reconstruction from request24
is `target/support-live/feedback-summary-limit.json`; this post-run reporting
addition did not change the live prompt or rerun inference. A focused test checks
state/search filtering and preserves the distinction from infeasibility.
Final checks: 58 driver tests passed, Python compilation and diff checks passed;
independent review covers the context abstraction and reporting. Existing Rust
selection/authority semantics and protected tests are unchanged. Corpus remains
151 qualified tasks; mature eligible independent episode count remains unknown.

The next meaningful provider comparison would use an already-installed compatible
model with the same preserved state and controller semantics, under explicit
provider provenance and a separately bounded allowance sufficient for both stages.
It must not silently change this run's frozen model configuration or count a
hand-authored repair as generation. This is a concrete verification dependency,
not a request to invent new desires or solve general intelligence.
