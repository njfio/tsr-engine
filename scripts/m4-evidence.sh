#!/usr/bin/env bash
# scripts/m4-evidence.sh — regenerate the M4 (Calibrate) exit evidence, docs/m4-evidence.md.
#
# Run from the workspace root (Linux, GNU coreutils, python3, git):
#
#     bash scripts/m4-evidence.sh > out.md
#
# It builds the binary with rust 1.90 and works only in fresh directories under target/
# (m4-evidence-work, -fresh, -cargo-home and -tmp; never in fixtures/ or the tree). For every claim
# the evidence doc makes it prints a heading and a fenced block: each command it ran (prefixed `$ `,
# paths relative to the workspace root, `.`) followed by its real output and `exit <code>`. Setup
# commands (copies, edits) are printed the same way, before the command under test. Long outputs
# are summarised (line count, sha256 and the lines that carry the claim); every summary line the
# script adds starts with `# `. Cargo's `Compiling`/`Checking`/`Finished` progress lines are
# omitted; every other line is kept. A short summary, "At a glance", is printed first.
#
# No network: every cargo step runs offline (CARGO_NET_OFFLINE=true) with a private CARGO_HOME
# under target/, seeded from a local registry (./.cargo-home, else $CARGO_HOME, else ~/.cargo).
# Without a usable one the script prints one line on stderr and exits 3 before running anything.
#
# Exit status: 0 when every step exits as the script expects; 1 when any step it reports exits
# otherwise (each is marked `UNEXPECTED` in its block, listed in "At a glance" and on stderr); 3
# when a precondition fails. The two m0 tests that fail where a docker CLI answers `--version`
# without a daemon are expected and named, not fatal.
#
# The base for any diff it prints is $TSR_MAIN if set, else `git merge-base HEAD origin/main`.
#
# Re-running on the same commit reproduces every line except the `finished in <n>s` test timings.
# Nothing it prints is derived from a clock. Across commits `git rev-parse HEAD` changes; across
# hosts the docker-dependent m0 tests and the unwritable-directory probes may differ (see the doc).
#
# The private target directory, CARGO_HOME and the tests' TMPDIR are removed when the script exits;
# the work directory target/m4-evidence-work is kept for inspection and removed at the next run.

set -u
umask 022
export LC_ALL=C TZ=UTC

die() { printf 'm4-evidence: %s\n' "$*" >&2; exit 3; }
note() { printf 'm4-evidence: %s\n' "$*" >&2; }

# --- preconditions -----------------------------------------------------------------------------
[ -f Cargo.toml ] && [ -d fixtures/m4 ] && [ -f scripts/m4-evidence.sh ] ||
    die "run from the workspace root: bash scripts/m4-evidence.sh > out.md"
for tool in git python3 sha256sum; do
    command -v "$tool" >/dev/null 2>&1 || die "$tool is required"
done
ROOT=$PWD

# The diff base, read before the environment is cleaned.
if [ -n "${TSR_MAIN:-}" ]; then
    BASE_FROM="\$TSR_MAIN=$TSR_MAIN"
    BASE=$(git rev-parse --verify -q "$TSR_MAIN^{commit}" 2>/dev/null) || BASE=""
else
    BASE_FROM="git merge-base HEAD origin/main"
    BASE=$(git merge-base HEAD origin/main 2>/dev/null) || BASE=""
fi

# Nothing from the caller's environment may change what the binary or cargo prints.
unset TSR_DB TSR_CODE_COMMIT TSR_SESSION TSR_REGEN_FIXTURES TSR_MAIN \
    RUSTFLAGS RUSTDOCFLAGS CARGO_ENCODED_RUSTFLAGS CARGO_BUILD_TARGET CARGO_TARGET_DIR \
    CARGO_BUILD_TARGET_DIR CARGO_INCREMENTAL RUST_BACKTRACE RUST_LIB_BACKTRACE

# Rust 1.90: a rustup install selects the 1.90.0 toolchain; otherwise the cargo on PATH must be it.
export PATH=/usr/local/cargo/bin:$PATH
if [ -z "${RUSTUP_TOOLCHAIN:-}" ] && command -v rustup >/dev/null 2>&1 &&
    rustup toolchain list 2>/dev/null | grep -q '^1\.90\.0'; then
    export RUSTUP_TOOLCHAIN=1.90.0
fi
case "$(rustc --version 2>/dev/null)" in
    "rustc 1.90."*) ;;
    *) die "needs rust 1.90 with rustfmt and clippy (found: $(rustc --version 2>&1 | head -1))" ;;
esac
cargo fmt --version >/dev/null 2>&1 && cargo clippy --version >/dev/null 2>&1 ||
    die "needs rustfmt and clippy for rust 1.90"

# Paths, relative to the workspace root, exactly as the printed commands use them.
W=./target/m4-evidence-work
FRESH=./target/m4-evidence-fresh
CHOME=./target/m4-evidence-cargo-home
TMPD=./target/m4-evidence-tmp
BIN=$FRESH/debug/tsr
FX=./fixtures/m4

[ -d "$W" ] && chmod -R u+w "$W" 2>/dev/null
rm -rf "$W" "$FRESH" "$CHOME" "$TMPD"
mkdir -p "$W/.out" "$CHOME/registry"
trap 'rm -rf "$ROOT/$FRESH" "$ROOT/$CHOME" "$ROOT/$TMPD"' EXIT

# A usable offline registry, checked before anything runs: copy a local index and crate cache
# into the private CARGO_HOME and ask cargo to fetch every host crate in Cargo.lock offline.
SRC_HOME=""
for h in "$ROOT/.cargo-home" "${CARGO_HOME:-}" "${HOME:-/nonexistent}/.cargo"; do
    if [ -n "$h" ] && [ -d "$h/registry/index" ] && [ -d "$h/registry/cache" ]; then
        SRC_HOME=$h
        break
    fi
done
[ -n "$SRC_HOME" ] ||
    die "no offline cargo registry (none of ./.cargo-home, \$CARGO_HOME, ~/.cargo has registry/index and registry/cache): run 'cargo fetch' once with network, then re-run"
cp -R "$SRC_HOME/registry/index" "$SRC_HOME/registry/cache" "$CHOME/registry/" ||
    die "cannot copy the cargo registry from $SRC_HOME"
export CARGO_HOME="$ROOT/${CHOME#./}" CARGO_NET_OFFLINE=true
HOST=$(rustc -vV | sed -n 's/^host: //p')
CARGO_TARGET_DIR=$FRESH cargo fetch --locked --target "$HOST" >"$W/.out/fetch.log" 2>&1 ||
    die "no usable offline cargo registry: $SRC_HOME lacks a crate Cargo.lock needs ($(grep -m1 'error' "$W/.out/fetch.log")); run 'cargo fetch' once with network, then re-run"
note "registry from $SRC_HOME, toolchain $(rustc --version)"

# --- helpers -------------------------------------------------------------------------------------

OUT="$W/.out/last.out"
ERR="$W/.out/last.err"
CUR=""
UNEXPECTED=()

# The two m0 tests that fail where a docker CLI answers `--version` without a daemon.
ALLOWED_FAILURES="m0_forecast_default::exec_resolves_the_session_task_and_refuses_without_one
m0_harness::selftest_is_reproducible_structurally_excluding_duration_ms_by_name"

section() { printf '## %s\n\n' "$1"; }
block() { CUR=$1; printf '### %s\n\n```text\n' "$1"; }
endblock() { printf '```\n\n'; }
say() { printf '# %s\n' "$*"; }

# A command line as a shell would read it: plain words as they are, others single-quoted.
qcmd() {
    local out="" a
    for a in "$@"; do
        if [[ "$a" =~ ^[A-Za-z0-9_./:=@%+,-]+$ ]]; then
            out+=" $a"
        else
            out+=" '${a//\'/\'\\\'\'}'"
        fi
    done
    printf '%s' "${out# }"
}

show() { # print a captured file, ending with a newline
    if [ -s "$1" ]; then
        cat "$1"
        [ -z "$(tail -c1 "$1")" ] || printf '\n'
    fi
}

check() { # check <expected> <rc> <command text>
    LAST_RC=$2
    printf 'exit %d\n' "$2"
    if [ "$2" != "$1" ]; then
        printf 'UNEXPECTED: expected exit %s\n' "$1"
        UNEXPECTED+=("$CUR: \`$3\` exited $2, expected $1")
    fi
}

# x <expected exit> <cmd...> — print `$ cmd`, run it, print stdout and stderr as they interleave.
x() {
    local exp=$1 c rc
    shift
    c=$(qcmd "$@")
    printf '$ %s\n' "$c"
    "$@" >"$OUT" 2>&1 </dev/null
    rc=$?
    show "$OUT"
    check "$exp" "$rc" "$c"
}

# xs <expected exit> <shell text> — the same for a command that needs the shell (redirects, cd).
xs() {
    local exp=$1 rc
    printf '$ %s\n' "$2"
    bash -c "$2" >"$OUT" 2>&1 </dev/null
    rc=$?
    show "$OUT"
    check "$exp" "$rc" "$2"
}

# xcargo <expected exit> <cargo args...> — cargo in the private target directory, printed with it;
# progress lines are omitted, warnings and errors are kept.
xcargo() {
    local exp=$1 c rc
    shift
    c=$(qcmd env "CARGO_TARGET_DIR=$FRESH" cargo "$@")
    printf '$ %s\n' "$c"
    env "CARGO_TARGET_DIR=$FRESH" cargo "$@" >"$OUT" 2>&1 </dev/null
    rc=$?
    say "Compiling, Checking and Finished progress lines omitted; every other line follows"
    grep -vE '^[[:space:]]*(Compiling|Checking|Finished|Blocking waiting for file lock) ' "$OUT" \
        >"$OUT.kept"
    show "$OUT.kept"
    check "$exp" "$rc" "$c"
    CARGO_KEPT=$(wc -l <"$OUT.kept")
    CARGO_RC=$rc
}

sha() { sha256sum "$1" | cut -d' ' -f1; }
nlines() { wc -l <"$1" | tr -d ' '; }

# xsum <expected exit> <mode> <cmd...> — run with stdout and stderr captured apart, print a summary
# of stdout and all of stderr. Modes:
#   head:<n>          the first n lines and the last line, plus the count of `uncalibrated` lines
#   json              the top-level keys and the headline fields of `report --json`
#   nobins            the report without its reliability `bin` lines and its floor sweep
#   same:<file>:<what> whether stdout is byte-identical to <file>
xsum() {
    local exp=$1 mode=$2 c rc n
    shift 2
    c=$(qcmd "$@")
    printf '$ %s\n' "$c"
    "$@" >"$OUT" 2>"$ERR" </dev/null
    rc=$?
    n=$(nlines "$OUT")
    case "$mode" in
        head:*)
            local k=${mode#head:}
            if [ "$n" -le $((k + 1)) ]; then
                say "stdout: $n lines, sha256 $(sha "$OUT"); all of it"
                show "$OUT"
            else
                say "stdout: $n lines, sha256 $(sha "$OUT"); the first $k lines, then the last"
                head -n "$k" "$OUT"
                say "($((n - k - 1)) lines not shown)"
                tail -n 1 "$OUT"
            fi
            say "lines printing \`uncalibrated\`: $(grep -c ' uncalibrated ' "$OUT")"
            ;;
        json)
            say "stdout: $n lines, sha256 $(sha "$OUT")"
            python3 -I - "$OUT" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
print("# top-level keys, in order: " + " ".join(d))
print("# headline fields, as the JSON holds them (reliability bins omitted):")
for k in ("bundle_id", "split_digest", "calibrator", "seed", "scored", "excluded", "uncalibrated"):
    print(f"{k}={json.dumps(d[k])}")
for k in ("controller", "generator"):
    f = d[k]
    print(f"{k} n={f['n']} brier={json.dumps(f['brier'])} log_loss={json.dumps(f['log_loss'])}")
for fam, by in d["families"].items():
    for k in ("controller", "generator"):
        f = by[k]
        print(f"families.{fam}.{k} n={f['n']} brier={json.dumps(f['brier'])} log_loss={json.dumps(f['log_loss'])}")
print("paired=" + json.dumps(d["paired"], separators=(",", ":")))
for fam, p in d["paired_by_family"].items():
    print(f"paired_by_family.{fam}=" + json.dumps(p, separators=(",", ":")))
PY
            ;;
        nobins)
            say "stdout: $n lines, sha256 $(sha "$OUT"); shown without its reliability \`bin\` lines and its floor sweep"
            python3 -I - "$OUT" <<'PY'
import sys
skip = False
for line in open(sys.argv[1]).read().splitlines():
    if line.startswith("floor sweep"):
        skip = True
    elif line == "":
        skip = False
    if skip or line.lstrip().startswith("bin ["):
        continue
    print(line)
PY
            ;;
        same:*)
            local rest=${mode#same:}
            local ref=${rest%%:*} what=${rest#*:}
            if cmp -s "$OUT" "$ref"; then
                say "stdout: $n lines, sha256 $(sha "$OUT"), byte-identical to the stdout of $what"
            else
                say "stdout: $n lines, sha256 $(sha "$OUT"), NOT identical to the stdout of $what"
            fi
            ;;
    esac
    if [ -s "$ERR" ]; then
        say "stderr:"
        show "$ERR"
    else
        say "stderr: empty"
    fi
    check "$exp" "$rc" "$c"
}

keep() { cp "$OUT" "$W/.out/$1"; } # keep the last captured stdout under a name

# --- the evidence --------------------------------------------------------------------------------

body() {
    note "toolchain and build"
    section "Toolchain and tip"
    block "Toolchain, commit and tree state"
    x 0 rustc --version
    keep rustc.txt
    x 0 cargo --version
    x 0 git rev-parse HEAD
    keep head.txt
    x 0 git status --short
    say "every cargo step below runs offline (CARGO_NET_OFFLINE=true) with CARGO_HOME set to a"
    say "private copy of a local registry under ./target/m4-evidence-cargo-home, LC_ALL=C, and"
    say "TSR_DB, TSR_CODE_COMMIT, TSR_SESSION and TSR_REGEN_FIXTURES unset"
    endblock

    section "Build, format, lint"
    block "cargo build"
    xcargo 0 build --workspace
    BUILD_RC=$CARGO_RC BUILD_KEPT=$CARGO_KEPT
    endblock
    if [ "$BUILD_RC" != 0 ] || [ ! -x "$BIN" ]; then
        printf 'The build failed; every later step needs the binary, so the run stops here.\n\n'
        return
    fi
    block "cargo fmt"
    x 0 cargo fmt --all -- --check
    FMT_RC=$LAST_RC
    endblock
    block "cargo clippy"
    xcargo 0 clippy --workspace --all-targets -- -D warnings
    CLIPPY_RC=$CARGO_RC CLIPPY_KEPT=$CARGO_KEPT
    endblock

    section "The tsr train command line"
    block "tsr train --help and each subcommand's --help"
    x 0 "$BIN" train --help
    for s in dataset fit verify report apply; do
        x 0 "$BIN" train "$s" --help
    done
    endblock

    note "the pipeline"
    section "The pipeline over fixtures/m4/ (--by time)"
    local DS=$W/m4ds B=$W/m4-synth
    block "dataset --by time --calibrate 0.2 --holdout 0.2 --seed 7"
    x 0 "$BIN" train dataset --events "$FX/events.jsonl" --tasks "$FX/tasks.jsonl" \
        --labels "$FX/labels.jsonl" --by time --calibrate 0.2 --holdout 0.2 --seed 7 --out "$DS"
    endblock
    block "fit"
    x 0 "$BIN" train fit "$DS" --out "$B" --support-floor 30
    endblock
    block "the bundle's files, its manifest and the keys of each JSON file"
    x 0 ls "$B"
    x 0 cat "$B/manifest.toml"
    x 0 python3 -I -c 'import json,sys; [print(f, " ".join(json.load(open(f)))) for f in sys.argv[1:]]' \
        "$B/features.json" "$B/head.json" "$B/isotonic.json" "$B/split.json" "$B/support.json"
    endblock
    block "verify"
    x 0 "$BIN" train verify "$B"
    endblock
    block "verify --rebuild"
    x 0 "$BIN" train verify "$B" --rebuild "$DS"
    endblock
    block "report --seed 7 (the full human report)"
    x 0 "$BIN" train report "$B" --dataset "$DS" --seed 7
    keep report.txt
    endblock
    block "report --seed 7 --json (summarised)"
    xsum 0 json "$BIN" train report "$B" --dataset "$DS" --seed 7 --json
    endblock
    block "apply --side held_out (summarised)"
    xsum 0 head:10 "$BIN" train apply "$B" --dataset "$DS" --side held_out
    endblock
    block "holdout-access.jsonl after this section's dataset, fit, verify, report and apply"
    x 0 wc -l "$DS/holdout-access.jsonl"
    x 0 head -n 1 "$DS/holdout-access.jsonl"
    endblock

    section "--by repo"
    local RDS=$W/m4-repo RB=$W/m4-repo-models
    block "dataset --by repo --calibrate 0.2 --holdout 0.2 --seed 7"
    x 0 "$BIN" train dataset --events "$FX/events.jsonl" --tasks "$FX/tasks.jsonl" \
        --labels "$FX/labels.jsonl" --by repo --calibrate 0.2 --holdout 0.2 --seed 7 --out "$RDS"
    endblock
    block "fit (--by repo)"
    x 0 "$BIN" train fit "$RDS" --out "$RB"
    endblock
    block "report --seed 7 (--by repo, summarised)"
    xsum 0 nobins "$BIN" train report "$RB" --dataset "$RDS" --seed 7
    endblock

    section "apply on the fit side"
    block "apply --side fit (summarised)"
    xsum 0 head:10 "$BIN" train apply "$B" --dataset "$DS" --side fit
    endblock

    note "refusals"
    section "verify, report and apply on edited inputs"
    block "verify: manifest.toml state edited to \"active\""
    x 0 cp -R "$B" "$W/tamper-state"
    x 0 sed -i 's/^state = .*/state = "active"/' "$W/tamper-state/manifest.toml"
    x 0 grep -n '^state = ' "$W/tamper-state/manifest.toml"
    x 2 "$BIN" train verify "$W/tamper-state"
    endblock
    block "verify: one byte of head.json edited"
    x 0 cp -R "$B" "$W/tamper-head"
    x 0 sed -i 's/"l2": 0.001,/"l2": 0.002,/' "$W/tamper-head/head.json"
    x 0 grep -n '"l2"' "$W/tamper-head/head.json"
    x 2 "$BIN" train verify "$W/tamper-head"
    endblock
    block "verify: code_commit edited (recorded provenance, not verified)"
    x 0 cp -R "$B" "$W/edit-code-commit"
    x 0 sed -i 's/^code_commit = .*/code_commit = "deadbeef"/' "$W/edit-code-commit/manifest.toml"
    x 0 grep -n '^code_commit = ' "$W/edit-code-commit/manifest.toml"
    x 0 "$BIN" train verify "$W/edit-code-commit"
    endblock
    block "report on a foreign dataset (the --by repo one)"
    x 2 "$BIN" train report "$B" --dataset "$RDS" --seed 7
    endblock
    block "apply on a foreign dataset (the --by repo one)"
    x 2 "$BIN" train apply "$B" --dataset "$RDS"
    endblock
    block "apply with a tampered bundle (the copy from \"verify: one byte of head.json edited\")"
    x 2 "$BIN" train apply "$W/tamper-head" --dataset "$DS"
    endblock
    block "apply with a tampered row (rows.jsonl line 1, family edited)"
    x 0 cp -R "$DS" "$W/tamper-row"
    x 0 sed -i '1s/"family":"calibrated"/"family":"forged-family"/' "$W/tamper-row/rows.jsonl"
    x 0 grep -c '"family":"forged-family"' "$W/tamper-row/rows.jsonl"
    x 2 "$BIN" train apply "$B" --dataset "$W/tamper-row"
    endblock

    section "dataset input rules"
    block "--tasks missing"
    x 2 "$BIN" train dataset --events "$FX/events.jsonl" --out "$W/in-no-tasks"
    endblock
    block "a label whose reviewer is not the operator"
    xs 0 "sed '1s/\"reviewer\":\"operator\"/\"reviewer\":\"generator\"/' $FX/labels.jsonl > $W/labels-reviewer.jsonl"
    x 0 grep -c '"reviewer":"generator"' "$W/labels-reviewer.jsonl"
    x 2 "$BIN" train dataset --events "$FX/events.jsonl" --tasks "$FX/tasks.jsonl" \
        --labels "$W/labels-reviewer.jsonl" --out "$W/in-reviewer"
    endblock
    block "a diff_label outside accepted, rejected, disputed, null"
    xs 0 "sed '1s/\"diff_label\":\"accepted\"/\"diff_label\":\"maybe\"/' $FX/labels.jsonl > $W/labels-diff-label.jsonl"
    x 0 grep -c '"diff_label":"maybe"' "$W/labels-diff-label.jsonl"
    x 2 "$BIN" train dataset --events "$FX/events.jsonl" --tasks "$FX/tasks.jsonl" \
        --labels "$W/labels-diff-label.jsonl" --out "$W/in-diff-label"
    endblock
    block "a malformed JSONL line in the task index"
    xs 0 "{ head -n 1 $FX/tasks.jsonl; echo '{\"task\":'; tail -n +2 $FX/tasks.jsonl; } > $W/tasks-malformed.jsonl"
    x 0 sed -n 2p "$W/tasks-malformed.jsonl"
    x 2 "$BIN" train dataset --events "$FX/events.jsonl" --tasks "$W/tasks-malformed.jsonl" \
        --out "$W/in-malformed"
    endblock
    block "a task id that is not a hash"
    xs 0 "sed '1s/\"task\":\"1456cc2f/\"task\":\"zz56cc2f/' $FX/tasks.jsonl > $W/tasks-bad-hash.jsonl"
    x 0 grep -c '"task":"zz56cc2f' "$W/tasks-bad-hash.jsonl"
    x 2 "$BIN" train dataset --events "$FX/events.jsonl" --tasks "$W/tasks-bad-hash.jsonl" \
        --out "$W/in-bad-hash"
    endblock
    block "a field of the wrong type (timestamp as a number)"
    xs 0 "sed '1s/\"timestamp\":\"2026-02-01T00:00:00Z\"/\"timestamp\":17/' $FX/tasks.jsonl > $W/tasks-wrong-type.jsonl"
    x 0 grep -c '"timestamp":17' "$W/tasks-wrong-type.jsonl"
    x 2 "$BIN" train dataset --events "$FX/events.jsonl" --tasks "$W/tasks-wrong-type.jsonl" \
        --out "$W/in-wrong-type"
    endblock
    block "a duplicate task id in the task index"
    xs 0 "{ cat $FX/tasks.jsonl; head -n 1 $FX/tasks.jsonl; } > $W/tasks-duplicate.jsonl"
    x 0 wc -l "$W/tasks-duplicate.jsonl"
    x 2 "$BIN" train dataset --events "$FX/events.jsonl" --tasks "$W/tasks-duplicate.jsonl" \
        --out "$W/in-duplicate"
    endblock
    block "--by outside time, repo"
    x 2 "$BIN" train dataset --events "$FX/events.jsonl" --tasks "$FX/tasks.jsonl" --by bogus \
        --out "$W/in-by"
    endblock
    block "a share that leaves a side empty (--holdout 0)"
    x 2 "$BIN" train dataset --events "$FX/events.jsonl" --tasks "$FX/tasks.jsonl" \
        --labels "$FX/labels.jsonl" --holdout 0 --out "$W/in-empty-side"
    endblock
    block "a --store that is not a sqlite database"
    xs 0 "printf 'not a database\\n' > $W/not-sqlite.db"
    x 2 "$BIN" train dataset --store "$W/not-sqlite.db" --tasks "$FX/tasks.jsonl" \
        --out "$W/in-not-sqlite"
    endblock

    section "Leakage probe and minimum data"
    block "label_reach: labels rewritten so the family separates them"
    x 0 python3 -I -c 'import json,sys; fam={r["task"]: r["family"] for r in map(json.loads, open(sys.argv[1]))}; rows=[json.loads(l) for l in open(sys.argv[2])]; [r.update(criteria_met=fam[r["task"]]=="calibrated", diff_label="accepted" if fam[r["task"]]=="calibrated" else "rejected") for r in rows]; open(sys.argv[3], "w").write("".join(json.dumps(r, separators=(",", ":")) + "\n" for r in rows))' \
        "$FX/tasks.jsonl" "$FX/labels.jsonl" "$W/labels-by-family.jsonl"
    x 0 grep -c '"diff_label":"accepted"' "$W/labels-by-family.jsonl"
    x 2 "$BIN" train dataset --events "$FX/events.jsonl" --tasks "$FX/tasks.jsonl" \
        --labels "$W/labels-by-family.jsonl" --by time --calibrate 0.2 --holdout 0.2 --seed 7 \
        --out "$W/probe-leak"
    x 1 test -e "$W/probe-leak"
    endblock
    block "fit below 150 tasks with a resolved label (no --labels: every row unresolved)"
    x 0 "$BIN" train dataset --events "$FX/events.jsonl" --tasks "$FX/tasks.jsonl" \
        --by time --calibrate 0.2 --holdout 0.2 --seed 7 --out "$W/no-labels"
    x 2 "$BIN" train fit "$W/no-labels" --out "$W/no-labels-bundle"
    endblock

    note "known limitations"
    section "Known limitations"
    # An unwritable directory: one this script creates and chmods. Root writes a 0555 directory,
    # so as root the probes drop to uid 65534 with setpriv; without that they are not shown.
    local AS=() NOTSHOWN="" ASNOTE=""
    if [ "$(id -u)" = 0 ]; then
        AS=(setpriv --reuid=65534 --regid=65534 --clear-groups --)
        ASNOTE="the script runs as root, which writes a 0555 directory, so each command under test runs as uid 65534 (nobody) through setpriv"
        if ! command -v setpriv >/dev/null 2>&1 ||
            ! "${AS[@]}" test -x "$BIN" -a -r "$FX/events.jsonl" -a -r "$DS/rows.jsonl" 2>/dev/null; then
            NOTSHOWN="not shown: the script runs as root, which writes a 0555 directory, and cannot drop to uid 65534 with setpriv here"
        fi
    fi
    block "--out is an existing directory the user cannot write (dataset and fit)"
    if [ -n "$NOTSHOWN" ]; then
        say "$NOTSHOWN"
    else
        [ -z "$ASNOTE" ] || say "$ASNOTE"
        x 0 mkdir "$W/lim-ro-out"
        x 0 chmod 0555 "$W/lim-ro-out"
        x 1 ${AS[@]+"${AS[@]}"} "$BIN" train dataset --events "$FX/events.jsonl" \
            --tasks "$FX/tasks.jsonl" --labels "$FX/labels.jsonl" --by time --calibrate 0.2 \
            --holdout 0.2 --seed 7 --out "$W/lim-ro-out"
        x 1 ${AS[@]+"${AS[@]}"} "$BIN" train fit "$DS" --out "$W/lim-ro-out"
    fi
    endblock
    block "fit and apply on a dataset the user cannot write"
    if [ -n "$NOTSHOWN" ]; then
        say "$NOTSHOWN"
    else
        [ -z "$ASNOTE" ] || say "$ASNOTE"
        x 0 cp -R "$DS" "$W/lim-ro-ds"
        x 0 chmod -R a-w "$W/lim-ro-ds"
        x 0 mkdir -m 0777 "$W/lim-ro-ds-out"
        x 1 ${AS[@]+"${AS[@]}"} "$BIN" train fit "$W/lim-ro-ds" --out "$W/lim-ro-ds-out/bundle"
        x 1 ${AS[@]+"${AS[@]}"} "$BIN" train apply "$B" --dataset "$W/lim-ro-ds"
    fi
    endblock
    block "fit with holdout-access.jsonl as a directory"
    x 0 cp -R "$DS" "$W/lim-holdout-dir"
    x 0 rm "$W/lim-holdout-dir/holdout-access.jsonl"
    x 0 mkdir "$W/lim-holdout-dir/holdout-access.jsonl"
    x 1 "$BIN" train fit "$W/lim-holdout-dir" --out "$W/lim-holdout-dir-bundle"
    endblock
    block "report with <bundle>.report.json as a directory"
    x 0 cp -R "$B" "$W/lim-report-json"
    x 0 mkdir "$W/lim-report-json.report.json"
    xsum 1 "same:$W/.out/report.txt:\"report --seed 7 (the full human report)\"" \
        "$BIN" train report "$W/lim-report-json" --dataset "$DS" --seed 7
    endblock
    block "report with <bundle>.report.txt as a directory"
    x 0 cp -R "$B" "$W/lim-report-txt"
    x 0 mkdir "$W/lim-report-txt.report.txt"
    xsum 1 "same:$W/.out/report.txt:\"report --seed 7 (the full human report)\"" \
        "$BIN" train report "$W/lim-report-txt" --dataset "$DS" --seed 7
    endblock
    block "the --out create probe leaves empty ancestors behind when a later step refuses"
    x 0 touch "$W/empty-tasks.jsonl"
    x 1 test -e "$W/lim-out"
    x 2 "$BIN" train dataset --events "$FX/events.jsonl" --tasks "$W/empty-tasks.jsonl" \
        --out "$W/lim-out/a/b/c"
    xs 0 "find $W/lim-out -type d | sort"
    endblock
    block "--store with a valid SQLite header and a corrupt body"
    xs 0 "{ printf 'SQLite format 3\\000'; head -c 4080 /dev/zero | tr '\\000' x; } > $W/corrupt-body.db"
    xs 0 "head -c 16 $W/corrupt-body.db | od -An -c"
    x 0 stat -c '%s bytes' "$W/corrupt-body.db"
    x 1 "$BIN" train dataset --store "$W/corrupt-body.db" --tasks "$FX/tasks.jsonl" \
        --out "$W/lim-corrupt-body"
    endblock
    block "the default store path (no --store, no --events, TSR_DB unset) creates an empty store"
    x 0 mkdir "$W/lim-default-store"
    x 1 test -e "$W/lim-default-store/.tsr/tsr.db"
    xs 2 "cd $W/lim-default-store && env -u TSR_DB ../../../${BIN#./} train dataset --tasks ../../../${FX#./}/tasks.jsonl --out ./ds"
    x 0 stat -c '%s bytes' "$W/lim-default-store/.tsr/tsr.db"
    x 0 python3 -I -c 'import sqlite3,sys; c=sqlite3.connect("file:"+sys.argv[1]+"?mode=ro", uri=True); print("events:", c.execute("select count(*) from events").fetchone()[0])' \
        "$W/lim-default-store/.tsr/tsr.db"
    endblock
    block "counts_digest sits outside the bundle id"
    x 0 cp -R "$DS" "$W/lim-count-only"
    x 0 sed -i 's/"unindexed_tasks": 0/"unindexed_tasks": 1/' "$W/lim-count-only/data-manifest.json"
    x 0 grep -n '"unindexed_tasks"' "$W/lim-count-only/data-manifest.json"
    x 2 "$BIN" train fit "$W/lim-count-only" --out "$W/lim-count-only-bundle"
    # The digest the binary recomputes for the edited count, taken from its own refusal.
    local CD
    CD=$(sed -n 's/.*recomputed \([0-9a-f]\{64\}\).*/\1/p' "$OUT")
    x 0 cp -R "$DS" "$W/lim-count-and-digest"
    x 0 sed -i -e 's/"unindexed_tasks": 0/"unindexed_tasks": 1/' \
        -e "s/\"counts_digest\": \"[0-9a-f]*\"/\"counts_digest\": \"$CD\"/" \
        "$W/lim-count-and-digest/data-manifest.json"
    x 0 grep -nE '"(unindexed_tasks|counts_digest)"' "$W/lim-count-and-digest/data-manifest.json"
    x 0 "$BIN" train fit "$W/lim-count-and-digest" --out "$W/lim-count-and-digest-bundle"
    endblock
    chmod -R u+w "$W" 2>/dev/null

    note "workspace tests (a few minutes)"
    section "The whole workspace test suite"
    block "cargo test --workspace --no-fail-fast"
    # The tests' temporary directories go under target/ too, removed when the script exits.
    x 0 mkdir "$TMPD"
    local c="env CARGO_TARGET_DIR=$FRESH TMPDIR=\"\$PWD/${TMPD#./}\" cargo test --workspace --no-fail-fast"
    printf '$ %s\n' "$c"
    bash -c "$c" >"$W/.out/test.log" 2>&1 </dev/null
    TEST_RC=$?
    say "each test binary's Running or Doc-tests line (without the build-hash path), then its"
    say "\`test result:\` line; then the totals and every failed test, parsed from the"
    say "\`test <name> ... FAILED\` and \`---- <name> stdout ----\` lines"
    python3 -I - "$W/.out/test.log" "$W/.out/test.summary" "$ALLOWED_FAILURES" <<'PY'
import os, re, sys
log, summary, allowed = sys.argv[1], sys.argv[2], set(sys.argv[3].split("\n"))
binary = "?"
results, failed = [], []
running = re.compile(r"Running (\S+(?: \S+)?) \(([^)]*)\)")
doctests = re.compile(r"Doc-tests (\S+)")
result = re.compile(r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; "
                    r"(\d+) measured; (\d+) filtered out; finished in [0-9.]+s")
for line in open(log, errors="replace").read().splitlines():
    m = running.search(line)
    if m:
        binary = re.sub(r"-[0-9a-f]{16}$", "", os.path.basename(m.group(2)))
        print(f"Running {m.group(1)} ({binary})")
        continue
    m = doctests.search(line)
    if m and "test result" not in line:
        binary = m.group(1)
        print(f"Doc-tests {binary}")
        continue
    m = result.search(line)
    if m:
        print(m.group(0))
        results.append([int(m.group(i)) for i in range(2, 7)])
        continue
    for pat in (r"test (\S+) \.\.\. FAILED", r"^---- (\S+) stdout ----$"):
        m = re.search(pat, line)
        if m and f"{binary}::{m.group(1)}" not in failed:
            failed.append(f"{binary}::{m.group(1)}")
tot = [sum(r[i] for r in results) for i in range(5)]
print(f"# totals over {len(results)} test result lines: {tot[0]} passed; {tot[1]} failed; "
      f"{tot[2]} ignored; {tot[3]} measured; {tot[4]} filtered out")
print("# failed tests:" + ("" if failed else " none"))
for f in failed:
    print(f + ("  (expected where a docker CLI answers --version without a daemon)" if f in allowed else "  (UNEXPECTED)"))
unexpected = [f for f in failed if f not in allowed]
consistent = tot[1] == len(failed)
with open(summary, "w") as s:
    s.write(f"{len(results)} {tot[0]} {tot[1]} {tot[2]} {len(unexpected)} {int(consistent)}\n")
    s.write("\n".join(failed) + "\n")
PY
    local nres npass nfail nign nunexp consistent
    read -r nres npass nfail nign nunexp consistent <"$W/.out/test.summary"
    TEST_SUMMARY="$nres test result lines: $npass passed, $nfail failed, $nign ignored"
    TEST_FAILED=$(tail -n +2 "$W/.out/test.summary" | grep . | tr '\n' ' ')
    if [ "$nunexp" != 0 ] || [ "$consistent" != 1 ]; then
        check 0 "$TEST_RC" "$c"
        [ "$TEST_RC" != 0 ] || UNEXPECTED+=("$CUR: a test failed outside the expected set")
    elif [ "$nfail" != 0 ]; then
        check 101 "$TEST_RC" "$c"
    else
        check 0 "$TEST_RC" "$c"
    fi
    endblock

    section "Diff base"
    block "The base, and the diff against it"
    if [ -n "$BASE" ]; then
        say "diff base: $BASE, from $BASE_FROM"
        x 0 git diff --stat "$BASE" HEAD -- Cargo.toml Cargo.lock README.md crates fixtures profiles
    elif [ "$BASE_FROM" = "git merge-base HEAD origin/main" ]; then
        say "diff base: none -- \$TSR_MAIN is unset and \`git merge-base HEAD origin/main\` found no"
        say "commit (this clone has no origin/main), so no diff is printed"
    else
        say "diff base: none -- $BASE_FROM does not name a commit"
        UNEXPECTED+=("Diff base: $BASE_FROM does not name a commit")
    fi
    endblock
}

# --- the summary first, then the evidence --------------------------------------------------------

LAST_RC="" FMT_RC="" BUILD_RC="" BUILD_KEPT="" CLIPPY_RC="" CLIPPY_KEPT="" TEST_RC="" TEST_SUMMARY="" TEST_FAILED=""
body >"$W/.out/body.md"

cargo_line() { # cargo_line <rc> <kept lines>
    if [ -z "$1" ]; then printf 'not run'; return; fi
    printf 'exit %s, %s' "$1" "$([ "$2" = 0 ] && echo 'no warning or error lines' || echo "$2 warning or error lines")"
}

printf '# M4 (Calibrate) — exit evidence\n\n'
section "At a glance"
printf '### The run\n\n```text\n'
printf 'commit   %s\n' "$(cat "$W/.out/head.txt" 2>/dev/null)"
printf 'rustc    %s\n' "$(cat "$W/.out/rustc.txt" 2>/dev/null)"
printf 'build    %s\n' "$(cargo_line "$BUILD_RC" "$BUILD_KEPT")"
printf 'fmt      %s\n' "$([ -n "$FMT_RC" ] && echo "exit $FMT_RC" || echo 'not run')"
printf 'clippy   %s\n' "$(cargo_line "$CLIPPY_RC" "$CLIPPY_KEPT")"
if [ -n "$TEST_RC" ]; then
    printf 'tests    exit %s; %s\n' "$TEST_RC" "$TEST_SUMMARY"
    for f in $TEST_FAILED; do printf '         failed: %s\n' "$f"; done
else
    printf 'tests    not run\n'
fi
if [ "${#UNEXPECTED[@]}" = 0 ]; then
    printf 'steps that did not exit as expected: none\n'
else
    printf 'steps that did not exit as expected: %d\n' "${#UNEXPECTED[@]}"
    for u in "${UNEXPECTED[@]}"; do printf '  %s\n' "$u"; done
fi
printf '```\n\n'
if [ -s "$W/.out/report.txt" ]; then
    printf '### The headline, as `report --seed 7` prints it\n\n```text\n'
    python3 -I - "$W/.out/report.txt" <<'PY'
import sys
lines = open(sys.argv[1]).read().splitlines()
print(lines[0])
section, prev = None, ""
for line in lines:
    if prev == "":
        section = line
        if section in ("forecasters", "families") or section.startswith("paired"):
            print(line)
        prev = line
        continue
    prev = line
    if section in ("forecasters", "families"):
        if line.startswith("family ") or line.startswith("  controller ") or line.startswith("  generator "):
            print(line)
    elif section and section.startswith("paired") and line:
        print(line)
PY
    printf '```\n\n'
fi
cat "$W/.out/body.md"

if [ "${#UNEXPECTED[@]}" != 0 ]; then
    note "${#UNEXPECTED[@]} step(s) did not exit as expected:"
    for u in "${UNEXPECTED[@]}"; do note "  $u"; done
    exit 1
fi
note "done; every step exited as expected"
exit 0
