# Independently reviewed finite checks

This adds a bounded evidence path, not a more capable language model or a new
acceptance policy. No model call or candidate execution was performed in this
change. Deterministic tests use a fake runner and cannot count as operational
correctness evidence.

## Specification and coverage

The invoice specification requires multiplication for subtotal and subtraction
with a zero floor for discount. Those requirements independently imply these
cases; no reference repair or generated candidate was used to choose outputs:

| Requirement | Inputs | Expected | Limit |
| --- | --- | --- | --- |
| subtotal | 7, 0 | 0 | Zero quantity only |
| subtotal | 3, 4 | 12 | One ordinary nonoverflowing product |
| discount | 7, 3 | 4 | One ordinary subtraction |
| discount | 7, 8 | 0 | One excess discount |

Discount is called directly, independently of subtotal. The existing original
probes compose the functions and can confound their failures. The new harness
separates them. Equality, other boundaries and all other inputs remain uncovered.
Overflowing products are unspecified by the admitted requirement, so these checks
do not invent overflow semantics. A finite pass is **Unknown** for universal
semantic correctness. A completed, nontruncated exit-zero output mismatch is a
counterexample to its independently reviewed mapped requirement. Compilation,
transport, timeout, signal, truncation and missing cases remain Unknown here.
They are not automatically evidence of a semantic counterexample.

The template is
[`examples/engine-invoice/supplemental-checks.template.json`](../examples/engine-invoice/supplemental-checks.template.json).
It intentionally needs current source/requirement identities and explicit review
attestation. Its harness is source text, not an execution receipt. Independent
read-only agent review checked the four-case mapping; an operational admission
must still attest the exact bound mapping and harness for that episode.

## Mechanism and authority

`acceptance/reviewer/supplemental.rs` exposes `Bundle`, `Mapping`, `Observation`
and `CaseOutcome`. Operator admission freezes the exact current candidate,
requirements revision, harness, cases, per-requirement mapping, coverage limits,
and distinct author/reviewer/generator identities. Identity strings are operator
attestations, not authentication or proof of independent errors. All originals,
including protected tests, are preserved byte-for-byte. Additional immutable
harness files must occupy the `supplemental/` namespace without collisions. The
v1 adapter supports existing non-Cargo Rust probe projects and the existing
four-case/file/timeout limits; it does not add arbitrary host commands.

The generated check ProjectSpec needs a **separate prospective episode admission**
with tool/wall budgets and a source-bound verification-only action. The existing
verification planner already supported bare Rust; its named-Cargo-only admission
guard was the integration defect fixed here. Verification still has no patch,
requires resolved resources, binds exact configuration/source, freezes forecasts,
and executes through the existing sealed runner. A check manifest itself neither
authorizes nor executes anything.

The original episode imports a receipt only from an explicitly trusted local
controller-owned engine store. It validates that store's event/CAS history,
configuration, unchanged original and harness sources, source-bound verification
commitment, Rules receipt and matching completed execution event, individual
case identities/arguments/expected outputs, and prospective resource admission.
It freezes and rechecks main/resource roots and sources before append. Arbitrary
receipt JSON from a generator is not an import interface. A locally fabricated
store under the operator's control is outside this provenance trust boundary;
these are not cryptographic attestations from a remote executor.

Imported observations include the exact bundle, source and resource roots,
receipt identity, checked snapshot, mapping and finite findings. Import does not
charge execution twice: resources remain charged in the check episode and their
snapshot is attached as evidence. Historical records remain immutable. Repeated
linking of the same admission/receipt is idempotent. Source/requirements drift
marks imported observations historical and prevents relinking them as current.

`Prerequisite::SupplementalFinite { admission }` and
`Basis::SupplementalFinite { admission, receipt }` bind model findings to the
exact current imported observation. Missing, fabricated or mismatched evidence
is refused; this basis can establish only Unknown or an observed NotMet.

New observations participate in verifier freshness and appear in the isolated
verifier request. A current counterexample prevents a contradictory semantic
`met` report from validation/adoption. Existing acceptance projects as
`supplemental_conflict_review_required`, reopening the residual-desire view and
blocking accepted-only export without rewriting or rejecting the historical
review. An explicit operator `supplemental-revoke --admission ID --reason ...`
withdraws a mistaken mapping; receipts remain visible and verifier requests become
stale. This can restore a still-current earlier operator acceptance; it does not
create a new acceptance record. Finite passes never become `met` through
this mechanism. Supplemental evidence does not replace original protected test
receipts, perform operator review, satisfy desires, mint mature labels, or grant
automatic acceptance. Separate operator review remains the positive acceptance
boundary; report text is not observed execution evidence.

## Runnable interface (not executed here)

Use `tsr engine --directory ORIGINAL project verifier` with:

- `supplemental-admit --file BOUND_BUNDLE.json` — append reviewed check admission.
- `supplemental-spec --admission ID` — print the immutable separate ProjectSpec.
- `supplemental-inspect` — inspect current/historical finite evidence.
- `supplemental-link --admission ID --from CHECK_RUN --receipt RECEIPT_ID` —
  validate and append a separately obtained controller receipt.

Initialize a fresh check run with the printed ProjectSpec using the ordinary
`project init --spec ...`. Admit a new lifecycle/tool budget with zero model
calls and zero patch bytes, then use `project verify-admit --state-root ...
--snapshot ...` and normal `project run --response ...`. No extra execution path
or second edit is introduced. `verify-admit` fails without a sufficient fresh
budget. This workflow is prepared, **not admitted or executed in this turn**.
Do not feed imported expected outputs or held-out comparison labels to a repair
generator as a reference answer.

## Remaining engine work, ranked against PRD

1. **Useful Intuition under conflicting wants remains the principal operational
   gap** (§B8). Shared desire conservation, standing loss, related expectation,
   revised commitment, two-stage possibilities and next attention are implemented.
   Preserved live conflicting-wants generation failed; sequential invoice repair
   does not erase that failure or demonstrate successful joint choice. Another
   identical retry has no evidence-backed justification.
2. **Domain-sufficient observed return is partial** (§B7/B8). This finite-check
   path can refute bounded arithmetic behavior without asking a model to do the
   arithmetic. The smallest next validation is a separately admitted, zero-model
   sealed check episode on a preserved synthetic candidate, followed by review
   of exact receipt binding and coverage. Positive semantic review still requires
   independent reasoning or a domain-specific complete decision procedure; finite
   examples cannot replace it. The prior Mistral shadow arithmetic error and
   invented execution claims remain failures, not solved by typed transport.
3. **Calibrated Expectation/Tier2 and superiority are data-dependent** (§B8/B9).
   Keep default fallback/shadow. The 151 qualified corpus tasks are not a count
   of mature independent reviewed episodes. Actual closure plus seven days,
   compatible event/family support, calibration and review/resource provenance
   are still required. This change adds no qualifying operational evidence.
4. **Expansion beyond admitted scopes remains an authority/domain dependency.**
   Existing attention and unchanged-guidance continuation work within admitted
   scope. New fundamental values, arbitrary observation discovery and unrestricted
   execution are neither specified nor authorized. Belief in the PRD is revisable
   momentary commitment, not a missing general semantic truth engine.

No additional deterministic core-selection defect was established by this audit.
The concrete correction was the missing evidence bridge and Rust verification
admission, not a new supervisor or endless verifier sampling campaign. A stronger
model might improve semantic inference, but these arithmetic requirements do not
require one to obtain useful counterexamples. They do require an independently
reviewed specification-to-check mapping and a real, separately admitted execution.

## Verification record

- `cargo +1.90.0 test -p tsr-cli --lib supplemental --offline`: **7 passed**.
  Tests use a fake runner, including fabricated/stale receipts, wrong case bindings,
  source/configuration drift, missing budgets, restart, idempotent import, no double
  charge, finite pass versus counterexample/timeout, typed overclaims, revoked
  mapping, and accepted-to-conflict projection with unchanged controls.
- Final `cargo +1.90.0 test -p tsr-cli --lib --offline`: **194 passed, 0 failed,
  3 existing ignored**. Ephemeral loopback-stub tests required execution outside
  the filesystem/network sandbox; no live provider or project code was invoked.
- All-target CLI clippy with `-D warnings`, CLI build, changed-file rustfmt and
  `git diff --check` passed. Original protected tests and fixtures were unchanged.
- Independent read-only review identified and verified fixes for mixed import
  snapshots, stale verifier reports, and already-accepted export after a new
  counterexample. Final typed-path review found no remaining blocker.

Failures were preserved, not omitted: the first sandboxed broad run had 187 passes
and five socket-permission failures. The first unrestricted broad run had 192
passes and one stub result-quarantine assertion failure. The same unchanged
assertion passed isolated (1.40 seconds) and in the final complete run (42.65
seconds overall). Additional failure diagnostics were added; the earlier transient
failure's exact cause was not captured, so it is **not established** as a timing
issue. No deadline or acceptance assertion was weakened.

Logs are under `target/supplemental-evidence/`: `targeted.log`, `cli-lib.log`,
`cli-lib-unrestricted.log`, `transport-isolated.log`, `cli-lib-final.log`,
`clippy-final.log`, `build.log`, and `format.log`. These are local verification
artifacts, not mature operational episodes. No full workspace integration campaign,
model comparison, real supplemental execution or external action was run here.

A subsequent separately admitted execution is recorded in
[the one-episode live report](engine-supplemental-live.md). It established two
subtotal counterexamples and exposed an expected-output newline defect in the
example template. That historical episode is preserved; the future template and
finite-case outcome regression were corrected without rerunning it. No finite
pass or semantic acceptance is claimed for the affected discount observations.
