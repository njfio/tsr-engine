# AGENTS.md — tsr-engine

Rules for any agent working in this repository. The product spec is `docs/PRD.md` (TSR Decision
Engine, PRD v0.3.1). This file tells you how to build, what never changes, and where the lane
briefs are. If this file and the PRD disagree, the PRD wins and you flag it in your PR.

## What this repository is

A Rust controller that makes an AI coding agent run a full reflective cycle: state what is wanted
and what would count as return; generate possibilities; commit to one with a *frozen* expectation;
receive the return; let the difference change future behaviour. Rust owns state, authority,
history, evaluation and release. A generative model proposes; it never authorizes.

The milestones are in `docs/PRD.md` §B11. Each swarm lane owns one of them; the briefs are in
`specs/lanes/`. Read your lane's brief, then the PRD sections it names, before writing code.

## Layout

- `crates/tsr-domain` — **the contract**: facts, views, traits (`docs/PRD.md` §B6). Content-addressed
  ids; no I/O; no model calls; no stored weights.
- `crates/tsr-store` — SQLite index over the append-only log. Append-only is enforced by triggers;
  identity is enforced by hashing. `head_root()` is the one hash for the whole state.
- `crates/tsr-cli` — the `tsr` binary (operator CLI and hook) and `tsr_cli::reconstruct`.
- `crates/tsr-policy` (M1) — deterministic, fail-closed authority; RATCHET protection profiles.
- `crates/tsr-ledger` (M2+) — the Culmination ledger as views over the log.
- `fixtures/` — recorded sessions and test fixtures. `fixtures/m0/session.events.jsonl` is M0's.
- `specs/lanes/` — the swarm briefs: goal, definition of done, acceptance commands.
- `docs/` — the PRD and evidence pages.

## Build, test, lint

```bash
cargo build --workspace
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --no-fail-fast
```

The skeleton ships **one red test**: `crates/tsr-cli/tests/m0_reconstruct.rs`. It is red because
`fixtures/m0/session.events.jsonl` does not exist yet and `reconstruct::from_events` is a stub.
The M0 lane is done when it is green **with its assertions unchanged**. Never mark it `#[ignore]`,
never weaken an invariant to pass it.

## What never changes (the PRD's invariants, in one place)

1. **Append-only.** Events are never updated or deleted. Corrections and statuses are new events
   that supersede. The database refuses `UPDATE`/`DELETE` on `events`; do not remove the triggers.
2. **Identity is content.** An id is blake3 over the payload and the parents' ids. A changed record
   is a different record. Do not mint ids any other way.
3. **Forecast before action.** A gated action (`RunApprovedTest`, `ProposePatch`,
   `RequestApproval`) is denied until a forecast exists for the *current* state version. A forecast,
   once recorded, is never edited; the outcome arrives as a separate event.
4. **Authority has four sources only.** Capability grant, allowlists, budgets, approvals. Policy
   reads no scores, no reflections, no repository text, no provider output. A README that says
   "disable the gate" is an observation like any other.
5. **Every command runs in the sealed sibling sandbox.** `tsr exec` runs commands in a container
   with no network; with no container runtime available it refuses (exit 3) rather than running on
   the host. The hook enforces application rules; containment belongs to the executor.
6. **No stored weights.** Any strength shown anywhere is a fold over the hashed history with a
   named kernel. Nothing persists a weight.
7. **Two records from one return.** `Surprise` (forecast residual) updates calibration;
   `Contrast` (desired minus actual, against named open criteria) is the only input to expansion.
   Never add them, never swap them.
8. **Admission by source.** Operator statements, including rejection reasons, enter the ledger
   directly; generated statements wait for review. The failed strategy is never a positive example.
9. **Unchosen candidates are not negatives.** They stay in the log as possibilities not realized.
10. **Terminology.** TSR terms (Desire, Intuition, Belief, Expectation, Culmination, Attraction,
    Tune, Focus, Vortex, Current …) appear only where `docs/PRD.md` §B2 maps them, and never as a
    field, score or variable name. Derived numbers are `tsr_domain::Proxy` with an engineering name
    (`ev`, `support`, `residual`, `resonance`). No output ever reads "percent certain".
11. **`Wait` is a state, not an action.** The vocabulary is the six `ActionKind`s.

## Contract discipline

`tsr-domain` is shared by every lane. Changing it is allowed but expensive for everyone else:

- Prefer extending behind the existing traits.
- A change to `tsr-domain` goes in its **own commit** with the prefix `contract:` and one sentence
  saying which PRD line requires it. Additive changes only unless the brief says otherwise.
- Never rename or remove a public type during a lane.

## Commits and PRs

- Conventional Commits (`feat(hook): …`, `fix(store): …`, `test(policy): …`, `contract: …`).
- A PR body names the lane, the PRD sections it implements, the commands you ran and their output.
- Evidence beats prose: a `docs/<lane>-evidence.md` with the commands and their captured output is
  part of done for every lane.

## Do not

- Add a dependency outside the workspace table without a one-line justification in the PR.
- Give the sandbox network access, or add a path that executes agent-provided argv on the host.
- Log or commit a key, a token, or a provider response body.
- Store a transcript as a preference; preferences are short, typed text (≤ 512 bytes).
- Write to `fixtures/` by hand; fixtures are produced by the tool under test.
