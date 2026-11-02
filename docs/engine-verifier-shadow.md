# Two-request local verifier shadow result

**Transport succeeded; semantic verification failed.** Exactly two advisory calls
were made on installed `mistral:latest`, pinned digest
`61e88e884507ba5e06c49b40e6226884b2a16e872382c2b44a42f2d119d804a5`.
This is distinct from the Qwen32B generator documented in
[the transport plan](engine-local-verifier.md). No retries, generator calls, project
executions, downloads, external transmission, adoption or acceptance occurred.

The two fresh synthetic invoice copies used the exact prior generated patches:
partial = proposal 1; complete = proposals 1 and 2 from `target/guidance-live`.
Original files/episodes were untouched. An independent reviewer froze expected
labels before seeing responses; labels and prior generator rationale were excluded
from both request inputs. Original subtotal/discount criteria were retained, with
an explicit observed-probe criterion corresponding to the original all-probes-pass
goal. Fresh copies had no execution receipt; historical receipts were not imported.

| Case | Expected subtotal / discount / current probes | Returned | Matched |
|---|---|---|---|
| Partial | not_met / met / unknown | met / met / met | 1/3 |
| Complete | met / met / unknown | met / met / met | 2/3 |

Both outputs passed strict schema and exact evidence-reference coverage. Neither
abstained. Neither matched the full expected vector. The partial source plainly
contained addition instead of multiplication, yet the verifier called it correct.
Both reports claimed independently observed passing probes despite explicit null
receipt and verdict fields. The complete subtotal is correct only for representable
`u32` products; overflow semantics were not invented or established. The completed
report also loosely said discount keeps totals “positive,” although zero is allowed.

## Metered resources and terminal status

| Case | Input tokens | Output tokens | Runtime-reported duration | Host invocation |
|---|---:|---:|---:|---:|
| Partial | 703 | 346 | 11.676 s | 12.205 s |
| Complete | 700 | 351 | 9.843 s | 10.396 s |

Total: **2,100 tokens**, 21.519 seconds runtime-reported processing, **zero external
charge**. These are measured local resources, not zero cost/resource use. Each
admission reserved 18,432 tokens, one call, ≤240 seconds, context 16,384/output
2,048; each project wall ceiling was 300 seconds with zero patch bytes. Both calls
completed, usage settled, no unknown/unsettled requests. Both verifier admissions
were revoked after the sole allowed request. The owned loopback runtime was idle
(`/api/ps` empty), stopped, and its session exited 0. No human review minutes or
mature evidence are claimed.

`project-lifecycle-close` refused the partial episode with “current independent
accepted/rejected review required.” This is an application contract refusal, not an
approval denial or uncertain execution. No adoption/review was fabricated to satisfy
it, and closure was not retried on the second case. Native project episodes remain
unclosed; shadow requests are terminal with spent/revoked allowances. This exposes
a separate lifecycle gap for advisory-only episodes, not permission to relabel them
as accepted or rejected project outcomes.

## Diagnosis and next bounded correction

Independent assessment confirmed 3/6 matched criteria and both fabricated execution
claims. Frozen requests contain the correct source and null receipt facts, and both
runtime responses completed with measured usage. Missing source, truncation or
transport corruption is not established. The present model failed to interpret
available evidence. Required reference lists establish structural coverage only.

The smallest justified correction is a **typed, operator-admitted evidence
prerequisite** for claims requiring observed execution. Expose receipt availability
as structured controller data; classify/reject a `met` assertion when the required
current-source passing receipt is absent. Preserve the original report and conflict;
do not silently rewrite findings or change expected labels. This bounds fabricated
execution evidence but does not solve the missed addition-versus-multiplication
error. No further comparison is proposed until that change and deterministic
regressions are independently reviewed. This Mistral path is unsuitable for
unattended acceptance on the demonstrated task.

Evidence: `target/verifier-shadow/held-out-labels.json`, per-case `request.json`,
`records-final.json`, `budget-final.json`, `status-final.json`, `comparison.json`,
`terminal.json`, and `independent-assessment.md`. Raw HTTP/provider response bodies
were not logged; validated typed reports and response hashes are retained by the
existing quarantine contract. No protected tests or original target files changed.
