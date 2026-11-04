#!/usr/bin/env python3
"""The single reading (spec #10). Opt-in only; a run without the profile is unchanged.

A reading comes before any proposal and carries no action contract. Its want crosses with no
review and grants nothing. Returns sit beside readings with no verdict.
"""
import importlib.util
from pathlib import Path

PROFILE = 'reading@v1'
TEXT_MAX = 4096  # tsr_domain::READING_TEXT_MAX
DIRECTIONS = ('want', 'sense', 'belief', 'expect')
FIELDS = DIRECTIONS + ('expected_return', 'falsifier')
_s = importlib.util.spec_from_file_location('reading_project_output', Path(__file__).with_name('engine-project-output.py'))
_output = importlib.util.module_from_spec(_s)
_s.loader.exec_module(_output)

READING_SYSTEM = (
    'Before any work is proposed, write one reading of where this public browser project stands for you now. '
    'Use the supplied brief, source, returns and earlier readings. '
    'Start with want: what do you want here? Answer in your own words from where you stand now, not by restating the brief or listing the work ahead. '
    'A want can be small, tentative or mistaken. Stating one commits nothing, and the next reading can change it. '
    'Then sense (what you sense is current), belief (what you believe now), and expect (what you expect, given what has happened). '
    'Nothing here has to be felt: if a field is not true for you now, make it null rather than inventing something. Any of want, sense, belief and expect can be null, including all four. '
    'If you state a want, also give expected_return {from, what} (what should come back, and from whom) and falsifier '
    '(what would show this reading was wrong). Otherwise both are null. '
    'The reading is recorded as generated. It approves, executes and accepts nothing, and nothing judges it. '
    'What actually comes back will be shown beside it later. '
    'Return only a JSON object with exactly the six fields in reading_contract.')

PROPOSAL_SYSTEM = (
    'Decide what to do with the artifact of this public browser project. '
    'current_reading holds your latest readings and any returns beside them. It is context, not a target or an approval. '
    'choice_summary lists the proposal forms available now; it is not approval or a ranking. '
    'Use the exact user words, current source and attributed observations. State uncertainty, and distinguish delivered evidence from the evidence you rely on. '
    'Unknown is not unmet. Citations and reasons are not proof of causal learning or player satisfaction. '
    'Positive feedback alone does not request a change, but an explicit creation or improvement request is an active goal, including when phrased as a question. '
    'Open design choices are delegated work, not inherently missing information. Passing checks are not satisfaction of a renewed goal. '
    'Consider alternatives and return 1..4 distinct candidates allowed by the supplied response contracts. You may instead inspect an admitted source window. '
    'Exact edits use old_text/new_text. A complete incremental edit is allowed; never return partial JSON or unfinished code. Preserve protected checks, runtime, permissions and acceptance. '
    'The controller previews proposals, obtains independent review of edits, and executes only with authorization; never claim these later steps happened already. '
    'To keep the artifact unchanged, return {retain:reason}. Retention ends this checkpoint, not future separately admitted input. '
    'Use {abstain:reason} for inability or declining further work; this stops the scheduled session. Neither creates user acceptance. Return only the supplied JSON contract.')

READING_CONTEXT_KEYS = ('public_brief', 'parents', 'files', 'return_evidence', 'browser_returns', 'supplemental_returns', 'criterion_returns',
                        'independent_assessments', 'feedback_provenance', 'review_criticism')
ARTIFACT_FORMS = ('abstain', 'retain', 'edit', 'inspect', 'verify')
DEVELOPMENT_CONTRACTS = ('relational', 'public_desire', 'provisional_preference', 'contrast_preference', 'success_preference', 'deliberation')
DEVELOPMENT_CONTEXT = ('relational_contract', 'relational_history', 'public_meaning_references', 'public_desire_history', 'generated_preferences',
                       'current_success', 'current_contrast', 'eligible_identity_aspirations', 'public_meaning_policy', 'generated_preference_statuses')
STRIPPED_CONTRACTS = DEVELOPMENT_CONTRACTS + ('relational_envelope',)
DEVELOPMENT_ALTERNATIVES = frozenset(DEVELOPMENT_CONTRACTS)
RETAIN_FORM = 'Keep the current artifact unchanged for this checkpoint: {retain:reason}. Not fulfillment or acceptance.'
RESPONSE_ENVELOPE = {
    'instruction': 'The generator response is {alternatives:[candidate,...]}, {retain:reason}, {abstain:reason}, or {inspect:{path,offset}}. Independent reviewers return their separate judgment contract, never inspect. '
                   'edits/rationale are candidate fields, never top-level fields. Each candidate matches one of response_contracts. Byte limits count UTF-8, not characters. '
                   'resulting_file in diagnostics means the complete file after applying that candidate, not a response field.',
    'max_alternatives': 4,
    'retention': '{retain:reason} is a current-condition no-change choice for the artifact; {abstain:reason} stops further work. Neither proves fulfillment or acceptance.'}
RESPONSIBILITIES = {
    'generator': 'Return proposed alternatives, not executed results. You may propose an untested initial design from the public brief, active wants and current source. '
                 'Prior test returns or player feedback are not prerequisites for an initial edit proposal. You do not invoke native preview yourself.',
    'review_stage': 'Independent proposal review assesses whether a complete incremental edit is useful and safe for the current stage; it need not finish every unresolved aim in one proposal. '
                    'Review must still reject misleading claims, regressions or unsupported actions. Proposal approval is not final acceptance; unresolved aims remain pending.'}


def admission_fields(profile):
    if profile is None:
        return {}
    if profile != PROFILE:
        raise ValueError('reading profile')
    return {'reading_profile': PROFILE}


def calls_per_decision(profile):
    """A reading adds one model call to each decision's proposal and review."""
    return 3 if profile else 2


def _text(path, value):
    if not isinstance(value, str):
        _output.fail(path, 'string_required', expected_type='string')
    if not value.strip() or len(value.encode()) > TEXT_MAX:
        _output.fail(path, 'text_bound', nonblank=True, max_utf8_bytes=TEXT_MAX)


def is_reading(value):
    return isinstance(value, dict) and set(value) == set(FIELDS)


def check_shape(value):
    """Mechanical checks only; the native contract repeats them. None reads meaning."""
    if not isinstance(value, dict):
        _output.fail('$', 'object_required', expected_type='object')
    if set(value) != set(FIELDS):
        _output.fail('$', 'unsupported_field', allowed_fields=list(FIELDS), required_fields=list(FIELDS))
    for key in DIRECTIONS + ('falsifier',):
        if value[key] is not None:
            _text('$.' + key, value[key])
    expected = value['expected_return']
    if expected is not None:
        if not isinstance(expected, dict):
            _output.fail('$.expected_return', 'object_required', expected_type='object')
        if set(expected) != {'from', 'what'}:
            _output.fail('$.expected_return', 'unsupported_field', allowed_fields=['from', 'what'], required_fields=['from', 'what'])
        _text('$.expected_return.from', expected['from'])
        _text('$.expected_return.what', expected['what'])
    if value['want'] is not None:
        for key in ('expected_return', 'falsifier'):
            if value[key] is None:
                _output.fail('$.' + key, 'required_field', required_fields=[key])
    elif expected is not None or value['falsifier'] is not None:
        _output.fail('$.want', 'required_field', required_fields=['want'])


def check_no_development(value):
    """No development route exists in an opted-in run, so none is accepted."""
    if not isinstance(value, dict):
        return
    if 'deliberation' in value:
        _output.fail('$.deliberation', 'unsupported_field', allowed_fields=['alternatives', 'retain', 'abstain', 'inspect'])
    if 'development_declined' in value:
        _output.fail('$.development_declined', 'unsupported_field', allowed_fields=['retain'])
    alternatives = value.get('alternatives')
    for index, item in enumerate(alternatives if isinstance(alternatives, list) else []):
        found = sorted(set(item) & DEVELOPMENT_ALTERNATIVES) if isinstance(item, dict) else []
        if found:
            _output.fail('$.alternatives[' + str(index) + '].' + found[0], 'unsupported_field', allowed_fields=['edits', 'rationale', 'expectation', 'existing'])


def reading_context(wire, view):
    """An allowlist, so no action contract or form can reach the reading turn."""
    context = {k: wire[k] for k in READING_CONTEXT_KEYS if k in wire}
    context['earlier_readings'] = view['bounded']
    context['reading_contract'] = {'fields': list(FIELDS), 'expected_return': ['from', 'what'], 'max_utf8_bytes_per_text': TEXT_MAX}
    return context


def strip_development(wire):
    """The proposal turn's context, with every development route removed."""
    value = {k: v for k, v in wire.items() if k not in DEVELOPMENT_CONTEXT}
    contracts = wire.get('response_contracts', {})
    value['response_contracts'] = {k: v for k, v in contracts.items() if k not in STRIPPED_CONTRACTS}
    if 'response_envelope' in wire:
        value['response_envelope'] = RESPONSE_ENVELOPE
    if 'proposal_responsibilities' in wire:
        value['proposal_responsibilities'] = {**wire['proposal_responsibilities'], **RESPONSIBILITIES}
    summary = wire.get('choice_summary', {})
    forms = {k: v for k, v in summary.get('forms', {}).items() if k in ARTIFACT_FORMS}
    if 'retain' in forms:
        forms['retain'] = RETAIN_FORM
    value['choice_summary'] = {'version': summary.get('version'), 'source_request_sha256': summary.get('source_request_sha256'), 'forms': forms,
                               'interpretation': 'Available proposal forms, not approved outcomes or a ranking.'}
    return value


def check_return(execution, reading):
    """A controller check as a return, or None when the execution left no receipt. No verdict."""
    completion = execution.get('state', {}).get('goal_completion', {}) if isinstance(execution, dict) else {}
    receipt = completion.get('test_receipt')
    if not receipt:
        return None
    return {'reading': reading, 'evidence': receipt,
            'text': 'Controller checks ran on source ' + str(completion.get('snapshot')) + ' with receipt ' + str(receipt) + '. Test result: ' + str(completion.get('test_verdict')) + '.'}
