# Prospective native inquiry / information-feedback validation

Status: prepared locally; **new user approval required**. No live dispatch, price
lookup, paid call, campaign inspection or generated adoption during preparation.
Historical abstention and frozen earlier projections remain unchanged.

## Exact scope

Same two evaluator-authored fictional desires: find a dependable starting point
quickly while unfamiliar with a topic; keep unfamiliar perspectives and unresolved
disagreements discoverable. Same six fictional notes: two each about seed storage,
path marking and water timing. No new corpus, private/native history, outside files,
credentials or real-world assertions are exported.

Start with all six bodies uniformly withheld. Export original attributed parents,
constraints, index topics and provenance, eligible native parent support and exact
current bindings. Subsequent exports may include this run's generated inquiry,
independent review decision, admitted advisory memory, model-selected IDs and
pre-read relevance, exact selected fictional record contents and access receipt,
then a quarantined reflection for separate review. Private review prose and full
native state are excluded by explicit projection. No preferred question, topic,
record IDs or answer is supplied.

## Six-call ceiling, one read group

1. Luna proposes an attributed question through native attention, or abstains.
2. A separate Luna context independently reviews that exact question. Only an
   admitted inquiry enters isolated persistent possibility memory; wanting and
   permissions are unchanged.
3. Luna selects 1–4 allowlisted IDs with a relevance hypothesis, or abstains.
   One counted native bounded inspection reads those bytes and records a receipt.
4. Luna reflects using the actual receipt and fictional assertions, distinguishing
   prior relevance from post-read support. Retention/abstention is legitimate.
5. A separate Luna context independently reviews the reflection. Only justified
   advisory possibility/relationship/meaning may be admitted locally. Reject and
   unknown stop. If changed meaning makes the receipt basis stale, stop rather
   than manufacture a fresh return.
6. The ordinary native proposal protocol previews a subsequent possibility using
   that persistent state. No final proposal adoption or further action follows.

All six calls use **CheaperInference gpt-6-luna**, at most 8,192 requested completion
tokens, 240 seconds and 32 KiB request bytes per call. No retries or extra round.
Fresh grant requires over 1,600 seconds headroom and expires within two hours.
Any inability, malformed/stale output, insufficient evidence, rejection, unknown,
resource exhaustion or valid abstention ends the run and expires unused slots.
Unknown effects retain campaign reservations; restart cannot repeat them.

Existing total ceilings remain **200 calls / 60 groups / $5**. Starting totals are
**193 calls / 57 groups / $0.161719**; a full run ends at most 199/58. Whole-envelope
preflight is six calls/one group and **$0.30 conservative allowance**, with fresh
catalog validation and per-call durable reservations through the existing ledger.
Preflight is not an exclusive campaign reservation; every effect rechecks limits.
At the previously recorded $0.065/$0.325 input/output per million token rates,
counting each of 32,768 input bytes conservatively as a token plus 8,192 output
tokens gives about $0.0288 for six calls, excluding separately metered categories.
This is not a current quote or guaranteed bill. Actual/unknown charges remain
separate. No numeric extension is required.

## Reproducible local preparation

```sh
python3 scripts/engine-inquiry-export.py target/qualitative-inquiry-prepared-2
python3 -m unittest discover -s scripts -p 'test_engine_inquiry_validation.py' -v
```

The prepared directory contains `inquiry-manifest.json`, `inquiry-body.json`, pinned
origin and isolated native state. The old preparation artifacts are retained only
as prerequisites, not authority. The new runner requires an exact fresh grant:
`plan`, `max_calls:6`, `max_operations:1`, `reservation_microusd:300000`, `deadline`,
`scope:frozen-fiction-native-inquiry@v1` and `runner_sha256`. **No grant is created
by preparation.** After approval only, the command is:

```sh
python3 scripts/engine-inquiry-validation.py target/qualitative-inquiry-prepared-2 --grant APPROVED_GRANT.json
```

The runner derives each next request from the current native question view. It
checks source/binary/transitive script pins, original corpus, state and campaign
before effects; every dispatch has durable intent. Both reviews are actual separate
provider invocations with pinned candidates. Semantic independence remains limited
by using the same model, not proven by reviewer labels.

## Preregistered interpretation

Assess whether the inquiry is grounded in active desires and an honest uncertainty;
whether selected records are relevant without claiming unseen support; whether
reflection faithfully distinguishes actual access from the truth of assertions;
and whether the next possibility substantively uses that reviewed understanding.
No change, disagreement or abstention can be appropriate. Mere persistence, JSON
validity, wording changes or a model's narrative do not establish culmination,
choice improvement, emergence or causal superiority. No baseline is present.

Local controls use an explicitly authored fake provider with real native commands
and temporary fixture reads. They cover all six stages, both reviews, privacy,
receipt/question tampering, authority drift, stale context, interruption, unknown
provider/read outcomes, and no-repeat terminal replay. They are wiring evidence
only. Existing legacy export tests remain unchanged.

## Frozen preparation evidence

Prepared directory: `target/qualitative-inquiry-prepared-2`.
Manifest digest: `2dc523dabb47e4f3ff86fc8d58ca578e64d4213338bfacbcba98431a7cf9e6e8`.
Initial body digest: `526dd8e0a3dea4eac562d924c14e019a8e90cf0d26107598463163c65d0eba4a`.
Initial body size: **5,161 bytes**. Approval is pending; no grant/admission exists.

Verification: four new local validation tests passed (including full six-stage
fake-provider flow, unknown/interrupted effects, expired inspection and payload
drift), plus one existing fresh-process inquiry test and five unchanged legacy
retrieval tests: **10 passed**. Python compilation passed. No Rust changed in this
preparation. Independent review identified a missing inspection deadline recheck;
fixed with 30-second headroom matching the native subprocess timeout and a test
that expires the clock immediately after selection. Local terminal counters are
explicitly attempted slots; campaign ledger status remains consumption authority.

Suggested approval question:

> May I run up to six CheaperInference gpt-6-luna calls and one read-only inspection
> of the same six fictional notes, beginning with their index only? The native
> engine would propose a question, have it independently reviewed, choose what
> to read, reflect on the actual returned text, independently review that
> interpretation, and preview a next possibility. Reviewed advisory updates stay
> in isolated test state. No private data, outside files or real-world actions;
> same $5 total, 200-call and 60-operation caps, with a $0.30 conservative allowance.

A second review also required a fresh call deadline check after payload assembly.
That guard and a slow-assembly/zero-dispatch control pass. The earlier prepared-1
packet remains unchanged and is stale against the corrected runner; only prepared-2
is proposed for approval.
