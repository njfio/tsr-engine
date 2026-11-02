#!/usr/bin/env python3
"""Bounded model-backed engine loop. Local test mode never loads credentials.

Uses the existing broker's price check, reservations, accounting and timeout.
The typed engine boundary alone decides whether a proposal can execute.
"""
import argparse
import fcntl
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import urllib.parse
import uuid

ROOT = Path(__file__).resolve().parents[1]


def module(name, file):
    spec = importlib.util.spec_from_file_location(name, ROOT / 'scripts' / file)
    value = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(value)
    return value


broker = module('engine_budgeted_broker', 'm3-cheaper-broker.py')
codec = module('engine_codec', 'engine-provider-envelope.py')
recovery = module('engine_response_recovery', 'engine-response-recovery.py')


class Stop(ValueError):
    pass


def cli(directory, *args, timeout=180):
    try:
        result = subprocess.run([str(ROOT / 'target/debug/tsr'), 'engine', '--directory',
                                 str(directory), *args], capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired:
        raise Stop('controller_timeout_reconcile_required') from None
    if result.returncode or len(result.stdout) > 1024 * 1024:
        raise Stop('engine_command_refused')
    return codec.strict_json(result.stdout)


def execution_timeout(plan):
    """Let the sealed executor enforce each deadline before timing out its controller."""
    duration = plan.get('timeout_ms')
    commands = plan.get('commands')
    if (type(duration) is not int or not 1 <= duration <= 600000
            or not isinstance(commands, list) or not 1 <= len(commands) <= 16):
        raise Stop('invalid_execution_deadline')
    return (duration * len(commands) + 999) // 1000 + 30


def require_ready(request):
    coding = request.get('coding', {})
    if not coding.get('generated_edits'):
        raise Stop('initialize_generated_scope_first')
    if coding.get('kind') == 'cargo_project@v1' and coding.get('dependencies_ready') is not True:
        raise Stop('prepare_project_dependencies_before_transport')


def completion_result(directory, request, result):
    """Test observations cannot authorize semantic goal completion."""
    if request.get('coding', {}).get('kind') in ('cargo_project@v1', 'rust_project@v1'):
        status = cli(directory, 'project', 'status')
        completion = status['goal_completion']
        return {**result(status['status']),
                'execution_status': status['execution_status'], 'goal_completion': completion,
                'acceptance_cycle': status.get('acceptance_cycle')}
    return result('success')


def ledger_status(path):
    with path.open('r', encoding='utf8') as log:
        try:
            fcntl.flock(log, fcntl.LOCK_SH | fcntl.LOCK_NB)
        except BlockingIOError:
            raise Stop('ledger_busy') from None
        lines = list(log)
    used = broker.account(lines)
    events = [json.loads(line) for line in lines]
    uncertain = {e['id'] for e in events if e['kind'] == 'reserve'} - {e['id'] for e in events if e['kind'] == 'settled'}
    return used, uncertain


def private_file(path, flags):
    fd = os.open(path, flags | os.O_NOFOLLOW, 0o600)
    try:
        parent = os.open(path.parent, os.O_RDONLY)
        try:
            os.fsync(parent)
        finally:
            os.close(parent)
        return os.fdopen(fd, 'a+', encoding='utf8')
    except Exception:
        os.close(fd)
        raise


def run(args):
    directory = args.directory.resolve(strict=True)
    request = cli(directory, 'request')
    require_ready(request)
    if cli(directory, 'project-budget').get('active') is True:
        raise Stop('active_project_budget_requires_accounted_local_transport; use engine-local-model.py and project run/resume')
    scope_request = request
    local = args.local_provider is not None
    ledger_arg = args.fixture_ledger if local else args.ledger
    if ledger_arg is None or (local and args.ledger) or (not local and args.fixture_ledger):
        raise Stop('explicit_mode_ledger_required')
    ledger = ledger_arg.absolute()
    if ledger.is_symlink():
        raise Stop('ledger_symlink')
    ledger = ledger.resolve()
    if ledger.is_relative_to(directory / 'coding-work'):
        raise Stop('ledger_inside_candidate_workspace')
    endpoint = args.local_provider if local else broker.HOST
    if local:
        parsed = urllib.parse.urlsplit(endpoint)
        if (parsed.scheme != 'http' or parsed.hostname != '127.0.0.1' or not parsed.port
                or parsed.username or parsed.password or parsed.query or parsed.fragment
                or parsed.path != '/v1/'):
            raise Stop('local_provider_must_be_literal_loopback_v1')
        marker = ledger.with_name(ledger.name + '.fixture')
        if ledger.exists() and (marker.is_symlink() or not marker.is_file() or marker.stat().st_size > 64 or marker.read_text() != 'engine-local-accounting@v1\n'):
            raise Stop('existing_ledger_is_not_a_local_fixture')
        if not ledger.exists():
            ledger.parent.mkdir(parents=True, exist_ok=True)
            with private_file(ledger, os.O_RDWR | os.O_CREAT | os.O_EXCL):
                pass
            with private_file(marker, os.O_RDWR | os.O_CREAT | os.O_EXCL) as file:
                file.write('engine-local-accounting@v1\n'); file.flush(); os.fsync(file.fileno())
    elif (not args.approved_live or ledger != (ROOT / 'target/m3-provider-cost.jsonl').resolve()
          or not ledger.is_file() or ledger.with_name(ledger.name + '.fixture').exists()):
        raise Stop('live_requires_approval_and_existing_aggregate_ledger')
    limit = broker.D(args.max_usd)
    if not limit.is_finite() or not 0 < limit <= broker.CAP:
        raise Stop('invalid_run_cost_limit')
    if not 1 <= args.max_steps <= 64 or not 1 <= args.call_timeout_seconds <= 55 or args.max_tokens < 1 or not 0 <= args.steps_this_invocation <= 64:
        raise Stop('invalid_run_bounds')
    config = {'directory': str(directory), 'ledger': str(ledger), 'endpoint': endpoint,
              'mode': 'local_fixture' if local else 'live', 'model': broker.MODEL,
              'max_steps': args.max_steps, 'max_usd': str(limit), 'max_tokens': args.max_tokens,
              'call_timeout_seconds': args.call_timeout_seconds}
    journal_path = directory / 'engine-transport.jsonl'
    with private_file(journal_path, os.O_RDWR | os.O_CREAT | os.O_APPEND) as journal:
        try:
            fcntl.flock(journal, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise Stop('another_engine_runner_is_active') from None
        journal.seek(0)
        raw = journal.read(2 * 1024 * 1024 + 1)
        if len(raw.encode()) > 2 * 1024 * 1024:
            raise Stop('transport_journal_limit')
        events = [codec.strict_json(line) for line in raw.splitlines()]

        def record(kind, **fields):
            event = {'kind': kind, **fields}
            journal.seek(0, 2)
            journal.write(json.dumps(event, ensure_ascii=False, separators=(',', ':')) + '\n')
            journal.flush(); os.fsync(journal.fileno())
            events.append(event)
            return event

        def result(status):
            used, uncertain = ledger_status(ledger)
            return {'status': status, 'mode': config['mode'], 'steps': sum(e['kind'] == 'started' for e in events),
                    'aggregate_usd_or_reserved': str(used), 'run_usd_or_reserved': str(used - baseline),
                    'uncertain_reservations': len(uncertain), 'directory': str(directory),
                    'journal': str(journal_path), 'provider': endpoint, 'model': broker.MODEL}

        used, uncertain = ledger_status(ledger)
        if not events:
            if request['pending'] is not None:
                raise Stop('existing_commitment_requires_manual_completion')
            record('config', config=config, baseline=str(used))
        elif events[0].get('kind') != 'config' or events[0]['config'] != config:
            raise Stop('restart_configuration_mismatch')
        baseline = broker.D(events[0]['baseline'])
        if uncertain:
            return result('uncertain_charge_stop')
        recover_id = getattr(args, 'recover_settled_decode', None)
        prior_recovery = next((e for e in events if e['kind'] == 'recovery'), None)
        if recover_id and prior_recovery is None:
            ledger_events = [codec.strict_json(l) for l in ledger.read_text().splitlines()]
            diagnostics = [codec.strict_json(l) for l in ledger.with_name(ledger.stem + '-diagnostics.jsonl').read_text().splitlines()]
            try:
                details = recovery.validate_recovery(events, ledger_events, diagnostics, recover_id, request, args.max_steps)
            except ValueError as error:
                raise Stop(str(error)) from None
            settled = next(e for e in ledger_events if e['kind'] == 'settled' and e['id'] == recover_id)
            if ledger_events[-1]['id'] != recover_id or used - baseline != broker.D(settled['usd']):
                raise Stop('recovery_receipt_not_bound_to_run_cost')
            prior_recovery = record('recovery', **details)
        elif recover_id and prior_recovery['reservation_id'] != recover_id:
            raise Stop('recovery_receipt_mismatch')
        recovered_step = prior_recovery['failed_step_id'] if prior_recovery else None
        if any(e['kind'] == 'failed' and e['step_id'] != recovered_step for e in events):
            return result('previous_failure_stop')
        steps_this_invocation = 0
        while True:
            starts = [e for e in events if e['kind'] == 'started']
            active = starts[-1] if starts and starts[-1]['step_id'] != recovered_step else None
            phase = [e for e in events if active and e.get('step_id') == active['step_id']]
            completed = next((e for e in phase if e['kind'] == 'completed'), None)
            if completed and completed['verdict'] == 'passed':
                terminal = completion_result(directory, scope_request, result)
                cycle = terminal.get('acceptance_cycle') or {}
                if not cycle.get('repair_ready'):
                    return terminal
                # An independently rejected candidate may be repaired within the SAME budgets.
                # Guidance is separately published by the operator; raw review evidence stays private.
                active = completed = None
            if completed and completed['verdict'] != 'failed_tests':
                return result('censored_execution_stop')
            if active and not completed:
                decoded = next((e for e in phase if e['kind'] == 'decoded'), None)
                committed = next((e for e in phase if e['kind'] == 'committed'), None)
                executing = next((e for e in phase if e['kind'] == 'execution_started'), None)
                if decoded is None:
                    # Even if billing settled, losing the response must never trigger a duplicate call.
                    return result('proposal_recovery_required')
                current = cli(directory, 'request')
                if executing:
                    receipts = current['coding']['recent_observations']
                    receipt = next((r for r in receipts if r['commitment'] == executing['commitment']), None)
                    if receipt is None:
                        return result('execution_recovery_required')
                    record('completed', step_id=active['step_id'], verdict=receipt['verdict'])
                    continue
                try:
                    if committed is None:
                        if current['state_root'] != decoded['proposal']['state_root']:
                            # Do not guess whether an unrelated process committed this proposal.
                            return result('commit_recovery_required')
                        with tempfile.NamedTemporaryFile(mode='w', dir=directory, suffix='.proposal.json', encoding='utf8') as file:
                            json.dump(decoded['proposal'], file, ensure_ascii=False, separators=(',', ':'))
                            file.flush()
                            applied = cli(directory, 'propose', '--response', file.name, '--apply')
                        if not applied['committed']:
                            record('failed', step_id=active['step_id'], reason='no_eligible_proposal')
                            return result('no_eligible_proposal')
                        pending = applied['state']['pending']
                        committed = record('committed', step_id=active['step_id'], commitment=pending['commitment'])
                    plan = cli(directory, 'coding-plan')
                    if plan['commitment'] != committed['commitment']:
                        raise Stop('commitment_changed')
                    record('execution_started', step_id=active['step_id'], commitment=plan['commitment'], source=plan['source_before'])
                    outcome = cli(directory, 'coding-run', '--commitment', plan['commitment'], '--expected-source', plan['source_before'], timeout=execution_timeout(plan))
                    record('completed', step_id=active['step_id'], verdict=outcome['verdict'])
                    steps_this_invocation += 1
                    if outcome['verdict'] not in ('passed', 'failed_tests'):
                        record('failed', step_id=active['step_id'], reason='censored_execution')
                        return result('censored_execution_stop')
                except Stop as error:
                    record('failed', step_id=active['step_id'], reason=str(error))
                    return result(str(error))
                continue
            if len(starts) >= args.max_steps:
                return result('step_budget_stop')
            if args.steps_this_invocation and steps_this_invocation >= args.steps_this_invocation:
                return result('paused')
            used, uncertain = ledger_status(ledger)
            if uncertain:
                return result('uncertain_charge_stop')
            ceiling = min(broker.CAP, baseline + limit)
            billed = sum(e['billed_tokens'] for e in events if e['kind'] in ('decoded', 'recovery'))
            remaining = args.max_tokens - billed
            current = cli(directory, 'request')
            if current['pending']:
                return result('unexpected_pending_commitment')
            require_ready(current)
            def encode_current(value):
                body = codec.envelope_request(value)['body']
                if prior_recovery:
                    body['max_tokens'] = prior_recovery['completion_tokens']
                return body
            body = encode_current(current)
            _, reservation = broker.sized_cost_reservation(body)
            if used + reservation > ceiling:
                return result('cost_budget_stop')
            if len(json.dumps(body).encode()) + 8192 + broker.MAX_OUTPUT > remaining:
                return result('token_budget_stop')
            step_id = str(uuid.uuid4())
            record('started', step_id=step_id, state_root=current['state_root'])
            request = {'engine_request': current, 'remaining_tokens': remaining,
                       'deadline_ms': args.call_timeout_seconds * 1000}

            def decode(response, transport_request):
                tokens = response.get('usage', {}).get('total_tokens')
                if isinstance(tokens, bool) or not isinstance(tokens, int) or tokens < 0 or tokens > remaining:
                    raise broker.BrokerFailure('usage_missing_or_outside_budget')
                try:
                    proposal = codec.decode_response(transport_request['engine_request'], response)
                except (ValueError, KeyError, TypeError, AttributeError) as error:
                    raise broker.BrokerFailure('output_truncated' if str(error) == 'output_truncated' else 'response_decode_invalid') from None
                return {'proposal': proposal, 'billed_tokens': tokens}

            options = {'encode_fn': lambda r: encode_current(r['engine_request']),
                       'response_observer': lambda response, key: recovery.save_response(directory / (step_id + '.response-private.json'), response, key),
                       'decode_fn': decode, 'ceiling': ceiling, 'refuse_uncertain': True, 'ledger_nonblocking': True, 'sized_reservation': True}
            if local:
                options.update(credential_provider=lambda: None,
                    transport=lambda route, key, data=None, timeout=30: broker.request_json(route, key, data, timeout, endpoint=endpoint))
            try:
                response = broker.complete_with_diagnostics(request, ledger, **options)
                record('decoded', step_id=step_id, **response)
            except Exception as error:
                record('failed', step_id=step_id, reason=error.code if isinstance(error, broker.BrokerFailure) else 'transport_or_decode_failure')
                return result('provider_failure_stop')


def main():
    parser = argparse.ArgumentParser(description=__doc__, allow_abbrev=False)
    parser.add_argument('--directory', type=Path, required=True)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--approved-live', action='store_true')
    mode.add_argument('--local-provider', help='Credential-free test server: http://127.0.0.1:PORT/v1/')
    parser.add_argument('--ledger', type=Path, help='Existing shared aggregate ledger; live mode only')
    parser.add_argument('--fixture-ledger', type=Path, help='Separate local-test accounting ledger')
    parser.add_argument('--max-steps', type=int, required=True)
    parser.add_argument('--max-usd', required=True)
    parser.add_argument('--max-tokens', type=int, default=200000)
    parser.add_argument('--call-timeout-seconds', type=int, default=55)
    parser.add_argument('--recover-settled-decode', help='Explicit receipt ID for the single remaining call after a verified settled decode failure; retains all history and budgets')
    parser.add_argument('--steps-this-invocation', type=int, default=0, help='Optional pause after this many completed steps; restart retains overall bounds')
    args = parser.parse_args()
    try:
        print(json.dumps(run(args), indent=2))
    except Exception as error:
        reason = str(error) if isinstance(error, Stop) else 'controller_configuration_or_journal_error'
        print(json.dumps({'status': 'stopped', 'reason': reason}))
        raise SystemExit(2)


if __name__ == '__main__':
    import runpy, sys
    runpy.run_path(str(Path(__file__).with_name('engine-output-release.py')))['check_cli'](__file__,sys.argv[1:])

if __name__ == '__main__':
    main()
