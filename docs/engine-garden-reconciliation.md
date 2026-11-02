# Garden request reconciliation — 2026-10-22

**Unresolved; no inference retry or account change.** The original intent is
`75e5e3086f67f46bd36898e22f23258f6e78dbf71d643743729d8ad481567517`,
payload SHA256 `1d2e058cf4fd1e995d76f302ba294ffe4c9afc2b676ef3e7695265d9b483d9d9`,
recorded at 19:39:08.582788 UTC. These are local hashes, not provider request IDs.
The response contained no stored HTTP status, body hash or provider ID. No extra
historical exception log exists in the run journal. The exact original exception
cannot be recovered from its generic `transport_or_parse_error` receipt.

## Official contract and bounded read attempt

Official [documentation](https://www.cheaperinference.com/docs), retrieved 2026-10-22,
confirms `https://api.cheaperinference.com/v1`, the buffered Chat Completions endpoint,
response IDs in `x-ci-request-id` / `X-Cheaper-Inference-Request-Id`, and direct
`GET /usage/requests?request_ids=…` lookup. Request detail retention is 45 days;
missing IDs are ambiguous and never establish nonexecution. The same-day saved
[OpenAPI schema](https://www.cheaperinference.com/api/openapi.json) documents narrow
`start_at` / `end_at` filters, but not a payload-hash lookup.

There is no exact provider ID to query. One bounded authorized read-only request used
`GET /v1/usage/requests?start_at=2026-10-22T19%3A39%3A08Z&end_at=2026-10-22T19%3A39%3A10Z&limit=10`.
Even a matching timestamp/model would be only candidate correlation, not proof of
identity. No general account poll or unrelated history was requested.

At 19:43:34 UTC the read failed at `open`: `URLError` wrapping `gaierror`, errno 8.
No HTTP status or provider records were received. Only fixed diagnostic fields were
saved in `target/garden-ready/reconciliation-read.json`; credential values and exception
messages were never printed or stored. This establishes a present DNS-resolution
failure for that read, not the original inference failure's cause. No alternate host,
network route, escalation, security change or retry was attempted.

The published 795-second text-request ceiling would place the original processing
ceiling around 19:52:24 UTC if the provider received it promptly. It is not a promised
billing-record visibility deadline. With no provider ID/record and failed lookup,
there is no identified in-flight record to monitor safely. Absence and elapsed time
cannot reconcile usage, charge or execution. All original uncertainty reservations
and the consumed slot remain intact; the other four slots remain expired.

## Proven local observability defect and prospective repair

The prior transport collapsed DNS, permission, credential, read and parse exceptions
into one category and discarded response ID headers before reading a body. This
prevented diagnosis and could lose lookup identity on partial responses. It is a
proven observability defect, not evidence that inference itself is broken.

New receipts retain a fixed stage/category and bounded numeric errno; no arbitrary
exception text. UUID-only provider response header IDs survive read/parse failure or
HTTP errors. Conflicting header IDs are labeled conflicting, never silently selected.
These fields do not prove execution, settle billing, refund reservations, authorize
retry or rewrite old receipts. No new request identifier is fabricated for the old run.
The change is exercised with fake credentials/HTTP only, without sockets or providers.

## Next condition

A fresh live attempt is **not justified as a demonstrated transport fix yet**. The
current DNS error needs resolution through ordinary permitted environment access;
then an authorized narrow read can seek a provider record. Any candidate must be
correlated honestly; missing records still do not establish zero charge. If exact
identity remains unavailable, preserve this run permanently as unknown. A later
separate admission could test recovered transport, but must not recycle this slot or
claim the original request failed terminally. No semantic or model-capability result
exists for this attempt.

## Verification

10 mocked transport tests and 12 real-controller/fake-provider cycle tests passed.
Independent review found no blocker; header IDs remain correlation hints, not
settlement proof. `git diff --check` passed. No denied socket tests were retried,
no full-suite clean claim, and no historical receipt or reservation was rewritten.

## Ordinary-access follow-up and offline baseline inspection

At 19:46:34 UTC, one unauthenticated GET to the exact configured official
`https://api.cheaperinference.com/v1/models` through the normal direct client route
failed with `URLError` / `gaierror(8)`, no HTTP response. No alternate proxy, host,
DNS setting, account setting or escalation was tried. Evidence:
`target/garden-ready/ordinary-metadata-read.json`.

The current session policy is workspace-write with restricted network and automatic
approval review. The operation returned DNS failure, not an explicit permission or
approval denial. This does not prove unrestricted network access or identify whether
the restriction is responsible. Because ordinary access failed, no further authenticated
billing lookup or general poll was made. Environmental recovery is now the prerequisite
for read-only reconciliation; provider execution/usage/cost remain unknown.

Unknown garden evidence now has its own pinned replay index. The explicit garden
replay command prints null total cost, a separately labeled settled subtotal, unknown
usage and retained reservations. Four replay tests passed, including fresh-process
identity, no dispatch, unchanged files and rejection of tampered/missing evidence.

Read-only M0 inspection compared against main baseline
`8b74019f9073dbaa5f9d13ebf9f7df7c40694e79`. Both protected test files are byte-identical;
the runtime detection/probe section is also identical. `m0_forecast_default.rs:503`
expects exit 3 based on no available runtime; Runtime::detect instead accepts a binary
whose `--version` works, independently of daemon/container availability. The preserved
suite observed exit 126; that log does not identify its deeper executor cause.
`m0_harness.rs:258` calls `scratch("run1")` again before reading; `scratch` deletes the
whole directory at line 41. This proves the pre-existing fixture-removal defect when
the real-runtime branch is entered. These are baseline contract/environment issues,
not introduced garden/replay regressions. No container, real selftest or protected test
was rerun or edited to make the suite green. Prior socket-bind denials remain respected.
