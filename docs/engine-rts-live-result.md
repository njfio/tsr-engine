# Approved autonomous RTS demonstration: stopped at response decoding

Implementation: `dd61636`; task `80d827e3a4c1`; prepared run directory
`target/engine-rts-autonomous-ready`. The precise payload/run received approval
before dispatch. One call used `deepseek-v4.1-flash` at
`https://api.cheaperinference.com/v1/chat/completions`.

The credential and model-catalog checks succeeded, including the broker's price
ceilings ($0.30/million input and $1.20/million output). The actual credential and
raw provider response were not printed or retained. The call settled at
**$0.002149**, using **1,360 prompt + 2,048 completion = 3,408 total tokens**.
The shared aggregate is **$0.620022**, with no unresolved reservations.

The response failed strict decoding after settlement. The sanitized diagnostic
records `stage=decode`, `code=transport_or_internal`, elapsed **21,291 ms**.
Completion usage exhausted the configured 2,048-token response allowance, which
is consistent with truncation, but the retained evidence cannot establish the
exact decoder failure. Do not report truncation as proven or claim that a valid
repair was produced.

The transport recorded `provider_failure_stop`. There were **zero accepted
proposals, zero edits and zero test executions**. All 56 protected files and the
mutant implementation retained their hashes; the original repository stayed
unchanged. A local restart returned `previous_failure_stop` with byte-identical
billing ledger and no further call. The controller remains `ready_for_proposal`,
with no pending commitment and zero completed repairs; transport restart correctly
remembers its terminal failure separately.

The two-proposal/$0.25/100,000-token bounds were not exceeded. One authorized call
remains numerically, but it was not spent on a blind retry. The response body is
not retained, so this attempt cannot be locally re-decoded. No proved recoverable
source or executor fault was found, and the persistent failure gate was not
relaxed. There is no model repair to export or validate against the full suite.
This is an unsuccessful product demonstration, not a successful repair benchmark.

[Structured evidence](engine-rts-live-result.json) includes settlement, sanitized
diagnostics, state, restart and integrity checks. No push, deployment, publication,
cohort, global Docker change or additional source export occurred.

A later explicitly authorized recovery is recorded in [the recovery report](engine-rts-live-recovery.md). This first-attempt receipt remains unchanged.
