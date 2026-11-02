#!/usr/bin/env python3
"""Pinned evaluator-authored repair evaluation. Preparation only; no execution or inference."""
import hashlib
import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('repair_suite_legacy', Path(__file__).with_name('engine-b91-prepare.py'))
legacy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(legacy)

TASKS = {
    'singleton-clamp': {
        'fault_family': 'degenerate interval versus invalid ordering',
        'source': '''pub fn clamp(value: i64, lower: i64, upper: i64) -> Result<i64, String> {
    if lower >= upper { return Err("invalid range".to_owned()); }
    Ok(value.max(lower).min(upper))
}
''',
        'failed': '''pub fn clamp(value: i64, lower: i64, upper: i64) -> Result<i64, String> {
    Ok(value.max(lower).min(upper))
}
''',
        'requirements': {
            'requested_behavior': 'Accept a singleton interval (lower equals upper), returning that endpoint for every i64 input value.',
            'existing_behaviour': 'Preserve all prior behavior when lower differs from upper: reversed bounds return Err("invalid range"); ordered bounds return the inclusive clamp without arithmetic overflow for all i64 values.'},
        'tests': '''use parser::clamp;
#[test] fn requested_behavior_handled() {
    for endpoint in [i64::MIN, -4, 0, 7, i64::MAX] {
        for value in [i64::MIN, -9, 0, 11, i64::MAX] {
            assert_eq!(clamp(value, endpoint, endpoint), Ok(endpoint));
        }
    }
}
#[test] fn existing_behaviour_unchanged() {
    for (lower, upper) in [(5, -5), (1, 0), (i64::MAX, i64::MIN)] {
        for value in [i64::MIN, 0, i64::MAX] {
            assert_eq!(clamp(value, lower, upper), Err("invalid range".to_owned()));
        }
    }
    for (value, lower, upper, expected) in [(-9,-5,5,-5),(9,-5,5,5),(0,-5,5,0),(-5,-5,5,-5),(5,-5,5,5),(i64::MIN,i64::MIN,i64::MAX,i64::MIN),(i64::MAX,i64::MIN,i64::MAX,i64::MAX)] {
        assert_eq!(clamp(value, lower, upper), Ok(expected));
    }
}
''',
        'oracle_limits': 'Finite endpoint/order cases; universal preservation requires independent source reasoning. No arithmetic-based reference implementation is exported.'},
    'unicode-prefix': {
        'fault_family': 'Unicode scalar count versus UTF-8 byte count',
        'source': '''pub fn prefix(text: &str, count: usize) -> Result<String, String> {
    if count == 0 || count > text.chars().count() { return Err("invalid count".to_owned()); }
    Ok(text.chars().take(count).collect())
}
''',
        'failed': '''pub fn prefix(text: &str, count: usize) -> Result<String, String> {
    if count > text.len() { return Err("invalid count".to_owned()); }
    Ok(text.chars().take(count).collect())
}
''',
        'requirements': {
            'requested_behavior': 'Accept count zero for every UTF-8 string, including the empty string, returning the empty string.',
            'existing_behaviour': 'For positive counts preserve the original Unicode scalar-value behavior: return exactly the first count chars when available, otherwise Err("invalid count"). Count Unicode scalar values, not UTF-8 bytes or grapheme clusters; never panic.'},
        'tests': '''use parser::prefix;
#[test] fn requested_behavior_handled() {
    for text in ["", "abc", "é", "🌱a", "e\\u{301}"] { assert_eq!(prefix(text, 0), Ok(String::new())); }
}
#[test] fn existing_behaviour_unchanged() {
    for (text, count, expected) in [("abc",1,"a"),("abc",3,"abc"),("éx",1,"é"),("🌱a",2,"🌱a"),("e\\u{301}x",2,"e\\u{301}")] {
        assert_eq!(prefix(text,count), Ok(expected.to_owned()));
    }
    for (text, count) in [("",1),("abc",4),("é",2),("🌱",3),("éx",3),("e\\u{301}",3),("a",usize::MAX)] {
        assert_eq!(prefix(text,count), Err("invalid count".to_owned()));
    }
}
''',
        'oracle_limits': 'Finite ASCII, multibyte and combining-scalar examples. No claim about grapheme segmentation; universal behavior needs source review.'}
}

METRICS = {
    'design': 'Two newly authored held-out tasks, followed by the original parser as regression. No task replacement or criterion changes after results.',
    'order': ['singleton-clamp', 'unicode-prefix', 'original-parser-regression'],
    'success': 'Normal controller generates experiment, observes actual failure, generates repair, passes immutable tests and unchanged generated experiment, receives independent final acceptance, and stops with replay producing zero effects.',
    'failure': 'Invalid/inability/unknown/exhausted/semantic rejection/failed verification is a recorded failed attempt, not omitted. No within-task restart or alternate seed selected after outcomes.',
    'assistance': 'Report all manual actions; final independent assessment is permitted and distinguished from unattended intermediate stages. No replacement code or operator-selected model actions.',
    'corrections': 'Existing bounded shared protocol correction only; preserve and count every request, including invalid responses. No unknown-outcome retry.',
    'resources': {'runs': 3, 'reserved_calls_per_run': 14, 'execution_groups_per_run': 4, 'completion_tokens_per_call': 8192, 'seconds_per_call': 240, 'total_calls_envelope': 42, 'total_groups_envelope': 12},
    'denominators': 'Report fresh tasks separately (0/2 through 2/2) and original regression (0/1 through 1/1), plus stage results, calls, corrections, costs and unknown reservations for every attempted run.',
    'nonclaims': ['general reliability', 'causal TSR advantage', 'arbitrary repository autonomy', 'human desire fulfillment'],
    'review_only_exclusions': ['protected test bodies', 'oracle commentary', 'reference patches', 'reviewer conclusions'],
}

def encoded(value): return legacy.encoded(value)
def digest(data): return hashlib.sha256(data).hexdigest()
def suite_manifest():
    return {'version': 'heldout-rust-repair-suite@v1', 'origin': 'new evaluator-authored non-private programs',
            'tasks': {key: digest(encoded(value)) for key, value in TASKS.items()},
            'original_regression': digest(encoded({k:digest(v) for k,v in legacy.artifacts().items()})),
            'metrics': METRICS}
def suite_sha256(): return digest(encoded(suite_manifest()))

class Scenario:
    FIRST_CASE = 'requested_behavior_handled'
    SECOND_CASE = 'existing_behaviour_unchanged'
    FIRST_REQUIREMENT = 'requested_behavior'
    SECOND_REQUIREMENT = 'existing_behaviour'
    def __init__(self, key):
        self.scenario_id = key
        data = TASKS[key]
        self.SOURCE, self.FAILED, self.TESTS = data['source'], data['failed'], data['tests']
        self.REQUIREMENTS = dict(data['requirements'])
    def artifacts(self):
        files = legacy.artifacts()
        scope = json.loads(files['scope.json'])
        scope['goal'] = 'Implement the requested behavior while preserving the second standing requirement. Both remain traceable; a proposal cannot grant authority or replace either.'
        scope['cases'][0]['id'] = self.FIRST_CASE
        plan = {'schema':'heldout-repair-preparation@v1','origin':'evaluator-authored synthetic Rust program',
                'scenario_id':self.scenario_id,'suite_sha256':suite_sha256(),'original_desires':self.REQUIREMENTS,
                'seed_origin':'evaluator-authored attempted change, not generated discovery',
                'execution_authorized':False,'model_calls_authorized':0,
                'oracle_limits':TASKS[self.scenario_id]['oracle_limits'],'metrics':METRICS}
        files.update({'source/src/lib.rs':self.SOURCE.encode(),'source/tests/contract.rs':self.TESTS.encode(),
                      'review-only/failed-candidate.rs':self.FAILED.encode(),'requirements.json':encoded(self.REQUIREMENTS),
                      'scope.json':encoded(scope),'plan.json':encoded(plan)})
        return files
    def manifest(self):
        return {'schema':'heldout-source-manifest@v1','scenario_id':self.scenario_id,'suite_sha256':suite_sha256(),
                'files':{k:digest(v) for k,v in self.artifacts().items()},'prd_sha256':digest((ROOT/'docs/PRD.md').read_bytes())}
    def prepare(self,directory):
        directory=Path(directory); directory.mkdir(parents=True,exist_ok=False)
        for name,data in self.artifacts().items():
            path=directory/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)
        (directory/'manifest.json').write_bytes(encoded(self.manifest()))
        return self.validate(directory)
    def validate(self,directory):
        directory=Path(directory); expected=self.artifacts()
        if json.loads((directory/'manifest.json').read_text())!=self.manifest():raise ValueError('heldout manifest drift')
        paths=list(directory.rglob('*'))
        if any(p.is_symlink() for p in paths):raise ValueError('heldout symlink forbidden')
        if {p.relative_to(directory).as_posix() for p in paths if p.is_file()} != set(expected)|{'manifest.json'}:raise ValueError('heldout file set drift')
        for name,data in expected.items():
            if (directory/name).read_bytes()!=data:raise ValueError('heldout bytes drift: '+name)
        return {'status':'prepared_not_executed','scenario_sha256':digest(encoded(self.manifest())), 'files':len(expected),'calls':0,'actions':0,'execution_authorized':False,'b91_passed':False}

def select(scenario_id):
    if scenario_id not in TASKS:raise ValueError('unregistered heldout scenario')
    return Scenario(scenario_id)
