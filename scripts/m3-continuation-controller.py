"""Three-arm one-shot controller. No dispatch without a separate matching approval.

Policy decisions use controller-owned exports, settlement ledger and guard checks.
Tests inject local fake operations; no model/provider values grant authority.
"""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sqlite3
import struct
import subprocess
import sys
import time
from contextlib import closing
import m3_smoke_budget as budget

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / 'docs/m3-continuation-readiness.json'


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / (name + '.py'))
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    return module


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sync_directory(path):
    fd=os.open(path, os.O_RDONLY)
    try: os.fsync(fd)
    finally: os.close(fd)


def save_new(path, value):
    with path.open('x') as handle:
        json.dump(value, handle, sort_keys=True, indent=2); handle.write('\n')
        handle.flush(); os.fsync(handle.fileno())
    sync_directory(path.parent)


def verify_freeze(manifest):
    for relative, expected in manifest['sha256'].items():
        path = ROOT / relative
        if path.is_symlink() or sha(path) != expected:
            raise ValueError('frozen input mismatch')


def eligible(diagnostic):
    shape = diagnostic.get('response_shape', {})
    if (diagnostic.get('stage') != 'decode' or diagnostic.get('status') != 'failed'
            or diagnostic.get('settled') is not True or shape.get('version') != 1
            or shape.get('choices') != 'one' or shape.get('message_type') != 'object'
            or shape.get('refusal_truthy') is not False
            or shape.get('finish_reason') not in ('stop', 'tool_calls', 'function_call', 'missing', 'null', 'other_string')):
        return False
    if diagnostic.get('code') == 'tool_shape':
        return shape.get('tool_calls') in ('missing', 'zero', 'multiple') or (
            shape.get('tool_calls') == 'one' and shape.get('call_type') == 'object' and (
                shape.get('function_type') == 'missing' or
                shape.get('function_type') == 'object' and shape.get('name_is_propose') is False))
    return (diagnostic.get('code') == 'json_invalid' and shape.get('tool_calls') == 'one'
            and shape.get('name_is_propose') is True and shape.get('argument_parse') == 'invalid_json')


def settled_diagnostics(before, after, diagnostics):
    if not after.startswith(before): raise ValueError('ledger prefix changed')
    budget.total(before); budget.total(after)  # rejects partial, duplicate, unknown, unresolved
    events = [json.loads(line) for line in after[len(before):].splitlines()]
    reserves = [e['id'] for e in events if e['kind'] == 'reserve']
    settlements = {e['id']:e for e in events if e['kind'] == 'settled'}
    ids = [d.get('reservation_id') for d in diagnostics]
    if len(ids) != len(set(ids)) or ids != reserves or set(ids) != set(settlements):
        raise ValueError('diagnostics do not bind exact arm reservations')
    if not ids or len(ids) > 20: raise ValueError('arm call bound')
    if any(d.get('settled') is not True for d in diagnostics): raise ValueError('unsettled diagnostic')
    return ids


def disposition(result, diagnostics, before, after, proposed_count):
    settled_diagnostics(before, after, diagnostics)
    if any(d.get('code') != 'ok' or d.get('status') != 'completed' for d in diagnostics[:-1]):
        raise ValueError('earlier provider failure')
    if result['outcome'] == 'unavailable':
        if not eligible(diagnostics[-1]) or proposed_count != len(diagnostics)-1:
            raise ValueError('ineligible unavailable arm')
        return 'unavailable'  # never reclassify, retry, or repair
    if result['outcome'] not in ('accepted', 'rejected') or any(d.get('code') != 'ok' for d in diagnostics):
        raise ValueError('ineligible outcome')
    return result['outcome']


def coordinate(manifest, state, ops):
    """At most once across restarts: state directory and intent precede dispatch."""
    if state.exists(): raise ValueError('one-shot state exists; no resume or retry')
    arms = manifest['arms']
    keys = [(a['task'], a['mode']) for a in arms]
    if len(arms) != 3 or len(set(keys)) != 3 or set(keys) != set(map(tuple, manifest['unattempted'])):
        raise ValueError('three-arm scope mismatch')
    ops.guard(initial=True)
    state.mkdir()
    sync_directory(state.parent)
    save_new(state/'plan.json', manifest)
    results = []
    started = time.monotonic()
    try:
        for index, arm in enumerate(arms):
            if manifest['global_wall_seconds']-(time.monotonic()-started) < 1800: raise ValueError('global deadline headroom')
            ops.guard(initial=False)
            before = ops.ledger()
            # Intent is durable before any subprocess/network, and never removed.
            save_new(state/(str(index)+'-intent.json'), dict(arm=arm, index=index))
            raw = ops.run(arm, state)
            if time.monotonic()-started >= manifest['global_wall_seconds']: raise ValueError('global deadline')
            ops.guard(initial=False) # real quiescence/ledger/freeze checks before export
            checked = ops.export_and_verify(arm, raw, state/('evidence-'+str(index)))
            result, diagnostics = checked['result'], checked['diagnostics']
            if result.get('task') != arm['task'] or result.get('system') != arm['mode']:
                raise ValueError('result identity')
            status = disposition(result, diagnostics, before, ops.ledger(), checked['proposed_count'])
            decision = dict(task=arm['task'], mode=arm['mode'], outcome=status,
                            evidence_sha256=checked['manifest_sha256'], reservations=[d['reservation_id'] for d in diagnostics])
            save_new(state/(str(index)+'-decision.json'), decision)
            results.append(decision)
        save_new(state/'terminal.json', dict(status='completed', results=results))
    except Exception:
        save_new(state/'terminal.json', dict(status='halted-review-required', results=results))
        raise
    return results


class LocalOperations:
    def __init__(self, manifest):
        self.m = manifest
        self.old = load('m3-progress-validation-controller')
        self.worker = load('m3-progress-validation-worker')
        self.receipts = load('m3-progress-validation-evidence')
        self.ledger_path = ROOT / manifest['ledger']
        self.diagnostic_path = self.ledger_path.with_name(self.ledger_path.stem+'-diagnostics.jsonl')

    def ledger(self): return self.ledger_path.read_bytes()

    def guard(self, initial):
        verify_freeze(self.m)
        self.old.require_idle()
        baseline = json.loads((ROOT/self.m['baseline']).read_text())
        budget.check(self.ledger(), baseline)
        minimum = self.m['initial_minimum_free_gib'] if initial else self.m['live_guard_gib']
        if shutil.disk_usage(ROOT).free < minimum*1024**3: raise ValueError('disk reserve')

    def run(self, arm, state):
        base = ROOT / arm['input']; mode = arm['mode']
        out = base / mode
        startup = base / ('startup-'+mode+'.sqlite')
        if out.exists() or list(base.glob(startup.name+'*')): raise ValueError('arm not fresh')
        self.diagnostic_before = self.diagnostic_path.read_bytes() if self.diagnostic_path.exists() else b''
        self.worker.STATE = state
        env = self.worker.env_for(False)
        env.update(TSR_M3_APPROVED_LIVE='100', TSR_M3_BROKER_SCRIPT=str(ROOT/self.m['broker']),
            TSR_M3_PYTHON=sys.executable, TSR_M3_COST_LEDGER=str(self.ledger_path), TSR_DB=str(startup))
        record = self.worker.run(arm['task'][:12]+'-'+mode,
            [str(ROOT/self.m['engine']), 'task','run','--mode',mode,'--tasks',str(base/'tasks.jsonl'),
             '--driver','cheaper-inference','--profile',str(base/'profile.toml'),'--out',str(out)],env)
        if record.get('status') != 'completed' or record.get('exit') != 0 or record.get('errors'):
            raise ValueError('arm process/containment failure')
        # Detect descendants even if no matching executable name remains.
        pid = record['pid']
        rows = subprocess.check_output(['ps','-axo','pid=,pgid='],text=True,timeout=15).splitlines()
        if any(pid in [int(v) for v in row.split()] for row in rows): raise ValueError('arm process group remains')
        return record

    def digest(self, raw):
        return subprocess.check_output([str(ROOT/'target/m3-blake3-checksum')], input=raw, timeout=15).decode().strip()

    def export_and_verify(self, arm, process, destination):
        base=ROOT/arm['input'];mode=arm['mode'];label=arm['task'][:12]+'-'+mode;out=base/mode
        destination.mkdir()
        # Refuse links/special files: evidence is controller-owned, never candidate paths.
        for directory in (out/'cas',):
            if directory.is_symlink() or not directory.is_dir(): raise ValueError('missing CAS')
            for p in directory.iterdir():
                if p.is_symlink() or not p.is_file(): raise ValueError('unsafe CAS entry')
        shutil.copytree(out/'cas',destination/label/'cas')
        shutil.copytree(base/'cas',destination/'input-cas')
        for name in ('profile.toml','broker-contexts.json','preparation.json'):
            shutil.copyfile(base/name,destination/name)
        result_path=out/('results-'+mode+'.jsonl')
        if result_path.is_symlink(): raise ValueError('result symlink')
        shutil.copyfile(result_path,destination/(label+'-results.jsonl'))
        shutil.copyfile(base/'tasks.jsonl',destination/(arm['task'][:12]+'-tasks.jsonl'))
        stores=list((out/'state').glob('*/*.sqlite'))
        if len(stores)!=1 or stores[0].is_symlink(): raise ValueError('store count/type')
        target=destination/(label+'-state.sqlite')
        with closing(sqlite3.connect('file:'+str(stores[0])+'?mode=ro',uri=True)) as source, closing(sqlite3.connect(target)) as dest:
            source.backup(dest);dest.execute('pragma journal_mode=DELETE')
        with closing(sqlite3.connect('file:'+str(target)+'?mode=ro&immutable=1',uri=True)) as db:
            if db.execute('pragma integrity_check').fetchone()[0]!='ok': raise ValueError('store integrity')
            events=[(i,json.loads(p)) for i,p in db.execute('select id,payload from events order by seq')]
        rows=[json.loads(line) for line in result_path.read_text().splitlines()]
        if len(rows)!=1: raise ValueError('result count')
        result=rows[0]
        root=self.digest(b''.join(struct.pack('<Q',len(i.encode()))+i.encode() for i,_ in events))
        if result.get('state_root_at_start')!=self.digest(b'') or result.get('state_root_at_end')!=root:
            raise ValueError('state root mismatch')
        closures=[e['kind']['reason'] for _,e in events if e['kind']['kind']=='task_closed']
        if len(closures)!=1 or not events or events[-1][1]['kind']['kind']!='task_closed': raise ValueError('closure evidence')
        if result.get('outcome')=='unavailable':
            if result.get('oracle') is not None or closures!=['unavailable:broker failed; see controller cost ledger for reserved charges']:
                raise ValueError('unavailable closure mismatch')
        for p in list((destination/label/'cas').iterdir())+list((destination/'input-cas').iterdir()):
            if self.digest(p.read_bytes())!=p.name: raise ValueError('CAS digest mismatch')
        receipt=self.receipts.verify_receipts(destination,label,dict(result=result,task=arm['task'],mode=mode))
        if result.get('oracle') is not None and receipt['status']!='checked': raise ValueError('oracle receipts missing')
        if result.get('denials'):
            raise ValueError('action or policy denial')
        if result.get('outcome') in ('accepted','rejected') and (result.get('oracle') is None or receipt.get('status')!='checked' or closures!=['oracle:'+str(result['oracle']['accepted']).lower()]):
            raise ValueError('non-oracle rejection or closure mismatch')
        if result.get('outcome') in ('accepted','rejected') and (result['oracle'].get('accepted') is not (result['outcome']=='accepted') or receipt.get('verified_success') is not (result['outcome']=='accepted')):
            raise ValueError('outcome/oracle receipt mismatch')
        if result.get('outcome')=='accepted' and (not receipt.get('verified_success') or closures!=['oracle:true']):
            raise ValueError('acceptance evidence')
        raw=self.diagnostic_path.read_bytes()
        if not raw.startswith(self.diagnostic_before) or not raw.endswith(b'\n'): raise ValueError('diagnostic prefix')
        diagnostics=[json.loads(line) for line in raw[len(self.diagnostic_before):].splitlines()]
        save_new(destination/'diagnostics.json',diagnostics)
        save_new(destination/'process.json',process)
        save_new(destination/'receipt-verification.json',receipt)
        (destination/'ledger.jsonl').write_bytes(self.ledger())
        hashes={str(p.relative_to(destination)):sha(p) for p in destination.rglob('*') if p.is_file()}
        for p in hashes:
            with (destination/p).open('rb') as handle: os.fsync(handle.fileno())
        save_new(destination/'sha256.json',hashes)
        for directory in sorted([p for p in destination.rglob('*') if p.is_dir()],key=lambda p:len(p.parts),reverse=True): sync_directory(directory)
        sync_directory(destination)
        sync_directory(destination.parent)
        for p,h in hashes.items():
            if sha(destination/p)!=h: raise ValueError('export changed')
        return dict(result=result, diagnostics=diagnostics,
                    proposed_count=sum(e['kind']['kind']=='proposed' for _,e in events),manifest_sha256=sha(destination/'sha256.json'))


def main():
    # Approval is separate from the immutable readiness manifest. No implicit launch.
    if len(sys.argv)!=3 or sys.argv[1]!='--approval': raise ValueError('separate scoped approval required')
    manifest=json.loads(MANIFEST.read_text());approval=json.loads(Path(sys.argv[2]).read_text())
    if approval != dict(status='approved', readiness_sha256=sha(MANIFEST), incremental_cap_usd='10', aggregate_cap_usd='100', arms=manifest['unattempted']):
        raise ValueError('approval scope mismatch')
    coordinate(manifest,ROOT/manifest['state'],LocalOperations(manifest))

if __name__=='__main__': main()
