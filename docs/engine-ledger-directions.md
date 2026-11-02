# Standing desires → directions → return

The current loop also supports [evidence-grounded generated refinements and context-sensitive expectation transfer](engine-refinements.md).

This local entry point connects the existing preference ledger to a repeated decision
cycle. The operator supplies standing wants and admissible evidence once, rather than
a sequence of tasks. An installed local model generates alternative directions; the
Rust controller quarantines invalid alternatives, requires at least two valid distinct
alternatives, selects one, freezes its forecast, performs its typed notebook action,
observes the result, and supplies that return to the next generation call. Each
iteration runs a new CLI process against the persistent ledger.

This is a bounded evidence-notebook adapter. It establishes a connected decision
process in that domain, not a general autonomous coding agent or a proof of TSR theory.
The earlier coding workflow remains separate. There is no authority to execute model
commands, modify another repository, contact external services, or publish anything.

## Run locally

From the repository root, using the existing Rust 1.90 toolchain and installed Ollama:

```sh
cargo build --offline -p tsr-cli
# Separate terminal; stop this process when finished. No model downloads.
OLLAMA_HOST=127.0.0.1:11435 OLLAMA_NO_CLOUD=1 OLLAMA_NOPRUNE=1 OLLAMA_NUM_PARALLEL=1 OLLAMA_MAX_LOADED_MODELS=1 ollama serve
```

In another terminal, choose a new run directory:

```sh
target/debug/tsr engine --directory target/my-directions direction-init \
  --spec examples/engine-directions/standing-wants.json
python3 scripts/engine-directions.py --directory target/my-directions \
  --model qwen2.5-coder:32b --calls 8
target/debug/tsr engine --directory target/my-directions direction-request \
  > target/my-directions/state.json
```

The example contains standing wants and repository facts, not proposals, a reference
answer, or a prescribed next task. Substitute another **already installed** local model
name if appropriate. The adapter checks local inventory and rejects cloud models,
redirects, non-loopback endpoints, and tool calls. A call has a 240-second timeout and
2,048 output-token ceiling. The call cap is cumulative and frozen in the run journal;
restart with identical arguments. A transport failure leaves an uncertain-call record
and stops instead of silently repeating. Two identical invalid offers stop the run.
A run can remain unresolved or exhaust its budget; neither is reported as success.

`direction-request` exposes all standing wants, current eligibility/selection trace,
observations, notebook and exact evidence account, prior returns, and terminal status.
`directions-N.json` contains the decoded model proposal with controller-bound state
root; `turn-N.json` contains the controller's assessments, chosen action, and return.
The JSONL local-call journal records model digest and proposal/context hashes.
No raw provider chat body is committed. A standalone `deliberate --response FILE`
command is available for adapters and tests; its offers are not genuine inference
merely because they pass validation.

## What determines a decision

- **Desire:** admitted standing preferences retain lineage. Existing ledger diffusion
  selects eligible preferences. A read-only eligibility projection removes currently
  satisfied wants; it does not delete them or erase their admission. They become
  eligible again if later notebook changes leave them unmet.
- **Intuition:** the model creates 2–4 different directions from current wants,
  known evidence and returns. A direction names its purpose, supports, anticipated
  effects, and a typed action. There is no supplied candidate list or cycle-index plan.
- **Belief:** the last productive commitment imposes a small switching cost while
  its supported wants remain unmet. It is capped at one quarter of positive net
  expected benefit (and 0.04), so it cannot eliminate every productive option. A failed
  return releases that persistence.
- **Expectation:** action-family progress rates and mean forecast-effect residuals
  are folded from recorded returns. They discount and calibrate newly proposed
  effects before ranking. Typed action invariants bound those forecasts: inspection
  cannot erase an account and composition cannot erase known evidence. Unchosen
  directions never count as failures.
- **Joint choice:** rank expected improvement in the worst-served selected want,
  plus one quarter of mean improvement, minus effort and switching cost. This exposes
  conflicts between coverage and brevity instead of counting supported criteria.
- **Return:** the controller materializes and observes the notebook, separately logs
  forecast residual and named unmet criteria, and builds the next generation context.
  Generated refinements receive a bounded notebook review before entering this
  domain’s selection view; their semantic wording remains unverified and they grant no
  authority. Global ledger admission is unchanged.

All fractions, calibration, continuity costs and ranking constants are explicit
engineering proxies. They are not measurements of qualitative TSR concepts or learned
stored weights. The action-family estimator is deliberately coarse and can overgeneralize.

## Domain and evidence limits

`inspect` chooses admitted note IDs and adds their text to available evidence.
`compose` materializes an account from inspected IDs and a bounded generated title.
The controller permits no invented evidence claims: account text is the original note
text. Notes are data, never executable paths. The observer measures topic coverage
and a word budget, not human understanding, factual truth of source notes, or prose
quality. This gives an independent mechanical return rather than model self-approval,
but it is not an independent general semantic verifier.

Standing wants are operator-admitted. Evidence-backed typed refinements can be
autonomously admitted within the notebook scope; unsupported proposals remain rejected. The process cannot invent external authorization or resolve
fundamentally incompatible wants. Other action domains need their own bounded execution
and independent observation adapters. The coding pipeline is not silently wired in as
one. Fine-tuning, broad autonomy, and superiority/statistical M3 acceptance remain
unproven and are not implied by a successful notebook run.

## Targeted checks

```sh
cargo test --offline -p tsr-cli --lib engine::deliberation
python3 -m unittest discover -s scripts -p test_engine_directions.py
```

Controlled tests hold observations and offers fixed while intervening separately on
selected desire, commitment, and historical expectation, with repeated unchanged-input
and restoration controls. The desire test intervenes on selected standing focus; it
does not establish autonomous formation of new values. Expectation transfer is tested
across novel direction identities in the same typed action family; it is not semantic
embedding-based generalization. These are synthetic causal
checks, not model-discovery evidence. Other tests verify actual failed returns preserve
unmet wants and change calibration across restart, output-driven selection changes,
unknown-note rejection, and stale-offer rejection without ledger mutation. Live local
inference evidence is reported separately in `engine-directions-evidence.md`.
