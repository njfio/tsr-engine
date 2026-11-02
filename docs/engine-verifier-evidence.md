# Typed verifier evidence and advisory closure

New local verifier activity requires `verifier-admission@v2`. Historical v1
admissions, requests and reports remain readable and unchanged; they do not acquire
v2 guarantees. Legacy configured local requests cannot be dispatched again through
the transport. Generic trusted-host v1/report compatibility remains for existing
operator-assisted history; it is not a typed or automatic acceptance route.

## Typed claims, not prose filtering

The admission's `prerequisites` map covers exactly the same requirement keys as
`coverage`. Each value is one of:

- `{"kind":"static_semantic"}`: source-based inference, never observed execution.
- `{"kind":"observed_tests","test_identity":"HASH"}`: observed approved suite outcome.
- `{"kind":"mixed","test_identity":"HASH"}`: static inference and observed suite outcome.

`project verifier inspect` exposes `test_identity`, computed by the controller from
a versioned suite contract: Cargo/probe mode, named package, full-workspace oracle,
probe entry, cases and expected outputs, protected files, runtime image/user and
timeout. Candidate source has its own exact snapshot identity. A change in suite
selection cannot reuse the same test identity.

The controller freezes `observed_tests` as a structured object in the request:
`receipt`, `snapshot`, `test_identity`, `outcome` (`passed|failed|unknown`), or null
if there is no matching current-source observation. Only the immutable Rules-sourced
receipt may supply this object. Stale receipts remain in history without becoming
current evidence. The source/evidence map remains available independently.

Every v2 assessment adds `basis`:

- `{"kind":"unknown"}` requires `finding: unknown` and no invented receipt.
- `{"kind":"static_semantic"}` is allowed only for a static prerequisite.
- `{"kind":"observed_tests","observation":{...}}` must match the frozen controller
  object exactly; `met` requires passed, `not_met` requires failed.
- `{"kind":"mixed","observation":{...},"static_finding":"met|not_met|unknown"}`
  requires both the declared static component and the exact observation. `met`
  requires static met plus observed pass. A static counterexample can yield not_met
  even when tests passed; unknown never becomes met. Missing observation cannot
  silently downgrade a mixed criterion to static-only.

The normal local prompt and JSON schema carry these explicit prerequisites and
structured availability. No keyword sanitizer judges arbitrary natural language.
Explanations remain untrusted advisory prose; they cannot create an observation,
receipt, permission or proof. The operator must classify requirements correctly;
classifying an execution requirement as static would be an incorrect admission.
This mechanism rejects unsupported **typed** success claims. It does not establish
the truth of static findings, catch every misleading sentence, or fix Mistral's
observed addition-versus-multiplication error.

Structurally parsed but inadmissible reports receive a separate `rejected_report`
event, retaining their typed claims and rejection reason. Their response digest
and resource receipt remain linked through `finished`, with no adoptable report.
Malformed raw responses remain hash/metadata only under the repository's prohibition
on provider-body logging. Historical reports are not reinterpreted or rewritten.

## Honest terminal advisory episodes

```
tsr engine --directory RUN project-lifecycle-close-advisory --reason TEXT
```

This appends `TaskClosed` with an `advisory_only:` reason in the native lifecycle;
it records neither accepted nor rejected review, no outcome label and no maturity
eligibility. It leaves the desire ledger and acceptance state unchanged. It refuses
pending executions, executed project episodes, unfinished verifier invocations
(including the pre-reservation window), and unsettled resources. Settled unknown
usage remains charged. A repeated close is idempotent; a reviewed closure cannot
be converted into an advisory one. Closed episodes cannot dispatch or adopt a
verifier report. Reconstruction explicitly yields
`advisory_only_no_execution_label`, rather than treating advisory closure as corrupt
reviewed data or eligible training evidence.

The two prior shadow episodes were closed through this CLI after implementation
validation. Original reports and acceptance status were unchanged; token charges
remain 1,049 and 1,051. Their elapsed wall budgets had expired while the old closure
interface was unavailable; the overrun is retained, not forgiven or backdated.
They remain unlabeled, not accepted/rejected, and operationally ineligible.

## Verification and next dependency

No live model calls or project executions were performed in this implementation
turn. Tests use synthetic receipt fixtures and fabricated loopback transport stubs.
CLI library: **187 passed, 3 existing ignores**. Learning library: **137 passed**.
CLI/learning all-target clippy with `-D warnings`, offline CLI build, changed-file
format checks and diff checks passed. Independent review identified the missing
suite-selection fields; they were added and re-reviewed with collision controls.
Evidence and CLI closure comparisons: `target/verifier-typed/`.

Tests cover missing/stale/foreign/wrong receipt, wrong source/test identity/outcome,
fabricated passing claims, source-only analysis without execution, mixed inference,
uncertainty, replay and rejected-report retention. Closure controls cover restart,
idempotence, unchanged desires, unfinished invocations, unknown budget reservations,
blocked subsequent dispatch/adoption, and explicit eligibility withholding.

No unchanged model comparison is recommended. The new gate should first prevent
unsupported observed claims deterministically, which these regressions demonstrate.
A future comparison would have to evaluate **static semantic competence separately**
from mechanically checked receipt claims, with independently frozen counterexamples
and no automatic adoption. These changes alone do not justify expecting the same
Mistral weights to detect arithmetic defects. No new inference allowance is requested
or consumed here; the remaining semantic-verifier limitation is explicit.
