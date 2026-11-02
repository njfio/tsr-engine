# Preregistered fixed 12-task paired comparison

Preparation only: no paid launch in this turn. Twelve real-repository mutation tasks, six per lineage, each once under record and rules. These are admitted constructed mutations in real repositories, not historical human bug fixes or an untouched holdout.

## Selection

The original draft remains in m3-fixed12-selection.json. Its unsupported >64KiB sources led to a separately versioned, outcome-blind compatibility filter. Version 2 uniformly applies UTF-8, one-file and exact materialized-mutant source <=65,536-byte eligibility to all 149 complete strict admissions. Twenty size-incompatible tasks are listed individually; 129 remain. Two existing admissions from an incomplete qualification run are excluded from the sampling pool without revoking their admission status. No generator outcomes influenced selection or replacement.

Seed: tsr-m3-fixed12-real-corpus-v1. Within each lineage, round-robin family buckets in the frozen SHA256 order; within each family prefer unseen file paths then task hash. Select six distinct function groups per lineage. Alternate lineages; counterbalance first mode within each lineage (three record-first and three rules-first). Exact inventory, ranks, source measurements, exclusions, task IDs and ordering are in m3-fixed12-v2-selection.json. Selection is fixed; no replacement following failed preflight or model performance.

The balanced/stratified sample is not a prevalence-weighted estimate of all 151 admissions. Size compatibility and only two lineages narrow generalization. Known previously paid selections are disclosed separately, never filtered by their result.

## Treatment and immutable inputs

Same frozen engine, exact-edit@v3 schema/system instructions, DeepSeek catalog ID deepseek-v4.1-flash, 16,384 output cap, omitted temperature/reasoning, and existing provider routing defaults in both modes. Prompt bytes are fixed by encoder source and deterministic per-task examples; source observations and model history naturally differ after each mode's actions. No retries or hidden substitutions. Requested model identity is not independent verification of upstream provider routing.

Within every pair the tasks JSON, CAS, profile, preparation and broker context are byte-identical; each arm uses an independent fresh workspace and empty store. Record and rules differ only in the native mode and its policy intervention. Preparation images differ by repository lineage and are pinned identically within pairs. Do not change engine, wrapper, prompts, selected test, caps or oracle after freeze. A necessary change requires a separate version, preserved earlier evidence, and explicit accounting of which arms used which version.

## Oracle and outcomes

Acceptance requires all independently protected failing tests actually executing and passing by exact test name, full-suite success, no protected-file/test tampering, independently reread final-oracle receipts with verified_success=true, correct event roots/CAS hashes and oracle:true closure. Native evaluation judges the candidate workspace; it does not apply a hidden reversal as a model repair. Original corpus outcome prose about reversal describes admission construction, not this repair criterion.

Visible named-test success is not completion. Keep accepted, oracle-rejected, policy-blocked, budget-rejected, schema/refusal unavailable, deterministic pre-dispatch budget unavailable, exact-edit precondition failure, infrastructure failure and unattempted arms distinct. Ordinary recorded model/policy/budget failures advance after clean accounting, durable evidence and quiescence. Unknown transport/billing, containment, integrity or unknown native errors halt all remaining arms. Never silently retry, replace, discard, impute acceptance or weaken a criterion. Outstanding reservations require reconciliation before any future dispatch.

## Paired analysis

Primary operational metric: independently verified successes divided by all 12 assigned tasks for each mode. Rejected, unavailable and precondition failures count as no verified success; their reason counts remain visible. Unattempted arms stay in the denominator with no verified success and are additionally reported as missing, with lower/upper bounds success/12 and (success+unattempted)/12. This is not an estimate that missing tasks would have failed an oracle.

Publish a 12-row pair table and all 24 arm records. For pairs with both arms terminal, tabulate both-success, record-only success, rules-only success, neither-success, plus the underlying missingness/reason categories. Report paired success difference (rules minus record), discordant counts, and an exploratory exact two-sided McNemar binomial calculation only when all 24 arms are terminal. Report Wilson 95% intervals for each mode's operational proportion; disclose that shared tests/function families and two lineages violate simple independent-task generalization. No superiority declaration from n=12, no pooling with the six synthetic attempts or historical pilots. Costs, requests, broker duration, worker duration, timeouts and schema/stale failures are summarized per mode and pair, including failed attempts.

## Recommended caps and resource gates

Serial execution: 20 generator requests, 200,000 billed generator tokens, 15 tool minutes, 1,800 wall seconds and 131,072 cumulative patch bytes per arm; 480 inference requests total. Recommend $60 incremental and $100 aggregate, from $0.239583 currently settled. Conservative per-request bound at existing maximum prices is $0.1121184; 480 such requests cost at most $53.816832, with room for the existing $2 reservation guard inside $60. The $100 ceiling is cumulative, not additional.

Controller wall ceiling: 46,800 seconds (13 hours), with at least 1,900 seconds remaining before dispatch (1,800 arm budget plus 100 seconds guard/export allowance). Checks after each arm halt if the ceiling is reached. No concurrency or automatic retry. Require 240GiB initially and maintain the 200GiB live floor. Current retained-workspace growth may exhaust disk before 24 arms; no deletion is authorized or automatic. Stop and retain unattempted assignments if a resource gate fails. This makes the launch bounded, not a guarantee that all arms can finish with current storage.

Catalog/runtime/image/disk/accounting checks must be fresh at launch. This preparation does not authorize launching in this turn.
