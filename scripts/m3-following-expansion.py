"""Sequential, frozen qualification cohorts after paired-1, stopping at resource/error bounds."""
from pathlib import Path
import hashlib
import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[1]
CREATED = False


def main():
    plan = json.loads((ROOT / 'docs/m3-expansion-following-freeze.json').read_text())
    if hashlib.sha256(Path(__file__).read_bytes()).hexdigest() != plan['controller_sha256']:
        raise RuntimeError('Frozen continuation controller changed')
    predecessor = json.loads((ROOT / plan['predecessor']).read_text())
    def terminal_ok(value):
        if isinstance(value, list):
            return bool(value) and all(terminal_ok(item) for item in value)
        return isinstance(value, dict) and value.get('status') == 'completed' and value.get('exit') == 0 and not value.get('errors')
    if not predecessor.get('results') or not terminal_ok(predecessor['results']):
        raise RuntimeError('Paired predecessor has incomplete or failed processes')
    events = [json.loads(line) for line in (ROOT / 'target/m3-provider-cost.jsonl').read_text().splitlines()]
    reserves = [e['id'] for e in events if e['kind'] == 'reserve']
    settled = [e['id'] for e in events if e['kind'] == 'settled']
    if len(set(reserves)) != len(reserves) or len(set(settled)) != len(settled) or any(e['kind'] not in ('reserve', 'settled') for e in events) or {e['id'] for e in events if e['kind'] == 'reserve'} != {e['id'] for e in events if e['kind'] == 'settled'}:
        raise RuntimeError('Provider ledger requires reconciliation')
    state = ROOT / 'target/m3-following-expansion-state'
    state.mkdir()
    global CREATED
    CREATED = True
    (state / 'plan.json').write_text(json.dumps(plan, indent=2) + '\n')
    outcomes = []
    for step in plan['steps']:
        reason = None
        if shutil.disk_usage(ROOT).free < plan['minimum_free_gib'] * 1024**3:
            reason = 'Disk reserve reached; cleanup remains unapproved'
        names = subprocess.check_output(['docker', 'ps', '--format', '{{.Names}}'], text=True, timeout=15).splitlines()
        if any(name.startswith('tsr-m3-') for name in names):
            reason = 'Prior M3 container remains'
        for relative, expected in step['hashes'].items():
            if hashlib.sha256((ROOT / relative).read_bytes()).hexdigest() != expected:
                reason = 'Frozen input changed: ' + relative
        repo = str(ROOT / step['source'])
        if subprocess.check_output(['git', '-C', repo, 'rev-parse', 'HEAD'], text=True).strip() != step['source_revision'] or subprocess.check_output(['git', '-C', repo, 'status', '--porcelain'], text=True).strip():
            reason = 'Source changed'
        if reason:
            outcomes.append(dict(cohort=step['cohort'], status='blocked-before-start', reason=reason))
            break
        env = os.environ.copy()
        env.pop('TSR_DB', None)
        with (state / (step['cohort'] + '.log')).open('x') as log:
            result = subprocess.run([sys.executable, '-B', str(ROOT / step['controller'])], cwd=ROOT,
                                    env=env, stdout=log, stderr=subprocess.STDOUT)
        terminal = ROOT / step['state'] / 'complete.json'
        detail = json.loads(terminal.read_text()) if terminal.exists() else {}
        bad_jobs = [r for r in detail.get('results', []) if not isinstance(r, dict) or r.get('status') != 'completed' or r.get('exit') != 0 or r.get('errors')]
        outcomes.append(dict(cohort=step['cohort'], exit=result.returncode, terminal=detail))
        (state / 'progress.json').write_text(json.dumps(outcomes, indent=2) + '\n')
        if result.returncode != 0 or detail.get('status') != 'completed' or bad_jobs:
            break
    (state / 'complete.json').write_text(json.dumps(dict(finished=datetime.now(timezone.utc).isoformat(), outcomes=outcomes), indent=2) + '\n')


if __name__ == '__main__':
    try:
        main()
    except Exception as error:
        if CREATED:
            path = ROOT / 'target/m3-following-expansion-state/error.json'
            with path.open('x') as stream:
                json.dump(dict(status='controller-error', error_class=type(error).__name__, recorded_at=datetime.now(timezone.utc).isoformat()), stream)
        raise
