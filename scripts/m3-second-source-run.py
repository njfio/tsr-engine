"""Bounded local queue: reuse a worker slot after the daemon cohort finishes.

Only the trusted controller/probe run on the host. All source commands use the
existing sealed workers. No provider calls. Existing outputs are never replaced.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument("--retry-agentid", action="store_true")
retry = parser.parse_args().retry_agentid
root = Path(__file__).resolve().parents[1]
status = root / ("target/m3-second-source-queue-v2.json" if retry else "target/m3-second-source-queue.json")
baseline_name = "m3-agentid-baseline-v2" if retry else "m3-agentid-baseline"
image_file = "/tmp/m3-agentid-v2-image-id" if retry else "/tmp/m3-agentid-image-id"
preparation_path = "docs/m3-preparations/agentid-head-v2/preparation.json" if retry else "docs/m3-preparations/agentid-head/preparation.json"
suffix = "-v2" if retry else ""
engine = root / "target/m3-qualification-engine-238ac39"


def record(stage, **extra):
    status.write_text(json.dumps({"stage": stage, **extra}, indent=2) + "\n")
    print(stage, flush=True)


def run(label, argv, env=None):
    record(label)
    log = root / "target" / (label + "-controller.log")
    with log.open("x") as stream:
        result = subprocess.run(argv, cwd=root, env=env, stdin=subprocess.DEVNULL,
                                stdout=stream, stderr=subprocess.STDOUT)
    record(label + "-finished", exit=result.returncode)
    return result.returncode


def environment(image, preparation):
    env = os.environ.copy()
    env.update(TSR_M3_RUNTIME="docker", TSR_M3_USER="65534:65534",
               TSR_M3_IMAGE=image,
               TSR_M3_PREPARATION=str(root / preparation))
    return env


if status.exists():
    raise SystemExit("Queue already exists; inspect outcomes instead of duplicating it")
record("waiting-for-cancel-slot" if retry else "waiting-for-daemon-slot")
deadline = time.monotonic() + 3600
while True:
    reports = root / ("target/m3-qualified-cancel-tokio/admission.jsonl" if retry else "target/m3-qualified-daemon-retry/admission.jsonl")
    if reports.exists() and len(reports.read_text().splitlines()) == (1 if retry else 2):
        break
    if time.monotonic() >= deadline:
        record("blocked", reason="preceding cohort did not finish within queue wait bound")
        raise SystemExit(1)
    time.sleep(1)

run(baseline_name, ["python3", "scripts/m3-history-probe.py",
    "--image-file", image_file,
    "--copy", str(root / ("target/m3-second-source/baseline-copy-v2" if retry else "target/m3-second-source/baseline-copy")),
    "--out", str(root / ("docs/" + baseline_name + ".json")),
    "--seconds", "600", "--tmp-exec", "--", "sh", "-c",
    "mkdir -m 700 /tmp/m3-home /tmp/m3-runtime /tmp/m3-state && "
    "export HOME=/tmp/m3-home XDG_RUNTIME_DIR=/tmp/m3-runtime XDG_STATE_HOME=/tmp/m3-state && "
    "exec cargo test --locked --offline --workspace --no-fail-fast -j 1 -- --test-threads=1"])
baseline = json.loads((root / ("docs/" + baseline_name + ".json")).read_text())
if baseline["exit"] == 0 and not baseline["output_truncated"]:
    image = Path(image_file).read_text().strip()
    env = environment(image, preparation_path)
    code = run("m3-agentid-screen" + suffix, [str(engine), "corpus", "mutants",
        str(root / "target/m3-second-source/repo"), "--package", "agentid-core",
        "--file", "crates/agentid-core/src/validation.rs", "--mutant",
        r"(replace >= with > in is_valid_e164|replace \|\| with && in is_valid_email)$",
        "--screen-target", "lib", "--out", str(root / "target/m3-agentid-screen")], env)
    manifest = root / "target/m3-agentid-screen/tasks.jsonl"
    if code == 0 and manifest.exists() and manifest.read_text().strip():
        run("m3-agentid-qualify" + suffix, [str(engine), "corpus", "qualify", "--tasks",
            str(manifest), "--out", str(root / "target/m3-qualified-agentid")], env)
else:
    record("agentid-baseline-not-admissible", exit=baseline["exit"])

if not retry:
    # This retry is independent of the second source's baseline result.
    env = environment("sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6",
                      "docs/m3-preparations/rts-head-v2/preparation.json")
    run("m3-cancel-tokio", [str(engine), "corpus", "qualify", "--tasks",
        str(root / "target/m3-cancel-tokio-input/tasks.jsonl"), "--out",
        str(root / "target/m3-qualified-cancel-tokio")], env)
record("complete")
