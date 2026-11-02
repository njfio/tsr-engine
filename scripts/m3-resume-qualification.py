"""Resume only unfinished local qualifications, with at most two sealed workers."""
from concurrent.futures import ThreadPoolExecutor
import json
import os
from pathlib import Path
import subprocess

root = Path(__file__).resolve().parents[1]
engine = root / "target/m3-qualification-engine-238ac39"
state = root / "target/m3-resume-state"
state.mkdir()  # Never overwrite an existing queue.


def environment(image, preparation):
    env = os.environ.copy()
    env.update(TSR_M3_RUNTIME="docker", TSR_M3_USER="65534:65534",
               TSR_M3_IMAGE=image, TSR_M3_PREPARATION=str(root / preparation))
    return env


def run(label, argv, env=None):
    path = state / (label + ".json")
    with (state / (label + ".log")).open("x") as stream:
        child = subprocess.Popen(argv, cwd=root, env=env, stdin=subprocess.DEVNULL,
                                 stdout=stream, stderr=subprocess.STDOUT)
        path.write_text(json.dumps({"status": "running", "pid": child.pid,
                                    "argv": argv}) + "\n")
        code = child.wait()
    path.write_text(json.dumps({"status": "completed", "exit": code,
                                "argv": argv}) + "\n")
    return code


def qualify(task):
    identifier = task["id"]
    env = environment("sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6",
                      "docs/m3-preparations/rts-head-v2/preparation.json")
    return run(identifier, [str(engine), "corpus", "qualify", "--tasks",
        str(root / "target/m3-resume-input" / identifier / "tasks.jsonl"),
        "--out", str(root / "target" / ("m3-qualified-resume-" + identifier[:12]))], env)


def agentid():
    run("agentid-baseline", ["python3", "scripts/m3-history-probe.py",
        "--image-file", "/tmp/m3-agentid-v2-image-id", "--copy",
        str(root / "target/m3-second-source/baseline-copy-v2"), "--out",
        str(root / "docs/m3-agentid-baseline-v2.json"), "--seconds", "600",
        "--tmp-exec", "--", "sh", "-c",
        "mkdir -m 700 /tmp/m3-home /tmp/m3-runtime /tmp/m3-state && "
        "export HOME=/tmp/m3-home XDG_RUNTIME_DIR=/tmp/m3-runtime XDG_STATE_HOME=/tmp/m3-state && "
        "exec cargo test --locked --offline --workspace --no-fail-fast -j 1 -- --test-threads=1"])
    baseline = json.loads((root / "docs/m3-agentid-baseline-v2.json").read_text())
    if baseline["exit"] != 0 or baseline["output_truncated"]:
        return baseline["exit"] or 125
    image = Path("/tmp/m3-agentid-v2-image-id").read_text().strip()
    env = environment(image, "docs/m3-preparations/agentid-head-v2/preparation.json")
    code = run("agentid-screen", [str(engine), "corpus", "mutants",
        str(root / "target/m3-second-source/repo"), "--package", "agentid-core",
        "--file", "crates/agentid-core/src/validation.rs", "--mutant",
        r"(replace >= with > in is_valid_e164|replace \|\| with && in is_valid_email)$",
        "--screen-target", "lib", "--out", str(root / "target/m3-agentid-screen")], env)
    manifest = root / "target/m3-agentid-screen/tasks.jsonl"
    if code != 0 or not manifest.exists() or not manifest.read_text().strip():
        return code
    return run("agentid-qualify", [str(engine), "corpus", "qualify", "--tasks",
        str(manifest), "--out", str(root / "target/m3-qualified-agentid")], env)


tasks = json.loads((root / "target/m3-resume-plan.json").read_text())
with ThreadPoolExecutor(max_workers=2) as pool:
    # The second source starts alongside one existing-source task.
    futures = [pool.submit(qualify, tasks[0]), pool.submit(agentid)]
    futures.extend(pool.submit(qualify, task) for task in tasks[1:])
    results = [future.result() for future in futures]
(state / "complete.json").write_text(json.dumps({"results": results}) + "\n")
