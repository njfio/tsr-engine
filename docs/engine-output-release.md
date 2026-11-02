# TSR output-boundary release scope — 2026-10-27

This is a **release hold**, not a claim that routine malformed output is now
invisible or impossible. No paid calls or target operations were used in this pass.
The completed trials, failures and expired admissions remain historical evidence.

## Operator entry points

| Family | Current disposition |
|---|---|
| `engine-exploration-validation.py` with `engine-exploration-export.py` | Prospective supported inquiry path. All six assembled messages request JSON-object output, use explicit bounded quarantine and strict transport/native contracts. No fresh live grant is supplied by this release. |
| `engine-inquiry-validation.py`, `engine-retrieval-validation.py` | New launches disabled. Existing terminal-file replay remains available; returns before provider/reader initialization. |
| `engine-b91-driver.py`, `engine-b91-campaign.py` | `run` disabled before dispatch or execution. Preparation/inspection retain original validation. |
| B9.1 continuation, final-recovery, patch-recovery, schema-patch-recovery | `run` disabled. Old recovery/admission records are not rewritten. |
| B9.1 shadow, semantic-shadow, next-proposal-shadow | `run` disabled; read-only/prepare interfaces retained. |
| `engine-native-cycle.py` / project-cycle adapter | Operator `run` disabled, including real repair; `check` remains offline. |
| `engine-campaign-next-choice.py`, `engine-directions.py --synthetic-admission` / synthetic-cycle | Live operator launches disabled. Deterministic injected-provider library fixtures remain available. |
| `engine-run.py --approved-live` | Disabled; local-provider mode remains available. |
| `engine-provider-envelope.py` | Offline envelope/decode only; never a dispatch adapter. |

`engine-output-release.py` supplies one compatibility error, before legacy CLI
argument handling can start effects. Option ordering cannot bypass the `run` gate.
These gates govern supported operator entry points, not arbitrary Python programs
importing internal transport/library functions. Those functions are not independently
supported paid launchers. Local-provider and unrelated M3 utilities are outside this
CheaperInference TSR release inventory; this is not certification of every script.

A disabled workflow must be migrated and independently reviewed before re-enabling;
using an internal API to evade the gate is not a supported workaround. Source changes
invalidate source-pinned admissions/upgrades as before. Inspection requiring an old
implementation hash may reject the current checkout: use the preserved terminal
receipts for read-only history or prepare a separately reviewed source upgrade. Do
not edit an old manifest or relax its check to make it match. No grants or ledger
records were modified here.

## What is actually enforced

The active prospective path's serialized request includes
`response_format: {"type":"json_object"}`. This requests JSON syntax; it does **not**
enforce native schema, semantic truth or duplicate-key uniqueness. Provider support
is advertised, not live-proven on the corrected path. Full schemas remain native
owned. Strict parsing rejects malformed/duplicate/nonfinite content, wrong wrappers,
truncation and unsupported shape before any admission. Raw response capture precedes
parsing; safe eligible bytes are retained only in bounded local quarantine. Sensitive
or unseparable wrappers are withheld, with classification/hash retained. Quarantine
is never fed back as model context. Details and official source limits are in
[the transport audit](engine-transport-output-boundary.md).

Legacy callers shared the strict parser, but not consistently the serialized output
mode, opt-in quarantine or native validation contract. They are disabled rather than
misrepresented as covered by tests of the active family. Historical schema request
fields are not retroactively altered.

## Admission-governed format correction

The active inquiry runner now uses `engine-bounded-correction.py`, pinned in the
prepared manifest. New admissions may explicitly include `correction_allowance`
(integer 0..2, default zero), `max_calls = 6 + allowance`, and a conservative
50,000 microUSD reservation per call. The allowance is shared across generators
and reviewers, never per stage. Old grants retain zero corrections; historical
terminal attempts remain terminal. This code change authorizes no live request.

Only positively settled content JSON syntax/duplicate/nonfinite failures and a
narrow whitelist of native structural type/field/length diagnostics qualify.
Native union/identity/lineage/semantic failures remain terminal rather than guessing
which branch was intended. Review reject/unknown/abstain is never retried. Raw
failed output and billing remain unchanged. Fresh requests retain the exact base
body/source context and append a bounded diagnostic requesting a full replacement
response; no raw failed content, repaired IDs or coerced JSON is supplied.

Under the run lock a durable, admission-bound correction journal reserves each
correction before its new call intent. Every attempt is separately counted and
billed. Duplicate correction identities, changed body/source, malformed journal,
unknown usage, overruns and exhaustion stop. Final checks and prior valid results
are retained per stage/call. A completed runner replays without dispatch.

**Crash limitation:** an interrupted runner with an admission or effect intent stops
for reconciliation on restart. It does not automatically resume or repeat any
stage, inspection or billing. This is safe containment, not automatic crash recovery.
There is no revival of unused terminal allowances. Known format correction works
within an uninterrupted admitted run; continuation after interruption still needs
reviewed reconciliation. Quarantine now permits fixed call names 1..8 (at most1MiB
of eligible retained byte prefixes across an eight-call run, plus bounded metadata).

The older B9.1 correction mechanism remains tied to its separate native journals
and admissions. Its legacy live launcher is disabled rather than pretending those
allowances apply to inquiry. No action is ever repeated by the new component.

## Acceptance still needed

No new live attempt is authorized. After a reviewed fresh admission, the minimal
check of the existing corrected boundary is one complete six-call/one-inspection
inquiry (early stop valid), exact serialized format and raw-failure diagnostics,
strict native validation, independent advisory review, truthful receipt interpretation,
persistent next-proposal preview and fresh-process no-dispatch replay. A failure must
retain classification and cost without adoption; it is not permission to retry.
A zero-correction six-call grant tests the corrected transport; a prospective
seven- or eight-call grant can also admit correction. Neither guarantees reliability.
Testing automatic correction live additionally requires a new explicit bounded
correction envelope, including all reviewer calls. Local failure injection proves
containment and wiring, not provider reliability.

Campaign remains **199/220 calls, 57/60 groups, $0.164806/$5**; no campaign unknowns
were introduced. The older Garden unknown is separate. No numerical allowance was
reset, reserved or consumed by this work.

## Local verification

59 targeted tests ran: 58 passed and one skipped (explicit native fake-worker
binary prerequisite). A final 17-test rerun of real serialized transport/native
flow and inquiry integration passed, including malformed generator and malformed
reviewer responses followed by fresh valid responses. Both corrected flows used
seven distinct calls and one temporary-fixture read; failed exact bytes and settled
charges remained recorded. These are injected local fixtures, not live results.
Other controls cover repeated invalid exhaustion, changed source/context, unknown
and interrupted effects, duplicate identities, admission drift, privacy exclusions,
legacy CLI gates and terminal no-dispatch replay. Independent source review passed
after fixing the call-seven quarantine bound and missing correction-module pin.
No workspace-wide pass is claimed. Final admission deadline preflight includes all
admitted correction slots as well as the six ordinary stages.

Reproduce:
`PYTHONPATH=scripts python3 -m unittest test_engine_output_release test_engine_bounded_correction test_engine_synthetic_transport test_engine_response_evidence test_engine_exploration_contract test_engine_exploration_validation test_engine_b91_campaign -q`
