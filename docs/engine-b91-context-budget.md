# B9.1 decision context: compaction without changing historical outcomes

Offline continuation after `38cee19`, no model calls, target executions, adoption,
new credentials, downloads or security changes. Trial11 remains incomplete, its STOP
and expired allowance intact. This is representation work, not semantic validation.

## Representation and preserved information

`b91-decision-context@v1` is constructed only after the existing full native source,
origin, lineage, generated-experiment and evidence checks. The full pre-compaction
view has a SHA-256 binding; original records remain in the native store and journals.
That hash does not replace evidence the model needs to see.

The visible context retains:

- Both original parent meanings and canonical identities; complete relational
  proposals, development, admissions and current/withdrawn/superseded flags.
- Current state/source/attention bindings, exact allowed revision ancestry,
  initial/revision/null rules and verbatim no-insight retention requirements.
- All current canonical observation IDs, their evidence binding, receipt-table
  indexes, outcomes and known/unknown exits. Assertion execution and returned values
  remain unknown; source predictions remain inferences. Timeout booleans do not become
  observed passes/failures merely because a raw summary contains them.
- Source code, the generated experiment's exact harness, requirement mappings,
  **rationales and finite limits**, findings, receipt/admission/source/bundle identity.
- Complete applicable envelope and payload contracts, immutable criteria, explicit
  no-authority semantics and unchanged validators/resource limits.

Compaction is mechanical, with no generative summarizer:

1. Case outcomes appear once in the conservatively normalized evidence catalog,
   joined to the displayed return tables by the original `return_index`. Duplicate
   case maps and repeated interpretation text are removed.
2. Supplemental audit-only `checked_snapshot`, `source_episode_root` and internal
   generated-origin/readmission fields stay host-side. The visible view retains the
   original generated proposal ID and current receipt/source/bundle/admission links.
   Native validation still checks the complete origin before compaction.
3. Historical per-entry bindings are omitted; current binding and allowed lineage
   remain explicit. No historical meaning/development/status is removed.
4. Repeated empty experiment `args`/`expected_stdout` are factored into explicit
   `case_defaults`, alongside exact case IDs. Nonempty or unknown invocation fields
   are rejected, not discarded. Reconstruction is tested byte-value exactly.
5. Repeated system-message field inventories move to their already-supplied contracts.
   A defined `text512` type avoids repeating its nonblank/512 UTF-8-byte description.
   Required wrappers, fields, schemas, limits and semantic rules are unchanged.

The model authored experiment rationale was deliberately **not** removed: independent
review found it can carry the hypothesis or the relationship between a check and a
want. No raw logs, hidden acceptance-test source or private review text are introduced.

## Measured capacity and limits

The matrix applies all six stage contracts to the **same retained terminal Trial11
snapshot**. It is a pressure comparison, not a claim about historical dispatch sizes.
Exact measurements are in `target/b91-context-growth-{before,after}.json`.

| Stage contract | Before bytes | After bytes | Headroom to 16,384 |
|---|---:|---:|---:|
| Initial relation | 16,261 | 13,894 | 2,490 |
| Experiment | 15,845 | 13,643 | 2,741 |
| Post-experiment relation | 16,273 | 13,906 | 2,478 |
| Boundary patch | 14,950 | 12,781 | 3,603 |
| Post-verification relation | 16,277 | 13,910 | 2,474 |
| Final proposal only | 18,656 | 16,290 | 94 |

Both pending stage requests now prepare through the ordinary projection without
calling a provider or changing source/state. The user context is source-bound and
full-history hashes are unchanged. However, the final projection is **before** a new
revision is generated/admitted: 94 bytes is not sufficient evidence of readiness.

Size-only future-revision fixtures (not generated output or adopted guidance) each
have a legal 3,363-byte compact proposal, with individual meaning fields below 512
UTF-8 bytes. Accounting for the extra history entry and removal of the now-unneeded
allowed-lineage entry yields:

- ASCII fixture: final request **19,446 bytes**.
- Unicode fixture: final request **25,046 bytes**, because the existing transport's
  ASCII JSON escaping expands non-ASCII text.

`target/b91-context-future-growth.json` records these calculations. Meaning is not
truncated, older failures are not deleted to fit, and the 16 KiB limit is not raised.
Growth refusal remains pre-dispatch with no slot consumption. These counterexamples
show that individually valid proposal limits do not guarantee cumulative context fits.

**Capacity remains a real limit.** A faithful continuation must either use a separately
reviewed, explicitly bounded larger context envelope or a further demonstrated
lossless representation. There is no claim that safe semantic compaction can fit
arbitrary valid history into 16 KiB, or that a live retry under that cap will finish.

## Smallest next admission and implementation boundary

The passed seed/experiment/patch/reverification need not be repeated. The minimal live
work is **two Luna requests, zero target executions**: post-verification revision and
then a final unexecuted proposal, with independent review and fresh accounting on a
source-bound continuation of the preserved state. The original trial must stay closed;
its STOP, journal and expired allowance must not be reopened.

A continuation-specific admission path still needs to bind that historical prefix,
new model-call reservations and zero execution authority. The current fresh-trial
runner is not permission to rerun passed stages or append under an expired grant.
A practical candidate for separate review is a **32 KiB wire cap** for this two-request
continuation, with an explicitly reviewed input-token reservation. That is a proposed
bounded change, **not implemented or authorized here**. The measured 19–25 KiB growth
cases explain the need; it is not a blanket unlimited-context request or a bill guarantee.
Per-call completion/time controls can remain 8,192 tokens and 240 seconds. No live
admission is currently granted.

## Verification

- Focused assembly/schema/lineage/native rejection checks: **33 passed**, 25.364s,
  `target/b91-context-focused-final.log`.
- New projection tests: **3 passed**, 1.686s, `target/b91-context-growth-tests.log`:
  immutable inputs, exact observation IDs/index joins, timeout unknowns, preserved
  meanings and rationales, reversible defaults, and valid-sized history overflow
  without truncation or slot consumption.
- Real Rust/Python six-stage fixture sequences (original DeepSeek, Luna, and native
  lineage): **3 passed**, 171.796s, `target/b91-context-native-six-stage.log`.
  These use the explicit cfg(test) worker and authored provider fixtures; no target
  commands or live requests run, and fixture success is not a live B9.1 pass.
- Full Python discovery: **633 tests: 615 passed, 18 skipped**, zero failures/errors,
  189.934s, `target/b91-context-full-python.log`. Live/target opt-ins stayed disabled.
- Final stricter rejection of unknown invocation metadata was checked by the affected
  suites afterward; exact result below. No new full Rust workspace sweep is claimed.
- Independent review by `readiness_review` found no blocking semantic/authority loss,
  while explicitly confirming that final-stage capacity remains unresolved.

Initial focused failures were stale representation assertions (duplicate case table,
old system phrase/type spelling), not ignored semantic failures. Assertions now check
canonical normalized evidence and the unchanged semantic rule in its actual contract.
The original failing log remains `target/b91-context-focused.log`; no frozen scenario,
acceptance test, historical output or protected file was edited.

Final guard check: `PYTHONPATH=scripts python3 -m unittest test_engine_b91_context test_engine_b91_project_envelope -v` — **20 passed**, 13.606s,
`target/b91-context-final-strictness.log`. Unknown extra invocation fields are rejected
rather than silently removed. `git diff --check` passed. The full-suite result above
precedes this final strictness-only guard; affected tests were rerun afterward.


## Finite continuation profile and final preflight (2026-10-25)

The 16,384-byte cap is a **nonprotected implementation default**, introduced in
`engine-synthetic-transport.py` at `5c1b7a1` and the B9.1 envelope at `994d632`.
It is not an exact user data boundary or a frozen PRD acceptance criterion.
The synthetic-only allowlist, source/provenance checks and old admissions remain
unchanged. The earlier proposed profile is now implemented offline, explicitly
selected only for Luna's last two stages: 32,768 UTF-8 wire bytes, 32,768 reserved
input tokens, 8,192 completion tokens and 240 seconds per request. Default callers
retain the original 16 KiB profile and serialization. No live grant is created.

The read-only capacity tool is:

```sh
python3 scripts/engine-b91-continuation-capacity.py \
  --parent-admission target/b91-live-trial11.admission.json
```

It verifies the exact settled four-call prefix, successful patch and unchanged
experiment receipts, source/criteria/current commitment, no pending operation,
and two retained relational revisions. It does not dispatch or adopt anything.
`target/b91-continuation-capacity-utf8.json` records the successfully measured
prefix before its native admission expired: pending revision 13,910 bytes and
final proposal 16,290 bytes. One additional native-valid proposal is at most
4,096 compact UTF-8 bytes. UTF-8 outer JSON escaping expands that by at most 2x;
512 additional bytes conservatively cover fixed identity/status/list metadata
(measured bound 177). Thus final worst-case wire size is **24,994 bytes**, leaving
**7,774 bytes** under 32 KiB. This is a prefix-specific bound, not arbitrary
history capacity. Actual size must still be checked before every dispatch.
ASCII escaping is deliberately not used for this new profile: DEL and Unicode
would invalidate a naive 3x expansion assumption. Wire bytes are not a tokenizer
proof; metered input tokens remain independently reserved and checked.

The final fresh-process preflight refused `changed desires or requirements`.
Read-only diagnosis established that the source, requirements revision and state
root were unchanged, but `acceptance/guidance.rs` correctly requires an open
scope-bound resource admission to carry guidance across the previous source
change. Its wall allowance has expired, so the current request exposes no parents
and consequently no mapped supplemental context. This is not lost historical
intent, nor permission to replace it with authored context. Do not bypass this
check or remove Trial 11's STOP.

**Capacity repair is verified; live continuation is not yet launch-ready.** The
remaining ordinary integration is a new, separate prefix-bound continuation
admission/ledger, with fresh native resource lifecycle admission, exactly two
calls, zero execution/patch authority, and checks after legitimate guidance
rebinding. It must preserve the old closed journal/STOP, reject source/intent
changes, and reconcile interruption without redispatch. Tests must demonstrate
that a final unexecuted proposal remains representable without granting action
authority. This wiring is not supplied by the capacity calculator or by passing
an optional Python argument; current fresh-trial driver still authorizes only its
existing full-trial shape. No protected-file exception is needed for this work.
The passed four execution groups need not be repeated.

Proposed subsequent live envelope, after that offline wiring and fresh approval:
**two gpt-6-luna calls, zero target executions**, 32 KiB wire and 32,768 input tokens
per call, 8,192 completion tokens and 240 seconds each; total model reservation
81,920 tokens. Post-verification revision followed by final proposal only, with
independent review, no forced adoption, no retries of unknown outcomes. Historical
incomplete Trial 11 remains incomplete regardless of any separately linked result.

Official public metadata retrieved 2026-10-25 from
<https://api.cheaperinference.com/public/models/gpt-6-luna> is retained in
`target/b91-continuation-luna-public.json`. Uncached input/output prices were
$0.065/$0.325 per million tokens. Doubling the two input reservations adds an
estimated **$0.00212992** at the uncached input rate. The full two-call token-budget
estimate is **$0.00958464**, not a guaranteed bill; cache-write/read or other
applicable billing must be reconciled from actual receipts. Published cache-write
rate was $0.1625/M, so the incremental input estimate at that rate is $0.0053248.
No authenticated request or inference was needed for this public metadata check.

Verification: full Python discovery **638 tests, 619 passed, 19 skipped**, no
failures, 186.116 seconds (`target/b91-capacity-full-python.log`). This precedes
one added growth-boundary case; all five capacity boundary cases then passed
(`target/b91-capacity-boundary.log`). Native late-stage reservation accounting
passed with the cfg(test)-only worker and fake provider, 52.161 seconds
(`target/b91-capacity-native-accounting.log`); focused transport/context checks
passed 21 tests (`target/b91-capacity-focused.log`). No real target execution,
model call, acceptance, maturity or completed B9.1 claim follows from fixtures.

Independent final review (`readiness_review`) found no blocking cap implementation issue,
and explicitly confirmed the remaining admission/carryover gap and token-versus-byte
qualification. Final five-case rerun passed in a fresh process
(`target/b91-capacity-final-boundary.log`). Parent driver/provider/origin/STOP SHA-256
checks all still match the measured prefix. No historical evidence was rewritten.

The separate continuation implementation and final offline evidence now live in
[engine-b91-continuation.md](engine-b91-continuation.md). It resolves the admission
and native guidance-carryover wiring described above while preserving the stopped
parent. No new live allowance is granted by this implementation.
