# Reviewed public desire meaning, independent of task acceptance

Local slice following the four-call complementarity result. No provider calls,
external exports, target execution or model-output adoption occurred in this slice.
Original held-out/tuned 0/2 results and the null-aspiration shadow remain unchanged.

## Implemented boundary

The native project previously rebuilt relational parent meanings exclusively from
published acceptance guidance. A new append-only public-desire revision view now
retains original public meanings and allows separately reviewed development of those
existing identities. It preserves participating parent origins, generator attribution,
review attribution, exact ancestry, clarification/development labels and typed basis
(unknown, actual current receipt, or applicable attributed judgment).

`Suggestion::PublicDesire` travels through the existing native proposal preview and
cycle advance. It is deferred, never executable. Advancing a reviewed proposal stores
its generated origin; a separate `project desire-admit` operation reviews/adopts the
public meaning. Native subsequent requests read that admitted meaning through the
same algorithm. No seed/experiment/patch sequence or winning action is introduced.

Task criteria, configuration, protected tests, acceptance evidence and permissions
remain separate. The public view creates no utility weights or scored obligations.
Unknown basis stays unknown; text is not proof of an observation or emergence.
Clarification is explicitly distinguishable from claimed development. Exact unchanged
text is refused as an update; arbitrary semantic paraphrase cannot be detected by
schema and must be judged by the independent reviewer. A model declaring development
is not automatically endorsed as having produced new meaning.

Origins are stored with admissions so later acceptance publication or requirement
changes do not erase public history. Old and new identities can coexist. Withdrawal
removes an identity and dependent guidance transitively; history persists. Restoration
is explicit. Superseded revisions cannot be restored through a stale status request.
Private review notes remain stored locally; only their digest appears in the view.
Reviewer/generator labels record attribution, not cryptographic independence.

## Compatibility and local use

Old histories without public-desire records retain v1/v2 relational input bindings.
New records append without rewriting old events. Once used, the native relational
input is `project-relational@v3`, with `parents` (active evolving meanings),
`acceptance_parents` (published immutable-task mapping), and `public_desires`
(current/history/origins/status). Experiment mappings still use acceptance parents.
Existing Python export adapters reject this version: no private or new public text
is silently authorized for transmission. No database migration or replay rewrite.

On an already initialized local project, inspect the exact native contract:

```sh
target/debug/tsr engine --directory "$RUN" project cycle request
target/debug/tsr engine --directory "$RUN" project cycle preview --response RESPONSE.json
target/debug/tsr engine --directory "$RUN" project cycle advance --file REVIEWED_RESPONSE.json
target/debug/tsr engine --directory "$RUN" project desire-admit --file DESIRE_REVIEW.json
target/debug/tsr engine --directory "$RUN" project desire-status --file DESIRE_STATUS.json
```

The request advertises `public_desire_contract`. A proposal carries binding, existing
active identity, exact prior revision (null for a legacy parent), participating parent
IDs including itself, bounded meaning, change label, typed basis and generator identity.
Admission takes current state_root, generated proposal event ID, index, distinct reviewer
and bounded review note. Status takes state_root, identity, exact latest revision (null
for an unrevised original), and withdrawn boolean. These are local state commands;
they do not invoke models, workers or formal task acceptance.

Current scope is ordinary `propose`, not a new option in the deliberately narrow
`reconsider` branch. Already accepted/closed tasks still stop; this does not reopen them.
This slice develops existing public parent meanings/relationships; it does not create
arbitrary new independently scored desires or implement the full continuous field.

## Controls and interpretation

Authored deterministic fixtures exercise actual native proposal preview/advance,
separate admission, request projection and persistent restart, with zero target commands.
Controls include acceptance/configuration invariance, unknown versus observed basis,
clarification versus asserted development, unchanged input/read-only root, stale ancestry,
forged authority field, self-attribution review, replay, publication changes, requirement
changes after admission, transitive withdrawal and retained relational history.

These prove state flow and boundaries, not generated emergence or better decisions.
Independent review found two defects during development (publication-dependent replay
and only one-level withdrawal); both were repaired before completion. See the review
record and final validation results below.

## Smallest meaningful later live validation (not authorized here)

Use one already approved non-private task with two published parents. Freeze exact
v3 public projection and immutable acceptance side; explicitly review the new export
profile first. One native generation call may propose a public meaning development
or legitimately abstain. Only a separate source-bound independent review may admit it.
A second call through the same native proposal algorithm can test whether the admitted
meaning is actually used in a relevant next possibility. No execution is needed to
establish this flow; no-difference and paraphrase remain legitimate outcomes. This would
be evidence of integration, not causal superiority. A controlled causal comparison
would require further separately specified calls, not invented authored winners.
Campaign remains 190/200 calls, 57/60 groups, $0.160693/$5, untouched by this patch.

## Verified checks

- `cargo check -p tsr-cli --offline`: passed, including CLI handlers.
- `cargo build -p tsr-cli --bin tsr --offline`: passed.
- `cargo test -p tsr-cli --lib project:: --offline --no-fail-fast`: **130 passed,
  5 failed, 3 ignored**, 118.90 seconds. All eight new desire tests passed. The five
  failures reported `Operation not permitted` in existing loopback verifier stubs.
- Supported per-command permission for those local sockets, with unchanged tests:
  `cargo test -p tsr-cli --lib project::acceptance::reviewer::local --offline
  --no-fail-fast`: **10 passed**, 9.97 seconds, including all five blocked cases.
  No deadlines/assertions weakened; no inference or target worker invoked.
- Whitespace check passed. New module formatted with installed Homebrew rustfmt;
  the pinned 1.90 toolchain lacks rustfmt. No software installed.

This is affected native-project coverage, not a fresh whole-workspace/Python green
claim. Existing external Python profiles intentionally cannot dispatch v3. All
native regression observations are authored fixture evidence, not live culmination.
