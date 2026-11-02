# Behavior continuation and supplementary compatibility

The reviewed five-candidate behavior continuation launched at 2026-10-15T14:03:01Z, PID 15629, after all 46 frozen hashes matched,352.03 GiB free exceeded248 GiB preflight, and no qualification workers/containers were active. Max 2 workers, frozen engine 238ac39,600 second sealed commands,1 MiB output,2700 second outer limit,200 GiB live launch guard and stopped historical outer 29246 remain unchanged. Observed container: pinned RTS image, user 65534:65534, network none, privileged false, capabilities ALL dropped. No retries, paid calls, cleanup or publication occurred.

The independently reviewed finite observer PID 21995 polls every 30 seconds for at most 12 hours. It exports completed outcomes, checks prior attempts unchanged, byte-compares exports against source evidence, and invokes the native admission audit. Initial audit: 80  admissions,7,238 evidence files,exit 0,no anomalies. It writes docs/m3-behavior-continuation-final.json and its final audit only after terminal reconciliation. At report creation the cohort is still running; no behavior admission is claimed. App scheduler callbacks remain unavailable in this execution context; the local observer saves durable evidence and does not launch a successor.

## Offline compatibility results

The17 supplementary proposals were previously source-reviewed; this review specifically resolves the 15 daemon compatibility questions. An offline Rust probe extracted the actual resolver functions from 238ac39 and the module-v2 implementation in the current source. It parsed pinned RTS source as data; no candidate code or tests were executed. The current resolver region matches commit 38196f9; the later source change affects an unrelated budget reason. Existing module-v2 binary SHA256 is 7ee73bc691b8d90d2de8fc9c7615ebaf4a5a6afee5b368303ba387603efaadc5, matching its prior successful strict revalidation receipt.

| Proposals | Frozen 238ac39 resolver | Existing module-v2 resolver | Remaining action |
|---|---|---|---|
|13 nested daemon methods|Rejects identities|All resolve|Fresh separately pinned module-v2 campaign, then sealed screening and full strict qualification|
|Response::into_line; ProtocolError::to_wire|Both resolve|Both resolve|Normal sealed screening and strict qualification|
|Route::to_wire; RootSet::match_one|Both resolve|Both resolve|Normal sealed screening and strict qualification|

The earlier blanket15 compatibility-blocked classification was too conservative. The old direct resolver does not reject the daemon root safety lint; module-v1 introduced that restriction, and module-v2 added the exact deny(unsafe_code) exception. No new resolver fix is indicated by this offline review. Neither running engine nor historical rejection records were changed. The existing module-v2 preserves entire ancestor bytes and rejects ambiguous layouts, conditional/redirection attributes and top-level ancestor macros. Independent review confirmed plain external chains, no competing layouts, ordinary test identities and no relevant unsupported attributes in these proposals.

Resolver compatibility is only one gate. These 17 remain proposals: every candidate must pass baseline-full-green, named-red, full-red, exact reversal and full-green with trusted evidence. No admission or outcome-performance claim follows from this probe.

## Capacity and operational limits

The latest audited corpus remains 80 distinct groups (26 AgentID, 54 RTS),70 short of150. There are 63 original remaining slots including the five running behavior candidates; perfect original yield reaches 143. Adding four supplementary proposals compatible with 238ac39 reaches 147; at least three further successes would still be needed. All 17 supplementary proposals plus all 63 original candidates give a conditional ceiling 160, ten failures of headroom, not a yield forecast. Prior dedup against current admitted mutation spans found no collisions; correlated mechanisms and two-repository concentration remain.

Remaining original build-only estimate 315.768 GiB exceeds available storage above the 100 GiB reserve; the 200 GiB launch guard is stricter. The finite behavior batch fits its conservative 48 GiB allowance, but the whole inventory needs more storage or separately authorized cleanup. No further deletion is authorized. M3 acceptance remains unmet; real paired outcome coverage and the 150-task criterion are not replaced by static checks or synthetic M4 tests.

## Verification

- Offline extracted resolver probe: exit 0,4/17 resolve 238ac39,17/17 resolve module-v2; per-function results and source hashes in docs/m3-validation/supplementary-compatibility-probe.
- cargo +1.90.0 test --offline -p tsr-batch nested_module_identity_regression -- --nocapture: 6 passed,0 failed; captured log m3-validation/m3-supplementary-module-regressions.log. This is scoped regression coverage, not a full workspace rerun.
- Observer Python compile check passed; independent mapping review found no blocker.
- Observer initial native audit 80/80 passed;7238 exported files byte-equal to source.
- No hooks claimed; Lefthook is unavailable.

Independent follow-up review verified both extracted resolver sections against their exact commits and confirmed the 4/17 versus 17/17 result; the reviewer did not rerun the probe.
