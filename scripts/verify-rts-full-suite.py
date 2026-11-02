#!/usr/bin/env python3
"""Verify an offline full-suite closeout without executing evaluated source."""
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys

root = Path(__file__).resolve().parents[1]
out = Path(sys.argv[1]).resolve()
receipts = out / sys.argv[2] if len(sys.argv) > 2 else out
initial = json.loads((out / 'initial.json').read_text())
source = out / 'source'
actual = {}
for path in source.rglob('*'):
    assert not path.is_symlink(), path
    if path.is_file():
        actual[str(path.relative_to(source))] = hashlib.sha256(path.read_bytes()).hexdigest()
assert actual == initial['source_sha256'], 'source inventory or bytes changed'
assert subprocess.check_output(['git', '-C', initial['source_repo'], 'status', '--porcelain']).decode() == initial['original_status']
assert hashlib.sha256(Path(initial['patch']).read_bytes()).hexdigest() == initial['patch_sha256']
build = json.loads((receipts / 'build.json').read_text())
test = json.loads((receipts / 'test.json').read_text()) if (receipts / 'test.json').exists() else None
summaries = []
counts = dict(passed=0, failed=0, ignored=0, measured=0, filtered=0)
if test:
    stdout = (receipts / 'test.stdout').read_text()
    pattern = r'^test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;.*$'
    for match in re.finditer(pattern, stdout, re.M):
        summaries.append(match.group(0))
        for key, value in zip(counts, match.groups()):
            counts[key] += int(value)
    assert len(summaries) == len(re.findall(r'^test result:', stdout, re.M)), 'unrecognized summary'
exit_code = test['result']['exit'] if test else build['result']['exit']
status = 'inconclusive'
if summaries and exit_code == 0 and counts['failed'] == 0:
    status = 'passed'
elif summaries and exit_code == 101 and counts['failed'] > 0:
    status = 'failed_tests'
if status == 'passed':
    assert build['result']['exit'] == 0 and counts['passed'] > 0
    assert 'test tests::test_supported_languages ... ok' in stdout
    prior_path = root / 'target/m3-qualified-rts-core-80d827e3a4c1' / initial['task'] / 'cas/87f5d6c6da0631452b6df81355fdca0f5f7496d04650b1ddaba4c615c0b01a13'
    prior = prior_path.read_text()
    names = lambda text: Counter(re.findall(r'^test (.+?) \.\.\. (?:ok|FAILED|ignored)', text, re.M))
    assert names(stdout) == names(prior), 'test inventory differs from prior successful full suite'
    assert counts == dict(passed=999, failed=0, ignored=15, measured=0, filtered=0)
    assert len(summaries) == 96
record = dict(task=initial['task'], implementation_commit=initial.get('implementation_commit', '9fa4705'), base=initial['base'],
              image='sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6',
              status=status, matches_prior_successful_inventory=(status == 'passed'), counts=counts, summary_groups=len(summaries), summaries=summaries,
              build=build, test=test, source_files_verified=len(actual), source_unchanged=True,
              original_repository_unchanged=True, same_exported_patch_sha256=initial['patch_sha256'],
              copied_build_from=initial['copied_build_from'], writable_shared_cache=False,
              paid_calls=0, run_directory=str(out), receipts_directory=str(receipts),
              scope=initial.get('scope', 'Full workspace suite for one local reference-correction replay; no model or cohort claim'))
for stage in ('build', 'test'):
    for stream in ('stdout', 'stderr'):
        path = receipts / f'{stage}.{stream}'
        if path.exists():
            record[f'{stage}_{stream}'] = dict(path=str(path), bytes=path.stat().st_size, sha256=hashlib.sha256(path.read_bytes()).hexdigest())
report_name = initial.get('report_name', 'engine-rts-full-suite-result.json')
assert Path(report_name).name == report_name and report_name.endswith('.json')
(root / 'docs' / report_name).write_text(json.dumps(record, indent=2) + '\n')
print(json.dumps(dict(status=status, matches_prior_successful_inventory=(status == 'passed'), counts=counts, source_files_verified=len(actual))))
