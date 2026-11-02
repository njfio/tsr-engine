"""Retry the corrected two-function filter when the bounded queue has a free slot."""
import json
import os
from pathlib import Path
import subprocess
import time

root = Path(__file__).resolve().parents[1]
state = root / "target/m3-agentid-filter-retry.json"
if state.exists():
    raise SystemExit("Retry already exists; inspect it before rerunning")
tasks = json.loads((root / "target/m3-resume-plan.json").read_text())
state.write_text(json.dumps({"stage": "waiting-for-worker-slot"}) + "\n")
deadline = time.monotonic() + 5400
while True:
    completed = 0
    for task in tasks:
        path = root / "target/m3-resume-state" / (task["id"] + ".json")
        if path.exists() and json.loads(path.read_text()).get("status") == "completed":
            completed += 1
    # Five native jobs were queued. With four finished, at most one remains.
    if completed >= len(tasks) - 1:
        break
    if time.monotonic() >= deadline:
        state.write_text(json.dumps({"stage": "blocked", "reason": "No worker slot within 90-minute queue bound"}) + "\n")
        raise SystemExit(1)
    time.sleep(1)

engine = str(root / "target/m3-qualification-engine-238ac39")
env = os.environ.copy()
env.update(TSR_M3_RUNTIME="docker", TSR_M3_USER="65534:65534",
           TSR_M3_IMAGE=Path("/tmp/m3-agentid-v2-image-id").read_text().strip(),
           TSR_M3_PREPARATION=str(root / "docs/m3-preparations/agentid-head-v2/preparation.json"))
screen = root / "target/m3-agentid-screen-v2"
argv = [engine, "corpus", "mutants", str(root / "target/m3-second-source/repo"),
        "--package", "agentid-core", "--file", "crates/agentid-core/src/validation.rs",
        "--mutant", r"(replace >= with < in is_valid_e164|replace \|\| with && in is_valid_email_address)$",
        "--screen-target", "lib", "--out", str(screen)]
state.write_text(json.dumps({"stage": "screening", "argv": argv}) + "\n")
with (root / "target/m3-agentid-filter-retry.log").open("x") as stream:
    result = subprocess.run(argv, cwd=root, env=env, stdin=subprocess.DEVNULL,
                            stdout=stream, stderr=subprocess.STDOUT)
    manifest = screen / "tasks.jsonl"
    if result.returncode == 0 and manifest.exists() and manifest.read_text().strip():
        state.write_text(json.dumps({"stage": "qualifying"}) + "\n")
        result = subprocess.run([engine, "corpus", "qualify", "--tasks", str(manifest),
            "--out", str(root / "target/m3-qualified-agentid-v2")], cwd=root, env=env,
            stdin=subprocess.DEVNULL, stdout=stream, stderr=subprocess.STDOUT)
state.write_text(json.dumps({"stage": "complete", "exit": result.returncode}) + "\n")
