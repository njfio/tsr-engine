"""Append-only split-integrity audit of the currently exported strict admissions.

Run after admission milestones and before evaluation. This invokes only the local
split checker, never source tests or a provider. Original evidence is untouched.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--out', type=Path, required=True)
args = parser.parse_args()
out = args.out.resolve()
out.mkdir()  # Refuse replacement of a prior audit.
(out / 'cas').mkdir()
summary_bytes = (ROOT / 'docs/m3-native-admission/summary.json').read_bytes()
summary = json.loads(summary_bytes)
rows, sources, ids, groups = [], [], set(), set()
for attempt in summary['attempts']:
    if not attempt['strict_protocol']:
        continue
    accepted = [r for r in attempt['reports'] if r['accepted']]
    if not accepted:
        continue
    if not re.fullmatch('m3-qualified-[a-z0-9-]+', attempt['attempt']):
        raise ValueError('Unexpected attempt name')
    source = ROOT / 'docs/m3-native-admission' / attempt['attempt']
    original_reports = [json.loads(line) for line in (source / 'admission.jsonl').read_bytes().splitlines()]
    if original_reports != attempt['reports']:
        raise ValueError('Summary differs from original admission reports: ' + attempt['attempt'])
    if any(r.get('protocol') != 'baseline-full-green/named-red/full-red/exact-reversal/full-green-v1'
           for r in accepted):
        raise ValueError('Accepted record lacks strict full-red protocol: ' + attempt['attempt'])
    manifest = (source / 'tasks.jsonl').read_bytes()
    tasks = [json.loads(line) for line in manifest.splitlines()]
    for report in accepted:
        matches = [t for t in tasks if t['id'] == report['task_id']]
        if len(matches) != 1:
            raise ValueError('Ambiguous or missing original task: ' + report['task_id'])
        if report['task_id'] in ids or report['function_group'] in groups:
            raise ValueError('Duplicate accepted task or function: ' + report['task_id'])
        ids.add(report['task_id'])
        groups.add(report['function_group'])
        rows.append(matches[0])
        sources.append(dict(attempt=attempt['attempt'], task_id=report['task_id'],
                            function_group=report['function_group'],
                            original_manifest_sha256=hashlib.sha256(manifest).hexdigest()))
    for entry in (source / 'cas').iterdir():
        if entry.is_symlink() or not entry.is_file() or not re.fullmatch('[0-9a-f]{64}', entry.name):
            raise ValueError('Unexpected CAS entry: ' + str(entry))
        data = entry.read_bytes()
        dest = out / 'cas' / entry.name
        if dest.exists():
            if dest.read_bytes() != data:
                raise ValueError('Conflicting CAS bytes: ' + entry.name)
        else:
            dest.write_bytes(data)
if len(rows) != summary['admitted_production_tasks']:
    raise ValueError('Admission count differs from original unique records')
(out / 'tasks.jsonl').write_text(''.join(json.dumps(row) + '\n' for row in rows))
(out / 'sources.json').write_text(json.dumps(sources, indent=2) + '\n')
engine = out / 'split-checker'
shutil.copyfile(ROOT / 'target/m3-split-checker-v5', engine)
engine.chmod(0o500)
engine_digest = hashlib.sha256(engine.read_bytes()).hexdigest()
verified_engine = json.loads((ROOT / 'docs/m3-combined-dedup-diagnostic-v5.json').read_text())
if engine_digest != verified_engine['engine_sha256']:
    raise ValueError('Checker differs from the recorded repaired engine; review before auditing')
result = subprocess.run([str(engine), 'corpus', 'split', str(out), '--by', 'repo',
                         '--holdout', '0.2', '--seed', '42'], capture_output=True,
                        text=True, timeout=120, cwd=ROOT)
report = dict(admissions_checked=len(rows), original_records_preserved=True,
              summary_sha256=hashlib.sha256(summary_bytes).hexdigest(),
              input_manifest_sha256=hashlib.sha256((out / 'tasks.jsonl').read_bytes()).hexdigest(),
              engine_sha256=engine_digest, exit=result.returncode,
              stdout=result.stdout, stderr=result.stderr,
              scope='Split-input integrity diagnostic only; not comprehensive qualification-run '
                    'evidence validation, a reserved holdout, or final evaluation. '
                    'CAS uses native BLAKE3 verification in the repaired splitter. '
                    'Snapshot identity checks do not establish snapshot-content coverage.')
if result.returncode == 0:
    report['split'] = json.loads((out / 'split.json').read_text())
    report['invalidated_cases'] = []
else:
    report['status'] = 'Not validated; inspect checker diagnostics before evaluation.'
(out / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(dict(admissions_checked=len(rows), exit=result.returncode, output=str(out))))
raise SystemExit(result.returncode)
