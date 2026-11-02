#!/usr/bin/env python3
"""Prepare local prospective state only. Never dispatch, inspect, or amend a campaign."""
import argparse
import datetime
import hashlib
import importlib.util
import json
from pathlib import Path

PACKET = Path(__file__).resolve().parent
ROOT = PACKET.parents[1]

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--campaign-manifest', type=Path, required=True)
    parser.add_argument('--packet-sha256', required=True)
    args = parser.parse_args()
    manifest_path = PACKET / 'manifest.json'
    if sha(manifest_path) != args.packet_sha256:
        raise ValueError('reviewed packet identity mismatch')
    manifest = json.loads(manifest_path.read_text())
    for base, records in ((PACKET, manifest['files']), (ROOT, manifest['implementation'])):
        for name, expected in records.items():
            path = base / name
            if path.is_symlink() or not path.resolve().is_relative_to(base) or sha(path) != expected:
                raise ValueError('reviewed packet/source drift: ' + name)
    output = args.output.absolute()
    if not output.resolve().is_relative_to(ROOT / 'target') or any(p.is_symlink() for p in (output, *output.parents)):
        raise ValueError('fresh repository target directory required')
    output.mkdir(parents=True, exist_ok=False)
    spec = importlib.util.spec_from_file_location('objective_prepare_adapter', ROOT / 'scripts/engine-objective-adapter.py')
    adapter = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(adapter)
    public = adapter.e.strict((PACKET / 'public.json').read_bytes())
    project = adapter.e.strict((PACKET / 'project-spec.json').read_bytes())
    campaign = adapter.e.strict(args.campaign_manifest.read_bytes())
    conditions = adapter.e.strict((PACKET / 'conditions.json').read_bytes())['rounds']
    source = output / 'source'
    source.mkdir()
    for group in ('editable', 'protected'):
        for name, text in project[group].items():
            path = source / name
            if not path.resolve().is_relative_to(source):
                raise ValueError('packet source path escape')
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
    def save(name, value):
        (output / name).write_text(json.dumps(value, sort_keys=True, indent=2) + '\n')
    save('project-spec.json', project)
    requirements = {p['id']: p['meaning'] for p in public['parents']}
    save('requirements.json', requirements)
    state = output / 'state'
    state.mkdir()
    controller = adapter.d.Native(state)
    controller.cli('project', 'init', '--spec', str(output / 'project-spec.json'), '--source', str(source), '--requirements', str(output / 'requirements.json'))
    controller.json_cli(('project-lifecycle-open',), {
        'state_root': controller.request()['state_root'], 'training': None,
        'tool_ms_ceiling': 300000, 'reason': 'Prospective finite objective preparation; external launch authority pending',
        'non_tool': {'model_calls': 24, 'model_tokens': 983040, 'wall_ms': 7200000,
                     'patch_bytes': 4096, 'external_cost_microusd': 0}}, '--admission')
    controller.json_cli(('project', 'acceptance', 'feedback'), {
        'requirements_revision': controller.cli('project', 'acceptance', 'status')['requirements_revision'],
        'decision_record': None, 'guidance': requirements})
    controller.json_cli(('project', 'objective-configure'), {
        'state_root': controller.request()['state_root'],
        'conditions': [{'id': x['id'], 'input': x['body']} for x in conditions],
        'service_entry': 'service.rs', 'max_output_bytes': 2048, 'timeout_ms': 20000})
    controller.cli('project', 'resource-unpriced-admit', '--state-root', controller.request()['state_root'])
    expiry = (datetime.datetime.now(datetime.timezone.utc) + datetime.timedelta(minutes=120)).isoformat()
    origin = adapter.freeze_origin(controller, public, args.packet_sha256, checker_profile='guide-opportunity@v1')
    save('admission.json', adapter.admission(output / 'adapter', origin, campaign, expiry))
    save('cycle-admission.json', {'version': 'local-project-cycle@v5', 'journal': str(output / 'cycle'),
         'scope': origin['scope'], 'max_decisions': 9, 'max_executions': 5, 'max_service_rounds': 3,
         'expires_at': expiry, 'implementation_sha256': sha(ROOT / 'scripts/engine-project-cycle.py')})
    save('PREPARED_ONLY.json', {'version': 'objective-preparation@v1', 'packet_sha256': args.packet_sha256,
         'admission_sha256': adapter.e.sha(json.loads((output / 'admission.json').read_text())),
         'external_launch_authorized': False, 'provider_calls': 0, 'target_operations': 0,
         'required': 'Fresh reviewed campaign extension, implementation upgrade, price check and explicit user batch authority before run.'})
    print(json.dumps({'prepared': str(output), 'provider_calls': 0, 'target_operations': 0, 'launch_authorized': False}))

if __name__ == '__main__':
    main()
