# Four-request inexpensive-model shadow comparison

Executed 2026-10-25 on `911a267`, after explicit user approval. This is not B9.1
Trial 11 and grants no execution or adoption authority. Frozen synthetic source and
instructions were identical except model ID; provider defaults were not presumed
identical or deterministic. Order was GLM, DeepSeek, DeepSeek, GLM. Each request was
bounded to 8192 requested completion tokens and 240 seconds, with model lock enabled.
No Qwen or premium model was used.

The initial Mac disconnection occurred before inference. Preparation session 73952
was reconciled as exit 0; all six runner tests passed in 25.181s. The absent dispatch
journal was confirmed. Independent readiness review recomputed all payload,
validation, binary and implementation bindings. Public prices remained, per million
input/output tokens: GLM $0.0825/$0.2750 and DeepSeek $0.0825/$0.3300. The runner
rechecked prices and fresh validation controls immediately before each dispatch.

Canonical manifest hash:
`09a8ed7d54e2570349949ae3a7344eec615364becb9424fcaa20d8a9159d9c06`.
Evidence directory: `target/b91-budget-shadow-two-model/` (manifest, pricing,
events, summary, typed candidates and label-only reviewer inputs). Prior evidence
remains unchanged. Raw provider bodies, reasoning and credentials are not retained.

| Call | Model | Prompt | Completion | Reasoning subset | Settled USD | Structural outcome |
|---|---|---:|---:|---:|---:|---|
| 1 | glm-5.3-flash | 2448 | 771 | 413 | 0.000526 | malformed JSON |
| 2 | deepseek-v4.1-flash | 2492 | 444 | 0 | 0.000480 | native deferred candidate |
| 3 | deepseek-v4.1-flash | 2492 | 352 | 0 | 0.000170 | native deferred candidate |
| 4 | glm-5.3-flash | 2448 | 1130 | 764 | 0.000652 | malformed JSON |

Total: **4 calls, 9880 prompt + 2697 completion = 12577 tokens; $0.001828 settled;
zero unknown charges, executions or adoptions**. Reasoning is a subset, not added
again. Call 3 reported 2432 cached prompt tokens. No request exceeded the admitted
bound; no retries or additional calls occurred. All allowances are consumed.

The transport recorded `malformed_json` in content for both GLM responses, with
`finish_reason:stop`. Subsequent envelope refusal of null is secondary to that parser
failure, not a semantic verdict. Without retained assertions we cannot establish
whether their intended tests were correct, inverted or vacuous. There is no evidence
here for a specific formatting repair or justification to loosen parsing.

A fresh-process invocation on the terminal manifest refused the existing journal
before dispatch; the journal hash remained unchanged. The fresh native validator
only validates proposals; no lifecycle execution admission was created. Prior
comparison artifact hashes still match. No production implementation changed during
this live run, so the prior 607-pass/17-skip full Python verification remains the
implementation baseline, supplemented by the six preflight tests above.

## Blinded semantic review

The independent reviewer received only label-only typed candidates and the frozen
synthetic task, not model identities, order, billing or raw provider responses.
Unblinding happened after the judgments below:

- **Opal (DeepSeek call 2):** meaningful, nonvacuous coverage of both aims. Empty
  input is expected to succeed; exact vectors for ordinary and signed lists, plus
  rejection of an interior empty field, leading whitespace and positive overflow,
  align with requirements. No inversion. This is finite coverage, not preservation
  proof. Several boundary classes remain untested.
- **Cedar (DeepSeek call 3):** correct empty-input assertion but no substantive test
  of the second aim. Mapping that case to both requirements does not create coverage.
  Its own limitation text acknowledges the omission. This is wrong-aim coverage,
  not inverted values.
- **Juniper/Iris (GLM calls 1/4):** unavailable typed candidates; semantic quality
  unknown, not automatically incorrect.

Both retained DeepSeek candidates use declared case IDs with module prefixes absent
from their actual root-level Rust functions. Opal also reuses existing test names.
These are execution-readiness/provenance issues despite native structural validity.
No assertion was run. Read-only inspection confirms that the current Cargo return
adapter looks up exact declared names and emits `unavailable`/`passed:None` when
missing (`coding.rs`, Cargo report handling); it does not make a missing case pass.
The observed discrepancy therefore warrants rejection/correction before any future
execution, not weakening validation or claiming a demonstrated false-pass bug.

Recommendation: **do not switch to GLM on this evidence**. DeepSeek produced one useful
semantic proposal, one inadequate second-aim proposal, and both need case-identity
review. This is two samples per model on one identical synthetic state, with provider
nondeterminism, defaults and cache effects; it establishes neither broad reliability
nor causal superiority. The next useful offline check is case-name/declared-ID
agreement at experiment review, preserving the distinction between structural
admissibility and independently reviewed coverage. No additional live comparison,
B9.1 trial or execution is authorized by these findings.
