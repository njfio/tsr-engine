# B9.1 live trial 1 — stopped at projection boundary

2026-10-23, engine `a3e8821`, checkout `ce2a5a4`. This was the user-approved live
six-call/four-execution synthetic trial, not deterministic fixture evidence.
Admission SHA256: `75b53c9bb88a092964b979c37612cfed25d523795392547391ee9923e2fddf3c`.
Validity was 09:50:29–10:50:29 UTC. Independent manifest review confirmed all helper,
binary, scenario and scope hashes before dispatch. The pinned installed Rust image
was present. Authenticated read-only provider metadata confirmed deepseek-v4.1-flash;
the legacy 1 MiB metadata helper limit was exceeded, so a separately approved bounded
8 MiB metadata-only read verified availability. No model request was made by preflight.

## Actual outcome

The normal driver completed the evaluator-authored seed execution group in the sealed
networkless worker. The empty-input test passed and the existing-behaviour test failed.
The original preferences remained open; no semantic acceptance was recorded. The
explicit evaluator forecast 0.1 preceded the exact commitment and joined to the failed
return, residual 0.1. This is diagnostic evidence, not learned calibration. The native
request preserves dimensional Contrast against the observed return, separately from
forecast residuals. Original protected source/tests remained intact.

One genuine provider request generated relational guidance. Independent review admitted
it as a boundary reconciliation preserving both parents: exactly empty input differs
from an empty field within nonempty input. This is useful repair guidance, **not an
established non-additive culmination or fulfilled desires**. The independent review was
operator-assisted, separate from generation, and required a normal-driver restart.

The restart admitted that reviewed proposal, then failed while constructing the
experiment request, before its dispatch. Exact error:
`guidance from outside this synthetic trial`.
No second provider call, generated experiment, boundary patch or re-verification occurred.
**The B9.1 live acceptance sequence did not pass.**

## Established implementation defect

`engine-b91-project-envelope.py:284` compares raw proposal dictionaries. The validated
provider proposal contains `development: null`; native history omits that optional field.
Rust's relational Proposal uses `default, skip_serializing_if = Option::is_none`.
Comparison of the preserved records found exactly this one journal-only field, no
native-only fields and no different shared fields. Removing only this null optional
field makes them identical. Independent review confirmed this is a native serialization
round-trip mismatch, not foreign/private guidance or a model capability failure.

The minimum next engineering change is explicit optional-field canonical comparison,
with negative tests retaining strict rejection of unknown fields, changed parent/meaning,
non-null development drift and foreign lineage. It must not rewrite the original journal.
The driver implementation is admission-pinned; no mid-trial patch or retry was attempted.
Any later live continuation requires a newly reviewed admission; unused slots were expired.

## Accounting and preservation

- One completed execution group, three native tool receipts; tool charge **9,487 ms**.
- One provider call: **905 prompt + 2,455 completion = 3,360 tokens**, including
  2,242 reported reasoning tokens within completion usage.
- Exact settled charge **$0.002413**; no unknown usage, charges or pending effects.
- Evaluator seed patch charged **191 bytes**; no model-authored patch executed.
- Five unused call slots and three unused execution groups expired. A local `STOP`
  marker records the terminal halt without modifying historical events.
- Native project remains open/unaccepted; no fabricated accepted/rejected lifecycle
  closure, human reviewer minutes or maturity claim was added.

Two fresh-process `inspect` invocations returned identical state, `dispatch_enabled:false`,
no unresolved effects, and unchanged driver/provider journal hashes. This is read-only
history inspection, not a replayed execution or new semantic pass. Both owned driver
processes terminated; no persistent service was installed or started.

Local artifacts: `target/b91-live-trial1.admission.json`,
`target/b91-live-trial1/{driver.jsonl,provider.jsonl,terminal-summary.json,terminal-budget.json,terminal-status.json,terminal-evaluator-forecast.json}`,
`target/b91-live-trial1-reviews/`, and launch/resume logs. Provider request identity:
`4beda05d-dc76-406c-ad37-ad4aabea6838`. Raw generated proposal content remains in local
receipts, not this committed report. No private repository source, review content or
credentials were sent. No downloads, original target edits, protected changes, security
changes, push or publication occurred.
