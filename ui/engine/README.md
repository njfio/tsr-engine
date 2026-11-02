# TSR Studio — local browser-project interface

Studio is a loopback interface to the native browser-project engine, not a hosted
service. Build `tsr` from the repository root, then run:

```sh
cargo build --locked
python3 ui/engine/server.py
```

Open http://127.0.0.1:8766/ yourself. The server does not open or refresh a browser.
The optional `sh ui/engine/service.sh start|status|stop` uses a current-user launchd
session on macOS; it is not a login-item installation. React and ReactDOM are
vendored with their license and provenance; no frontend installation is needed.

## Fresh checkout versus existing local project

A fresh checkout starts without historical projects or messages. Existing local
project records and inputs are preserved when reopening. Delivered games, raw
runtime journals, credentials and the original campaign are intentionally absent
from the public repository.

The current accounting/preparation adapter still expects an operator-provisioned,
source-bound campaign at `target/tsr-integrated-campaign-1` and the associated local
runtime configuration. Consequently **a fresh checkout is not immediately ready
for paid Send**. An operator must configure and admit that workflow; missing state
must remain an explicit blocker, not be replaced with fake responses or authority.
There is no public credential/bootstrap service. Historical recovery helpers are
bound to their original records and must not be applied to a different campaign.

## Actual behavior and boundaries

Conversation is the primary flow. Send saves an idempotent input, prepares a
source-bound native continuation when permitted, and displays recorded responses,
progress and errors. Input received during a run stays queued. Retained desires,
relationships, proposals, review, execution, observations and recorded rationale
summaries are inspectable. These are recorded summaries, not hidden model thoughts.
Unknown human fulfillment and user acceptance remain distinct from test results.

In-app decisions use exact disclosures, expiring one-shot challenges and durable
approve/decline records. They represent interaction with a loopback single-user
application, **not remote identity authentication**. No historical approval or
user message is automatically manufactured on installation.

Limited mode enforces the admitted dollar/call/operation/time allowance. The
existing limited workflow uses at most $10, 200 calls, 100 groups and 120 minutes.
An explicitly admitted unlimited campaign can remove cumulative limits; this is
not the default or a grant for every user. Accounting, capacity checks, bounded
recovery, separate consequential-action authority and unknown-effect blocking
remain. Stop preserves history and reservations; it does not prove an in-flight
external effect never occurred.

Mutations require exact loopback Host and Origin plus a per-server CSRF token.
Packet registration is CLI-only and validates source and policy bindings. Artifact
previews are sandboxed without same-origin authority. Credentials do not enter the
frontend. Keep this service on loopback; do not expose it as a public multi-user API.

## Verification

```sh
PYTHONPATH=ui/engine:.:scripts python3 -m unittest discover -s ui/engine -p 'test_*.py'
```

Tests use labeled authored state/transport, not paid model calls. Native integration
and isolated-browser opt-ins have separate prerequisites; see the integration
report for exact executed checks and exclusions. Passing fixtures does not prove
live autonomous desire development. The historical reports describe their own
source versions and are not current-run evidence.
