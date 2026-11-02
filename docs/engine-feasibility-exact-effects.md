# Independent feasibility diagnosis and exact-effect correction

This analysis is not a generator prompt, runtime candidate, reference patch or
executed answer. It reads the preserved support-live specification, receipts and
controller code. No proposed action from it was submitted to the live run.

## Preserved-state diagnosis

State root: `3d736f20dfa9683eb7b1d6b0c7d1cf2589fdb4ccc16bf4950e01188f1bf51c4f`.
Coverage is 0.5 and brevity 0.8. The original max_cycles is eight, with two actual
returns. Authority permits only inspection of admitted IDs and local composition.
Neither remaining action budget nor external permission blocks this analysis.

The admitted immutable note lengths are three and seventeen words. The second
note remains unread by the generator. Independent analysis can read the original
owner-provided specification; its hidden text and this analysis are not inserted
into a model request.

- **Joint full satisfaction is impossible:** full coverage requires twenty source
  words plus a nonempty title, exceeding the four-word limit. This does not prove
  that every intermediate improvement is impossible.
- **Mechanical progress exists:** the known account uses three source words and
  a two-word title. A smaller nonempty title length can improve brevity while
  retaining coverage. This is a word-count feasibility witness, not a supplied
  title or runtime action. The schema permits such compositions and distinct
  dependent inspection plans; two valid alternatives need not both be selectable.
- **v8 policy selection was blocked:** the only previous composition predicted
  brevity 1.0 and observed 0.8. `decision::expectation` normalizes its positive
  related weight, giving a 0.2 residual. Every related composition therefore had
  scored brevity at most 0.8 because raw forecasts cannot exceed one. Short-only
  coverage cannot increase. Gain was nonpositive before effort/continuity costs.
  The exact fallback still required a positive estimated score, so it could not
  rescue independently established progress.
- **Inspection does not solve this gate:** it changes knowledge, not the account.
  Plans including the longer note incur a large standing brevity loss. Full
  coverage's best possible brevity is 4/21; the conserved valuation is negative.

An independent read-only reviewer confirmed this controller bottleneck. Repeated
model inability was not proof that no useful action exists. Repeated proposals occurred under deterministic sampling
(temperature zero), but further inference alone
could not overcome the v8 selection defect.

## Scoped correction

The notebook adapter now supplies equal lower and upper bounds for independently
simulated deterministic predicate effects, for direct actions as it already did
for planned terminal effects. Shared kernel clamping therefore prevents both
optimistic manufactured progress and pessimistic residual suppression. The adapter
version is `shared-obligation-kernel@v9`; new assessments include `projection_basis`
identifying `sealed-notebook-transition@v1`. Old receipts remain unchanged and
readable through the optional field.

Raw generator forecasts, their residuals, evidence provenance, effort, continuity,
standing losses, authority and support validation are unchanged. Historical
progress probability still weights valuation and can change choice; it is not
uncertainty about an exactly determined local transition. Simulated effects remain
projections until dispatch and return. Source truth is not established. Unknown
project/domain effects retain their existing intervals; no global certainty rule
or shared domain contract changed.

Raw support must still predict improvement. The fix does not turn a pessimistic
unsupported proposal into an accepted action or generate an action itself.

## Checks and live boundary

A focused positive-residual regression failed before the fix and passed afterward:
`target/exact-effect-before.log` and `target/exact-effect-after.log`. It also checks
raw forecasts unchanged, no-op rejection and proposal-consistency rejection.
The causal commitment/expectation fixtures now balance actual effects rather than
understating a composition forecast; winner reversals and restoration controls
remain. Their hypothetical failed-inspection history is a synthetic intervention,
not an execution receipt. The no-progress fixture retains all three standing wants
while only the two unmet ones remain selected after a completed inspection.
Independent review cleared those nonprotected fixture changes.

Final pre-live checks: CLI library 96 passed, one existing diagnostic ignored;
domain tests 15 passed; Python driver tests 50 passed. Clippy/build status and live
results are recorded below. Protected tests, original target repositories and old
receipts are unchanged. Corpus remains 151 qualified tasks; eligible independent
mature episode count remains unknown.

The new model-facing information is only the general exact-projection policy and
clarification that useful partial progress need not satisfy all wants at once.
No feasibility witness, hidden note length, title, candidate or action plan from
this analysis is supplied to the generator. The authorized new batch is bounded
to six local calls, 240 seconds each; composition and assessment each consume a
slot. No unchanged deterministic retry is authorized by this diagnosis.

### Terminal verification result

Clippy (`cargo clippy -p tsr-cli --all-targets -- -D warnings`) and
`cargo build -p tsr-cli` passed using Rust 1.90.0; logs are
`target/exact-effect-clippy.log` and `target/exact-effect-build.log`.
Final independent read-only review found no remaining implementation blocker.

Batch `exact-effects-v1` dispatched one of six authorized local calls. Request 20
used the changed general policy context and returned explicit possibility-stage
inability. It did not produce a frozen proposal, request assessment, or execute
an action. The response is preserved as
`target/support-live/run/inability-20.json`, SHA-256
`b36f4346371e9325c326ae8e07667a828ebc0ab7cb321936c391d25b287a89f4`.
The journal and `target/support-live/exact-effects-live.log` retain the unchanged
state root and observation [0.5, 0.8, 0.0]. Request 19 inability and request 13
unknown-abandoned outcome remain intact. No uncertain operation was retried.

The driver exited successfully after recording inability. Temporary provider
PID 85010 was terminated; its session exited zero. Five slots remain undispatched.
No paid or external inference occurred. This is a demonstrated controller fix,
not demonstrated live recovery. Generator inability remains distinct from joint
constraint infeasibility and from the formerly accidental controller exclusion.

The next concrete investigation is bounded diverse possibility search after this
recorded temperature-zero inability, with sampling configuration explicit in the
request receipt and no authored witness or hidden note data. Repeating the same
request unchanged is not justified. Any new trial must consume the existing five
remaining slots (two-stage attempts require at least two) and preserve inability
as a valid outcome; no successful action or full-engine completion is claimed.
