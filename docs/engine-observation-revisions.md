# Adapting an ongoing episode to revised evidence

[Bounded foreground collection](engine-local-collector.md) can now submit revisions
automatically from explicitly mapped local files and invoke ordinary continuation.

`tsr engine direction-observe --observation FILE` records an operator-observed content
revision to already admitted notebook evidence. It does not create a desire, propose
an action, add evidence IDs/topics, or grant authority. This is an observation ingress
command, not an automatic external watcher or a generator tool.

The JSON contract is:

```json
{
  "state_root": "CURRENT_ROOT_FROM_DIRECTION_REQUEST",
  "reason": "Why the admitted source observation changed",
  "notes": [{"id": "return", "topic": "decisions", "text": "Revised observed source text"}]
}
```

Use a current root from `direction-request`; IDs and topics must already exist. Text
and reasons are bounded. Unknown fields, duplicate IDs, new IDs/topics, and stale roots
are rejected before any record is appended. Identical content is an exact no-op.
Changed content appends a source-attributed record with old/new evidence hashes and
the observer's before/after vectors. Old events and CAS blobs are not rewritten.

The controller invalidates inspection and account membership for changed IDs. The
same standing wants and generated subsets therefore reopen when their current
coverage drops. Unaffected evidence remains available. Reverting to an earlier text
is another observation change; it does not resurrect old inspection satisfaction.
Lineage, preference IDs, historical commitment, outcomes, and action budget remain.
This path does not reset a cycle limit or inference allowance.

New returns bind the evidence revision and pre-action notebook. Old returns inherit
their revision from their position in the append-only history during reconstruction;
their saved bytes remain unchanged. Historical success/failure contributes to the
coarse related-context progress rate. Forecast residuals transfer only within the
same content revision. This conservative boundary avoids using an old source's
prediction errors as current calibration; the retained cross-revision progress rate
is an explicit engineering assumption, not proof that changed text has the same meaning.

Old contrasts cannot authorize new refinements after a content change. A new action
and measured return must precede refinement. Current observations determine eligibility;
historical receipts are exposed with revision identity and do not establish present
satisfaction. Counterfactual audit choices now return null when the policy would abstain.

## Operator flow

```sh
target/debug/tsr engine --directory RUN direction-request
# Write only the independently observed note revision to observation.json.
target/debug/tsr engine --directory RUN direction-observe --observation observation.json
python3 scripts/engine-directions.py --directory RUN --model qwen2.5-coder:32b \
  --endpoint http://127.0.0.1:11435 --calls ORIGINAL_FROZEN_LIMIT
```

Use an authorized installed local service and the same frozen journal configuration.
Resuming does not increase its allowance. No next task, candidate, or reference answer
is required: the driver generates from reopened desires and current observations.

## Deterministic checks and boundaries

The integration control completes an episode, admits a subset refinement, compares
unchanged and changed observations, verifies reopening with unchanged preference IDs,
rejects stale offers and new authority, checks preserved history and revised calibration
applicability, restarts, restores coverage, and then tests a contradictory source revision.
An additional control checks that an explicit original withdrawal is not reopened in
attention. This is not comprehensive preference revocation: scoring still retains the
original standing obligations and generated descendant hints have their existing
admission behavior. The observation command does not alter those authority mechanisms.

Affected checks: 81 CLI and 12 domain library tests passed (one explicit receipt
diagnostic ignored normally). Offline CLI build, workspace/all-target Clippy with
`-D warnings`, changed-file formatting and diff checks passed. Logs are
`target/observation-checks.log`, `target/observation-build.log`, and
`target/observation-clippy.log`. Independent review prompted the withdrawal-preservation
and counterfactual-abstention corrections and cleared the live continuation.

All satisfaction remains typed note coverage and word budget. Source truth and
qualitative understanding are not verified. The observation caller is responsible for
the source revision; a generated assertion alone is neither observed truth nor permission.
The shared decision kernel is unchanged; revision tracking belongs to the notebook
adapter (`shared-obligation-kernel@v3`). Project semantic review remains independent.

## Live changed-environment continuation

On 2026-10-21, an isolated copy of the completed `target/capacity-live/run` episode
was continued at `target/observation-live/run`. The original episode and receipts
were preserved. Source inspection of the implemented revision rule justified updating
the existing `return` note to describe revision-bound residuals and retained historical
progress rates. This was an independently introduced observation, not a generated
claim, new want, candidate, or reference answer.

The unchanged-content control returned exactly the same state. The changed-content
command reopened observation from `[1,1,1,1,1]` to `[2/3,2/3,1,1,1]`; it did not reset
the existing three action cycles or alter the five wants and their lineage.

Using installed local `qwen2.5-coder:32b`, the driver made **three additional calls**:

1. Journal call 6 generated alternatives and selected inspection of the revised
   `return` note. Current inspection became complete; account coverage stayed 2/3.
2. Call 7 proposed refinements; both were declined for citing already-covered parts.
   No additional wants or authority were admitted.
3. Call 8 generated alternatives and selected composition including the newly inspected
   note. All five operational predicates became 1, with five action cycles total.

Historical progress evidence affected scores. Old-revision residuals were zero in
call 6. In call 8, learned score for full composition was `0.248210` versus `0.163333`
without learning; both selected it. No winner-change or learning-superiority claim
is made. The caller supplied no next task or authored action after the observation.

Restart returned exactly the same satisfied state with no additional inference.
Eight journal calls exist in total, including five from the original run; the frozen
ten-call total was not raised. All calls were terminal, each subject to the existing
240-second timeout. The temporary cloud-disabled loopback service, PID 14490, was
stopped. No paid/external inference, downloads, credentials or publication occurred.

Evidence: `target/observation-live/{before.json,unchanged-state.json,changed-state.json,
final-state.json,restart-driver.json,verification-summary.json,driver.log}` and the
run's generated offers, reviews, turns and journal. The original journal is an exact
prefix of the continuation; original JSON receipts remain byte-identical.

Final root: `7299fc545c24ff7d4ce3e109a52d198423e47dda483c8a99afa1c64aac1ab8fe`.
Evidence revision: `bd0bef93f3937e5cb9369d669c8561cd730cb6ad809e9b1cd06e813d59301321`.
Journal SHA-256: `5d9d728663812765309f7eabf3fb549124970281b64c29f5024eeee1cf3361c8`.

This verifies bounded adaptation to an observed environment change within an ongoing
episode. It does not verify automatic source discovery, arbitrary new evidence
admission, qualitative understanding, or an unlimited run; budgets still terminate.
Final independent review verified the unchanged control, three terminal continuation
calls, revision-sensitive learning, restored satisfaction, restart equality, and
preserved receipt bytes. It found no blocker for the local commit.
