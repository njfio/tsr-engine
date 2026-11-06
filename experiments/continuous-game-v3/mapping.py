"""Offline reviewed mapping/probe compiler for the next continuous-game test.

Same reviewed-control-map contract as experiments/game-wants-v3/mapping.py (which
stays frozen: the v2 launch manifest pins its hash), plus two changes:

* The mapping carries ``coverage``: for each criterion, the reviewer's lower bound
  on inputs needed to reach a state where that criterion could be judged, with
  its basis. ``search`` sizes every predeclared word from it, up to 32 inputs
  (64 harness actions), instead of the fixed six inputs that left every v2
  finding Unknown.
* ``plan`` refuses a batch that scripts/engine-observation-coverage.py reports
  blocked, so a stage that cannot resolve its criterion is never dispatched.

No dispatcher, reviewer authentication or authority is created here.
"""
import copy
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def _load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


base = _load('continuous_v3_base_mapping', ROOT / 'experiments/game-wants-v3/mapping.py')
coverage_tool = _load('continuous_v3_coverage', ROOT / 'scripts/engine-observation-coverage.py')

VERSION = 'reviewed-control-map@v2'
BATCHES = ('crossing-first', 'growth-first', 'mixed-pursuit')
ROOTS = ('crossing', 'growth')
MAX_KEYS = 32  # 1 start click + 32 keys + 31 waits = 64 harness actions
MIN_KEYS = 12  # never shorter than twice the v2 words
MARGIN = 4     # inputs beyond the declared minimum, for hazards and detours
digest = base.digest
source = base.source


def split(mapping):
    """The v1 contract fields, and the coverage declaration."""
    if not isinstance(mapping, dict) or mapping.get('version') != VERSION or 'coverage' not in mapping:
        raise ValueError('mapping contract: ' + VERSION + ' with coverage required')
    legacy = {k: v for k, v in mapping.items() if k not in ('coverage', 'version')}
    legacy['version'] = 'reviewed-control-map@v1'
    declared = mapping['coverage']
    if not isinstance(declared, dict) or set(declared) != set(ROOTS):
        raise ValueError('coverage must declare exactly crossing and growth')
    coverage_tool.static(declared, {})  # validates each {min_inputs, basis}
    for root, item in declared.items():
        if item['min_inputs'] > MAX_KEYS - MARGIN:
            raise ValueError(root + ': declared min_inputs exceeds what a %d-input trace can cover' % MAX_KEYS)
    return legacy, declared


def validate(mapping, files, review):
    """Validate against the frozen v1 rules with the review bound to the full v2 mapping."""
    legacy, declared = split(mapping)
    if not isinstance(review, dict) or review.get('mapping_sha256') != digest(mapping):
        raise ValueError('mapping review binding')
    result = base.validate(legacy, files, {**review, 'mapping_sha256': digest(legacy)})
    if result.get('status') == 'mapped':
        result = {**result, 'mapping_sha256': digest(mapping), 'coverage': copy.deepcopy(declared)}
    return result


def root_at(batch, index):
    if batch not in BATCHES:
        raise ValueError('unknown observation batch')
    return 'growth' if batch == 'growth-first' or (batch == 'mixed-pursuit' and index % 2) else 'crossing'


def word_length(declared, batch):
    roots = {'crossing-first': ['crossing'], 'growth-first': ['growth'], 'mixed-pursuit': list(ROOTS)}[batch]
    need = max(declared[r]['min_inputs'] for r in roots)
    # mixed-pursuit alternates roots, so each root gets every other input.
    if batch == 'mixed-pursuit':
        need *= 2
    return max(MIN_KEYS, min(MAX_KEYS, need + MARGIN))


def search(mapping, batch):
    """Six predeclared words per batch, each sized from the reviewed coverage."""
    _, declared = split(mapping)
    n = word_length(declared, batch)
    cycles = (('Up', 'Right', 'Down', 'Left'), ('Up', 'Left', 'Down', 'Right'))
    words = [(d,) * n for d in ('Up', 'Right', 'Down', 'Left')] + [tuple((c * n)[:n]) for c in cycles]
    return [[mapping['controls'][root_at(batch, i)][d] for i, d in enumerate(word)] for word in words]


def batch_coverage(mapping, batch, words):
    """Coverage report for the criterion (or criteria) a batch is meant to resolve."""
    _, declared = split(mapping)
    if batch == 'mixed-pursuit':
        # Alternating roots: count each root's own inputs within every word.
        sequences = {r: [[k for i, k in enumerate(w) if root_at(batch, i) == r] for w in words] for r in ROOTS}
        need = declared
    else:
        root = root_at(batch, 0)
        sequences = {root: words}
        need = {root: declared[root]}
    return coverage_tool.static(need, sequences)


def plan(mapping, files, review, batch, keys):
    valid = validate(mapping, files, review)
    if valid['status'] != 'mapped':
        return valid
    if not 1 <= len(keys) <= MAX_KEYS:
        raise ValueError('unsupported sequence length; no truncation')
    for i, key in enumerate(keys):
        if key not in mapping['controls'][root_at(batch, i)].values():
            raise ValueError('input outside reviewed batch controls')
    report = batch_coverage(mapping, batch, search(mapping, batch))
    if report['status'] != 'resolvable':
        raise ValueError('observation stage cannot resolve %s; refusing dispatch' % ', '.join(report['blocked_criteria']))
    actions = [{'kind': 'click', 'value': mapping['start']}]
    for i, key in enumerate(keys):
        if i:
            actions.append({'kind': 'wait', 'value': 100})
        actions.append({'kind': 'key', 'value': key})
    if len(actions) > coverage_tool.HARNESS_MAX_ACTIONS:
        raise ValueError('probe exceeds harness action bound')
    probe = {'version': 'browser-probe@v1', 'assets': sorted(files), 'entry': mapping['entry'], 'actions': actions,
             'checks': [{'selector': v, 'text': ''} for v in dict.fromkeys(mapping['readouts'].values())]}
    body = {**valid, 'batch': batch, 'probe': probe, 'coverage_report': report,
            'coverage': 'Finite ordered sequence and final DOM readouts only; no inferred success or failure. Every probe '
                        'starts a fresh page and uses advertised restart. No cross-probe state continuation. Word length '
                        'comes from the reviewed coverage declaration, fixed before any result is seen.'}
    return {**body, 'plan_sha256': digest(body)}


def bind_observation(plan, value, files):
    return base.bind_observation(plan, value, files)
