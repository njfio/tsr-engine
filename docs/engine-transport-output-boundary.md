# Output boundary repair after the two terminal inquiry attempts

## Diagnosis: verified and unresolved

The harness omitted an advertised JSON-output control on the qualitative inquiry
path. It relied on prose asking for JSON, then discarded the exact failing model
content. These are request-configuration and observability defects. Strictly
rejecting malformed/duplicate-key JSON was correct and prevented state corruption.
Neither lost response can now be reconstructed. Missing JSON mode is a credible
contributor, not a proven explanation of the specific unretained syntax errors.

Inspection of stored request objects and the actual transport serializer shows:

| Historical packet | Serialized bytes | Serialized request SHA256 |
|---|---:|---|
| live-2 |11264|03a645034d7e3633ecb91713113b50ddbd0852695872174cde6f43dd9c639dac|
| live-3 |11264|4282382cda0edd911ed2610bd313a3f0be1418b01af2c6a10e39ad3c6b09003c|

Both specify exact Luna,8192 completion tokens, nonstreaming messages, and neither
`response_format` nor tools. These are deterministic serialized request bytes from
preserved bodies, not a claim to reconstruct response content or a server trace.
The response wrapper, model and usage/billing parsed successfully in both runs;
`choices[0].message.content` failed strict JSON parsing. Normal stop and settled
billing do not imply valid JSON. No native inquiry schema or review was reached.

## Official support, without inferred enforcement

Read-only official documentation checked2026-10-27:
- [CheaperInference parameters](https://www.cheaperinference.com/docs#parameters)
  says `response_format` is forwarded, documents JSON-object and supported schema
  output, and warns model/serving-provider support can vary or ignore/reject fields.
- [Exact Luna metadata](https://api.cheaperinference.com/public/models/gpt-6-luna)
  is preserved from the same day in `target/luna-structured-readonly/model.json`,
  SHA256`fd16b79d468b602c26b4e710fb63c9cf7eb64e4b09728f6950fdf0a0100db103`.
  It advertises `structured-output`, `tool-use` and Chat Completions. Direct public
  metadata access via the web tool failed during this audit; the existing official
  snapshot supplies model-specific evidence, not a new measurement of route support.

The prospective six-stage exporter now sets **response_format:{type:json_object}**.
The local helper rejects unsupported model/endpoint/capability metadata. A future
live admission must refresh capability/pricing, source pins and the expired grant.
This is a documented request configuration, **not verified provider enforcement**.
It is deliberately not `strict:true` JSON Schema: the native conditional/optional
profile is not automatically an accepted strict provider schema. Existing native
validation still enforces every field, lineage, authority and UTF-8 bound. Tool
calling would change the response protocol and is not silently substituted.
HTTP400/422 naming `response_format` records provider rejection; no fallback occurs.
A provider that silently ignores the field cannot be proven compliant by this patch.

## Capture, parse, validation and recovery

1. Serialize exact request, enforce token/model/byte/deadline controls; record actual
   request-byte count/hash and requested format, never headers or credentials.
2. Capture bounded response bytes before parsing. Preserve UUID response headers.
3. Strictly parse the wrapper; extract only bounded usage and settled billing.
4. Require exact model, one normal-stop text choice; strict-parse content. No code
   fence stripping, duplicate-key overwrite, coercion, identifier repair or retries.
5. Run the native-owned profile and native deserialization, then native semantic
   checks and independent admission. Contract rejection cannot apply an action.
6. On failure, retain eligible exact bytes locally; discard on successful native
   validation. Native rejection can retain the same captured content unchanged.

Quarantine is explicit opt-in by the admitted isolated runner. It uses a pinned
parent directory descriptor, fixed exclusive no-follow filenames,0700 directory
and0600 files. Each part is at most64KiB (wire+content at most128KiB per request;
six-call attempt at most768KiB plus bounded metadata). Larger parts are explicitly
labeled prefixes with observed-byte count/hash; never claimed complete. Network
reads themselves remain capped at1MiB+1. Content bytes are exact UTF-8 encoding of
the returned content string; eligible wire bytes preserve original JSON escaping.

Dedicated reasoning/private fields, known credential echoes, credential patterns
and sensitive keys (including escaped JSON tokens) cause withholding, not redaction
or output alteration. If content is withheld, wire is withheld too. Malformed or
unallowlisted wrappers are hashed but not retained because sensitive fields cannot
be separated reliably. Thus wrapper failures have safe classification/position/hash,
not guaranteed raw retention. This is bounded exclusion, not a universal secret
classifier; scope remains synthetic-only. Quarantine is never committed, exported,
used as model context or admitted as guidance. A failed disk write is reported as
retention_failed without changing the original charge/outcome classification.

Recovery metadata is advisory and never schedules a request. Unknown billing,
network/partial response or uncertain effects require exact-request reconciliation
and retain reservations. A known settled invalid response may only be retried under
a fresh bounded admission, with a new counted identity; it cannot repeat execution.
This patch grants zero retries. Previously consumed/expired attempts remain closed.

## Audit of other model boundaries

All callers of `engine-synthetic-transport.invoke` share strict wrapper/content
parsing and safe accounting; the new receipt diagnostics/recovery classification
apply there. Raw retention is opt-in, never silently enabled for unrelated scopes.

| Caller family | Output request/retention finding |
|---|---|
| exploration export/validation |All six stages now JSON-object mode plus explicit local quarantine; native contract and independent review retained.|
| historical inquiry/retrieval validation |Prompt-only format and no quarantine; old grants expired, no historical rewrite or release claim.|
| B9.1 driver/campaign, continuation, final/patch/schema recovery |Same strict transport; grounded experiment/boundary patch already use best-effort json_schema strict:false from engine-repair-response-format.py. Other stages do not uniformly request it. No automatic migration of frozen admissions.|
| project runtime review/cycle adapter |Independent reviews/proposals share strict transport but lack uniformly configured format/quarantine. Dormant profiles need explicit configuration/admission review before being called release-ready.|
| campaign-next-choice/synthetic-cycle |Same parser boundary with caller-owned durable intent and gates; no blanket output-mode/retention rollout claimed.|
| engine-provider-envelope |Already requests JSON-object mode; that alone is not native validity or semantic truth.|

This remains an engine release blocker until the intended operational path is
validated with the corrected configuration. The smallest evidence-backed next
change is this existing-model request/diagnostic repair, not a model substitution,
framework rewrite or weaker oracle. No further paid probe/run is authorized here.
A later complete inquiry run would require a fresh explicit admission (up to six
calls/one inspection), not reopening either failed attempt; numeric aggregate
capacity remains available but is not itself permission to run.

## Verification and limits

Failure injection checks actual serialized HTTP request bytes and parse order,
then use the same transport with a fake HTTP response through the real six-stage
native CLI. Controls include malformed/duplicate/empty content, malformed wrapper,
truncation, provider format rejection, network unknowns, credential/escaped-key
suppression, directory replacement, native contract rejection, success and replay.
Authored temporary fixtures prove containment/wiring only, not model reliability.
Independent review found two quarantine issues (escaped content leaking through
wire and unpinned destination); both were fixed with negative tests. A scanning
suffix-copy cost was removed. No production deadlines, parser strictness, tests or
acceptance criteria were weakened.

Verification command:
`PYTHONPATH=scripts python3 -m unittest test_engine_synthetic_transport test_engine_response_evidence test_engine_exploration_contract test_engine_exploration_validation test_engine_b91_campaign -q`

Campaign remains199/220calls57/60groups$0.164806/$5; older Garden unknown remains
separate. No inference, campaign inspection, external adoption, credential change
or target execution occurred. Only official documentation was read online.

Final result:46 tests run,45 passed,1 skipped (`explicit native fake worker binary
required`). Independent source review passed after the fixes. No full-workspace
pass is claimed. Known credential echoes in parsed request IDs are withheld and
in otherwise valid content are rejected (not rewritten), retaining billing. The
final ledger check confirms199calls57groups$0.164806 with no campaign unknowns;
both historical attempts still contain one call intent and zero inspection intents.

## Subsequent release closure

See [the release inventory and correction contract](engine-output-release.md).
Legacy live launchers are now explicitly gated; the prospective inquiry runner
has an admission-governed shared 0..2 format-correction budget. This supersedes the
no-correction implementation description above for newly prepared admissions only.
No historical grant or result changed. JSON-object mode still does not enforce
schema, duplicate-key semantics or truthful interpretation.
