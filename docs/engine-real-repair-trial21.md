# Trial 21: real repair passed tests; temporal review-context defect

At `f7e5ba1`, the fresh v4 normal controller ran without operator stage steering or correction attempts. It generated a substantive experiment, observed the preservation failure, reflected, generated a patch and passed both protected tests. It then stopped on a valid runtime-review rejection of unchanged-experiment readmission. No post-patch supplemental execution, final reflection or independent acceptance occurred. This is an incomplete trial, not a completed autonomy demonstration.

Independent review established a host request-assembly defect: the original experiment candidate was retained verbatim, including its then-accurate statement that the protected preservation case had failed. Readmission placed that historical proposal beside only the current post-patch passing catalog and source, without original evidence or an explicit temporal distinction. The reviewer rejected the apparent contradictory factual claim. The original experiment-review request and actual earlier logs confirm that the historical failure claim was true. The rejection is preserved; no manual override or favorable-review retry was performed.

The generated patch adds an exactly-empty branch and preserves the original nonempty parsing expression. Both protected tests pass, and protected source inputs remain unchanged. Earlier reflection used imprecise language suggesting supplemental tests were merely proposed despite observed case outcomes; this remains a semantic caveat, not silently corrected evidence.

## Accounting and replay

- Four generator calls: 15,440 tokens, $0.002972.
- Five reviewer calls: 36,930 tokens, $0.006173.
- Total: **9 calls, 52,370 tokens, $0.009145; 3 execution groups**.
- Campaign: **113/140 calls, 33/40 groups, $0.085842/$5**, no campaign unknowns. Older Garden Trial 1 unknown remains separate.
- Fresh-process replay reproduced the semantic rejection with no calls/executions and no changes to 150 run files or the ledger.

Evidence: `target/b91-integrated-trial21/accounting-summary.json`, `target/b91-trial21-launch.log`, `target/b91-trial21-replay.json`, and [independent result review](../specs/lanes/reviews/engine-trial21-result.md).

The justified next repair is a source-bound temporal distinction: preserve original proposal/evidence provenance alongside current readmission evidence, and state that the unchanged harness is being considered for rerun against the current patch. Do not rewrite the old proposal, leak the prior review conclusion as an answer, or weaken factual accuracy checks. A fresh trial must test the corrected normal path; historical Trial 21 remains rejected at this stage.

## Prospective temporal-context correction

The runtime reviewer now receives source-bound original experiment evidence separately from current readmission evidence. It verifies the unchanged candidate, prior admission, original request/intent/outcome identities and historical source before exporting that context. Prior reviewer prose and conclusions are not supplied. Historically false claims stay visible and judgeable; the host does not rewrite them or grant execution. The path requires the same run's original RuntimeReviewer provenance and fails closed if it is missing.

The actual retained Trial21 request rebuilt read-only to 21,107 bytes, with 11,661 bytes headroom under the unchanged 32 KiB limit; all 150 files stayed unchanged. Eight tests passed in 113.461 seconds (six focused controls plus ordinary and shared-correction native full flows). After adding an original-role/outcome tamper case, all seven pure controls passed in 0.056 seconds. These are offline fixtures; no inference or actual target execution was used. Final independent source review found no material blocker. A fresh live trial remains the behavioral check.
