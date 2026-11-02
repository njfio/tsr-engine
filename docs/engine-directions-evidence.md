# Ledger-driven direction-cycle evidence, 2026-10-21

The connected decision cycle is verified **within the bounded evidence-notebook
adapter**. This is not completion of the broader TSR decision-engine goal. See the
[runnable guide](engine-ledger-directions.md) and
[machine-readable receipt summary](engine-directions-live-summary.json).

## Genuine local generation and continuation

The fresh run at `target/engine-direction-verified/run` used the installed
`qwen2.5-coder:32b`, digest
`4bd6cbf2d094264457a17aab6bd6acd1ed7a72fb8f8be3cfb193f63c78dd56df`,
at the local-only Ollama endpoint. Three actual calls generated alternative directions;
no assistant-written replacement direction, reference answer, or next task was supplied.
The example specified three standing wants and three admitted source notes once.

| Decision | Observed before | Observed after | Action | Next selected wants |
|---|---|---|---|---|
| 1 | 0, 0, 0 | 1, 0, 0 | Inspect evidence | Account, brevity |
| 2 | 1, 0, 0 | 1, 2/3, 1 | Compose partial account | Account |
| 3 | 1, 2/3, 1 | 1, 1, 1 | Compose fuller account | None: satisfied |

Vectors measure topic knowledge coverage, account coverage and brevity—not qualitative
understanding. Decisions 2 and 3 each quarantined two invalid generated alternatives,
retained two valid alternatives, and selected one. Unsupported claims of improving
already satisfied wants were not allowed to change the wants, authority or notebook.
The independent mechanical observer, not model approval, determined actual coverage.

The model generated new alternatives as remaining wants changed, and the controller's
choice produced a different observed return. This establishes outcome-driven generation
in this domain. It does not isolate the causal effect of desire text on model output:
those live calls changed observation and context together. The isolated controller
interventions are described separately below.

Final root:
`881f575e4842c6732f7a334f77712d1737029ce832cfecfbff65fcea3f59fc49`.
Restart returned exactly the same state and `satisfied`, with the journal still showing
three calls. Every cycle used a new CLI process. The account is inspectable through
`direction-request`. The temporary Ollama service was stopped after verification.

## Causal controls and transfer

Eight focused Rust tests pass. The three controlled interventions keep observation and
offer set fixed, change one controller input, and verify a changed choice:

- Selected standing focus: coverage versus brevity reverses the preferred composition.
  This tests the desire-selection channel, not autonomous formation of values.
- Commitment: the same evidence/expectation with a different prior commitment reverses
  a close choice. Restoration and identical-input controls reproduce the initial result.
- Expectation: failed family outcomes change calibrated effects and reverse a choice
  while belief is held absent. Repeated unchanged inputs reproduce identical scores.
  A different direction purpose and note combination inherit the learned family
  prediction/calibration rather than resetting to a new-identity prior.

These are synthetic causal tests, not model-generated discoveries. Transfer uses typed
operation families (`inspect`/`compose`), not semantic embeddings; it is coarse and may
pool unrelated situations. The remaining tests verify durable failed-return feedback,
restart/eligibility, stale/unauthorized offers, quarantined alternatives without changing
standing wants, and action-invariant/commitment constraints that prevent deadlock.

## Recoverable faults found and fixed

Earlier runs remain preserved at `target/engine-direction-live` and
`target/engine-direction-final`. They exposed repeated stale proposals, all-or-nothing
rejection of an offer set, impossible calibrated regressions for read-only inspection,
and a fixed switching cost that could block all positive progress. The implementation
now sends compact actual returns, quarantines invalid alternatives, bounds projections
by operation invariants, and caps continuity cost below positive net benefit.
The older six-call journal was exhausted without changing its budget.

A separate replay of its original decoded sixth offer, against a copied historical
state, now quarantines the two invalid alternatives and advances knowledge from 2/3
to 1. This is explicitly replay evidence, not a new inference call:
`target/engine-direction-quarantine-replay/result.json`. The fresh three-call run above
independently verifies quarantine live on the final Rust implementation.

## Checks and scope

- `cargo test --offline -p tsr-cli --lib engine::deliberation`: **8 passed**.
- `python3 -m unittest discover -s scripts -p test_engine_directions.py`: **5 passed**,
  covering frozen cumulative budget, uncertain-call refusal, terminal restart and
  completion on the last budgeted call.
- Final offline build and workspace/all-targets Clippy with warnings denied: passed.
- Workspace run before the final ranking/quarantine refinements: **783 passed,
  2 failed, 5 ignored**. The final changes received the focused tests above and live
  CLI verification; the two failures were independently diagnosed below.
- Changed Rust files pass formatting; workspace formatting flags only unchanged
  `crates/tsr-batch/src/oracle_receipts.rs:249`. `git diff --check` passes.
- Frozen M0 assets, original test files, and M0 execution/hook code are unchanged.

Logs: `target/direction-{final-unit,python-tests,workspace-tests,final-clippy,fmt}.log`.
Structured proposals and controller receipts remain local under the run folders; raw
provider chat bodies are not committed.

**Still unmet in the broader confirmed goal:** this adapter does not generate new
admitted standing desires, learn general semantic action models, choose among open-ended
real-world action domains, or connect its decision process to the existing coding
executor. Its new refinements remain pending operator admission, preserving the project
contract. The observer measures mechanically defined notebook properties; it is not a
general semantic verifier. General TSR autonomy, fine-tuning, superiority and statistical
M3 acceptance remain unproven. The verified result is a reusable connected mechanism
plus one bounded operational domain, not whole-system completion.

No paid/external inference, target repository edits, push, merge or deployment occurred.
The paid ledger SHA256 remains
`5a3c0ec91b19db3a32b687dd307e9152c62d122c3ef4a68dc0e56ea1255d9952`.

## Runtime-dependent failures inspected

The two failing test files and `src/{exec,hook}.rs` have no diff from `d539259`.
`m0_forecast_default.rs:503` assumes runtime absence and requires exit 3; with Docker
on PATH the observed exit is 126. `m0_harness.rs:232` conditionally enters a real-runner
branch when a runtime is detected, then calls `scratch("run1")` again before reading
its fixture; `scratch` deletes that directory. Both defects and their runtime
sensitivity were already recorded in AGENTS.md at the starting commit.

The identical compiled tests were rerun with `PATH=/usr/bin:/bin`, which excludes the
runtime binary, and both passed without test/source edits. The harness pass exercises
its fake-runner half; its real-runner half explicitly skips when no runtime is found.
Captured output: `target/direction-m0-no-runtime.log`. This is diagnostic evidence,
not a claim that a real sandbox execution test passed.
