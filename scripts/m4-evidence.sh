#!/usr/bin/env bash
# scripts/m4-evidence.sh — regenerate the M4 (Calibrate) exit evidence.
#
# Run from the workspace root:
#
#     bash scripts/m4-evidence.sh > out.md
#
# It builds the binary, works only in a fresh directory under target/ (never in
# fixtures/ or the tree), and for every claim the evidence doc makes prints the
# exact command (prefixed `$ `) followed by its real stdout, stderr and exit
# code in a fenced block. No network is used. Exit code is 0.
#
# The base for any diff it prints is $TSR_MAIN if set, else
# `git merge-base HEAD origin/main`.
#
# Re-running on the same commit reproduces every block except test timings and
# timestamp-derived ids.

set -u

# --- environment -----------------------------------------------------------
export PATH=/usr/local/cargo/bin:$PATH
export CARGO_HOME=$PWD/.cargo-home

ROOT=$PWD
WORK="$ROOT/target/m4-evidence-work"
BIN="$ROOT/target/debug/tsr"

# The base for any diff we print.
if [ -n "${TSR_MAIN:-}" ]; then
    MAIN="$TSR_MAIN"
else
    MAIN=$(git merge-base HEAD origin/main 2>/dev/null || echo "")
fi

# --- helpers ---------------------------------------------------------------

# run <label> <cmd...>  — print `$ cmd`, then stdout+stderr and the exit code.
run() {
    local label="$1"; shift
    printf '$'
    printf ' %q' "$@"
    printf '\n'
    local out rc
    out=$("$@" 2>&1)
    rc=$?
    printf '%s\n' "$out"
    printf 'exit %d\n' "$rc"
    printf '\n'
}

# run_in <dir> <label> <cmd...> — same, but run in <dir>.
run_in() {
    local dir="$1"; shift
    local label="$1"; shift
    printf '$'
    printf ' %q' "$@"
    printf '\n'
    local out rc
    out=$(cd "$dir" && "$@" 2>&1)
    rc=$?
    printf '%s\n' "$out"
    printf 'exit %d\n' "$rc"
    printf '\n'
}

# run_show <label> <cmd...> — like run but prints only the exit code line after
# the output (used where the output is long and the exit code is the claim).
run_show() {
    run "$@"
}

# run_cargo <label> <cargo args...> — run cargo, and if it fails with the
# sandbox's transient "can't find crate" state (a concurrent cargo racing on the
# default target/), retry once in a private target directory that is removed
# first, so the retry starts from a clean state and its output is reproducible.
# The retry's output is what is printed.
run_cargo() {
    local label="$1"; shift
    printf '$'
    printf ' %q' cargo "$@"
    printf '\n'
    local out rc
    out=$(cargo "$@" 2>&1)
    rc=$?
    if printf '%s' "$out" | grep -q "can't find crate"; then
        printf '%s\n' "$out"
        printf 'exit %d\n' "$rc"
        local fresh="$ROOT/target/m4-evidence-fresh"
        rm -rf "$fresh"
        printf '# retrying in CARGO_TARGET_DIR=%s (removed first)\n' "$fresh"
        printf '$'
        printf ' %q' env "CARGO_TARGET_DIR=$fresh" cargo "$@"
        printf '\n'
        out=$(CARGO_TARGET_DIR="$fresh" cargo "$@" 2>&1)
        rc=$?
    fi
    printf '%s\n' "$out"
    printf 'exit %d\n' "$rc"
    printf '\n'
}

section() {
    printf '## %s\n\n' "$1"
}

# --- fresh work directory --------------------------------------------------
rm -rf "$WORK"
mkdir -p "$WORK"
cd "$WORK" || exit 1

# --- build -----------------------------------------------------------------
section "Build, format, lint"

run_cargo "build" build --workspace
run "fmt" cargo fmt --all -- --check
run_cargo "clippy" clippy --workspace --all-targets -- -D warnings

# --- toolchain and tip -----------------------------------------------------
section "Toolchain and tip"

run "rustc" rustc --version
run "git" git rev-parse HEAD

# --- the synthetic corpus --------------------------------------------------
section "The pipeline over fixtures/m4/"

DS="$WORK/m4ds"
BUNDLE="$WORK/m4-synth"

run "dataset" "$BIN" train dataset \
    --events "$ROOT/fixtures/m4/events.jsonl" \
    --tasks "$ROOT/fixtures/m4/tasks.jsonl" \
    --labels "$ROOT/fixtures/m4/labels.jsonl" \
    --by time --calibrate 0.2 --holdout 0.2 --seed 7 --out "$DS"

run "fit" "$BIN" train fit "$DS" --out "$BUNDLE" --support-floor 30

run "verify" "$BIN" train verify "$BUNDLE"
run "verify --rebuild" "$BIN" train verify "$BUNDLE" --rebuild "$DS"

run "report" "$BIN" train report "$BUNDLE" --dataset "$DS" --seed 7
run "report --json" "$BIN" train report "$BUNDLE" --dataset "$DS" --seed 7 --json

run "apply" "$BIN" train apply "$BUNDLE" --dataset "$DS" --side held_out

# --- --by repo -------------------------------------------------------------
section "--by repo"

REPO_DS="$WORK/m4-repo"
REPO_BUNDLE="$WORK/m4-repo-models"

run "dataset --by repo" "$BIN" train dataset \
    --events "$ROOT/fixtures/m4/events.jsonl" \
    --tasks "$ROOT/fixtures/m4/tasks.jsonl" \
    --labels "$ROOT/fixtures/m4/labels.jsonl" \
    --by repo --calibrate 0.2 --holdout 0.2 --seed 7 --out "$REPO_DS"

run "fit --by repo" "$BIN" train fit "$REPO_DS" --out "$REPO_BUNDLE"

run "report --by repo" "$BIN" train report "$REPO_BUNDLE" --dataset "$REPO_DS" --seed 7

# --- apply on a task -------------------------------------------------------
section "apply on a task"

run "apply --side fit" "$BIN" train apply "$BUNDLE" --dataset "$DS" --side fit

# --- verify on a tampered manifest -----------------------------------------
section "verify on a tampered manifest"

# (1) A manifest field edited to a value the canonical rendering does not
#     produce: `state` is a closed enum, so `active` is refused.
TAMPERED="$WORK/m4-tampered"
cp -R "$BUNDLE" "$TAMPERED"
sed -i.bak 's/^state = .*/state = "active"/' "$TAMPERED/manifest.toml"
rm -f "$TAMPERED/manifest.toml.bak"
run "verify tampered state" "$BIN" train verify "$TAMPERED"

# (2) `code_commit` edited to another value. It is recorded provenance, not a
#     verified property: `verify` accepts it and prints it labelled.
CODECOMMIT="$WORK/m4-codecommit"
cp -R "$BUNDLE" "$CODECOMMIT"
sed -i.bak 's/^code_commit = .*/code_commit = "deadbeef"/' "$CODECOMMIT/manifest.toml"
rm -f "$CODECOMMIT/manifest.toml.bak"
run "verify edited code_commit" "$BIN" train verify "$CODECOMMIT"

# --- report on a foreign dataset -------------------------------------------
section "report on a foreign dataset"

run "report foreign" "$BIN" train report "$BUNDLE" --dataset "$REPO_DS"

# --- known limitations -----------------------------------------------------
section "Known limitations"

# (0) an `--out` directory the user cannot write to still exits 1. The sandbox
#     runs as root, so a mode-bit test cannot make a directory unwritable (root
#     writes a 0555 directory); the read-only filesystem `/sys/kernel/debug` is
#     an existing, empty, unwritable directory that root cannot write either.
#     Nothing is created in the tree: the probe only reads /sys.
LIM_RO="/sys/kernel/debug"
if [ -d "$LIM_RO" ] && [ -w "$LIM_RO" ] 2>/dev/null; then
    printf '# %s is writable here; skipping the unwritable --out probe\n\n' "$LIM_RO"
else
    run "dataset, --out is an unwritable directory" "$BIN" train dataset \
        --events "$ROOT/fixtures/m4/events.jsonl" \
        --tasks "$ROOT/fixtures/m4/tasks.jsonl" \
        --labels "$ROOT/fixtures/m4/labels.jsonl" \
        --by time --calibrate 0.2 --holdout 0.2 --seed 7 --out "$LIM_RO"
    run "fit, --out is an unwritable directory" "$BIN" train fit "$DS" --out "$LIM_RO"
fi

# (1) `fit` with `holdout-access.jsonl` as a directory exits 1.
LIM_DS="$WORK/m4-lim-holdout"
cp -R "$DS" "$LIM_DS"
rm -f "$LIM_DS/holdout-access.jsonl"
mkdir -p "$LIM_DS/holdout-access.jsonl"
run "fit, holdout-access.jsonl is a directory" "$BIN" train fit "$LIM_DS" --out "$WORK/m4-lim-holdout-b"

# (2) `report` with `<bundle>.report.json` as a directory exits 1 after printing.
LIM_B="$WORK/m4-lim-report"
cp -R "$BUNDLE" "$LIM_B"
rm -f "$LIM_B.report.json"
mkdir -p "$LIM_B.report.json"
run "report, <bundle>.report.json is a directory" "$BIN" train report "$LIM_B" --dataset "$DS" --seed 7

# (3) the `--out` create probe leaves empty ancestor directories behind when a
#     later step refuses.
LIM_OUT="$WORK/m4-lim-out"
rm -rf "$LIM_OUT"
: > "$WORK/m4-lim-empty-tasks.jsonl"
run "dataset, empty tasks index, --out under new ancestors" "$BIN" train dataset \
    --events "$ROOT/fixtures/m4/events.jsonl" \
    --tasks "$WORK/m4-lim-empty-tasks.jsonl" \
    --labels "$ROOT/fixtures/m4/labels.jsonl" \
    --by time --calibrate 0.2 --holdout 0.2 --seed 7 --out "$LIM_OUT/a/b/c"
printf 'directories left behind:\n'
find "$LIM_OUT" -type d 2>/dev/null | sort || true
printf '\n'

# (4) `dataset --store` reads the whole file to check its header; a store with a
#     valid header but a corrupt body exits 1.
LIM_DB="$WORK/m4-lim-corrupt.db"
printf 'TSRDB1\n' > "$LIM_DB"
head -c 200 /dev/urandom >> "$LIM_DB"
run "dataset, --store with a valid header and a corrupt body" "$BIN" train dataset \
    --store "$LIM_DB" --tasks "$ROOT/fixtures/m4/tasks.jsonl" --out "$WORK/m4-lim-store-ds"

# (5) the default store path creates an empty store when none exists.
LIM_DEF="$WORK/m4-lim-default-store"
rm -rf "$LIM_DEF"
mkdir -p "$LIM_DEF"
printf '$ TSR_DB=%s/.tsr/tsr.db tsr train dataset --tasks fixtures/m4/tasks.jsonl --out %s/ds\n' "$LIM_DEF" "$LIM_DEF"
( cd "$LIM_DEF" && TSR_DB="$LIM_DEF/.tsr/tsr.db" "$BIN" train dataset \
    --tasks "$ROOT/fixtures/m4/tasks.jsonl" --out "$LIM_DEF/ds" 2>&1 )
printf 'exit %d\n' "$?"
printf 'store created:\n'
ls -l "$LIM_DEF/.tsr/tsr.db" 2>/dev/null || true
printf '\n'

# (6) `counts_digest` sits outside the bundle id: a consistent edit of a count
#     and its digest is accepted. `unindexed_tasks` 0 -> 1, with the digest
#     recomputed over the seven counts.
LIM_CD="$WORK/m4-lim-counts"
cp -R "$DS" "$LIM_CD"
python3 - "$LIM_CD/data-manifest.json" <<'PY'
import json, sys
p = sys.argv[1]
d = json.load(open(p))
d["unindexed_tasks"] = 1
d["counts_digest"] = "abb355d365e3a4e6fdc077f8a888371df1237c3e09cadc7298e1b161417e8e97"
json.dump(d, open(p, "w"), indent=2, sort_keys=True)
PY
run "fit, unindexed_tasks and counts_digest edited together" "$BIN" train fit "$LIM_CD" --out "$WORK/m4-lim-counts-b"

# --- the whole workspace test suite ----------------------------------------
section "The whole workspace test suite"

printf '$ cargo test --workspace --no-fail-fast\n'
TEST_LOG="$WORK/workspace-test.log"
cargo test --workspace --no-fail-fast >"$TEST_LOG" 2>&1
TEST_RC=$?
if grep -q "can't find crate" "$TEST_LOG"; then
    TEST_FRESH="$ROOT/target/m4-evidence-fresh"
    rm -rf "$TEST_FRESH"
    printf '# retrying in CARGO_TARGET_DIR=%s (removed first)\n' "$TEST_FRESH"
    printf '$ CARGO_TARGET_DIR=%s cargo test --workspace --no-fail-fast\n' "$TEST_FRESH"
    CARGO_TARGET_DIR="$TEST_FRESH" cargo test --workspace --no-fail-fast >"$TEST_LOG" 2>&1
    TEST_RC=$?
fi
# `cargo test` runs the test binaries in parallel and their stdout interleaves,
# so a `test result:` line can share a physical line with a test name. Extract
# the result substring wherever it appears.
sed -n 's/.*\(test result: [^ ]*\. [0-9][0-9]* passed; [0-9][0-9]* failed; [0-9][0-9]* ignored; [0-9][0-9]* measured; [0-9][0-9]* filtered out; finished in [0-9.]*s\).*/\1/p' "$TEST_LOG" || true
printf '\n'
printf 'failed tests:\n'
grep -oE 'test [A-Za-z0-9_:]+ \.\.\. FAILED' "$TEST_LOG" | sed 's/^test //; s/ \.\.\. FAILED$//' | sort -u || true
printf '\n'
printf 'exit %d\n' "$TEST_RC"
printf '\n'

# --- diff base -------------------------------------------------------------
section "Diff base"

if [ -n "$MAIN" ]; then
    run "diff base" git rev-parse "$MAIN"
else
    printf 'no base: $TSR_MAIN unset and `git merge-base HEAD origin/main` failed\n\n'
fi

exit 0
