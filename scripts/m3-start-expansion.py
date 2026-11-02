"""Verify the reviewed first expansion cohort before starting its finite controller."""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
plan = json.loads((ROOT / 'docs/m3-expansion-qualification-freeze.json').read_text())
for relative, expected in plan['hashes'].items():
    if hashlib.sha256((ROOT / relative).read_bytes()).hexdigest() != expected:
        raise RuntimeError('Reviewed input changed: ' + relative)
revision = subprocess.check_output(['git', '-C', str(ROOT / plan['source']), 'rev-parse', 'HEAD'], text=True).strip()
if revision != plan['source_revision']:
    raise RuntimeError('Source revision changed')
if subprocess.check_output(['git', '-C', str(ROOT / plan['source']), 'status', '--porcelain'], text=True).strip():
    raise RuntimeError('Source checkout modified')
if shutil.disk_usage(ROOT).free < plan['minimum_free_gib'] * 1024**3:
    raise RuntimeError('Disk reserve reached')
raise SystemExit(subprocess.call([sys.executable, '-B', str(ROOT / 'scripts/m3-agentid-db-cohort.py')], cwd=ROOT))
