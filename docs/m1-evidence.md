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
| Adversarial suite (17 named cases) | `crates/tsr-policy/tests/adversarial.rs` |
| Profile-format documentation | `docs/ratchet-profile.md` |

## Commands and output

### `cargo build --workspace`

```
$ cargo build --workspace
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 7.39s
BUILD_EXIT=0
```

### `cargo fmt --check`

```
$ cargo fmt --check
FMT_EXIT=0
```

No diff was printed: the tree is rustfmt-clean, so no `cargo fmt` fix-up was needed.

### `cargo clippy --workspace --all-targets -- -D warnings`

```
$ cargo clippy --workspace --all-targets -- -D warnings
    Checking tsr-policy v0.0.1 (/work/crates/tsr-policy)
    Checking tsr-cli v0.0.1 (/work/crates/tsr-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 24.12s
CLIPPY_EXIT=0
```

### `cargo test -p tsr-policy --test adversarial`

```
$ cargo test -p tsr-policy --test adversarial
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.88s
     Running tests/adversarial.rs (target/debug/deps/adversarial-0287258e070fb5da)

running 17 tests
test budget_exhaustion_blocks_dispatch ... ok
test candidate_without_forecast_at_current_version_is_denied_with_reason ... ok
test empty_path_prefix_is_rejected_or_matches_nothing ... ok
test expired_approval_is_invalid ... ok
test altered_diff_invalidates_approval ... ok
test injection_in_comment_cannot_self_approve ... ok
test forecast_at_a_stale_version_does_not_satisfy_the_gate ... ok
test injection_in_filename_cannot_widen_scope ... ok
test injection_in_readme_text_cannot_widen_scope ... ok
test approval_revoked_between_decision_and_dispatch_blocks ... ok
test injection_in_stack_trace_cannot_widen_scope ... ok
test model_proposed_command_outside_allowlist_is_denied ... ok
test scope_change_invalidates_approval ... ok
test unparseable_expiry_is_expired ... ok
test wait_action_does_not_exist ... ok
test prefix_without_separator_does_not_match_siblings ... ok
test altered_snapshot_invalidates_approval ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

ADV_EXIT=0
```

The suite carries the twelve required named cases plus five added by the security repair
(`prefix_without_separator_does_not_match_siblings`, `empty_path_prefix_is_rejected_or_matches_nothing`,
`unparseable_expiry_is_expired`) and the two extra approval-binding cases. Every case asserts the
exact decision (`allow | deny | ask`) and that the reason is a single non-empty line.

### `cargo test --workspace --no-fail-fast`

```
$ cargo test --workspace --no-fail-fast
   Compiling tsr-policy v0.0.1 (/work/crates/tsr-policy)
   Compiling tsr-cli v0.0.1 (/work/crates/tsr-cli)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1m 05s
     Running unittests src/lib.rs (target/debug/deps/tsr_cli-bfc110a9450f264d)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/main.rs (target/debug/deps/tsr-aa4906be1af6b52b)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/m0_reconstruct.rs (target/debug/deps/m0_reconstruct-df9928932b4ecd9a)

running 1 test
test m0_session_reconstructs_from_the_log_alone ... FAILED

failures:

---- m0_session_reconstructs_from_the_log_alone stdout ----

thread 'm0_session_reconstructs_from_the_log_alone' panicked at crates/tsr-cli/tests/m0_reconstruct.rs:23:5:
M0 fixture missing at /work/crates/tsr-cli/../../fixtures/m0/session.events.jsonl: record a session through `tsr hook` and save its events.jsonl here
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

failures:
    m0_session_reconstructs_from_the_log_alone

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p tsr-cli --test m0_reconstruct`
     Running tests/policy_check.rs (target/debug/deps/policy_check-ff55f6911e119c9d)

running 3 tests
test gated_action_without_forecast_is_denied_naming_what_to_record ... ok
test injection_readme_does_not_widen_scope ... ok
test malformed_event_exits_three_with_json_error_on_stderr ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.31s

     Running unittests src/lib.rs (target/debug/deps/tsr_domain-66289bdfb27e0bdc)

running 6 tests
test tests::candidate_id_commits_to_parents_and_rejects_self_conflict ... ok
test tests::proxy_prints_the_engineering_name_never_the_term ... ok
test tests::probability_rejects_nonfinite_and_out_of_range ... ok
test tests::hash_is_deterministic_and_length_prefixed ... ok
test tests::event_id_changes_with_parents_and_round_trips_json ... ok
test tests::preference_is_short_and_typed ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

     Running unittests src/lib.rs (target/debug/deps/tsr_policy-34a5154a1b509641)

running 52 tests
test action::tests::approval_and_stop_hooks_map_to_their_kinds ... ok
test action::tests::edit_tools_map_to_propose_patch ... ok
test action::tests::bash_maps_to_run_approved_test ... ok
test action::tests::exactly_the_three_gated_kinds_are_gated ... ok
test action::tests::generate_tools_map_to_generate ... ok
test action::tests::inspect_tools_map_to_inspect ... ok
test action::tests::unknown_tools_are_not_typed ... ok
test action::tests::wait_is_a_state_not_an_action ... ok
test approval::tests::altered_artifact_digest_invalidates ... ok
test approval::tests::altered_snapshot_digest_invalidates ... ok
test approval::tests::changed_scope_invalidates ... ok
test approval::tests::different_task_or_candidate_invalidates ... ok
test approval::tests::expiry_is_inclusive ... ok
test approval::tests::from_event_ignores_other_kinds ... ok
test approval::tests::from_event_rejects_a_scope_without_digests ... ok
test budget::tests::a_zero_cost_reservation_always_fits ... ok
test approval::tests::scope_encoding_round_trips_through_an_event ... ok
test budget::tests::commit_moves_the_hold_into_consumed ... ok
test budget::tests::release_frees_the_budget ... ok
test budget::tests::reservation_is_taken_before_dispatch ... ok
test budget::tests::the_error_names_the_first_exhausted_budget ... ok
test budget::tests::two_in_flight_reservations_cannot_jointly_overrun ... ok
test gate::tests::absolute_inspect_path_is_denied ... ok
test gate::tests::allowlisted_test_id_is_allowed_with_a_forecast ... ok
test gate::tests::candidate_id_is_used_for_the_forecast_match ... ok
test gate::tests::decision_as_str_is_the_lowercase_word ... ok
test gate::tests::boundary_without_an_approval_asks ... ok
test gate::tests::dispatch_allows_when_the_approval_still_holds ... ok
test gate::tests::dispatch_denies_when_the_approval_is_missing ... ok
test gate::tests::dispatch_denies_when_the_approval_was_revoked ... ok
test gate::tests::dispatch_recheck_uses_the_single_approval_predicate ... ok
test approval::tests::valid_approval_passes ... ok
test budget::tests::exhaustion_blocks_the_next_dispatch ... ok
test gate::tests::exhausted_budget_blocks_dispatch ... ok
test gate::tests::gated_action_without_a_forecast_is_denied_naming_what_to_record ... ok
test gate::tests::forecast_at_a_different_version_does_not_satisfy_the_gate ... ok
test gate::tests::inspect_path_inside_both_scopes_is_allowed ... ok
test gate::tests::dispatch_denies_when_the_snapshot_changed_since_approval ... ok
test gate::tests::generate_and_stop_are_allowed_when_in_the_grant ... ok
test gate::tests::inspect_path_escaping_with_dotdot_is_denied ... ok
test approval::tests::revoked_between_decision_and_dispatch ... ok
test gate::tests::patch_within_both_ceilings_is_allowed ... ok
test gate::tests::rts_tool_outside_the_grant_is_denied ... ok
test gate::tests::patch_over_the_smaller_ceiling_is_denied ... ok
test gate::tests::unused_imports_are_referenced ... ok
test gate::tests::inspect_path_outside_the_grant_scope_is_denied ... ok
test gate::tests::test_id_outside_the_allowlist_is_denied ... ok
test gate::tests::kind_outside_the_grant_is_denied ... ok
test profile::tests::duplicate_test_id_is_rejected ... ok
test profile::tests::default_profile_parses_and_round_trips ... ok
test profile::tests::empty_test_id_is_rejected ... ok
test profile::tests::test_command_lookup_by_id ... ok

test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/adversarial.rs (target/debug/deps/adversarial-32f3f628a7e2f9e1)

running 17 tests
test altered_snapshot_invalidates_approval ... ok
test altered_diff_invalidates_approval ... ok
test approval_revoked_between_decision_and_dispatch_blocks ... ok
test empty_path_prefix_is_rejected_or_matches_nothing ... ok
test forecast_at_a_stale_version_does_not_satisfy_the_gate ... ok
test injection_in_comment_cannot_self_approve ... ok
test expired_approval_is_invalid ... ok
test injection_in_filename_cannot_widen_scope ... ok
test injection_in_stack_trace_cannot_widen_scope ... ok
test model_proposed_command_outside_allowlist_is_denied ... ok
test prefix_without_separator_does_not_match_siblings ... ok
test scope_change_invalidates_approval ... ok
test unparseable_expiry_is_expired ... ok
test wait_action_does_not_exist ... ok
test budget_exhaustion_blocks_dispatch ... ok
test candidate_without_forecast_at_current_version_is_denied_with_reason ... ok
test injection_in_readme_text_cannot_widen_scope ... ok

test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.07s

     Running unittests src/lib.rs (target/debug/deps/tsr_store-4ef4425254935056)

running 4 tests
test tests::a_parent_must_already_be_in_the_log ... ok
test tests::head_root_changes_with_every_append ... ok
test tests::append_is_idempotent_by_identity_and_ordered_by_seq ... ok
test tests::the_database_refuses_update_and_delete ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

   Doc-tests tsr_cli

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests tsr_domain

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests tsr_policy

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests tsr_store

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: 1 target failed:
    `-p tsr-cli --test m0_reconstruct`
WS_EXIT=101
```

**The one red target is not tsr-policy's.** `cargo test --workspace --no-fail-fast` is red only on
`crates/tsr-cli/tests/m0_reconstruct.rs`, which panics with
`M0 fixture missing at .../fixtures/m0/session.events.jsonl`. That is the pre-existing lane-01 M0
gap (the fixture is recorded by `tsr hook`, which M1 does not implement); it is red by design and
unrelated to `crates/tsr-policy`. Every other target is green: tsr-policy 52 unit + 17 adversarial,
tsr-cli `policy_check` 3, tsr-domain 6, tsr-store 4.

### `tsr policy check` acceptance commands

```
$ ./target/debug/tsr policy check profiles/default.toml crates/tsr-cli/tests/fixtures/policy/injection-readme.json
{"decision":"deny","reason":"inspect path /etc/passwd is absolute; the read scope is relative to the repository copy"}
EXIT=1

$ ./target/debug/tsr policy check profiles/default.toml crates/tsr-cli/tests/fixtures/policy/no-forecast.json
{"decision":"deny","reason":"record a forecast for RunApprovedTest at state version 1 before this action"}
EXIT=1
```

Both acceptance cases deny with a single-line reason and exit 1, as the fixtures' `_comment` fields
specify. The first shows repository text (a README prompt injection) carrying no authority: the
`Inspect` of `/etc/passwd` is denied because the read scope is relative to the repository copy. The
second shows forecast-gated execution: the gated `RunApprovedTest` is denied until a `Forecast`
exists for the candidate at the current state version, and the reason names what to record.