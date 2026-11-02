#!/usr/bin/env python3
"""Offline integration check using a genuine aspiration and explicit fixture stages.

No provider calls. The fixture possibilities/forecasts/reaffirmation are NOT model
discovery or semantic success. Only a fresh copy's reversible notebook is changed.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil
import sqlite3
import subprocess

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('directions', ROOT / 'scripts/engine-directions.py')
d = importlib.util.module_from_spec(spec)
spec.loader.exec_module(d)


def check(evidence, destination):
    evidence = evidence.resolve(strict=True)
    source = evidence / '1-A-1-synthetic-import'
    original = json.loads((evidence / '1-A-1-typed-offer.json').read_text())['proposals'][0]
    provenance = json.loads((evidence / '1-A-1-import-provenance.json').read_text())
    receipt = json.loads((evidence / '1-A-1-receipt.json').read_text())
    assert hashlib.sha256((evidence / '1-A-1-typed-offer.json').read_bytes()).hexdigest() == provenance['typed_sha256']
    assert receipt['body_sha256'] == provenance['original_body_sha256']
    assert json.loads((source / 'SYNTHETIC-NOT-OPERATIONAL.json').read_text())['synthetic'] is True
    destination.mkdir(parents=True, exist_ok=False)  # Never overwrite or retry a used run.
    run = destination / 'run'
    run.mkdir()

    def save(name, value):
        (destination / name).write_text(json.dumps(value, indent=2) + '\n')

    def cli(directory, *args):
        result = subprocess.run([str(ROOT / 'target/debug/tsr'), 'engine', '--directory', str(directory), *args],
                                capture_output=True, text=True, timeout=45)
        if result.returncode:
            raise RuntimeError(result.stderr)
        return json.loads(result.stdout)

    before = cli(source, 'direction-request')
    # SQLite backup gives a consistent closed-source snapshot; root equality below
    # catches any intervening change. The original fixture is never mutated.
    with sqlite3.connect(f'file:{source / "events.sqlite"}?mode=ro', uri=True) as src:
        with sqlite3.connect(run / 'events.sqlite') as dst:
            src.backup(dst)
    shutil.copytree(source / 'cas', run / 'cas')
    state = cli(run, 'direction-request')
    assert state == before
    guidance = state['relational_guidance'][0]
    for key in ('relationship', 'aspiration', 'discriminator'):
        assert guidance['proposal'][key] == original[key]
    assert guidance['proposal']['parents'] == sorted(provenance['parent_mapping'][p] for p in original['parents'])
    save('provenance.json', dict(provenance, source_root=state['state_root'],
        binary_sha256=hashlib.sha256((ROOT / 'target/debug/tsr').read_bytes()).hexdigest(),
        script_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        source_revision=subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=ROOT, capture_output=True, text=True, check=True).stdout.strip(),
        implementation_sha256={str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest() for p in
            (ROOT / 'crates/tsr-cli/src/engine/deliberation/relational.rs', ROOT / 'scripts/engine-directions.py')},
        source_receipt_sha256=hashlib.sha256((evidence / '1-A-1-receipt.json').read_bytes()).hexdigest(),
        stage_origin='fixture-authored possibilities, exact predicate forecasts and unchanged reaffirmation',
        semantic_evaluation='unknown', inference_calls=0))
    save('before.json', state)
    context = d.possibility_context(d.identified_context(state, dict(state)))
    assert context['relational_guidance'] == state['relational_guidance']
    save('possibility-context.json', context)
    # This frozen pair tests wiring, not a hidden reference answer for any generator.
    notes = [n['id'] for n in state['available_notes']]
    assert len(notes) == 2 and set(notes) == set(state['notebook']['known'])
    proposal = {'state_root': state['state_root'], 'directions': [
        {'purpose': 'FIXTURE inclusion of both fictional notes; no interpersonal claim',
         'action': {'operation': 'compose', 'title': 'Fixture evidence account', 'claims': notes}},
        {'purpose': 'FIXTURE inclusion of one fictional note; no interpersonal claim',
         'action': {'operation': 'compose', 'title': 'Fixture evidence account', 'claims': notes[:1]}}]}
    d.validate_possibilities(state, proposal, {'prior_rejected_offers': []})
    save('fixture-possibilities.json', proposal)
    rows = []
    for node in proposal['directions']:
        values = {}
        for identity, want in zip(state['obligation_ids'], state['standing_desires']):
            assert want['aim']['predicate'] == 'account'
            relevant = [n['id'] for n in state['available_notes'] if n['topic'] == want['aim']['topic']]
            values[identity] = sum(n in node['action']['claims'] for n in relevant) / len(relevant)
        rows.append({'proposal_id': d.proposal_digest(node), 'forecasts': values,
                     'supports': [identity for identity, value in values.items() if value > 0]})
    assessment = {'state_root': state['state_root'], 'proposal_digest': d.proposal_digest(proposal), 'assessments': rows}
    save('fixture-assessment.json', assessment)
    offer = d.normalize_forecasts(state, d.complete_assessment(state, proposal, assessment))
    save('offer.json', offer)
    result = cli(run, 'deliberate', '--response', str(destination / 'offer.json'))
    save('actual-controller-return.json', result)
    current = cli(run, 'direction-request')
    save('after-action.json', current)
    assert current['cycles'] == state['cycles'] + 1
    assert current['status'] == 'satisfied'  # Mechanical coverage only.
    assert current['relational_request']['due'] is True
    assert current['relational_request']['review_only'] is True
    assert current['relational_request']['existing'][0]['discriminator_tested'] == 'unknown'
    assert not current['relational_guidance']
    rq = current['relational_request']
    reaffirmation = {key: rq[key] for key in ('state_root', 'return_commitment', 'observation_revision', 'evidence_key')}
    reaffirmation['proposals'] = [dict(guidance['proposal'], revises=guidance['revision'])]
    save('fixture-reaffirmation.json', reaffirmation)
    cli(run, 'direction-relate', '--response', str(destination / 'fixture-reaffirmation.json'))
    after = cli(run, 'direction-request')
    save('after-review.json', after)
    assert after['relational_request']['due'] is False
    assert after['relational_guidance'][0]['identity'] == guidance['identity']
    assert after['relational_guidance'][0]['revision'] != guidance['revision']
    for key in ('standing_desires', 'obligation_ids', 'authority', 'semantic_status'):
        assert state[key] == after[key]
    for key in ('cycles', 'observation', 'notebook'):
        assert current[key] == after[key]
    assert cli(source, 'direction-request') == before
    summary = {'integration': 'passed', 'model_calls': 0, 'notebook_actions': 1,
               'semantic_discriminator': 'unknown', 'behavioral_influence': 'not tested',
               'fixture_stages': ['possibility', 'assessment', 'reaffirmation'],
               'original_fixture_unchanged': True, 'parents_and_authority_preserved': True}
    save('summary.json', summary)
    return summary


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--evidence', type=Path, required=True)
    parser.add_argument('--destination', type=Path, required=True)
    args = parser.parse_args()
    print(json.dumps(check(args.evidence, args.destination.resolve()), indent=2))
