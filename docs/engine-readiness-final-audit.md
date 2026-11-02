# Engine readiness audit after 381b6d8

This is an implementation/readiness assessment, not an acceptance or maturity
claim. No inference, external transmission or target modification ran in this pass.
The two rejected episodes remain closed. Independent read-only review found no
new demonstrated local core correctness defect after the supporting-context and
diagnostic fixes; this is not a proof of absence of defects.

## Mechanisms and actual boundaries

| Capability | Evidence and readiness |
| --- | --- |
| Interacting persistent desires | Shared `tsr-domain/src/decision.rs::assess/assess_path` conserves original obligation influence, standing losses, uncertainty and prerequisite costs; notebook/project adapters use the kernel. Ledger lineage and restart persist. Numerical choices implement bounded operational proxies, not proof of TSR's qualitative theory. |
| Intuition and next direction | `deliberation_request` and `scripts/engine-directions.py` support genuine two-stage generation, state binding, reconsideration and outcome-driven continuation. Earlier notebook runs demonstrate this bounded cycle; provider inability remains a real operational limit, not solved by having a driver. Project coding requirements and independent review remain operator supplied. |
| Belief | PRD B2/B6 defines momentary committed possibility with frozen forecasts. `deliberate` and project execution bind forecasts before return; revised evidence and rejection release unsupported commitment continuity. This does not assert a general semantic truth-belief system. |
| Expectation and adaptation | Related-return learning, versioned observations, bounded refinements and shared selection affect later choices. Saved-draw interventions distinguish ranking from sampling changes. Coverage/brevity measurements and decomposition are not understanding or new fundamental values. |
| Real action/return | Sealed project execution, independent named/full oracles, budget reservations, receipt provenance, rejection and closure operate. Recent named-list episodes failed before full-suite acceptance. Earlier supported_languages repair is a different task and not evidence of current success. |
| Numerical learning | Receipt schema, causal prefixes, maturity reconstruction, leakage/support/calibration gates and Tier2 opt-in plumbing exist. `tier2_evaluate` requires qualifying evidence; default fallback/shadow stays active. No operational calibrated model or superiority is established. |
| Independent acceptance | Project review/export gates exist. Semantic review is manual/independent of generator; active reviewer effort is unknown in current episodes. Complete coverage/eligibility cannot be fabricated from wall time. |

Remaining operational data are independently reviewed, resource-complete common-event
episodes with original admission, event/family/clone provenance, causal pre-action
features/forecasts, actual closure, close-plus-seven-day horizon and explicit as-of
cutoff. PRD B8's starting support is 150 mature episodes per event and at least 30 per
family, plus validation/leakage/applicability checks. The 151 qualified corpus tasks
are task inventory, not this evidence; mature-eligible count remains unknown.
Missing data and generation usefulness are dependencies, not another export API gap.

## What the provider evidence does and does not show

Both `target/collection-one-ready-v4/provider-pin.json` and
`target/collection-support-episode/provider-pin.json` pin installed Ollama
`hhao/qwen2.5-coder-tools:latest` (7.6B Q4_K_M), digest
`b7fade0651b448e0e57af67d9edcbdcb686cd8a9647c2cb1f123858438af0a33`.
Across these two episodes: six calls, three sealed executions, no accepted repair.
Failures include silent filtering, lexical/type errors and unchanged-source output.
The second episode supplied 617 read-only validator bytes and bounded
compiler feedback; no recovery was established. This does not prove all local
models incapable or isolate context as the sole cause.

Installed manifest files were read without starting a provider:

- `qwen2.5-coder:32b`, digest `4bd6cbf2d094264457a17aab6bd6acd1ed7a72fb8f8be3cfb193f63c78dd56df`.
- `mistral:latest`, digest `61e88e884507ba5e06c49b40e6226884b2a16e872382c2b44a42f2d119d804a5`.

Neither has been tested on this exact named-list task in the preserved episodes.
Notebook Qwen32B/Mistral inability trials are different tasks. Qwen32B was also used
on the earlier real `supported_languages` coding task (`engine-rts-local-correction-result.json`,
attempt 6); it is inaccurate to call that model tested only on notebooks, or to
count that different coding task as a test of the current failure.

## Minimal next decision, prepared but not dispatched

Approve a NEW prospective local comparison using installed Qwen32B above, at most
three generation calls, each 240seconds, input context at most 16,384 tokens and
output at most 2,048 (total reservation 55,296 tokens), at most two sealed executions.
Zero external charge; this is not zero compute. Use the same 323-file frozen source,
700-byte Surface::parse edit authority and 617-byte read-only downstream validator,
original named-list/profile/unknown-name/full-suite requirements, and preserved
bounded rejection/compiler observations. The local payload includes those admitted
spans, task/criterion text, opaque state/source/inspection bindings and bounded
history; no reference repair, protected test answers, secrets or unrelated sessions.
No whole repository upload or new source admission is proposed.

Keep 32,768 reconstructed patch bytes, 2,400,000 tool-ms, 3,600,000 wall-ms and the pinned
existing no-network image. Freeze new admission and request hashes; retain every
outcome; require named/full-suite success AND independent semantic acceptance.
Unknown effort still blocks complete eligibility. Closed episodes are not reopened.
The larger already-installed model is a meaningful capability comparison to the
failed 7.6B model, not a guarantee or an unchanged retry. No hosted-provider approval
is necessary for this proposed next step. Hosted model choice/pricing should be
resolved only if that local comparison fails; no speculative paid request is made.

## Exact protected-test decisions

Request permission to add `--as-of 2028-01-01T00:00:00Z` ONLY to the synthetic dataset
invocations in these two tests, preserving every assertion and fixture:

1. `m4_corpus::b7_train_apply_prints_raw_and_calibrated_p_and_support`.
2. `m4_corpus::by_repo_seeds_1_to_8_give_three_non_empty_sides`.

They demand fit success without reproducible maturity provenance. Do not relax
fitting or silently select ambient time. These remain documented expected conflicts,
not passing tests. Separately, permission for a whitespace-only expansion of the
single `else { Vec::new() }` at `crates/tsr-cli/tests/m4_train.rs:45` would clear the
remaining formatter failure; that protected file was untouched in this pass.

## Formatting scope and checks

The original 169 formatter hunks covered 25 files. All baseline-existing affected
files were formatted at original M4 baseline 8b74019; newer code introduced these
differences. 24 implementation files are now exact Rust 1.90 rustfmt output of 381b6d8,
verified mechanically; no manual semantic edits. New modules and modified engine,
learning and oracle sections were formatted, with recursion disabled to avoid
unrelated files. Protected tests and unrelated untracked work are unchanged.

The sole remaining formatter difference is the protected m4_train.rs line 45 above,
introduced by 6d1ce769. It predates this pass but is not an original-baseline defect.
No broad clean-suite claim is made. Evidence and command logs live under
`target/readiness-audit/`; final validation results are recorded below.

Validation: 49 tsr-batch, 135 tsr-cli and 137 tsr-learn library tests passed (321 total; three CLI tests ignored). All-target clippy for those packages with warnings denied passed. Diff whitespace checks passed. Full formatting check exits 1 solely for the protected line above. No inference or new operational evidence was collected.
