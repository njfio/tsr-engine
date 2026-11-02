# Held-out repair evaluation

Preregistered design: [held-out plan](engine-heldout-repair-plan.md). Implementation and independent preflight review: `a8f04ca`, [review](../specs/lanes/reviews/engine-heldout-suite.md). Suite hash:`3c8dd0978d049f68d3bcbf8e89494ab6f4859c724ebf5207003c39d8a21e94ee`.

The approved campaign extension is append-only: total200 calls/60 execution groups/$5, retaining133 calls/40 groups/$0.106171 from prior work. Three sequential profiles fit the planned42-call/12-group envelope. Whole-run preflight is nonexclusive; each actual effect is separately reserved and settled. Older GardenTrial1 unknown reservation is separate.

Preparation verification: five source-suite tests;15 independently repeated campaign-extension tests;38 affected adapter/control tests; six adapter tests including complete real-native flows for both new tasks with authored responses and cfg(test) worker receipts. These are offline integration evidence, not real repair results. Installed worker digest was unchanged. Missing local lefthook remains disclosed; no install or hook-pass claim.

Actual run results will be recorded below without dropping failed attempts. The fixed tests, task contents, order and success criteria remain those frozen before dispatch. Target programs run only inside the sealed worker and are counted in the shared campaign. No raw provider body is stored in this report.

## Fresh tasks

| Task | Real execution evidence | Full-cycle result | Calls / groups | Cost |
|---|---|---|---:|---:|
| singleton-clamp | Seed and generated experiment failed; generated repair and unchanged experiment passed | Stopped on final runtime semantic rejection; no native acceptance | 12 / 4 | $0.014341 |
| unicode-prefix | Seed and generated experiment failed; no patch applied | Stopped after two format corrections, then a590-byte rationale exceeded the512-byte contract | 9 / 2 | $0.010116 |

**Fresh-task full-cycle completions:0/2.** One technically correct repair was verified, but it did not complete the full cycle. Neither failed attempt is omitted or rerun. Both had genuine generated second-aim experiments; Unicode's rejected patch body was not retained, so its correctness is unknown.

The clamp run used five generator and seven reviewer calls (77,428 tokens), including one bounded reviewer-format correction. It also exposes two semantic review limitations: an earlier reflection falsely described a failed case as passing, and runtime review missed that error; the final reflection was rejected for omitted summary qualifiers even though independent review found no actual narrowing of the immutable parent requirements. The valid rejection was not overridden. Later accurate statements do not erase the earlier error.

Unicode used six generator and three reviewer calls (48,064 tokens), including two full-response experiment-format corrections. Its retained experiment and reflections were meaningful and truthful; the final protocol refusal is known failure, not transport uncertainty. No manual replacement, identifier repair or silent response truncation was performed.

Fresh-process replays, with provider/worker callbacks forbidden, reproduced both terminal refusals. Clamp's200 files and Unicode's130 files remained unchanged, with no additional calls, executions or ledger changes. Source and fixed tests remained immutable throughout each replay. Both runs have zero unsettled requests/executions.

Independent semantic/source assessment and exact evidence IDs: [results review](../specs/lanes/reviews/engine-heldout-results.md). Local records: `target/heldout-clamp-1/`, `target/heldout-unicode-1/`, their `-accounting.json`, `-replay.json` and `-launch.log` siblings. Raw response bodies remain excluded under repository policy; structural rejection diagnostics and admitted proposal provenance are retained.

## Original parser regression

The separately reported regression **completed1/1**. The normal controller generated a meaningful experiment, observed its failure, generated the correct exactly-empty boundary repair, passed the immutable tests and the byte-identical experiment, and produced truthful final reflection accepted by runtime review. No protocol correction was needed. Independent final assessment accepted the repair using source equivalence and actual receipts; this was an explicit independent agent judgment, not measured human review or model self-approval.

The same normal driver adopted that source-bound review and closed with provider and worker callbacks forbidden. It skipped the optional next proposal. A fresh process replay then returned `completed_operator_reviewed`, with all200 run files and the campaign ledger unchanged and zero provider/target effects. No linked recovery, intermediate operator steering, replacement code or rerun of passed stages was used. Final independent acceptance remains a separate supervised boundary, as preregistered.

Native accepted decision:`b104956a2050a1476f776d1e7c902e505d52b1fe3cf46e08b51d1c5933f97da8`.
Source:`a241bf455a544034864f74a1aa304abf22926b39f12ebe46a5b3980007537ffc`.
Protected receipt:`d768d198c8a33e382a3ae255c7db4935ebefce38a37241a5e1c6d830b354cf7b`.

Regression usage: five generator/six reviewer calls,66,210 tokens, four actual execution groups,$0.012105. All settled, zero corrections. Local records: `target/heldout-parser-regression-1/`, sibling `-accounting.json`, `-finalization.json`, `-replay.json`, and `-reviews/`.

## Aggregate conclusion and next step

The three preregistered attempts consumed **32 unique calls,191,702 tokens,10 execution groups,$0.036562**. The campaign now stands at **165/200 calls,50/60 groups,$0.142733/$5**, with no campaign unknown usage/charges/executions or pending driver operations. Thirty-five call slots and ten groups remain; this evaluation is complete and makes no additional dispatch. The older GardenTrial1 unknown reservation remains distinct. Independent aggregate audit: `target/heldout-evaluation/final-accounting.json`; finalization independently proves no later ledger effects.

This supports repeatability on the previously successful parser and useful repair generation on one new task. It does **not** establish reliable full-cycle transfer: **0/2 fresh tasks completed**, even though clamp's repair passed. Two of the three programs have verified correct generated patches; Unicode has no executed repair and cannot be counted as a repair success. No probability of general reliability, causal TSR benefit or non-additive culmination is inferred from this tiny, structurally related sample.

The most useful next work is to reduce protocol noncompliance and improve semantic review discrimination before adding scenarios. The model violated an explicit512-UTF-8-byte rationale bound and explicit experiment field shapes; this evaluation found no justification to relax those contracts, silently repair output, or claim the rejected code would pass. Check provider-supported schema-constrained response generation before choosing an implementation. Separately, use the retained clamp evidence to test a review boundary that distinguishes unsupported observed facts from merely omitted summary wording, while preserving immutable parent requirements and allowing attributed disagreement. Do not fix review quality by forcing acceptance or repeating calls until a reviewer agrees. Any subsequent evaluation should be separately frozen and reported, not substituted into this denominator.

No protected tests, historical trials, credentials, security settings, private exports, production targets, publishing or deployment were changed. The source implementation is committed locally at `a8f04ca`; this result and independent review are documentation evidence. No fresh full-workspace test pass is claimed.
