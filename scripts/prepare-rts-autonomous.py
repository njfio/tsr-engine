#!/usr/bin/env python3
"""Prepare one fresh RTS mutant and bounded live-run argv; never call a provider."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=False)
    replay = ROOT / 'target/engine-rts-real-2'
    initial = json.loads((replay / 'init.json').read_text())
    original_scope = replay / 'scope.json'
    source = replay / 'source'  # Original admitted mutant, never the repaired coding-work.
    for path, expected in initial['protected_sha256'].items():
        assert hashlib.sha256((source / path).read_bytes()).hexdigest() == expected
    implementation = source / 'crates/rts-core/src/lib.rs'
    assert 'vec![] /* ~ changed by cargo-mutants ~ */' in implementation.read_text()
    # Bounded interface context, extracted from the mutant; no reference answer.
    text = implementation.read_text()
    start = text.index('pub struct LanguageInfo {')
    declaration = text[start:text.index('\n}', start)+2]
    signature = next(line for line in text.splitlines() if line.startswith('pub fn supported_languages()'))
    spec = json.loads(original_scope.read_text())
    spec['goal'] += '\nInterface context from admitted mutant:\n' + declaration + '\n' + signature
    assert len(spec['goal'].encode()) <= 512
    scope = out / 'scope.json'
    scope.write_text(json.dumps(spec, indent=2)+'\n')
    engine = out / 'engine'
    def cli(*argv):
        result = subprocess.run([str(ROOT / 'target/debug/tsr'), 'engine', '--directory', str(engine), 'project', *argv], capture_output=True, text=True, timeout=180, check=True)
        return json.loads(result.stdout)
    cli('init', '--spec', str(scope), '--source', str(source))
    prepared = cli('prepare')
    status = cli('status')
    request = cli('request')
    assert prepared['ready'] and status['status'] == 'ready_for_proposal'
    assert status['completed'] == 0 and status['protected_files_intact']
    assert not request['coding']['recent_observations']
    assert len(request['coding']['files']) == 1
    assert '#[cfg(test)]' not in request['coding']['files'][0]['source']
    assert len(json.dumps(request).encode()) <= 32768
    ledger = ROOT / 'target/m3-provider-cost.jsonl'
    loader = importlib.util.spec_from_file_location('rts_readiness_transport', ROOT / 'scripts/engine-run.py')
    transport = importlib.util.module_from_spec(loader)
    loader.loader.exec_module(transport)
    transport.require_ready(request)
    body = transport.codec.envelope_request(request)['body']
    _, reservation = transport.broker.sized_cost_reservation(body)
    used, uncertain = transport.ledger_status(ledger)
    assert not uncertain, 'unsettled aggregate reservation requires reconciliation'
    assert used + reservation <= transport.broker.CAP
    assert reservation <= transport.broker.D('0.25')
    argv = [str(ROOT / 'target/debug/tsr'), 'engine', '--directory', str(engine),
            'project', 'run', '--', '--approved-live', '--ledger', str(ledger),
            '--max-steps', '2', '--max-usd', '0.25', '--max-tokens', '100000',
            '--call-timeout-seconds', '55']
    config = dict(task=initial['task'], engine_directory=str(engine),
                  provider='https://api.cheaperinference.com/v1/', model='deepseek-v4.1-flash',
                  max_steps=2, max_usd='0.25', aggregate_cap_usd='100', max_tokens=100000,
                  call_timeout_seconds=55, argv=argv, execution_authorized_by_this_file=False,
                  source_scope='One admitted implementation window, bounded interface excerpt, task goal, named-test identity, controller state and bounded test feedback; no full protected files or reference correction',
                  stopping_scope='Named library test passes; full-suite closeout still required for a new model repair')
    (out / 'run-config.json').write_text(json.dumps(config, indent=2)+'\n')
    evidence = dict(task=initial['task'], status=status, offline_preparation=prepared,
                    request_bytes=len(json.dumps(request).encode()), visible_files=1,
                    source_sha256=hashlib.sha256(implementation.read_bytes()).hexdigest(),
                    no_reference_correction_or_receipts_imported=True, paid_calls=0,
                    source_transmitted=False, credentials_read=False,
                    aggregate_ledger_present=ledger.is_file(), aggregate_usd_or_reserved=str(used),
                    uncertain_reservations=len(uncertain), first_call_reservation_usd=str(reservation),
                    provider_credentials_and_current_prices_not_checked=True, config=str(out/'run-config.json'))
    (out / 'readiness.json').write_text(json.dumps(evidence, indent=2)+'\n')
    print(json.dumps(evidence, indent=2))

if __name__ == '__main__':
    main()
