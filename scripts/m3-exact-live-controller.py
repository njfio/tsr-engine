"""One-shot exact-edit pair. Import/preparation does not call providers or launch workers."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'docs/m3-exact-live-freeze.json'


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / (name + '.py'))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


previous = load('m3-continuation-controller')
budget = load('m3-exact-budget')
save_new = previous.save_new


def validate_scope(m):
    task = '0c5a91a6aaa862c279aed1e1602a74eda7c42c4fc0f72247de513fa948f50b70'
    if [(a['task'], a['mode']) for a in m['arms']] != [(task, 'record'), (task, 'rules')]:
        raise ValueError('exact pair scope')
    if (m['max_calls_per_arm'], m['max_calls_total'], m['max_wall_seconds_per_arm'],
        m['incremental_cap_usd'], m['aggregate_cap_usd'], m['automatic_retries']) != (20, 40, 1800, '5', '100', 0):
        raise ValueError('exact pair limits')
    if m['global_wall_seconds'] != 4200 or m['export_margin_seconds'] != 300:
        raise ValueError('global deadline scope')


def coordinate(m, state, ops):
    validate_scope(m)
    if state.exists():
        raise ValueError('one-shot state exists; no resume or retry')
    started = time.monotonic()  # Includes initial preflight and all reporting overhead.
    ops.guard(initial=True, dispatch=True)
    state.mkdir()
    previous.sync_directory(state.parent)
    save_new(state / 'plan.json', m)
    decisions = []
    try:
        for i, arm in enumerate(m['arms']):
            if m['global_wall_seconds'] - (time.monotonic() - started) < 1800 + m['export_margin_seconds']:
                raise ValueError('global deadline headroom')
            ops.guard(initial=False, dispatch=True)
            if m['global_wall_seconds'] - (time.monotonic() - started) < 1800 + m['export_margin_seconds']:
                raise ValueError('global deadline headroom after guard')
            before = ops.ledger()
            save_new(state / f'{i}-intent.json', dict(arm=arm, index=i))
            process = ops.run(arm, state)
            ops.guard(initial=False, dispatch=False)
            checked = ops.export_and_verify(arm, process, state / f'evidence-{i}')
            result, diagnostics = checked['result'], checked['diagnostics']
            if result['task'] != arm['task'] or result['system'] != arm['mode']:
                raise ValueError('result identity')
            ids = previous.settled_diagnostics(before, ops.ledger(), diagnostics)
            if any(d.get('code') != 'ok' or d.get('status') != 'completed' for d in diagnostics):
                raise ValueError('provider/protocol failure; halt pair')
            if result['outcome'] not in ('accepted', 'rejected') or result.get('oracle') is None:
                raise ValueError('no final oracle; halt pair')
            if checked['proposed_count'] != len(ids):
                raise ValueError('proposal/call count mismatch')
            if decisions and result['state_root_at_start'] != decisions[0]['start_root']:
                raise ValueError('paired roots differ')
            if time.monotonic() - started >= m['global_wall_seconds']:
                raise ValueError('global deadline')
            decision = dict(task=arm['task'], mode=arm['mode'], outcome=result['outcome'],
                start_root=result['state_root_at_start'], reservations=ids,
                evidence_sha256=checked['manifest_sha256'])
            save_new(state / f'{i}-decision.json', decision)
            decisions.append(decision)
        save_new(state / 'terminal.json', dict(status='completed', results=decisions))
    except Exception:
        # Partial exports/raw arm outputs are retained. Never mint a successful decision
        # for incomplete evidence. A failed export/process requires independent salvage.
        save_new(state / 'terminal.json', dict(status='halted-review-required', results=decisions,
            evidence='Preserve partial exports, worker logs and raw arm outputs; verify quiescence before salvage.'))
        raise
    return decisions


class LocalOperations(previous.LocalOperations):
    def __init__(self, manifest):
        super().__init__(manifest)
        # Bound verifier child processes without changing the frozen verifier's logic.
        self.receipts.subprocess = SimpleNamespace(
            check_output=lambda *a, **kw: subprocess.check_output(*a, **(kw | {'timeout': 15})),
            run=lambda *a, **kw: subprocess.run(*a, **(kw | {'timeout': 60})))

    def guard(self, initial, dispatch=True):
        previous.verify_freeze(self.m)
        if str(Path(sys.executable).resolve()) != self.m['python']:
            raise ValueError('controller Python interpreter mismatch')
        self.old.require_idle()
        baseline = json.loads((ROOT / self.m['baseline']).read_text())
        budget.check(self.ledger(), baseline, dispatch=dispatch)
        minimum = self.m['initial_minimum_free_gib'] if initial else self.m['live_guard_gib']
        if shutil.disk_usage(ROOT).free < minimum * 1024**3:
            raise ValueError('disk reserve')
        image = subprocess.check_output(['docker', 'image', 'inspect', '--format', '{{.Id}}', self.m['sandbox_image']], text=True, timeout=15).strip()
        if image != self.m['sandbox_image']:
            raise ValueError('sandbox image mismatch')
        if initial:
            if budget.history.freeze(self.ledger()) != baseline:
                raise ValueError('baseline is not exact current ledger')
            for arm in self.m['arms']:
                base = ROOT / arm['input']
                if (base / arm['mode']).exists() or list(base.glob('startup-' + arm['mode'] + '.sqlite*')):
                    raise ValueError('arm not fresh')

    def export_and_verify(self, arm, process, destination):
        checked = super().export_and_verify(arm, process, destination)
        # Require an explicit Stop proposal before any completed outcome; hitting a cap
        # must not be mistaken for model submission. The final evaluator is unchanged.
        import sqlite3
        label = arm['task'][:12] + '-' + arm['mode']
        db = sqlite3.connect((destination / (label + '-state.sqlite')).as_uri() + '?mode=ro&immutable=1', uri=True)
        try:
            kinds = [json.loads(row[0])['kind'] for row in db.execute('select payload from events order by seq')]
        finally:
            db.close()
        proposals = [e['candidate']['kind'] for e in kinds if e['kind'] == 'proposed']
        if not proposals or proposals[-1] != 'stop':
            raise ValueError('explicit submission missing')
        return checked


def main():
    if len(sys.argv) != 3 or sys.argv[1] != '--approval':
        raise ValueError('separate approval bound to exact freeze required')
    m = json.loads(MANIFEST.read_text())
    approval = json.loads(Path(sys.argv[2]).read_text())
    expected = dict(status='approved', freeze_sha256=hashlib.sha256(MANIFEST.read_bytes()).hexdigest(),
        incremental_cap_usd='5', aggregate_cap_usd='100', arms=[[a['task'], a['mode']] for a in m['arms']])
    if approval != expected:
        raise ValueError('approval scope mismatch')
    coordinate(m, ROOT / m['state'], LocalOperations(m))


if __name__ == '__main__':
    main()
