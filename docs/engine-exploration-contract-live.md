# Native typed-contract exploration validation

Prospective validation after `7cdca05`; prior `565de0d` failure is immutable.
Explicit approval permits at most two attempts on the same fictional corpus,
stopping after a complete result. A second attempt is only for a known recoverable
failure, not resampling to obtain a preferred answer. Unknown effects stop.

Each attempt: up to six total Luna generator/reviewer calls, one bounded read-only
inspection, 8192 completion tokens/240 seconds/32 KiB per request, $0.30 conservative
allowance. Independent inquiry and interpretation reviews count. Final proposal
is preview only. The observed receipt establishes access/content identity, never
truth of fictional assertions. No outside files/private data/real-world actions.

The exact append-only call-cap grant is 202→220; total 60 operation groups and $5
remain unchanged. Starting usage is197 calls/57groups/$0.163331, no campaign
unknowns. This preserves earlier Garden unknown accounting separately.

First prepared packet: `target/qualitative-exploration-live-2`.
Manifest `b38c8dcd2197cf44487cfef208b12cdddc406d5ee3133bde94a4fecba35674c9`.
Initial body `b136ef6f01c9483b7286211f1e8c45b5032a3166ece10b96e9c5d869e63137c9`,11250bytes.
The runner checks the manifest-pinned campaign baseline instead of silently
reusing the prior run's196-call baseline. Tests:7 exploration/contract tests and17
campaign-extension tests pass, including full six-stage authored fixture flow
through native contracts and rejection of altered/replayed cap grants.

Public pricing preflight: input$0.084153/output$0.420765 per million tokens;
conservative all-category maximum estimate$0.013650 per call, within the existing
$0.05 reservation. Rates are checked again immediately before dispatch. This is
not a provider-enforced hard billing cap. Whole-run readiness is nonexclusive;
each request reserves and checks allowance under the shared ledger lock.

## Attempt 1: known protocol failure

Live-2 stopped at the first inquiry response before native admission or inspection.
Provider HTTP200, normal `stop`, content parse `malformed_json`; request
`63d279f3-0fc5-492d-a399-33f77d83ad00`. Reported2882 input/1100 completion tokens
(648 reasoning included), settled$0.000765. Native initial state is unchanged;
fresh-process replay returns `dispatch:false`. No unknown outcome remains.
The retained receipt identifies the error class but does not retain exact malformed
content, so no precise syntax defect or semantic judgment is inferred.

## Attempt 2 admission

The single remaining authorized attempt uses unchanged code/instructions/rubric,
fresh isolated identities and baseline198/57/$0.164096. Independent review verified
this is recovery from known protocol failure, not selection of favorable content.
Packet `target/qualitative-exploration-live-3`, manifest
`defbed0c234bbca5eee18f2ca15e9ce2c8a829ef806b663a93963697ee0c7b95`;
initial body `20a252ae0c497f4630688eea791b7105c10bc21cc37b34a630d94d8cbdf78188`.
This is attempt2of2; no third attempt is authorized by this grant.

## Final outcome: neither attempt reached the information loop

Attempt2 stopped at the first inquiry response with `duplicate_key` during content
JSON parsing. HTTP200/normalstop, request
`65dba252-44a2-465b-9344-e592b9cc36ab`,2890 input/967 completion tokens
(481 reasoning included), settled$0.000710. No inquiry entered native quarantine,
no runtime review occurred, and no inspection, interpretation or memory update ran.
The transport records the error class, usage and response hash, not the exact
malformed wording or duplicate key. No unretained detail is inferred.

| Stage | Attempt1 | Attempt2 |
|---|---|---|
| Inquiry response | Invalid JSON | Duplicate JSON key |
| Inquiry native admission/review | Not run | Not run |
| Selection/inspection | Not run | Not run |
| Receipt-informed reflection/review | Not run | Not run |
| Persistent advisory update/next proposal | Not run | Not run |

Both original native states remain unchanged. Fresh-process terminal replay returns
`dispatch:false` for both; source pins are unchanged. Both attempts are terminal,
five unused call slots and the unused inspection in each are expired. No third
attempt is authorized. No candidate was silently repaired or selected for meaning.

Actual use: **2 calls,0 inspections,$0.001475 settled**,5772 input/2067 completion
reported tokens (1129 reasoning tokens included in completion). Campaign:
**199/220 calls,57/60groups,$0.164806/$5**, no campaign unknowns. Older Garden
Trial1 unknown reservation remains separate and unchanged.

This establishes neither changed understanding nor substantive no-change: the
protocol failed before interpretable native content. Local contract tests establish
producer/consumer wiring only. Provider-side JSON/schema enforcement is still not
configured; these failures do not justify weakening strict parsing or claiming a
specific unretained response defect. The next useful technical investigation is
why JSON output is unreliable and whether documented provider structured output
can be used, not another unchanged paid attempt. No such new run is included here.

Independent retained-evidence review confirmed the two parser failures, counts,
costs, unchanged scope and absence of semantic evidence. Actual fresh-process
replay outputs are retained as `fresh-process-replay.json` in both packet folders;
post-run checks record unchanged initial roots and source pins. Raw provider
content is not committed. No extra tests were repeated for this evidence-only
update. Commit hooks report `lefthook` unavailable; explicit checks above passed.
