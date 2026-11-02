# Native latency repair and source-provenance continuation

## Verified implementation

`08aed95b` adds verified per-process decoded-record reuse and avoids one repeated
relational projection. It does not cache authority or skip CAS/history integrity
checks. It also adds authenticated append-only recovery for an interrupted native
advance proven not committed. `c74d3be1` exports the already-authenticated receipt
source snapshot into the public projection and repairs an obsolete authored
browser transport test double. Historical rejected reviews are unchanged.

On equivalent retained history, request time changed from 26.974414s to
21.406616s, with exact JSON equality. A valid cloned admission changed from
72.544177s to 62.142565s; semantic results matched after derived event-identity
normalization, and all 191 original event payloads stayed unchanged. No timeout
was increased. Native verification remains expensive: repeated subprocess
reconstruction before provider calls adds minutes. This is a partial latency
improvement, not removal of the entire overhead.

Local tests: three decoded-cache tests, 20 native cycle tests, 12 recovery tests,
and 33 API/lifecycle/recovery checks passed (suites overlap). The final source
projection relational suite passed 15 tests with one optional fixture ignored;
the authored multi-file/restart/stale flow passed; all 24 Python cross-language
checks passed without skips. Independent source review approved the final diff.
An obsolete test parser and then a test-only type-name compile error were fixed;
failed logs remain retained. Authored observations are not live gameplay evidence.

The existing build directory caused prolonged `readdir` waits. Its stalled local
compiler was stopped and exit verified. An isolated offline build directory,
`target/native-latency-repair/fresh-build`, avoided that scan; final test compile
was 16.76s, binary build 15.75s. The old cache and binary are preserved. No packages
were acquired and no security settings changed.

## Live recovery and actual engine work

Original `c5ac0688-a44f-493c-a715-2b54f0effa6c` remains pending historically.
Its authenticated recovery proof establishes the interrupted native advance did
not commit. Fresh run `4940a823-d477-4047-8883-41a0affeb3f6` made five counted model
calls, no executions, costing $0.036617. Two valid reviewer rejections exposed
omitted source metadata. The third proposal removed the disputed assertion but
remains unreviewed/unexecuted. An operator pause settled that run before repair.

Fresh run `5c5acce1-fe51-4bf9-94d7-488503a629f5` uses the same CheaperInference
`gpt-6-luna`, 32768 maximum completion tokens, existing unlimited policy and
unchanged isolated runtime. Its binary SHA-256 is
`415f01257c40df0327ce72f36d5815df339aa0a13ac93fc8bd05f2c0b392e9ba`.
The retained request proves native receipt `f2b818a9…` was delivered as
current-source snapshot `889c654b…`, alongside the independent visual criticism.
That is evidence delivered, not proof of causal learning.

The model chose gentle frog bobbing/breathing and a halo, explicitly calling it
an incremental polish step rather than completion of the larger ambition. It
limited its smoke-test claim correctly. A separate runtime reviewer approved;
native admission and isolated execution passed, without a stage handoff inside
this fresh run. The automatic independent browser assessment returned, and the
controller autonomously continued into another decision.

The exact resulting snapshot is
`b7c599e58e5956d92369a8b2757a21049a51acc12616418faea098741252c343`.
A separate accounted control-driven verification passed all 15 gameplay checks:
keyboard/touch movement, traffic, collection/growth, body constraint, collisions,
reachable success, repeated restart, responsive width and no external requests
or browser errors. Independent visual comparison finds a localized enhancement,
not substantial redesign or demonstrated enjoyment. Screenshots cannot establish
animation timing quality. No new desire identity or admitted relationship
formation is demonstrated by this edit; contributing history remains retained.

## Preserved candidate and limitations

`artifacts/pond-courier-visual-v5/` contains byte-identical engine source, desktop
and mobile captures, exact gameplay receipt, visual review and provenance.
Studio serves all three source files with verified matching SHA-256 at
`http://127.0.0.1:8766/preview/pond-courier/visual-v5/index.html`.
Visual-v4 and all 16 previously recorded preservation hashes remain unchanged.
This is a verified candidate, not user acceptance or engine-selected completion.
The broader aesthetic request remains unfinished.

The Studio API returned real running state without worker/timeline errors in
9.0s. Host visual inspection remains unavailable because the Mac is locked;
no tab was refreshed or game played on the user's behalf.

This repair involved operator recovery/preparation and a source-projection pause;
it is not an intervention-free end-to-end history. No game code was manually
written, no reviewer overridden, and no settled external effect replayed.
The follow-on run remains active, not completed or paused. Its next retained
proposal chooses layered drifting water reflections. It explicitly identifies
`visual-889c654b` as historical, limits the current smoke claim to bounded checks,
and calls the new edit untested and insufficient to claim fulfillment. Native
proposal validation completed; independent review/execution of that next proposal
is not established at this report checkpoint. No arbitrary new iteration ceiling
or manual completion is imposed.

The exact next request contains current native receipt `6c73d1ae…` and the
automatic independent browser assessment, both bound to `b7c599e5…`. This proves
delivery; the rationale explicitly distinguishes current from historical evidence.
It does not establish causal learning. The separately accounted 15-check report
and independent visual critique are queued once in Studio for a future prepared
context; this run has frozen inputs and has NOT consumed those two later reports.

At 2026-10-30 22:57:09 UTC: 387 cumulative model calls, 123 execution groups,
$0.609181 known charges and $0.659181 committed accounting. No blocking unknown
calls or executions. The difference is the unchanged historical $0.05 quarantined
generation estimate, not a verified bound or settled charge. These are timestamped
running totals, not final costs. The current fresh run has used three model calls
and $0.023794, plus two engine-controlled execution/assessment groups and one
separate operator gameplay-verification group. Since the initial latency-repair
baseline, both preserved runs together used eight calls, three groups and
$0.060411 known charges.

A final Studio API check returned in 12.62s, contained the actual frog rationale,
the original message identity, seven timeline items and verified v5 artifact,
with running=true and no worker/timeline error. The ordinary worker remains PID
32533 in session `5c5acce1-fe51-4bf9-94d7-488503a629f5`. No new Start request is
needed or should be duplicated. The old paused adapter/cycle journal hashes and
all 16 recorded artifact preservation hashes still match.

Remaining concrete latency work: browser admission validation and callback
preparation repeatedly invoke full native reconstruction in separate processes.
The current decoded-record cache cannot survive those process boundaries. A
future consolidation must reuse one verified projection within a non-mutating
segment while retaining fresh source/Stop/accounting checks at effect boundaries;
this report does not claim that consolidation is implemented.
