# Trial 22: accepted review hash encoding blocked readmission

The normal controller at `d43c6c7` recovered automatically from an experiment response with unsupported fields, ran the corrected experiment, reflected on the failure, and generated/applied a patch that passed both protected tests. It stopped locally before unchanged-experiment readmission; no reviewer call was dispatched at that stage. The run is incomplete and unaccepted.

The new historical-review helper compared a recorded candidate hash using the envelope JSON serializer rather than the review serializer that wrote it. The accepted report contained non-ASCII text. Its recorded hash `0df3d09f55e098ce73cc7752e6798464dd8190e611dcb2721e9e7f4798b95b93` exactly matches `cycle.digest`; the other encoding produced `1ffd1552f1bf4632570fd2f1c911810725eaae97062d3a654c222d0fb7486de9`. This was a local implementation defect, not a semantic rejection or unknown execution.

The one-line fix uses the same digest as the existing review writer and replay validator. No alternate hash is tolerated and no historical record is changed. Eight focused tests passed in 0.054 seconds, including Unicode and forged-hash rejection. Actual Trial22 request reconstruction passed read-only at 21,421 bytes, leaving 11,347 bytes under the unchanged 32 KiB bound; 150 files and ledger were unchanged. Independent assessment is recorded in the follow-up to [the temporal review](../specs/lanes/reviews/engine-trial21-result.md).

## Accounting

Five generator calls:20,060 tokens/$0.003896; four reviewer calls:29,487 tokens/$0.004842. **Nine total calls, 49,547 tokens, $0.008738; three real execution groups.** The failed experiment attempt's charge is included. Seed and supplemental experiment failed preservation checks; patch commands all exited zero and protected tests passed2/2. Unchanged-experiment rerun, final reflection and acceptance were not performed.

Campaign total: **122/140 calls, 36/40 groups, $0.094580/$5**, no campaign unknowns. The older Garden Trial1 unknown remains separate. Trial22 and all earlier histories remain immutable. Remaining18 calls/four groups can cover one fresh14-call/four-group trial; no allowance resets.

Evidence: `target/b91-integrated-trial22/accounting-summary.json`, `correction-accounting-audit.json`, `target/b91-trial22-launch.log`, `target/b91-trial22-unicode-readiness.json`. No provider/target effects occurred during the fix or its verification.
