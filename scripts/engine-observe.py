#!/usr/bin/env python3
"""Bounded foreground collection of explicitly mapped local evidence. No discovery."""
import argparse
from contextlib import redirect_stdout
import fcntl
import hashlib
import importlib.util
import json
import os
from pathlib import Path, PurePosixPath
import stat
import sys
import time
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('directions', ROOT / 'scripts/engine-directions.py')
directions = importlib.util.module_from_spec(spec)
spec.loader.exec_module(directions)


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def identity(st):
    return st.st_dev, st.st_ino, st.st_size, st.st_mtime_ns, st.st_ctime_ns


def stable_read(root, relative, limit):
    """Open each component without symlinks and verify descriptor/path stability.

    This is a single-file snapshot, not a transaction over independent source files.
    A source changed after this snapshot is collected on a subsequent bounded poll.
    """
    path = PurePosixPath(relative)
    if not relative or path.is_absolute() or any(p in ('', '.', '..') for p in relative.split('/')):
        raise ValueError('source must be an explicit relative file path without traversal')
    if not Path(root).is_absolute() or str(Path(root).resolve(strict=True)) != root:
        raise ValueError('source root must be an existing canonical absolute directory')
    fds = []
    links = []
    try:
        anchor = os.stat(root, follow_symlinks=False)
        fd = os.open(root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
        fds.append(fd)
        opened = os.fstat(fd)
        if (anchor.st_dev, anchor.st_ino) != (opened.st_dev, opened.st_ino):
            raise ValueError('source root changed before read')
        for part in path.parts[:-1]:
            child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW, dir_fd=fd)
            fds.append(child); links.append((fd, part, child)); fd = child
        file_fd = os.open(path.name, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK, dir_fd=fd)
        fds.append(file_fd)
        before = os.fstat(file_fd)
        if not stat.S_ISREG(before.st_mode) or before.st_nlink != 1 or before.st_size > limit:
            raise ValueError('source must be a bounded, single-link regular file')
        raw = os.read(file_fd, limit + 1)
        after = os.fstat(file_fd)
        named = os.stat(path.name, dir_fd=fd, follow_symlinks=False)
        if len(raw) > limit or len(raw) != before.st_size or identity(before) != identity(after) or identity(after) != identity(named):
            raise ValueError('source changed during read; no observation submitted')
        for parent, name, child in links:
            expected, actual = os.fstat(child), os.stat(name, dir_fd=parent, follow_symlinks=False)
            if (expected.st_dev, expected.st_ino) != (actual.st_dev, actual.st_ino) or not stat.S_ISDIR(actual.st_mode):
                raise ValueError('source directory changed during read')
        current = os.stat(root, follow_symlinks=False)
        if (anchor.st_dev, anchor.st_ino) != (current.st_dev, current.st_ino) or str(Path(root).resolve(strict=True)) != root:
            raise ValueError('source root changed during read')
        text = raw.decode('utf-8', errors='strict')
        if not text.strip():
            raise ValueError('empty source is not a supported note revision')
        return text, {'sha256': hashlib.sha256(raw).hexdigest(), 'device': before.st_dev,
                      'inode': before.st_ino, 'size': before.st_size, 'mtime_ns': before.st_mtime_ns}
    finally:
        for fd in reversed(fds):
            os.close(fd)


def validate(mapping, state):
    if not isinstance(mapping, dict) or set(mapping) != {'root', 'max_bytes', 'sources', 'continuation'}:
        raise ValueError('mapping requires root, max_bytes, sources and continuation')
    if type(mapping['max_bytes']) is not int or not 1 <= mapping['max_bytes'] <= 512:
        raise ValueError('source byte limit must be 1..512')
    sources = mapping['sources']
    if not isinstance(sources, list) or not 1 <= len(sources) <= 32:
        raise ValueError('explicit 1..32 source mappings required')
    admitted = {(n['id'], n['topic']) for n in state['available_notes']}
    ids, paths = set(), set()
    for source in sources:
        if not isinstance(source, dict) or set(source) != {'id', 'topic', 'path'} or not all(isinstance(v, str) for v in source.values()):
            raise ValueError('source requires string id, topic and path only')
        if (source['id'], source['topic']) not in admitted or source['id'] in ids or source['path'] in paths:
            raise ValueError('mapping must uniquely name admitted evidence and explicit files')
        ids.add(source['id']); paths.add(source['path'])
    continuation = mapping['continuation']
    if continuation is not None:
        if not isinstance(continuation, dict) or set(continuation) != {'model', 'endpoint', 'calls'}:
            raise ValueError('continuation requires existing local model, endpoint and frozen calls')
        directions.local.endpoint(continuation['endpoint'])
        if not isinstance(continuation['model'], str) or 'cloud' in continuation['model'].lower() or type(continuation['calls']) is not int or not 1 <= continuation['calls'] <= 12:
            raise ValueError('bounded installed local continuation required')


def collect(directory, mapping, journal, events, record, stop):
    state = directions.cli(directory, 'direction-request')
    validate(mapping, state)
    pending = None
    for event in events:
        if event['status'] == 'observation_started':
            pending = event
        elif event['status'] in ('observation_confirmed', 'observation_not_committed'):
            pending = None
    if pending:
        matches = [u for u in state['observation_updates'] if u['reason'] == pending['reason'] and u['from'] == pending['revision']]
        if matches:
            record({'status': 'observation_confirmed', 'id': pending['id'], 'reconciled': True,
                    'revision': matches[-1]['to'], 'snapshot': pending['snapshot']})
        elif state['state_root'] == pending['state_root']:
            # Exact unchanged head proves no append occurred. An idempotent no-op
            # may have run; rereading and resubmitting cannot duplicate an event.
            record({'status': 'observation_not_committed', 'id': pending['id']})
        else:
            raise ValueError('uncertain observation outcome with advanced state; inspect journal before retry')
    if stop.exists():
        return {'status': 'operator_stop', 'state': state}
    notes, provenance = [], []
    for source in mapping['sources']:
        text, version = stable_read(mapping['root'], source['path'], mapping['max_bytes'])
        notes.append({'id': source['id'], 'topic': source['topic'], 'text': text})
        provenance.append({**source, **version})
    snapshot = digest(provenance)
    last = next((e for e in reversed(events) if e['status'] == 'observation_confirmed'), None)
    if last and last['snapshot'] == snapshot and last['revision'] == state['observation_revision']:
        return {'status': 'unchanged', 'state': state}
    if stop.exists():
        return {'status': 'operator_stop', 'state': state}
    submission = digest({'root': state['state_root'], 'snapshot': snapshot, 'mapping': mapping})
    reason = 'local collector snapshot ' + submission
    payload = {'state_root': state['state_root'], 'reason': reason, 'notes': notes}
    path = directory / ('collector-observation-' + submission + '.json')
    if path.exists():
        existing, _ = stable_read(str(directory), path.name, 32768)
        if directions.local.runner.codec.strict_json(existing) != payload:
            raise ValueError('existing observation artifact differs')
    else:
        directions.local.save(path, payload)
    record({'status': 'observation_started', 'id': submission, 'state_root': state['state_root'],
            'revision': state['observation_revision'], 'reason': reason, 'snapshot': snapshot, 'provenance': provenance})
    if stop.exists():
        record({'status': 'observation_not_committed', 'id': submission, 'reason': 'operator_stop_before_dispatch'})
        return {'status': 'operator_stop', 'state': state}
    # Errors/interruptions leave the durable started record. Next invocation must
    # reconcile the authoritative event chain; it cannot blindly duplicate a call.
    state = directions.cli(directory, 'direction-observe', '--observation', str(path))
    record({'status': 'observation_confirmed', 'id': submission, 'revision': state['observation_revision'], 'snapshot': snapshot})
    return {'status': 'observed', 'state': state}


def run(args):
    directory = args.directory.resolve(strict=True)
    raw, _ = stable_read(str(args.mapping.absolute().parent), args.mapping.name, 16384)
    mapping = directions.local.runner.codec.strict_json(raw)
    if not 1 <= args.polls <= 10 or not 1 <= args.interval <= 60:
        raise ValueError('foreground bounds: 1..10 polls and 1..60 seconds between polls')
    stop = directory / 'collector.stop'
    journal_path = directory / 'collector.jsonl'
    with os.fdopen(os.open(journal_path, os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600), 'r+') as journal:
        fcntl.flock(journal, fcntl.LOCK_EX | fcntl.LOCK_NB)
        events = [directions.local.runner.codec.strict_json(line) for line in journal]
        config = {'status': 'config', 'mapping': mapping}
        if events and events[0] != config:
            raise ValueError('collector mapping and continuation allowance are frozen for this run')
        def record(value):
            journal.seek(0, 2); journal.write(json.dumps(value) + '\n'); journal.flush(); os.fsync(journal.fileno()); events.append(value)
        if not events:
            validate(mapping, directions.cli(directory, 'direction-request'))
            record(config)
        for poll in range(args.polls):
            result = collect(directory, mapping, journal, events, record, stop)
            if result['status'] == 'operator_stop':
                return {'status': 'operator_stop'}
            if args.continue_run and mapping['continuation'] and result['state']['status'] == 'deliberate':
                record({'status': 'continuation_started', 'state_root': result['state']['state_root']})
                with redirect_stdout(sys.stderr):
                    continuation = directions.run(SimpleNamespace(directory=directory, **mapping['continuation'], batch=None, batch_calls=None, stop_file=stop))
                record({'status': 'continuation_returned', 'result': continuation['status']})
                result = {'status': continuation['status'], 'state': continuation['state']} if 'state' in continuation else continuation
                if continuation['status'] != 'satisfied':
                    return result
            record({'status': 'poll_complete', 'poll': poll + 1, 'result': result['status'],
                    'state_root': result.get('state', {}).get('state_root')})
            if poll + 1 < args.polls:
                # Foreground, interruptible, bounded polling; no scheduler installation.
                for _ in range(args.interval):
                    if stop.exists():
                        return {'status': 'operator_stop'}
                    time.sleep(1)
        return result


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--directory', type=Path, required=True)
    parser.add_argument('--mapping', type=Path, required=True)
    parser.add_argument('--polls', type=int, default=1)
    parser.add_argument('--interval', type=int, default=1)
    parser.add_argument('--continue', dest='continue_run', action='store_true')
    print(json.dumps(run(parser.parse_args()), indent=2))
