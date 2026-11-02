# Shared bounded near-tie selection

PRD §B8 calls for margin abstention and optional epsilon exploration over near-ties.
The notebook and adaptive/project paths now share `tsr_domain::selection::select`.
This operates on engineering proxy scores, **not** validated Tier 1 probabilities
or a calibrated Tier 2 release. It cannot generate missing alternatives.

## Activation and historical behavior

Existing runs without a policy retain their legacy argmax/propensity-1 behavior.
No historical receipt is rewritten. Activate the new policy explicitly using a
current `state_root`, a nonnegative finite `margin`, `epsilon` in [0,1], a u64 seed,
and an operator reason. The same policy command works with either adapter:

```
target/debug/tsr engine --directory RUN decision-policy-view
# Write policy.json using the returned state_root.
# {"state_root":"CURRENT_ROOT","policy":{"margin":0.02,"epsilon":0.0,"seed":19},"reason":"Require separation before committing"}
target/debug/tsr engine --directory RUN decision-policy --update policy.json
target/debug/tsr engine --directory RUN decision-policy-view
```

`decision-policy-view` exposes settings and durable selection traces. The update
is root-bound and accepts only operator provenance. Exploration requires explicit
`epsilon > 0`; zero abstains when at least two eligible scores are within the
margin, including an exact tie at margin zero. No eligible candidate also abstains.
No action is executed and no return is learned from abstention.

The notebook admits only existing weak-support-fallback-eligible alternatives.
The adaptive/project path retains existing action/patch validation and selects
only positive-score options under the new policy. Actual scoped execution and
independent accepted-export review remain separate. Unsupported or unauthorized
actions cannot enter exploration by changing epsilon.

## Reproducible selection and evidence

When exploration is enabled, the near-tie set receives an epsilon mixture:
nominal mass 1-epsilon on the stable best candidate plus epsilon uniformly over
the near-tie set. Outside candidates have probability zero. A separated best has
probability one. Canonical identity breaks exact ordering ties, not provider order.
A hash of policy version, operator seed, pre-offer state and sorted candidates
provides a reproducible 53-bit pseudorandom draw. Integer bucket widths give the
exact logged probability under that draw law (including floating-point rounding).
This is a reproducible pseudorandom policy, not a claim that a human-selected seed
or observational run establishes causal unbiasedness.

`bounded-selection-trace@v1` stores version, policy, state, scores, all probability
masses, draw, selection and reason. `Committed` stores the selected propensity.
Full traces replay without mutation; policies and old commits remain historical.
Configured-policy notebook runs suppress the older argmax-only learning
counterfactual rather than falsely treating exploratory selection as that control.

The local driver handles `selection_abstained` as terminal, records a controller
response explicitly distinguished from an observed action return, clears frozen
possibilities and stops. It does not spend more calls trying to evade a margin.

## Verification

Deterministic tests cover exact ties, near/non-ties, empty candidate sets, invalid
configuration, exact mixture mass, fixed-seed replay and candidate permutation.
Both adapter tests show durable abstention with no pending action/return, then
explicit opt-in exploration with propensity 0.5. Notebook tests preserve wants and
observations; policy provenance rejects generator-origin settings. Driver tests
cover terminal abstention after the two-stage boundary. These are offline checks,
not live policy superiority evidence.

Final checks: 17 domain tests passed; 105 CLI library tests passed with one
existing diagnostic ignored; 64 Python driver tests passed. Clippy with warnings
denied, CLI build, whitespace checks and disposable CLI policy activation/readback
passed. Independent review found no remaining scoped blocker after the controller
trace provenance correction. The CLI smoke evidence is
`target/selection-cli-smoke.log`.

Logs: `target/selection-domain-tests.log`, `target/selection-cli-tests.log`,
`target/selection-adapter-tests.log`, `target/selection-driver-tests.log`,
`target/selection-clippy.log`, `target/selection-build.log`.

## Corrected-contract live attempt before implementation

The prior journal had 28 dispatched calls against a lifetime allowance of 30.
Request 29 used installed `mistral:latest`, digest
`61e88e884507ba5e06c49b40e6226884b2a16e872382c2b44a42f2d119d804a5`,
seed 7304, temperature 0.7, and the corrected
`valid-first-action-alternatives@v2` contract. It returned explicit inability at
the possibility stage. No candidates, assessment, execution or partial progress
resulted. One slot was consumed; one remains unused. The preserved root remains
`3d736f20dfa9683eb7b1d6b0c7d1cf2589fdb4ccc16bf4950e01188f1bf51c4f`.

Evidence: `target/support-live/run/request-29-e7efa9221d6e.json`,
`provider-response-29.json`, `inability-29.json`, and `local-directions.jsonl`;
terminal log `target/support-live/admissibility-live.log`. Temporary Ollama PID
65773 was terminated and its session exited zero. No paid/external inference.
Initial automatic approval review rejected `--calls 4` as exceeding two remaining
calls; a read-only calculation verified that this was the original base budget,
with cumulative limit 30 and 28 dispatched, no new grant. One retry was approved
on that evidence. The denied command did not execute.

## Remaining precise gaps

The installed provider still did not produce valid alternatives in this fixed
conflicting-desire state after the corrected contract. This scoped capability
limit remains, rather than being replaced by authored actions. One remaining slot
cannot verify an entire new two-stage attempt; no additional dispatch is planned.

The new selector is opt-in to preserve existing behavior; operational migration
requires an operator to choose a margin and explicitly whether to explore. These
are existing PRD settings, not new foundational semantics. Validated calibration
still requires independently reviewed mature common-event episode data; 151
qualified tasks do not establish the unknown mature eligible episode count.
The [policy-aware learning intervention](engine-policy-learning-interventions.md)
now records same-draw comparisons and separately reports changed eligibility.
The old argmax-only audit remains deliberately suppressed.
