# AGENTS.md — tsr-engine

Rules for any agent working in this repository. The product spec is `docs/PRD.md` (TSR Decision
Engine, PRD v0.3.1); its milestones are §B11. What main does today, and where it falls short, is in
`README.md` (Status, Known limitations). If this file and the PRD disagree, the PRD wins and you
flag it in your PR.

## What this repository is

The design (PRD §B1): a Rust controller that makes an AI coding agent run a full reflective cycle:
state what is wanted and what would count as return; generate possibilities; commit to one with a
*frozen* expectation; receive the return; let the difference change future behaviour. Rust owns
state, authority, history, evaluation and release; a generative model proposes and never
authorizes. Main implements the recording, forecast and review parts of that (M0–M2); it does not
yet block actions, calibrate, decide or release (`README.md`).

## Layout

- `crates/tsr-domain` — **the contract**: facts, views, traits (`docs/PRD.md` §B6). Content-addressed
  ids; no I/O; no model calls; no stored weights.
- `crates/tsr-store` — SQLite index over the append-only log. Triggers refuse `UPDATE`/`DELETE` and an
  insert that names a missing parent; identity is enforced by hashing. `head_root()` is the one hash
  for the whole state.
- `crates/tsr-cli` — the `tsr` binary (`hook`, `exec`, `forecast`, `task`, `policy check`, `review`,
  `reflect`, `note`, `task reconcile`, `train`; `corpus`, `data`, `eval`, `model` exit 2) and
  `tsr_cli::reconstruct`.
- `crates/tsr-policy` (M1) — RATCHET profiles (TOML), the gate (`allow | deny | ask`), approvals,
  budgets. Only `tsr policy check` calls it; the hook does not.
- `crates/tsr-ledger` (M2) — the Culmination ledger as folds over the log.
- `crates/tsr-forecast` (M2) — named events, forecast resolution, Brier, log loss, reliability bins.
- `crates/tsr-learn` (M4) — the dataset and split, the logistic head, the isotonic calibrator, bundles
  and their verification, the controller-versus-generator report (`tsr train`).
- `profiles/default.toml` — the default RATCHET profile (`docs/ratchet-profile.md`).
- `fixtures/m0/` — M0's recorded sessions (frozen, below). `fixtures/m2/episodes/` — thirty synthetic
  episodes written by `crates/tsr-cli/tests/m2_episodes.rs`. `fixtures/m4/` — the synthetic M4 corpus
  (regenerate with `TSR_REGEN_FIXTURES=1`).
- `scripts/` — `m2-evidence.sh` (prints the text of `docs/m2-evidence.md` to stdout; needs
  `origin/main` or `TSR_MAIN`) and the two store builders it uses; `m4-evidence.sh` (prints the text
  of `docs/m4-evidence.md`).
- `specs/lanes/` — lane briefs; `specs/lanes/reviews/` — the independent reviews behind repair briefs.
- `docs/` — the PRD, the profile format, the evidence pages.

## Build, test, lint

The lanes use Rust 1.90 (`rust:1.90-bookworm` in the sandbox); `Cargo.toml` declares
`rust-version = "1.85"` and there is no `rust-toolchain` file, so name the toolchain:

```bash
cargo +1.90.0 build --workspace
cargo +1.90.0 fmt --all -- --check
cargo +1.90.0 clippy --workspace --all-targets -- -D warnings
cargo +1.90.0 test --workspace --no-fail-fast
```

Clippy with `-D warnings` is clean on 1.90 only: newer stable clippy (1.97 tested) fails on
`filter_next` in `crates/tsr-policy/src/gate.rs`. The lane sandboxes run plain `cargo` in the
`rust:1.90-bookworm` image.

When commands run inside ds-swarm's Docker sandbox, only the workspace is mounted, so keep the
cargo cache inside it or every command re-downloads the registry:

```bash
export CARGO_HOME="$PWD/.cargo-home"   # first line of every shell in the sandbox; it is gitignored
```

**Tests and the container runtime.** No test needs a Docker daemon. Two `tsr-cli` tests pass only
where no `podman` or `docker` binary answers `--version`, and fail wherever one does, with or
without a daemon:

- `m0_forecast_default::exec_resolves_the_session_task_and_refuses_without_one` expects `tsr exec` to
  exit 3.
- `m0_harness::selftest_is_reproducible_structurally_excluding_duration_ms_by_name` runs the real
  self-test only when a runtime is detected, then deletes its output directory before reading it.

Everything else (261 tests on main) passes in both cases.

**Fixtures.** `TSR_REGEN_FIXTURES=1 cargo +1.90.0 test -p tsr-cli --test m2_episodes` rewrites
`fixtures/m2/episodes/` (byte-identical while the generator is unchanged) and writes the git-ignored
`fixtures/m2/store/tsr.db`; a plain run only compares. Fixtures are produced by the tool under test,
never by hand.

## Frozen files

No lane edits these. The verifiers compare them byte for byte with the lane's base
(`specs/lanes/01c-real-session.md`; `scripts/m2-evidence.sh` checks
`git diff --stat <main> -- crates/tsr-cli/tests/m0_reconstruct.rs fixtures/m0` is empty):

- `crates/tsr-cli/tests/m0_reconstruct.rs` — the M0 exit test. Never edit its assertions, never mark
  it `#[ignore]`, never weaken an invariant to pass it.
- `fixtures/m0/session.events.jsonl` — the operator's real `tsr hook --selftest` record.
- `fixtures/m0/claude-session.events.jsonl` — the log of the real Claude Code session on a
  disposable copy of rts (M0's exit evidence, read by `tests/m0_real_session.rs`), and
  `fixtures/m0/claude-session.rts.diff` — the patch that session made (no test reads it).

Also pinned: `crates/tsr-cli/tests/fixtures/m0/real-record-4d8d7b8.events.jsonl`, an earlier real
record that the `exec.rs` unit tests replay. `tsr hook --selftest` writes `fixtures/m0/` under the
current directory, so never run it from the repository root.

Lane 01's M0 files belong to the M0 lanes. The M2 briefs from `04r` on list them under
**Never edits**; the fullest list (`04rrr`, `m2-r4a`) is
`crates/tsr-cli/src/{hook,exec,store_path,task,lib}.rs` (except a `pub mod` line), the M0 arms of
`main.rs` (task open/status/export, forecast, hook, exec, policy), and
`crates/tsr-cli/tests/{m0_real_session,m0_harness,m0_forecast_default}.rs`.

## Lanes

How the swarms are launched and watched: `specs/lanes/README.md`.

- A lane is one milestone or one repair, run by ds-swarm from one brief in `specs/lanes/`, in its own
  checkout on its own branch (`lane/<name>`). Two runs never share a checkout.
- A brief's header names the **Workspace** (branch and base), what the lane **Owns** and what it
  **Reads first**; repair briefs add **Never edits** (from `04r` on) or **Frozen** (`01c`). The body
  is **Goal** (the text handed to `ds-swarm run`), **Definition of done** (`--dod`), **Out of
  scope**, and in many briefs **Acceptance commands** and **Suggested run flags**. Read `AGENTS.md`,
  your brief, then the PRD sections it names, before writing code. Change only what the lane owns.
- An independent review lands in `specs/lanes/reviews/`, and its findings become the next repair
  brief (`04-reflect` → `04r` → `04rr` → `04rrr` → `m2-r4a` → `m2-r4b`). Each finding is one task: a
  test that fails on the current code and names the finding's id, then the fix, then a commit naming
  the id. A finding that is wrong at the tip gets a report naming the file and line that refute it.
- Evidence is part of done: `docs/<milestone>-evidence.md` with every command and its captured
  output. M2's comes from `bash scripts/m2-evidence.sh > out.md`; regenerate it, never hand-patch
  it.
- A lane lands on `main` by PR, reviewed against its Definition of done and the PRD, not against the
  director's summary.

## What never changes (the PRD's invariants, in one place)

These are rules for the code you write. Where main does not meet one yet, `README.md` (Known
limitations) says so; do not cite main's behaviour as permission.

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
5. **Every command runs in the sealed sandbox container.** `tsr exec` runs commands in a container
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

## Do not

- Add a dependency outside the workspace table without a one-line justification in the PR.
- Give the sandbox network access, or add a path that executes agent-provided argv on the host.
- Log or commit a key, a token, or a provider response body.
- Store a transcript as a preference; preferences are short, typed text (≤ 512 bytes).
- Write to `fixtures/` by hand; fixtures are produced by the tool under test.
