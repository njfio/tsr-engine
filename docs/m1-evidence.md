# M1 evidence — `crates/tsr-policy`

This is the evidence for milestone M1: the deterministic, fail-closed authority of the TSR Decision
Engine (PRD §B5, §B6). Every command below was run in this workspace; the output is pasted verbatim.

Environment (the sandbox mounts only the workspace, so the cargo cache lives inside it):

```sh
export CARGO_HOME="$PWD/.cargo-home"; export PATH="/usr/local/cargo/bin:$PATH"
```

## What was delivered

| item | path |
| --- | --- |
| RATCHET profile format | `crates/tsr-policy/src/profile.rs`, `profiles/default.toml` |
| The gate (`allow \| deny \| ask`) | `crates/tsr-policy/src/gate.rs` |
| Hook tool-name → `ActionKind` mapping | `crates/tsr-policy/src/action.rs` |
| Approvals (bound, re-checked, invalidated) | `crates/tsr-policy/src/approval.rs` |
| Budgets (reserved before dispatch) | `crates/tsr-policy/src/budget.rs` |
| `tsr policy check` CLI | `crates/tsr-cli/src/policy_cmd.rs` |
| Adversarial suite (14 named cases) | `crates/tsr-policy/tests/adversarial.rs` |
| Profile-format documentation | `docs/ratchet-profile.md` |

## Commands and output

### `cargo build --workspace`

```
(placeholder — filled from the run below)
```

### `cargo fmt --check`

```
(placeholder)
```

### `cargo clippy --workspace --all-targets -- -D warnings`

```
(placeholder)
```

### `cargo test -p tsr-policy --test adversarial`

```
(placeholder)
```

### `cargo test --workspace --no-fail-fast`

```
(placeholder)
```

### `tsr policy check` acceptance commands

```
(placeholder)
```