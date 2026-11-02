#!/usr/bin/env python3
"""Freeze a synthetic B9.1 coding scenario offline. No engine run or authority grant."""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
IMAGE = 'sha256:cff6dcceb8b9d203a818b7ff104b89a6aaa353f8576850ca4f58903539417945'
SOURCE = '''pub fn parse(input: &str) -> Result<Vec<i64>, String> {
    input.split(',').map(|field| field.parse::<i64>().map_err(|_| "invalid".to_owned())).collect()
}
'''
FAILED = '''pub fn parse(input: &str) -> Result<Vec<i64>, String> {
    input.split(',').filter(|field| !field.is_empty()).map(|field| field.parse::<i64>().map_err(|_| "invalid".to_owned())).collect()
}
'''
TESTS = '''use parser::parse;
#[test] fn empty_input_handled() { assert_eq!(parse(""), Ok(vec![])); }
#[test] fn existing_behaviour_unchanged() {
    assert_eq!(parse("1,-2,03"), Ok(vec![1,-2,3]));
    for input in ["1,,2", ",1", "1,", " ", "1, 2", "x", "9223372036854775808"] {
        assert!(parse(input).is_err(), "{input:?}");
    }
}
'''
REQUIREMENTS = {
    'empty_input': 'Handle the exactly empty input as a successful empty list without panic.',
    'existing_behaviour': 'For every nonempty input preserve the original result: comma-separated signed i64 fields, no trimming or dropped fields; malformed, empty fields and overflow remain errors.'}

def encoded(value):
    return (json.dumps(value, sort_keys=True, indent=2) + '\n').encode()

def digest(data):
    return hashlib.sha256(data).hexdigest()

def artifacts():
    scope = {'goal': 'Handle empty input while preserving all existing nonempty parsing behaviour. Both aims remain standing desires; experiments are not permission to discard either.',
             'image': IMAGE, 'user': '501:20', 'cargo': True,
             'full_workspace_oracle': False, 'editable': {'src/lib.rs': ''},
             'protected': {'Cargo.toml': '', 'Cargo.lock': '', 'tests/contract.rs': ''},
             'cases': [{'id': 'empty_input_handled', 'args': [], 'expected_stdout': ''},
                       {'id': 'existing_behaviour_unchanged', 'args': [], 'expected_stdout': ''}], 'timeout_ms': 20000}
    plan = {'schema': 'b91-preparation@v1', 'origin': 'evaluator-authored synthetic coding scenario',
            'original_desires': REQUIREMENTS, 'execution_authorized': False,
            'model_calls_authorized': 0, 'actions_authorized': 0,
            'seed': {'origin': 'evaluator-authored known-flawed strategy, not generated discovery',
                     'path': 'review-only/failed-candidate.rs', 'forecast': {'event': 'both_criteria_pass', 'p': 0.1},
                     'status': 'predicted only; no execution receipt or observed failure',
                     'forecast_role': 'scenario diagnostic, not task_accepted_in_budget calibration evidence'},
            'contrast': {'dimensions': ['empty_input', 'existing_behaviour'],
                         'rule': 'Record met/unmet/unknown with independently bound evidence per dimension; never replace with a magnitude.'},
            'conflict': 'The seed drops empty fields, helping the first aim while violating the second. This is strategy-level conflict, not proof that the desires are incompatible.',
            'acceptance': ['Freeze forecast before actual seed execution; independently observe common-event failure. Expected raw Brier is 0.01 only if that failure is observed.',
                           'Both original preferences remain open and traceable after the failed attempt; an individual passing example does not discharge a standing desire.',
                           'Generate and select a bounded experiment for existing behaviour before proposing the subsequent boundary patch; execute that experiment and use its actual return.',
                           'A generated multi-parent aspiration may develop the relationship, but cannot alter criteria, authority or declare parents fulfilled. No expected aspiration phrase is supplied.',
                           'The patch must respond to the boundary experiment, not repeat the failed filtering strategy. Protected tests plus independent semantic review are required.',
                           'Persist observed returns, separate Surprise/Contrast, guidance lineage and next proposal across restart. Unknown/failed execution cannot stand in for a receipt.'],
            'post_return_feedback': 'Only after authorized execution: admit the exact attempted change and independently measured failing requirement/receipt as untrusted evidence, not the seed diagnosis or a reference correction.',
            'scope_role': 'Project CLI source-import template; empty file values are populated by project init --source, not executable empty source.',
            'generator_excludes': ['review-only/', 'protected test bodies', 'reviewer verdicts', 'reference repair (none supplied)'],
            'admission_required': ['Exact source/state hash, expiry, model/provider and data scope', 'Bounded independent supplemental-test authoring/review and execution authority', 'Call/token/wall/tool/patch budgets; no automatic renewal'],
            'readiness': 'scenario frozen only; project relational-guidance and prerequisite-experiment bridge not yet integrated'}
    return {'source/Cargo.toml': b'[package]\nname = "parser"\nversion = "0.0.0"\nedition = "2021"\n',
            'source/Cargo.lock': b'version = 3\n\n[[package]]\nname = "parser"\nversion = "0.0.0"\n',
            'source/src/lib.rs': SOURCE.encode(), 'source/tests/contract.rs': TESTS.encode(),
            'review-only/failed-candidate.rs': FAILED.encode(),
            'requirements.json': encoded(REQUIREMENTS), 'scope.json': encoded(scope), 'plan.json': encoded(plan)}

def prepare(directory):
    directory = Path(directory)
    directory.mkdir(parents=True, exist_ok=False)
    files = artifacts()
    for name, data in files.items():
        path = directory / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)
    manifest = {'schema': 'b91-source-manifest@v1', 'files': {name: digest(data) for name, data in files.items()},
                'prd_sha256': digest((ROOT/'docs/PRD.md').read_bytes())}
    (directory/'manifest.json').write_bytes(encoded(manifest))
    return validate(directory)

def validate(directory):
    directory = Path(directory)
    manifest = json.loads((directory/'manifest.json').read_text())
    expected = {name: digest(data) for name, data in artifacts().items()}
    if manifest != {'schema': 'b91-source-manifest@v1', 'files': expected,
                    'prd_sha256': digest((ROOT/'docs/PRD.md').read_bytes())}:
        raise ValueError('manifest or source contract drift')
    paths = list(directory.rglob('*'))
    if any(p.is_symlink() for p in paths):
        raise ValueError('symlink is not admitted')
    if {p.relative_to(directory).as_posix() for p in paths if p.is_file()} != set(expected) | {'manifest.json'}:
        raise ValueError('unexpected or missing scenario files')
    for name, wanted in expected.items():
        if digest((directory/name).read_bytes()) != wanted:
            raise ValueError('scenario bytes changed: '+name)
    return {'status': 'prepared_not_executed', 'scenario_sha256': digest(encoded(manifest)),
            'files': len(expected), 'calls': 0, 'actions': 0,
            'execution_authorized': False, 'b91_passed': False}

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode', choices=['prepare', 'validate'])
    parser.add_argument('--directory', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps((prepare if args.mode == 'prepare' else validate)(args.directory), indent=2))
