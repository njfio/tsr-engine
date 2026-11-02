# B9.1 initial and revision contract alignment

Offline follow-up to Trial 3 (`00032c6`), 2026-10-23. No inference, target commands,
new admission or historical evidence changes. Trial 3's text and semantic quality
remain unknown; its observed structural mismatch motivates this correction.

## Agreement audit and change

The native project validator already distinguishes an initial hypothesis from a
revision. The model prose previously named development fields but left initial
nullness and identity types implicit. The request now explicitly says:

- Initial: `revises: null`; `development` null or absent, never an object.
- Revision: `revises` names an eligible actual admission; development is required.
  `original_revision` is its actual origin admission, `prior_revision` equals
  `revises`. These are hash references, not prose. Parent identities remain unchanged.
- No new insight: null/absent `proposed_development` retains the prior relationship,
  aspiration and discriminator. It does not erase observations or unresolved meaning.

Every B9.1 stage carries a source/state-bound `relational_contract`, including the
experiment, patch and proposal-only stages. Its explicit `allowed_lineage` is derived
from native history: latest nonwithdrawn/nonsuperseded entries, changed evidence
binding, available parents, and traced origin. Multiple independent chains retain
separate origins. An empty list means there is no eligible revision; it does not
forbid an initial hypothesis, grant execution, or prove a desire impossible.

The early Python validator checks the same identity relations before native submission.
It never fills in, repairs or substitutes model values. Native validation still owns
text bounds, catalysts, duplicate hypotheses, history bounds, source/provenance and
admission. Model input remains bounded by the existing 16KiB request limit.

The ordinary native project context prose was aligned too, so other project proposal
paths receive the same initial/revision distinction. The notebook relational schema
already has separate initial/revision branches and actual revision references; its
revision IDs differ legitimately from project admission IDs and were not replaced.
B9.1 has no canned output example or provider-enforced JSON schema: the system text,
explicit reference projection, envelope checks and native validator are the relevant
contract surfaces. This repair adds clarity, not a guarantee of provider compliance.

## Verification

- Offline Rust build passed (`cargo +1.90.0 build -p tsr-cli --offline`).
- Affected Rust project-relational suite: 12 passed, zero failures; fake runner evidence,
  not target execution (`cargo +1.90.0 test -p tsr-cli --lib
  engine::adaptive::coding::project::relational::tests --offline`).
- Focused Python lineage suite: four passed. Real Rust initial null/absent and malformed
  development checks; six-stage positive/negative reference matrix, mixed chains,
  unavailable parents, superseded/withdrawn/unchanged evidence and forged ancestry.
- Full Python discovery: **589 total, 572 passed, 17 opt-in skips**, zero failures/errors,
  129.274 seconds. One new opt-in native sequence accounts for the additional skip.
- Independent review found no remaining blocking issue. Its wording and mixed-chain
  coverage recommendations were incorporated. `git diff --check` passed.

The new full cross-language fixture checks the model projection against the real
controller after provider resource settlement, validates initial and subsequent
revision proposals, rejects invalid development ancestry without state mutation,
and replays the terminal result without redispatch. Inputs and worker returns are
explicitly authored fixtures; no generated semantic success or live B9.1 pass follows.
Initial harness failures (checking guidance during a pending reservation and assuming
all native refusals return an assessment) were corrected without changing production
validation. Logs are retained in `target/b91-lineage-*.log`.

## Readiness and limitations

This fixes the demonstrated contract ambiguity without relaxing original desires,
acceptance or authority. Any future trial must have a fresh admission bound to the
new binary/helper hashes; Trial 1–3 grants stay expired. The six-call/four-group scope
is unchanged, but no new live permission is implied. Provider ability to follow the
clarified contract and complete B9.1 remains unverified. Finite tests, evaluator seed
forecast and fixture revisions do not establish learned calibration, mature data,
semantic culmination or general autonomy.

The separately enabled native fixture completed successfully in **59.600 seconds**:
`PYTHONPATH=scripts TSR_B91_TEST_WORKER_BIN="$PWD/target/debug/deps/tsr_cli-62904bb7a7cdafee" python3 -m unittest test_engine_b91_driver.NativeSequence.test_native_lineage_contract_across_generated_fixture_stages`.
It exercised all six fake provider stages and four cfg(test) worker-return groups,
with zero target commands, followed by no-redispatch replay.
