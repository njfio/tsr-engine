# One sealed supplemental episode and a context hypothesis

Exactly one zero-model verification action was admitted and executed on 2026-10-22.
No generator/verifier calls, patch application, target changes, protected-test edits,
external transmission or payment occurred. The original partial invoice candidate
and its closed advisory episode remain unchanged; new evidence was appended.

## Frozen execution and actual return

Original: `target/verifier-shadow/partial/run`, snapshot
`316975ec017c226ff997d5d65345a9442d809b98a211dd836b095c98e132d5b1`.
Check episode: `target/supplemental-live/check-run`, snapshot
`98733d039d7991a574cf9ea32b87255df941aa17d394a93456d180bfe9cfa4c5`.
Installed image:
`sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6`.
Independent review checked the specification-to-case mapping before execution.
The manifest freezes all four file hashes, mapping, image, user and resource bounds.

Admission: one verification action, six commands, each ≤15 seconds, total tool
ceiling 90 seconds, episode wall ceiling 300 seconds, zero model calls/tokens,
zero patch bytes and zero external charge. Existing sealed worker limits remained
2 GiB, two CPUs, 256 PIDs, nonroot, no network; probes use a read-only source mount.
The Docker image preflight initially encountered the filesystem sandbox's socket
restriction; authorized local access confirmed the existing pinned image. No image
was downloaded and no daemon/security configuration changed.

Execution was dispatched once at 12:24:10.646 UTC and terminal at 12:24:23.610 UTC.
Measured commands totaled **12,109 ms**. Accounting reported no unknown or unsettled
requests, no patch bytes, and zero inference. No uncertain action was retried.

| Check | Actual stdout | Frozen expected stdout | Imported case evidence |
| --- | --- | --- | --- |
| subtotal(7,0) | `7\n` | `0` | Counterexample |
| subtotal(3,4) | `7\n` | `12` | Counterexample |
| discount(7,3) | `4\n` | `4` | Unknown |
| discount(7,8) | `0\n` | `0` | Unknown |

The actual aggregate verdict is `failed_tests`. The imported mapped findings are
**subtotal NotMet, discount Unknown**. No finding was promoted to semantic Met,
and the original protected probes were not executed or replaced. The separate
`current_probes` requirement remains unresolved.

Receipt: `f1c0df56c04b470223bc5ecbbb1446c3f3ca96d40615e211c64451cad789420a`.
Import: `e7d40ed1967f6fde2c72de058a16763242c880c083a661e5e17f105963023d92`.
Independent review confirmed both the arithmetic counterexamples and the narrower
interpretation of the discount observations. No actual accepted candidate was
used, so accepted-to-conflict behavior remains covered by deterministic tests,
not claimed as an additional live result here.

### Check contract defect and correction

The template omitted newlines while its harness uses `println!`. The existing
runner compares exact bytes; the supplemental importer conservatively refuses to
turn those failed-format receipts into finite passes. This is a demonstrated
check-specification defect, not a discount implementation defect. The future
example template now includes newlines. The deterministic fixture likewise uses
the exact output contract and asserts `passed_finite_case`, `counterexample` and
`unknown`, rather than checking only the final Unknown semantic result.

The consumed admission was **not rerun**. Frozen manifests, old expected outputs,
receipts and imported classifications were not changed. A corrected finite pass
would need a separately admitted new execution; it still would not prove universal
correctness or unspecified overflow behavior.

### Closure and evidence boundaries

The execution is terminal and its one-action allowance consumed. The native check
lifecycle remains open: `project-lifecycle-close-advisory` explicitly forbids an
Executed episode, while ordinary closure requires a genuine accepted/rejected
review. Neither was fabricated to obtain closure. No closure command was attempted
to bypass that contract. The wall allowance expires normally and prevents further
execution; time spent documenting an unclosed episode continues to appear in its
budget. This is a lifecycle limitation, not additional running computation.
The original advisory episode's closed projection compares equal before/after import.
There is no new accepted/rejected label, human-review-minute measurement or maturity
claim. The existing Docker runtime was not started or stopped by this task; its
sealed per-command workers are ephemeral. No model runtime was started.

## Conflicting-wants representation hypothesis

Read-only audit of the actual latest preserved request
`target/support-live/run/request-32-55ba3817299f.json` found no new admissibility
mismatch. It already permits partial progress, losses assessed by the controller,
deferred inspection and valid alternatives that need not all be selected. The
prior inability does not establish infeasibility, and the original wants remain.

It does reveal a stage mismatch: a possibility generator instructed not to forecast
receives old forecast values, residuals and obsolete rejection prose before current
obligations and action rules. `possibility_context()` now places current authority,
state, obligations, measurements, known/unknown evidence and action contract first.
It retains every observed before/after/contrast and every historical effect with
its evidence references. Forecast diagnostics and old rejection prose are removed
uniformly by field role, not by favorable/unfavorable outcome. Full context is saved
separately with a digest. Assessment still receives the full direction context;
validation, schema, sampling, controller selection, budgets and authority are unchanged.

Deterministic comparison on that preserved request:

| Measure | Before | State-first projection |
| --- | ---: | ---: |
| Context characters | 14,456 | 11,354 |
| Obligations offset | 10,036 | 317 |
| Action contract offset | 11,903 | 2,577 |

Authority, obligations, notes and action contract compare equal. No action, title,
claim set, successful response or independent feasibility solution was injected.
This is a representation experiment, **not evidence that model generation improves**.
The precise known limit remains inability under the recorded configuration.

The smallest discriminating next test is one separately admitted two-stage local
attempt on the same preserved state with the same pinned model, schema and sampling,
changing only this recorded context projection. The recorded configuration is
installed `qwen2.5-coder:32b`, digest
`4bd6cbf2d094264457a17aab6bd6acd1ed7a72fb8f8be3cfb193f63c78dd56df`,
temperature 0, context 32,768 and output ceiling 2,048 tokens, with the existing
240-second per-call bound. Two calls cover one generation and its assessment;
no extra call is implied by failure. If valid distinct possibilities
emerge, freeze them and apply unchanged assessment; if inability persists, preserve
it rather than add another hint or weaken wants. A fresh matched old-context control
would strengthen attribution but needs its own explicit bounded allowance. No calls
for either arm were made or reserved here. This targets the principal core Intuition
bottleneck, rather than adding unrelated supervision or asserting calibrated learning.

## Checks and evidence

- Python direction-driver suite: **69 passed** (deterministic/stub tests).
- Rust supplemental suite: **7 passed** (fake runner; not operational evidence).
- Independent review: actual receipt interpretation, newline fix and context
  projection reviewed; no remaining blocker found.
- Local evidence: `target/supplemental-live/frozen-manifest.json`, `dispatch.jsonl`,
  `plan.json`, `post-status.json`, `post-budget.json`, `observed.json`,
  `context-comparison.json`, before/after context files and test logs.

The shared decision mechanisms remain implemented within bounded adapters; useful
composition under conflicting wants and independently qualifying mature data remain
separate unresolved dependencies. The 151 qualified tasks are not a mature-episode
count. This episode does not prove TSR, superiority, general understanding or
unattended semantic acceptance.

### Subsequent explicit closure (2026-10-22)

The open-lifecycle limitation above was resolved in a later, separately authorized
change. [Verification-only inconclusive closure](engine-context-comparison.md)
closed this check episode at 12:46:14.305 UTC, without rerunning it or changing its
receipts, findings, costs or earlier evidence. No acceptance/rejection/training
label was created. The original advisory episode remains unchanged.
