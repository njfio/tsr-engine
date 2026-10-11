# Lane 02 — Policy, the RATCHET profile (milestone M1)

**Workspace:** a second clone of `tsr-engine` on branch `lane/policy`.
**Owns:** `crates/tsr-policy` (new), `profiles/`, `docs/ratchet-profile.md`, `docs/m1-evidence.md`,
the `tsr policy` subcommand.
**Reads first:** `AGENTS.md`; `docs/PRD.md` §B5 (actions, policy, approvals, budgets),
§B6 (contracts), §B4 (where the hook boundary sits), §B9 adversarial cases, §B11 (M1 row).
**Exit evidence (PRD B11, M1):** adversarial suite passes with zero unauthorized effects; every
denial explains itself to the model.

This lane is where RATCHET finally gets built: its protection-profile format is this crate's
configuration, and the engine's gate is its first customer.

## Goal (the text handed to `ds-swarm run`)

Implement `crates/tsr-policy`, the deterministic, fail-closed authority of the TSR Decision Engine,
per docs/PRD.md B5 and B6. Read AGENTS.md, then B5, B6, B4 and the adversarial cases in B9, then
specs/lanes/02-policy.md. Deliver: (1) a RATCHET protection-profile format in TOML — read-scope
allowlist (paths and rts tools), exact allowlisted test command ids, budgets (generator tokens, tool
minutes, wall clock, generator calls, patch bytes), approval requirements for boundary actions
(promote diff, non-allowlisted command, network); an example `profiles/default.toml`; (2) the gate:
given a profile, the task's grant, the log so far and a proposed candidate, return allow | deny | ask
with a one-line reason the model reads; (3) budgets reserved before dispatch and exhaustion blocking
further dispatch; (4) approvals bound to task + artifact digest + repository snapshot digest + scope +
expiry, re-checked at dispatch, and invalidated by any change; (5) forecast-gated execution: a gated
action (RunApprovedTest, ProposePatch, RequestApproval) is denied until a Forecast event exists for
the current state version, with the reason naming what to record; (6) the hook tool-name →
ActionKind mapping as a pure function; (7) `tsr policy check <profile.toml> <event.json>` printing
the decision as JSON. Policy reads no scores, no reflections, no repository text and no provider
output: authority comes only from the grant, the allowlists, the budgets and the approvals. Write an
adversarial test suite with at least twelve named cases, including: prompt injection in README text,
filenames, comments and stack traces cannot widen scope or self-approve; an altered diff or snapshot
invalidates an approval; an approval revoked between decision and dispatch blocks; budget
exhaustion blocks dispatch; a candidate without a forecast at the current version is denied with
the reason; a model-proposed command outside the allowlist is denied; a `Wait` action does not
exist. Do not implement execution and do not change the tsr-store schema. Keep changes to
tsr-domain additive and in a `contract:` commit. Write docs/ratchet-profile.md and
docs/m1-evidence.md with the commands you ran and their output.

## Definition of done (`--dod`)

cargo build, cargo fmt --check, cargo clippy --workspace --all-targets -- -D warnings and
cargo test --workspace --no-fail-fast pass (the M0 test may still be red if lane 01 has not landed;
every other test passes). `crates/tsr-policy` exists with the adversarial suite of at least twelve
named cases passing. `tsr policy check profiles/default.toml <event.json>` prints allow | deny | ask
with a reason. tsr-policy has no dependency on any model client, on tsr-decide, or on anything that
learns. docs/ratchet-profile.md documents the format with the example profile. tsr-domain changes,
if any, are additive and in a `contract:` commit.

## Out of scope for this lane

Running anything (the executor is lane 01). Calibration, the ledger, the reflect view. Any UI.

## Acceptance commands

```bash
cargo test -p tsr-policy
./target/debug/tsr policy check profiles/default.toml tests/fixtures/policy/injection-readme.json
./target/debug/tsr policy check profiles/default.toml tests/fixtures/policy/no-forecast.json
```

## Suggested run flags

```
--max-tokens 700000 --deadline-secs 7200 --max-agents 5 --reviews security,architect,correctness
```
