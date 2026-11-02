"""Durable completion receipts for prospective outer orchestration stages.

A receipt exists only after the callback returned successfully. It closes the
crash window before the outer journal records that return; it never authorizes
calling the callback again. Missing receipts remain uncertain. The caller must
also reject unresolved nested transport/native effects before recovery.
"""
import hashlib
import json
import os
from pathlib import Path
import tempfile

PROFILE = 'durable-stage-receipts@v1'
MAX_BYTES = 8 * 1024 * 1024


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False, allow_nan=False).encode()


def digest(value):
    return hashlib.sha256(canonical(value)).hexdigest()


def _unique(pairs):
    value = {}
    for key, item in pairs:
        if key in value:
            raise ValueError('duplicate stage receipt key')
        value[key] = item
    return value


class Receipts:
    def __init__(self, root, admission):
        self.root = Path(root) / 'stage-receipts'
        self.admission = admission

    def binding(self, rows, intent):
        if not rows or rows[-1]['event'] != intent or intent.get('kind') != 'intent':
            raise ValueError('stage receipt requires exact terminal intent')
        return {'profile': PROFILE, 'admission_sha256': digest(self.admission),
                'intent_sha256': digest(intent), 'journal_prefix_sha256': digest(rows),
                'stage': intent['name'], 'prefix_rows': len(rows)}

    def path(self, binding):
        return self.root / (digest(binding) + '.json')

    def _boundary(self):
        if any(p.is_symlink() for p in (self.root, *self.root.parents)):
            raise ValueError('stage receipt symlink')

    def save(self, rows, intent, value):
        self._boundary()
        binding = self.binding(rows, intent)
        receipt = {'binding': binding, 'value': value}
        encoded = canonical({**receipt, 'sha256': digest(receipt)}) + b'\n'
        if len(encoded) > MAX_BYTES:
            raise ValueError('stage receipt exceeds local journal resource capacity')
        self.root.mkdir(mode=0o700, exist_ok=True)
        path = self.path(binding)
        # Publish a completely fsynced immutable file atomically. An interrupted
        # temporary write is never mistaken for completion evidence.
        temporary = None
        try:
            with tempfile.NamedTemporaryFile(dir=self.root, prefix='.pending-', delete=False) as f:
                temporary = Path(f.name)
                f.write(encoded)
                f.flush()
                os.fsync(f.fileno())
            os.link(temporary, path, follow_symlinks=False)
            fd = os.open(self.root, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
            try:
                os.fsync(fd)
            finally:
                os.close(fd)
        finally:
            if temporary is not None:
                temporary.unlink(missing_ok=True)

    def load(self, rows, intent):
        self._boundary()
        binding = self.binding(rows, intent)
        path = self.path(binding)
        if path.is_symlink():
            raise ValueError('stage receipt symlink')
        if not path.exists():
            return None
        fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW)
        with os.fdopen(fd, 'rb') as f:
            raw = f.read(MAX_BYTES + 1)
        if len(raw) > MAX_BYTES:
            raise ValueError('stage receipt resource capacity')
        receipt = json.loads(raw, object_pairs_hook=_unique,
                            parse_constant=lambda _: (_ for _ in ()).throw(ValueError('nonfinite receipt')))
        if (not isinstance(receipt, dict) or set(receipt) != {'binding', 'value', 'sha256'}
                or receipt['binding'] != binding
                or receipt['sha256'] != digest({k: receipt[k] for k in ('binding', 'value')})):
            raise ValueError('stage receipt binding/content drift')
        return receipt
