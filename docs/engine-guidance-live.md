# Fresh guidance continuation validation — 2026-10-22

Code `c274ba8` was validated on a fresh disposable synthetic invoice episode at
`target/guidance-live/`. Original requirements, source/protected hashes, installed
model and resource admission were frozen before generation. Guidance was published
**once**. No new task, replacement answer, changed desire or second feedback
publication was supplied.

1. Normal attention selected `discount`. The installed model generated only the
   discount repair. Isolated execution failed overall: subtotal 152 rather than
   300, invoice 122 rather than 270, floor 0 passed. Separate static review found
   discount met and subtotal unmet; its explicit operator-adopted rejection was
   recorded. Before review, the observation shifted attention to `test:subtotal`.
2. After review, normal attention selected the published `subtotal` requirement.
   The original guidance text and publication ID remained unchanged, with status
   `unchanged_guidance_rebound`. The second genuine model response repaired subtotal
   only. Isolated execution passed all three protected probes.
3. The old semantic review remained stale after the second return; test success did
   not grant acceptance. Reapplying that review failed with `stale acceptance
   evidence binding`, leaving the factual root unchanged. Fresh independent semantic
   review is recorded separately; see terminal result below.

The audit script reads actual SQLite event references and CAS records. It verifies
one operator `AcceptanceFeedback` publication, two Rules `AttentionConsumed` records
matching saved request/response bindings, unchanged guidance and protected probe
SHA-256. Both proposed source replacements came from the local model, without a
hand-authored substitute. This establishes observed normal-path continuation, not
statistical proof that attention caused the generated edits.

## Resources and scope

Installed `qwen2.5-coder:32b` digest
`4bd6cbf2d094264457a17aab6bd6acd1ed7a72fb8f8be3cfb193f63c78dd56df`;
existing pinned fixture image
`sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6`.
Temperature 0, context 16384, output bound 2048, timeout 240 seconds per call.
Two calls and two isolated executions were used; no retries or further slots.

Measured tokens: 5707 input + 260 output = **5967**. Provider duration including
loading/runtime: **112.864 seconds**. Tool time: **25761 ms** of 150000 admitted;
patch bytes: **272** of 8192. Token ceiling 69632 and wall ceiling 900000 ms.
External charge/transmission: **zero**. No unknown usage or unsettled requests/tools.
Temporary loopback Ollama PID2811 was stopped; its process session exited 0.
No original repository, protected test, credential, security or daemon configuration
was changed. The initial sandbox listener failed before starting; the explicitly
scoped temporary loopback service and isolated runs used approved escalation.

## Evidence and limits

`manifest.json`, `admission.json`, `feedback.json`, requests, model receipt journals,
proposals and execution records preserve each phase. `provenance-records.json` and
`audit-result.json` establish publication/consumption; `stale-review-rejected.json`
establishes stale rejection without mutation. No historical episode was rewritten.

Reviews are explicit **operator-assisted** adoption of a separate review agent's
source/coverage assessment. They are not an implemented automatic reviewer.
Active human reviewer effort remains unknown. Arithmetic overflow policy beyond
representable u32 products remains unspecified. Three probes plus static reasoning
are not exhaustive input testing. This synthetic fixture is ineligible operational
training evidence; closure plus seven days cannot make it mature eligible data.
No calibration, general autonomy or statistical reliability claim follows.

The next concrete implementation is the separately admitted verifier request/report
boundary in [independent review plan](engine-independent-review-plan.md), initially
with explicit operator adoption. It requires authentic provenance and domain coverage,
not generator self-approval or a nominal different reviewer name. No automated reviewer
calls or external providers were used in this validation.

## Terminal result

Fresh independent review accepted both scoped requirements. The normal final request
reported `no_eligible_attention`; no further generation was attempted. Closure was
**2026-10-22T10:53:16.144Z**, with stopped budget and no unsettled work. Final elapsed
wall charge was **370043 ms**. All call/execution allowances are consumed and expired.
Native label remains null. Close plus seven days is 2026-10-29T10:53:16.144Z, which
still cannot qualify synthetic evidence or supply unknown reviewer effort.

Accepted-only export succeeded for snapshot
`4108daaf840eb0a7042d5f6743c038b33894b71dfe08a030770ec78e513348ad`;
`target/guidance-live/accepted-export.json` names the complete repair patch and
metadata. `summary.json` records terminal status/accounting. The independent reviewer
also confirmed the one-publication/two-consumption audit. No engine code changed in
this validation; prior code checks remain in `target/guidance-continuation/`.
