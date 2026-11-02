# Requirements, review, repair, and the next episode

The functioning local scope is a persistent coding cycle with independently reviewed goals:
operator requirements become ledger preferences; generated proposals act only on admitted
source; execution resolves test expectations; an independent semantic review resolves a
separate acceptance expectation; unmet dimensions guide another generated proposal. A later
episode can explicitly carry reviewed observations forward under preserved requirements, with new desires starting without outcome evidence.

This extends the existing TSR cycle rather than replacing it with a test gate. It keeps the
PRD's two records separate: the acceptance forecast residual updates empirical expectation,
while named unmet requirements produce contrast and keep preferences open. Historical test
success remains test success. Generated proposals cannot manufacture a review or satisfy goals.

## Practical completion criteria and verified behavior

1. **Persistent prospective state.** Requirement registration appends operator-origin expansions
   with the original goal's preference lineage. Requirements survive independent process restarts.
   Acceptance closes their statuses together. A changed source invalidates completion; the
   read-only state projects stale fulfilled aims as open, and the next explicit repair records
   their reopening. New requirements withdraw superseded aims without deleting history.
2. **Belief and expectation from return.** Per-requirement met counts and an overall acceptance
   estimate are folds over independently reviewed candidates. Both use a Beta(1,1) smoothing
   prior, reported as engineering proxies, not certainty. Before commitment, the engine freezes
   a separate `independent_operator_acceptance@v1` forecast. A decisive review records its
   residual separately from named contrast. No new stored weights or model training are involved.
3. **Repair informed by feedback.** The next request can contain explicitly published guidance,
   unmet dimensions, reviewed evidence IDs and estimates. The generator can produce new exact
   edits or multi-file edit sets; it is not limited to choosing a pre-existing action ID.
4. **Several desires together.** Current semantic preferences support candidate lineage without
   being silently inserted into executable test criteria. The local regression composes pricing
   and discount edits, verifies both outcomes, and closes both preferences with one independent
   review. Passing tests alone never closes them.
5. **Carryover across episodes.** `acceptance carry --from EPISODE` imports reviewed observations
   from one explicitly selected local engine store. All source requirements must be preserved exactly; the destination may add new ones. It copies
   booleans and provenance, not source, private evidence, transcripts or fitted scores. Imports
   are idempotent at the same source root; a newer import replaces that source episode's older
   snapshot, including withdrawn/undecided judgments. Duplicate receipt observations are counted
   once. Evidence for a smaller objective updates only matching dimensions; it does not update the overall acceptance estimate of the enlarged goal. Changed or removed source requirements are refused rather than silently relabeled.
6. **Bounded resumable execution.** After a passing test but independent rejection, the transport
   resumes only when current sanitized guidance covers every unmet requirement. The same call,
   cost and token limits still apply. Without guidance it waits; at exhaustion it stops; after
   acceptance it completes. A new transport attempt does not infer new spending permission.

The regression's counterfactual uses identical new source and requirements: without earlier
reviewed outcomes its fixture generator chooses pricing first; with the earlier rejection it
chooses discount first and bundles both repairs. Its forecast changes from 0.5 to 1/3 before
execution. It reads the current request and constructs fresh patches/inspection handles rather
than playing a numbered action queue. This is a deterministic synthetic inference fixture;
it demonstrates state and control flow, not an LLM's general ability to reason.

## Operator workflow

Initialize and prepare a project using the existing [project workflow](engine-project-workflow.md),
then register requirements and review results using [independent acceptance](engine-project-acceptance.md).
The original requirements and review evidence remain private to the controller. Publish only
sanitized guidance that is suitable for the proposal provider:

```json
{
  "requirements_revision": "<current requirements event ID>",
  "decision_record": "<current review event ID, or null before the first review>",
  "guidance": {
    "coverage": "Preserve every supported API entry while repairing its metadata.",
    "metadata": "Use the documented metadata contract for every supported entry."
  }
}
```

```sh
tsr engine --directory RUN project acceptance feedback --file feedback.json
tsr engine --directory RUN project request
```

This writes local state; it sends nothing. Guidance is limited to current requirement IDs and
512 bytes per text, up to 16 requirements. Private review text is never automatically copied.
Publishing stale guidance is refused; republish after a new review. Initial guidance can expose
operator-approved requirements before any result. Publication does not expand editable files,
commands, spending limits or independent acceptance authority.

Within an already authorized run, resume using its unchanged transport arguments. A locally
prepared proposal can also use `project run --response response.json`, which still invokes the
sealed executor. No model or external service is needed to replay saved/generated local responses.

For a later initialized episode, register the preserved requirements (plus any new ones) and publish its guidance, then:

```sh
tsr engine --directory NEXT project acceptance carry --from PREVIOUS
tsr engine --directory NEXT project request
```

The resulting request contains the carried observations' estimates. Added requirements have zero observations; `fully_reviewed_candidates` excludes historical judgments that did not cover the enlarged goal. Importing a previous accepted
outcome does not accept the new source: the later episode still needs its own execution and review.
The operator chooses the episode and matching requirements; this is not automatic mining of files
or sessions. At most 128 reviewed candidate observations may be carried in one episode snapshot.

## Verification

Rust 1.90 commands run:

```sh
cargo test -p tsr-cli --lib --test engine_cli --offline
cargo clippy -p tsr-cli --all-targets --offline -- -D warnings
python3 -m unittest discover -s scripts -p 'test_engine_*.py'
```

Results: **57 library + 2 CLI tests passed**, Clippy passed; **33 Python tests passed**, ten
optional sealed cases skipped in that default invocation. The additional dedicated sealed
end-to-end case was then exercised explicitly with the existing pinned image:

```sh
TSR_ENGINE_TEST_IMAGE=sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6 \
TSR_ENGINE_TEST_KEEP=1 python3 -m unittest discover -s scripts -p 'test_engine_acceptance_loop.py' -v
```

That case passed in 39.476 seconds. It used a loopback synthetic provider, real offline sealed
Cargo, independent source inspection and separate CLI processes. The first green patch had a
wrong special case; rejection plus published feedback led to a correct second patch within the
same two-call fixture budget. A later episode carried both observations and produced a complete
bundled patch on its first fixture call. Both episodes ended independently accepted. The three
calls were local fixture completions, with fixture accounting only, no credentials or external
inference. Original example files and protected test bytes remained unchanged. Structured evidence:
[engine-continuous-cycle-result.json](engine-continuous-cycle-result.json).

A separate offline regression proves that fresh rejection/feedback cannot exceed an exhausted
step budget. Other regressions cover stale feedback, rejection without publication, private-text
exclusion, corrected review imports, duplicate imports, independent forecast residuals, multiple
fulfilled preferences and refusal to keep repairing an already accepted goal.

## Operational boundary and remaining work

The real RTS two-call demonstration is unchanged and remains unaccepted; all 56 protected files
and the original repository are intact. No guidance was published to that live run and it is not
ready to dispatch a repair. Its two paid calls remain consumed. A real-model corrective validation
would need a **new explicitly bounded authorization** for `api.cheaperinference.com`, covering the
new sanitized acceptance guidance/estimates plus the existing admitted source context. The $100
aggregate ceiling remains a ceiling, not a renewal of the consumed run. No such call was attempted.

This delivers a functioning, independently reviewed local loop and explicit cross-episode state
carryover. Reviews still depend on operator judgment; evidence text is not a proof of correctness.
Matching requirement text does not prove statistical exchangeability across repositories. The
empirical estimates have thin support and are not calibrated benchmarks. Automatic long-running
model improvement, fine-tuning, superiority over a baseline and the PRD's M3 corpus/interval
acceptance criteria remain unverified. This work publishes, pushes and deploys nothing.
