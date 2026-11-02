"""Describe observed inventory capacity without extrapolating qualification yield."""
from collections import defaultdict
import hashlib
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
source = root / "target/m3-history/mutants-copy/mutants-all.json"
raw = source.read_bytes()
rows = json.loads(raw)
groups = defaultdict(set)
missing_function = 0
for row in rows:
    function = row.get("function")
    if function is None:
        missing_function += 1
        continue
    groups[row["package"]].add((
        row["file"], function["function_name"],
        json.dumps(function["span"], sort_keys=True),
    ))
progress = json.loads((root / "docs/m3-corpus-progress.json").read_text())
admission = json.loads((root / "docs/m3-native-admission/summary.json").read_text())
source_lineage = "https://github.com/njfio/rs-agent-code-utility"
result = {
    "inventory_lineage": source_lineage,
    "source_revision": progress["source_revision"],
    "inventory_sha256": hashlib.sha256(raw).hexdigest(),
    "descriptors": len(rows),
    "descriptors_without_function": missing_function,
    "identified_function_groups_by_package": {
        key: len(value) for key, value in sorted(groups.items())
    },
    "non_benchmark_function_inventory_ceiling": sum(
        len(value) for key, value in groups.items() if key != "rts-bench"
    ),
    "direct_source_file_function_inventory_ceiling": sum(
        1 for package, entries in groups.items() if package != "rts-bench"
        for file, _name, _span in entries
        if "/src/" in file and "/" not in file.rsplit("/src/", 1)[1]
        and file.rsplit("/", 1)[1] not in {"lib.rs", "main.rs", "mod.rs"}
    ),
    "qualified_tasks_observed": admission["admitted_by_lineage"].get(source_lineage, 0),
    "all_sources_qualified_tasks": admission["admitted_production_tasks"],
    "caught_candidates_observed": progress["tool_reported_caught"],
    "inventory_repository_lineages": 1,
    "all_sources_qualified_lineages": admission["repository_lineages"],
    "m3_task_gap": max(0, 150 - progress["admitted_production_tasks"]),
    "qualification_yield_estimate": None,
    "interpretation": [
        "Function inventory is an upper bound before coverage, supported syntax, completed red/green qualification, and near-clone deduplication; it is not usable task capacity.",
        "The direct-source-file subset is another ceiling, not an eligibility determination. Nested external test-module paths and non-doc function attributes remain unsupported in the frozen engine; tests may live elsewhere or lack coverage.",
        "The screens were purposive rather than random. Their caught rate cannot estimate population admission yield.",
        "The observed lower bound is the qualified-task count above. The existing source has enough inventory to investigate 150 tasks, but no evidence establishes 150 qualifying tasks.",
        "More revisions or mutations in this source cannot create a second repository lineage.",
        "AgentID now supplies an independently developed second owner-approved lineage. This makes repository-separated exploration possible, but a single held-out lineage cannot support informative repository-cluster uncertainty; actual paired results across multiple evaluation lineages remain needed.",
        "Both selected sources have pinned revisions, task provenance, public dependency preparation and bounded offline test commands. Additional sources, if needed, require the same provenance; no unrelated sessions or private logs are needed.",
        "No task count or extra repository alone supplies systems-1/2 generator outcomes. Offline scripted wiring checks are synthetic and paid calls remain paused.",
    ],
}
(root / "docs/m3-capacity.json").write_text(json.dumps(result, indent=2) + "\n")
