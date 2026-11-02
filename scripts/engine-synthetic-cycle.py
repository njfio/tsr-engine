"""Explicitly admitted fixed-synthetic archive cycle. Never exports normal/private context.

No request is retried. A lost dispatch or application acknowledgment blocks for
operator reconciliation. Recorded outcomes and controller receipts resume locally.
"""
import datetime as dt
import decimal
import fcntl
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
from types import SimpleNamespace

ROOT = Path(__file__).resolve().parents[1]
FIXTURE = '612f479aea815883e97d9b3de772ab3ffeb5056114d068770b94d648b565b200'
MODEL = 'deepseek-v4.1-flash'
LUNA = 'gpt-6-luna'
ENDPOINT = 'https://api.cheaperinference.com/v1/chat/completions'
PHASES = ('relational', 'possibilities', 'assessment')
WORKSHOP = '4cb905c9511c5e865bfe8940b8554f8a17e37f0cdadbb97187f251122241c756'
GARDEN = '3740151f01d728fece9178f11b2c467e821750c465a4cb8d725f06ebc2f7128b'
SCENARIOS = {'archive': ('docs/engine-synthetic-archive-inputs.json', FIXTURE),
             'workshop': ('docs/engine-synthetic-workshop-plan.json', WORKSHOP),
             'garden': ('docs/engine-synthetic-garden-plan.json', GARDEN)}


def load_transport():
    s = importlib.util.spec_from_file_location('synthetic_transport', ROOT/'scripts/engine-synthetic-transport.py')
    m = importlib.util.module_from_spec(s); s.loader.exec_module(m)
    return m


def load_reviewer():
    spec=importlib.util.spec_from_file_location('synthetic_runtime_reviewer',ROOT/'scripts/engine-synthetic-review.py')
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

def validate_review_output(value,schema,binding,transport,helper,receipt):
    """Static diagnostics only; never persist exception text or rejected report prose."""
    schema_codes={'schema_depth_or_shape','unsupported_schema_keyword','schema_anyof','schema_const','schema_enum',
                  'schema_object','schema_extra','schema_array','schema_duplicate','schema_text','schema_number',
                  'schema_boolean','schema_null','unsupported_schema_type'}
    contract_codes={
        'review identity/shape':'identity_or_shape','incomplete review coverage':'incomplete_coverage',
        'finding shape':'finding_shape','finding enum':'finding_enum','unsupported evidence':'unsupported_evidence',
        'observed receipt required':'observed_receipt_required','unknown basis cannot establish finding':'unknown_basis_cannot_establish',
        'bounded review text':'bounded_text','review verdict':'verdict_enum',
        'accept requires complete met coverage':'accept_requires_met','reject requires negative finding':'reject_requires_negative',
        'unknown requires unknown finding':'unknown_requires_unknown'}
    try:candidate_hash=digest(value)
    except (TypeError,ValueError):candidate_hash=None
    body_hash=receipt.get('body_sha256')
    diagnostic={'contract':'synthetic-review-validation@v1','stage':'schema','code':None,'candidate_sha256':candidate_hash,
                'response_body_sha256':body_hash if isinstance(body_hash,str) and re.fullmatch('[0-9a-f]{64}',body_hash) else None}
    structure={'root_type':'object' if isinstance(value,dict) else 'non_object','findings':{}}
    if isinstance(value,dict):
        structure['verdict']=value.get('verdict') if value.get('verdict') in ('accept','reject','unknown','abstain') else 'invalid'
        findings=value.get('findings')
        if isinstance(findings,dict):
            structure['unexpected_findings_count']=len(set(findings)-set(helper.CRITERIA))
            for key in helper.CRITERIA:
                finding=findings.get(key)
                item={'present':key in findings,'object':isinstance(finding,dict)}
                if isinstance(finding,dict):
                    for field,allowed in (('finding',('met','not_met','unknown')),('basis',('static_semantic','observed_simulator','unknown'))):
                        item[field]=finding.get(field) if finding.get(field) in allowed else 'invalid'
                    for field in ('explanation','limitations'):
                        text=finding.get(field);item[field+'_utf8_bytes']=len(text.encode(errors='replace')) if isinstance(text,str) else None
                    refs=finding.get('evidence')
                    item['unsupported_reference_count']=sum(not isinstance(ref,str) or ref not in binding['evidence_ids'] for ref in refs) if isinstance(refs,list) else None
                structure['findings'][key]=item
    diagnostic['structure']=structure
    try:transport.validate(value,schema)
    except (ValueError,TypeError,KeyError) as error:
        diagnostic['code']=str(error) if type(error) is ValueError and str(error) in schema_codes else 'schema_validation_error'
        return None,diagnostic
    diagnostic['stage']='semantic_contract'
    try:helper.validate(value,binding)
    except (ValueError,TypeError,KeyError) as error:
        diagnostic['code']=contract_codes.get(str(error),'contract_validation_error') if type(error) is ValueError else 'contract_validation_error'
        return None,diagnostic
    diagnostic.update(stage='validated',code='valid')
    return value,diagnostic

def review_profile(actions, recovery=False):
    helper=load_reviewer()
    return {'contract':'synthetic-runtime-review@v1','model':LUNA,'max_calls':1+3*actions+(2 if recovery else 0),
            'completion_tokens':8192,'input_bytes':32768,'timeout_seconds':240,
            'criteria_sha256':helper.digest(helper.CRITERIA),
            'independence':'separate invocation and context; same model may share errors',
            'effect':'fail-closed permission-independent veto; no acceptance or authority grant'}

def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':')).encode()).hexdigest()


def read(path):
    def pairs(items):
        out = {}
        for k, v in items:
            if k in out: raise ValueError('duplicate JSON key')
            out[k] = v
        return out
    return json.loads(Path(path).read_bytes(), object_pairs_hook=pairs,
                      parse_constant=lambda _: (_ for _ in ()).throw(ValueError('nonfinite JSON')))


def save(path, value):
    # Exclusive immutable files. A partial file is an explicit blocker, never overwritten.
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, 'w') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2); stream.flush(); os.fsync(stream.fileno())
    fd = os.open(Path(path).parent, os.O_RDONLY)
    try: os.fsync(fd)
    finally: os.close(fd)


def storage_inventory(directory):
    """Conservative evidence of unchanged authoritative files, not just head root."""
    directory = Path(directory)
    if directory.is_symlink() or (directory/'cas').is_symlink(): raise ValueError('storage root symlink')
    def collect():
        paths = list(directory.glob('events.sqlite*')) + list((directory/'cas').rglob('*'))
        result = {}; total = 0
        for path in sorted(paths):
            if path.is_symlink(): raise ValueError('storage symlink')
            if path.is_dir(): continue
            if not path.is_file(): raise ValueError('nonregular storage')
            before = path.stat(); total += before.st_size
            if total > 128*1024*1024: raise ValueError('storage evidence bound')
            raw = path.read_bytes(); after = path.stat()
            if (before.st_ino,before.st_size,before.st_mtime_ns) != (after.st_ino,after.st_size,after.st_mtime_ns):
                raise ValueError('storage changed during inventory')
            result[str(path.relative_to(directory))] = hashlib.sha256(raw).hexdigest()
        if 'events.sqlite' not in result: raise ValueError('missing authoritative database')
        return result
    first = collect()
    if first != collect(): raise ValueError('unstable storage inventory')
    return first


def verify_refusal(cr, apply, directory, root):
    diagnostic = cr.get('result',{}).get('diagnostic',{})
    if (set(diagnostic) != {'contract','code','exit_code','stderr_sha256','stderr_bytes'}
            or diagnostic['contract'] != 'controller-refusal@v1'
            or diagnostic['code'] not in ('no_supported_progress','stale_offer','insufficient_valid_alternatives')
            or type(diagnostic['exit_code']) is not int or diagnostic['exit_code'] != 2
            or not isinstance(diagnostic['stderr_sha256'],str)
            or not re.fullmatch('[0-9a-f]{64}',diagnostic['stderr_sha256'])
            or type(diagnostic['stderr_bytes']) is not int or diagnostic['stderr_bytes'] <= 0):
        raise ValueError('invalid controller refusal diagnostic')
    if (cr['before_root'] != root or cr['after_root'] != root or apply['state_root'] != root
            or cr['action_count'] != 0 or cr['storage_before'] != apply['storage_before']
            or cr['storage_before'] != cr['storage_after']
            or cr['storage_after'] != storage_inventory(directory)
            or cr['apply_hash'] != digest(apply)):
        raise ValueError('refusal evidence drift')


def checkpoint(_name):
    """Test seam: subprocess tests interrupt here; production does nothing."""


def instant(value):
    parsed = dt.datetime.fromisoformat(value.replace('Z', '+00:00'))
    if parsed.tzinfo is None: raise ValueError('explicit timezone required')
    return parsed.astimezone(dt.timezone.utc)


def implementation():
    files = ('engine-directions.py','engine-synthetic-cycle.py','engine-synthetic-transport.py','engine-archive-contract.py',
             'm3-cheaper-broker.py','engine-local-model.py','engine-run.py',
             'engine-provider-envelope.py','engine-response-recovery.py','engine-workshop-prepare.py','engine-synthetic-review.py')
    values = {name:hashlib.sha256((ROOT/'scripts'/name).read_bytes()).hexdigest() for name in files}
    values['controller_binary'] = hashlib.sha256((ROOT/'target/debug/tsr').read_bytes()).hexdigest()
    return values


def fixture(scenario='archive'):
    if scenario not in SCENARIOS: raise ValueError('unregistered synthetic scenario')
    path, pinned = SCENARIOS[scenario]
    raw = (ROOT/path).read_bytes()
    if hashlib.sha256(raw).hexdigest() != pinned: raise ValueError('synthetic fixture drift')
    return json.loads(raw)


def scope(state, d, scenario='archive'):
    f = fixture(scenario)
    if state.get('standing_desires') != f['spec']['wants'] or state.get('archive', {}).get('world') != f['spec']['archive']:
        raise ValueError('only frozen evaluator-authored synthetic archive/parents admitted')
    ids = state['obligation_ids']
    if len(ids) != 2 or len(set(ids)) != 2: raise ValueError('two original identities required')
    for value in [state['state_root'], state['observation_revision'], *ids]:
        if not isinstance(value, str) or not re.fullmatch('[0-9a-f]{64}', value): raise ValueError('invalid binding')
    prefix = []
    for event in state['archive']['events']:
        d.archive.validate({'archive':{'world':f['spec']['archive'], 'events':prefix}},
                           {'directions':[{'action':{'operation':'archive','event':event}}]})
        prefix.append(event)
    if len(prefix) > 64: raise ValueError('event bound')
    if len(state['observation']) != 2 or any(type(x) not in (int,float) or not 0 <= x <= 1 for x in state['observation']):
        raise ValueError('invalid observation')


def workshop_initial(state, scenario='workshop'):
    plan = fixture(scenario)
    if (state['archive']['events'] != plan['initial_events']
            or not state['relational_request']['due'] or state['relational_request']['existing']
            or state['relational_request']['history']):
        raise ValueError('workshop requires exact frozen initialization and no prior hypothesis')


def idle_initial(state,scenario):
    if state['archive']['events']!=fixture(scenario)['initial_events'] or state['active_desires'] or state['status']!='inactive' or state['relational_request']['history']:raise ValueError('all originals must be withdrawn in exact frozen setup')

def relation(value, ids):
    p = value['proposal']
    if set(p['parents']) != set(ids): raise ValueError('relational parents drift')
    result = {}
    for key in ('relationship','aspiration','discriminator'):
        text = p[key]
        if not isinstance(text,str) or not 0 < len(text.encode()) <= 512: raise ValueError('bounded synthetic guidance required')
        result[key] = text
    result.update(parents=sorted(ids), semantic_status='unknown')
    for key in ('identity','revision'):
        if key in value:
            if not re.fullmatch('[0-9a-f]{64}', value[key]): raise ValueError('invalid relation identity')
            result[key] = value[key]
    if 'origin' in value:
        origin = value['origin']
        if set(origin['parents']) != set(ids) or not re.fullmatch('[0-9a-f]{64}',origin['revision']):
            raise ValueError('relational origin binding')
        result['origin'] = {'revision':origin['revision'],'parents':sorted(ids)}
        for key in ('relationship','aspiration','discriminator'):
            text = origin[key]
            if not isinstance(text,str) or not 0 < len(text.encode()) <= 512: raise ValueError('bounded origin meaning required')
            result['origin'][key] = text
    # Development annotations are preserved in the controller history; the normal
    # projection keeps original/current meaning and current return separate.
    return result


def context(state, d, scenario='archive'):
    scope(state, d, scenario)
    # Reconstruct, never copy the full request, operator report or private review.
    out = {'synthetic_only':True, 'state_root':state['state_root'],
           'observation_revision':state['observation_revision'],
           'standing_desires':fixture(scenario)['spec']['wants'], 'obligation_ids':state['obligation_ids'],
           'selected_desires':state['selected_desires'], 'active_desires':state['active_desires'],
           'active_obligation_ids':state['active_obligation_ids'],'native_status':state['status'], 'observation':state['observation'],
           'archive':{'world':fixture(scenario)['spec']['archive'], 'events':state['archive']['events'],
                      'response_evidence':d.archive.response_evidence(state)},
           'action_contract':d.action_contract(state), 'minimum_directions':state['minimum_directions']}
    if any(type(i) is not int or i not in (0,1) for i in out['selected_desires']): raise ValueError('invalid attention')
    if scenario in ('workshop','garden'): out['semantic_boundary'] = fixture(scenario)['model_visible_boundary']
    out['epistemic_roles'] = d.archive.epistemic_roles()
    out['relational_guidance'] = [relation(g,state['obligation_ids']) for g in state['relational_guidance'] if g['applicable']]
    return out


def payload(state, phase, pending, admission, d, feedback=None):
    scenario = admission.get('scenario','archive')
    if phase == 'possibilities_final': phase = 'possibilities'
    ctx = context(state, d, scenario)
    if phase == 'relational':
        r = state['relational_request']
        ctx = {'synthetic_only':True,'state_root':state['state_root'],'obligation_ids':state['obligation_ids'],
               'active_obligation_ids':state['active_obligation_ids'],'native_status':state['status'],
               'parent_meanings':[{'id':key,'text':want['text']} for key,want in zip(state['obligation_ids'],fixture(scenario)['spec']['wants'])],
               'measurement_limits':d.action_contract(state),
               'archive':{'world':fixture(scenario)['spec']['archive'],'events':state['archive']['events'],
                      'response_evidence':d.archive.response_evidence(state)}}
        # Only current typed bindings, numeric returns and admitted operations. Purposes
        # and private reports never cross this boundary.
        ctx['review'] = {k:r[k] for k in ('state_root','return_commitment','observation_revision','evidence_key','review_only')}
        eligible = [g for g in r['existing'] if not g['withdrawn']
                    and g['observation_revision'] == state['observation_revision']
                    and set(g['proposal']['parents']) <= set(r['active_originals'])
                    and (not r['review_only'] or g['revision'] in r['reviewable_revisions'])]
        ctx['existing'] = []
        for g in eligible:
            projected = dict(g)
            origin = next((x for x in r['history'] if x['identity'] == g['identity']),None)
            if origin is None: raise ValueError('missing relational lineage origin')
            projected['origin'] = dict(origin['proposal'],revision=origin['revision'])
            ctx['existing'].append(relation(projected,state['obligation_ids']))
        ctx['reviewable_revisions'] = r['reviewable_revisions']
        if state['recent_returns']:
            t = state['recent_returns'][-1] if len(state['recent_returns']) == 1 else next(
                x for x in state['recent_returns'] if x['commitment'] == r['return_commitment'])
            action = t['offered'][t['chosen']]['direction']['action']
            d.archive.action_schema(state)  # Schema is controller-derived, no generated authority.
            ctx['observed_return'] = {'commitment':t['commitment'], 'action':action,
                                      'before':t['before'], 'after':t['after']}
        if scenario in ('workshop','garden'): ctx['semantic_boundary'] = fixture(scenario)['model_visible_boundary']
        ctx['epistemic_roles'] = d.archive.epistemic_roles(relational=True)
        schema = r['response_schema']
        system = d.relational_instructions()
    else:
        system = d.archive.instructions(phase)
        schema = d.possibility_schema(state) if phase == 'possibilities' else d.assessment_schema(state,pending)
        if phase == 'assessment':
            ctx['frozen_possibilities'] = pending
            ctx['proposal_digest'] = d.proposal_digest(pending)
    if feedback is not None:ctx['attributed_semantic_feedback']=feedback
    body = {'model':admission['model'], 'stream':False, 'temperature':0,
            'max_tokens':admission['stages'][phase]['completion_tokens'],
            'messages':[{'role':'system','content':system+' Return only schema-bound JSON; each text at most 512 UTF-8 bytes.'},
                        {'role':'user','content':json.dumps({'context':ctx,'response_schema':schema},sort_keys=True)}]}
    if admission['model'] == LUNA:
        body['max_completion_tokens'] = body.pop('max_tokens'); body.pop('temperature')
    if len(json.dumps(body).encode()) > admission['max_payload_bytes']: raise ValueError('synthetic payload byte ceiling')
    return body, schema


def continuation_initial(state):
    plan=fixture('garden');events=state['archive']['events']
    if (len(events)!=5 or events[:4]!=plan['initial_events'] or events[4].get('kind')!='record_response'
            or state['relational_request']['due'] or not state['relational_request']['existing']
            or state['status']!='deliberate'):
        raise ValueError('continuation requires reviewed first return and an unresolved second challenge')

def continuation_binding(a):
    if not a.get('continuation'): return
    b=a['continuation'];parent=Path(b['parent']);archive=Path(a['directory'])/'parent-evidence'
    def inventory(root):
        result={}
        for p in sorted(root.rglob('*')):
            if p.is_symlink():raise ValueError('parent evidence symlink')
            if p.is_file():result[str(p.relative_to(root))]=hashlib.sha256(p.read_bytes()).hexdigest()
        return result
    if inventory(parent)!=b['files'] or inventory(archive)!=b['files']:raise ValueError('continuation parent/archive drift')
    review=read(Path(a['directory'])/'parent-review.json')
    if digest(review)!=b['review_sha256'] or review.get('approved') is not True:raise ValueError('continuation review drift')


def prepare(args, driver):
    d = SimpleNamespace(**driver); directory = args.directory.resolve(strict=True)
    scenario = getattr(args,'synthetic_scenario','archive')
    state = d.cli(directory,'direction-request'); scope(state,d,scenario)
    if args.model not in (MODEL,LUNA): raise ValueError('fixed configured model required')
    if getattr(args,'synthetic_review_recovery',False) and not getattr(args,'synthetic_runtime_review',False):raise ValueError('recovery requires runtime review')
    calls = args.calls; actions = args.synthetic_actions; idle=getattr(args,'synthetic_idle',False)
    if type(calls) is not int or not (0 if idle else 1) <= calls <= 12 or type(actions) is not int or not 0 <= actions <= 6:
        raise ValueError('bounded calls/actions required')
    multi=getattr(args,'synthetic_multi',False);reviewed_multi=getattr(args,'synthetic_reviewed_multi',False)
    if reviewed_multi and (scenario not in ('garden','workshop') or args.model!=LUNA or calls!=8 or actions!=2 or not getattr(args,'synthetic_runtime_review',False) or not getattr(args,'synthetic_review_recovery',False)):raise ValueError('reviewed multi profile')
    if multi and (scenario not in ('garden','workshop') or args.model!=LUNA or calls!=8 or actions!=2):raise ValueError('synthetic multi profile')
    if idle and (multi or scenario not in ('garden','workshop') or args.model!=LUNA or calls!=0 or actions!=0):raise ValueError('idle profile')
    continuation=getattr(args,'synthetic_continuation',None)
    if sum(bool(x) for x in (multi,idle,continuation,reviewed_multi))>1:raise ValueError('incompatible synthetic profiles')
    if continuation:
        if scenario!='garden' or args.model!=LUNA or calls!=3 or actions!=1:raise ValueError('continuation profile')
        continuation_initial(state)
    elif idle: idle_initial(state,scenario)
    elif scenario in ('workshop','garden'): workshop_initial(state,scenario)
    if not continuation and not multi and not reviewed_multi and not idle and scenario in ('workshop','garden') and (calls != 5 or actions != 1 or not state['relational_request']['due'] or state['relational_request']['existing']):
        raise ValueError('workshop requires fresh pending initial relation, five calls and one action')
    expires = instant(args.synthetic_expires)
    if expires <= dt.datetime.now(dt.timezone.utc): raise ValueError('future expiry required')
    target = args.prepare_synthetic_admission.resolve()
    stages = {'possibilities':{'completion_tokens':8192,'timeout_seconds':240},
              'assessment':{'completion_tokens':32768,'timeout_seconds':240},
              'relational':{'completion_tokens':8192,'timeout_seconds':240}}
    if args.model == LUNA: stages['assessment']['completion_tokens'] = 8192
    # The source snapshot is local evidence; only the projection is exportable.
    source = target.with_suffix(target.suffix+'.source.json')
    save(source,state)
    manifest = {'contract':'synthetic-archive-cycle@v1','model':args.model,'endpoint':ENDPOINT,
                'directory':str(directory),'source_snapshot':str(source),'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
                'initial_root':state['state_root'],'fixture_sha256':SCENARIOS[scenario][1],'calls':calls,'actions':actions,
                'stages':stages,'max_payload_bytes':16384,'reserved_input_tokens_per_call':16384,
                'max_reserved_completion_tokens':calls*32768,'max_transport_seconds':calls*240,
                'expires_at':expires.isoformat(),'synthetic_only_attestation':True,'implementation_sha256':implementation()}
    if scenario in ('workshop','garden'):
        manifest.update(scenario=scenario, stop_empty_initial_aspiration=True, proposal_only_tail=True,
                        max_reserved_completion_tokens=65536)
    if args.model == LUNA:
        manifest.update(model_profile='luna-bounded@v1', max_reserved_completion_tokens=calls*8192)
    if multi:
        manifest.update(autonomy_profile=scenario+'-multi@v1',proposal_only_tail=False,stop_empty_initial_aspiration=True,max_payload_bytes=32768,reserved_input_tokens_per_call=32768,
            operator_boundary={'after_request':4,'after_actions':1,'preference':state['obligation_ids'][1],'status':'withdrawn','source':'preregistered evaluator/operator; not model choice'})
    if reviewed_multi:manifest.update(reviewed_multi_profile='reviewed-multi@v1',proposal_only_tail=False)
    if idle:manifest.update(idle_profile='all-originals-withdrawn@v1',stop_empty_initial_aspiration=False,proposal_only_tail=False)
    if getattr(args,'synthetic_runtime_review',False):
        if args.model!=LUNA or idle:raise ValueError('runtime review requires non-idle Luna profile')
        recovery=bool(getattr(args,'synthetic_review_recovery',False))
        if recovery:
            if multi or idle or continuation or scenario not in ('garden','workshop'):raise ValueError('recovery requires ordinary reviewed synthetic profile')
            manifest['review_recovery']='single-attributed-reconsideration@v1'
            manifest['calls']=calls+2;manifest['max_reserved_completion_tokens']=(calls+2)*8192;manifest['max_transport_seconds']=(calls+2)*240
        manifest['runtime_review']=review_profile(actions,recovery)
        manifest['total_call_limit']=manifest['calls']+manifest['runtime_review']['max_calls']
        manifest['max_payload_bytes']=32768;manifest['reserved_input_tokens_per_call']=32768
    if continuation:
        manifest.update(continuation=continuation,stop_empty_initial_aspiration=False,proposal_only_tail=False,max_reserved_completion_tokens=24576)
        continuation_binding(manifest)
    save(target,manifest)
    return {'status':'prepared_not_dispatched','admission':str(target),'source_projection':context(state,d,scenario)}


def validate_admission(a, args):
    if sum(bool(a.get(k)) for k in ('autonomy_profile','idle_profile','continuation','reviewed_multi_profile'))>1:raise ValueError('incompatible synthetic profiles')
    if a['contract'] != 'synthetic-archive-cycle@v1' or a['model'] not in (MODEL,LUNA) or a['endpoint'] != ENDPOINT or args.model != a['model']:
        raise ValueError('fixed external admission required')
    if a['model']==LUNA and (a.get('model_profile')!='luna-bounded@v1' or a['max_reserved_completion_tokens']!=a['calls']*8192):
        raise ValueError('Luna profile required')
    if a.get('runtime_review'):
        if a['model']!=LUNA or a.get('idle_profile') or a['runtime_review']!=review_profile(a['actions'],bool(a.get('review_recovery'))) or a.get('total_call_limit')!=a['calls']+a['runtime_review']['max_calls']:raise ValueError('runtime review admission mismatch')
    elif 'total_call_limit' in a:raise ValueError('unadmitted review budget')
    if a.get('review_recovery') and (a['review_recovery']!='single-attributed-reconsideration@v1' or not a.get('runtime_review') or any(a.get(k) for k in ('autonomy_profile','idle_profile','continuation'))):raise ValueError('invalid recovery profile')
    scenario = a.get('scenario','archive')
    fixture(scenario)  # Registered file and semantic-boundary hash checked before transport.
    if a.get('reviewed_multi_profile'):
        if (a['reviewed_multi_profile']!='reviewed-multi@v1' or scenario not in ('garden','workshop') or a['model']!=LUNA or a['calls']!=10 or a['actions']!=2 or not a.get('runtime_review') or not a.get('review_recovery') or a.get('proposal_only_tail') or not a.get('stop_empty_initial_aspiration') or 'operator_boundary' in a):raise ValueError('reviewed multi admission shape')
    elif a.get('autonomy_profile'):
        if (scenario not in ('garden','workshop') or a['autonomy_profile']!=scenario+'-multi@v1' or a['model']!=LUNA or a['calls']!=8 or a['actions']!=2 or a.get('proposal_only_tail') or not a.get('stop_empty_initial_aspiration')):raise ValueError('multi admission shape')
        b=a['operator_boundary']
        if b!={'after_request':4,'after_actions':1,'preference':read(a['source_snapshot'])['obligation_ids'][1],'status':'withdrawn','source':'preregistered evaluator/operator; not model choice'}:raise ValueError('operator boundary changed')
    elif a.get('idle_profile'):
        if (a['idle_profile']!='all-originals-withdrawn@v1' or scenario not in ('garden','workshop') or a['model']!=LUNA or a['calls']!=0 or a['actions']!=0 or a.get('proposal_only_tail') or a.get('stop_empty_initial_aspiration')):raise ValueError('idle admission shape')
    elif a.get('continuation'):
        if (scenario!='garden' or a['model']!=LUNA or a['calls']!=3 or a['actions']!=1 or a.get('proposal_only_tail') or a.get('stop_empty_initial_aspiration') or a['max_reserved_completion_tokens']!=24576):raise ValueError('continuation admission shape')
        continuation_binding(a)
    elif scenario in ('workshop','garden'):
        if (a['calls'] != (7 if a.get('review_recovery') else 5) or a['actions'] != 1 or a.get('stop_empty_initial_aspiration') is not True
                or a.get('proposal_only_tail') is not True or a['max_reserved_completion_tokens'] != ((57344 if a.get('review_recovery') else 40960) if a['model']==LUNA else 65536)):
            raise ValueError('workshop admission shape')
    elif a.get('proposal_only_tail') or a.get('stop_empty_initial_aspiration'):
        raise ValueError('unadmitted scenario behavior')
    if a['synthetic_only_attestation'] is not True or a['fixture_sha256'] != SCENARIOS[scenario][1]:
        raise ValueError('explicit synthetic attestation required')
    if Path(a['directory']).resolve() != args.directory.resolve(strict=True): raise ValueError('admission directory mismatch')
    bounds = {'calls':((0,0) if a.get('idle_profile') else (1,12)),'actions':(0,6),'max_payload_bytes':((32768,32768) if a.get('autonomy_profile') or a.get('runtime_review') else (1,16384)),'reserved_input_tokens_per_call':((32768,32768) if a.get('autonomy_profile') or a.get('runtime_review') else (16384,16384)),
              'max_reserved_completion_tokens':((0,0) if a.get('idle_profile') else (1,393216)),'max_transport_seconds':((0,0) if a.get('idle_profile') else (1,2880))}
    for k,(lo,hi) in bounds.items():
        if type(a[k]) is not int or not lo <= a[k] <= hi: raise ValueError('invalid admission '+k)
    if set(a['stages']) != set(PHASES): raise ValueError('stage contract')
    for p,v in a['stages'].items():
        if set(v) != {'completion_tokens','timeout_seconds'}: raise ValueError('stage bounds required')
        cap = 32768 if p == 'assessment' and a['model']==MODEL else 8192
        if type(v['completion_tokens']) is not int or not 1 <= v['completion_tokens'] <= cap: raise ValueError('completion bound')
        if type(v['timeout_seconds']) is not int or not 1 <= v['timeout_seconds'] <= 240: raise ValueError('deadline bound')
    instant(a['expires_at'])
    if a['implementation_sha256'] != implementation(): raise ValueError('admitted implementation drift')


def run(args, driver):
    d = SimpleNamespace(**driver); a = read(args.synthetic_admission); validate_admission(a,args)
    if getattr(args,'expected_admission_hash',digest(a)) != digest(a): raise ValueError('campaign admission binding mismatch')
    source = Path(a['source_snapshot']); raw = source.read_bytes()
    if hashlib.sha256(raw).hexdigest() != a['source_sha256']: raise ValueError('admitted source drift')
    initial = read(source); scope(initial,d,a.get('scenario','archive'))
    if a.get('continuation'): continuation_initial(initial)
    elif a.get('idle_profile'):idle_initial(initial,a['scenario'])
    elif a.get('scenario') in ('workshop','garden'): workshop_initial(initial,a['scenario'])
    if initial['state_root'] != a['initial_root']: raise ValueError('initial root mismatch')
    directory = args.directory.resolve(strict=True)
    out = directory/'synthetic-external'; out.mkdir(exist_ok=True)
    if out.is_symlink(): raise ValueError('symlink journal refused')
    lockfd = os.open(out/'lock', os.O_CREAT|os.O_RDWR|os.O_NOFOLLOW, 0o600)
    with os.fdopen(lockfd,'a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        if (out/'admission.json').exists():
            if read(out/'admission.json') != a: raise ValueError('admission is frozen; no budget reset')
        else: save(out/'admission.json',a)
        def current(): return d.cli(directory,'direction-request')
        def verify_refusals():
            for path in out.glob('controller-*.json'):
                cr = read(path)
                if cr.get('result',{}).get('status') != 'controller_refused': continue
                number = path.stem.split('-')[-1]
                apply = read(out/f'apply-intent-{number}.json')
                intent = read(out/f'intent-{number}.json'); outcome = read(out/f'outcome-{number}.json')
                if (intent['admission_hash'] != digest(a) or outcome['intent_hash'] != digest(intent)
                        or cr['outcome_hash'] != digest(outcome) or apply['outcome_hash'] != digest(outcome)
                        or apply['value_hash'] != digest(read(out/f'apply-value-{number}.json'))
                        or intent['state_root'] != cr['before_root']
                        or digest(read(out/f'request-{number}.json')) != intent['payload_hash']
                        or digest(read(out/f'schema-{number}.json')) != intent['schema_hash']):
                    raise ValueError('refusal provenance drift')
                verify_refusal(cr,apply,directory,current()['state_root'])
        def finish(reason):
            intents = list(sorted(out.glob('intent-*.json')))
            receipts = [read(p)['receipt'] for p in sorted(out.glob('outcome-*.json'))]
            review_intents=list(sorted(out.glob('review-*.intent.json')))
            review_outcomes=[read(p) for p in sorted(out.glob('review-*.outcome.json'))]
            receipts += [x['receipt'] for x in review_outcomes]
            review_unknown=[read(p) for p in review_intents if not p.with_name(p.name.replace('.intent.','.outcome.')).exists() or read(p.with_name(p.name.replace('.intent.','.outcome.')))['receipt'].get('status')!='settled' or read(p.with_name(p.name.replace('.intent.','.outcome.')))['receipt'].get('usage_status')!='reported' or read(p.with_name(p.name.replace('.intent.','.outcome.')))['receipt'].get('usd') is None]
            unmatched = [p.name for p in intents if not (out/p.name.replace('intent-', 'outcome-')).exists()]
            unresolved = [read(p) for p in intents if p.name in unmatched or (
                (out/p.name.replace('intent-', 'outcome-')).exists() and
                (read(out/p.name.replace('intent-', 'outcome-'))['receipt'].get('status') != 'settled' or
                 read(out/p.name.replace('intent-', 'outcome-'))['receipt'].get('usage_status') != 'reported'))]
            uncertain_applications = [p.name for p in sorted(out.glob('apply-intent-*.json'))
                                      if not (out/p.name.replace('apply-intent-', 'controller-')).exists()]
            verify_refusals()
            value = {'status':reason,'calls_dispatched':len(intents),'review_calls_dispatched':len(review_intents),'total_calls_dispatched':len(intents)+len(review_intents),
                     'semantic_review_authority':'veto/hold only; no formal acceptance or desire fulfillment',
                     'unknown_application_outcome':bool(uncertain_applications),
                     'unacknowledged_applications':uncertain_applications,
                     'acknowledged_actions':sum(read(p)['action_count'] for p in out.glob('controller-*.json')),
                     'unmatched_intents':unmatched,
                     'retained_reservations':{'calls':len(unresolved)+len(review_unknown),
                         'input_tokens':sum(i['input_reserved'] for i in unresolved+review_unknown),
                         'completion_tokens':sum(i['completion_reserved'] for i in unresolved+review_unknown),
                         'transport_seconds':sum(i['wall_reserved'] for i in unresolved+review_unknown)},
                     'settled_usd':str(sum((decimal.Decimal(r['usd']) for r in receipts if r.get('usd') is not None),decimal.Decimal(0))),
                     'unknown_usage_or_cost':bool(unmatched) or bool(review_unknown) or any(r.get('usage_status') != 'reported' or r.get('status')!='settled' for r in receipts),
                     'state_root':current()['state_root'],'remaining_slots_expired':True}
            if reason == 'controller_refused':
                refusals = [p for p in out.glob('controller-*.json') if read(p).get('result',{}).get('status') == reason]
                if len(refusals) != 1: raise ValueError('exact terminal refusal receipt required')
                value['controller_refusal'] = {'file':refusals[0].name,'sha256':digest(read(refusals[0]))}
            save(out/'terminal.json',value); return value
        def stable(path,value):
            if path.exists():
                if read(path)!=value:raise ValueError('review frozen evidence drift')
            else:save(path,value)
        def verify_reviews():
            for ip in out.glob('review-*.intent.json'):
                prefix=ip.name[:-len('.intent.json')];intent=read(ip)
                if not a.get('runtime_review') or intent['admission_hash']!=digest(a):raise ValueError('runtime review admission drift')
                helper=load_reviewer();runtime_a=dict(a,runtime_review_id=digest({'admission':digest(a),'review':prefix}))
                body,schema,binding=helper.build(read(out/f'{prefix}.state.json'),intent['kind'],read(out/f'{prefix}.subject.json'),runtime_a)
                if intent['payload_hash']!=digest(body) or intent['schema_hash']!=digest(schema) or intent['binding_hash']!=digest(binding):raise ValueError('runtime review source binding drift')
                for suffix,key in (('request','payload_hash'),('schema','schema_hash'),('binding','binding_hash')):
                    if digest(read(out/f'{prefix}.{suffix}.json'))!=intent[key]:raise ValueError('runtime review provenance drift')
                op=out/f'{prefix}.outcome.json'
                if op.exists():
                    outcome=read(op)
                    if outcome['intent_hash']!=digest(intent):raise ValueError('runtime review outcome drift')
                    if outcome['value'] is not None:load_reviewer().validate(outcome['value'],read(out/f'{prefix}.binding.json'))
        def review_gate(kind,number,review_state,subject):
            if not a.get('runtime_review'):return 'accept'
            helper=load_reviewer();cap=a['runtime_review'];prefix=f'review-{number}-{kind}'
            ip=out/f'{prefix}.intent.json';op=out/f'{prefix}.outcome.json'
            runtime_a=dict(a,runtime_review_id=digest({'admission':digest(a),'review':prefix}))
            body,schema,binding=helper.build(review_state,kind,subject,runtime_a)
            if len(json.dumps(body).encode())>cap['input_bytes']:return 'payload_limit'
            if current()['state_root']!=review_state['state_root']:return 'state_drift'
            if (out/'STOP').exists() or (getattr(args,'stop_file',None) is not None and args.stop_file.exists()):return 'operator_stop'
            if dt.datetime.now(dt.timezone.utc)>=instant(a['expires_at']):return 'expired'
            if ip.exists():
                intent=read(ip)
                if (intent['payload_hash']!=digest(body) or intent['binding_hash']!=digest(binding) or intent['schema_hash']!=digest(schema)):raise ValueError('review request binding drift')
                if not op.exists():return 'generation_unknown'
            else:
                count=len(list(out.glob('review-*.intent.json')))
                total=count+len(list(out.glob('intent-*.json')))
                if count>=cap['max_calls'] or total>=a['total_call_limit']:return 'call_budget'
                stable(out/f'{prefix}.state.json',review_state);stable(out/f'{prefix}.subject.json',subject)
                stable(out/f'{prefix}.request.json',body);stable(out/f'{prefix}.schema.json',schema);stable(out/f'{prefix}.binding.json',binding)
                intent={'payload_hash':digest(body),'schema_hash':digest(schema),'binding_hash':digest(binding),
                        'input_reserved':cap['input_bytes'],'completion_reserved':cap['completion_tokens'],'wall_reserved':cap['timeout_seconds'],
                        'role':'independent_context_reviewer','kind':kind,'generator_request':number,'model':a['model'],'admission_hash':digest(a)}
                save(ip,intent);checkpoint('review_intent')
                timeout=min(cap['timeout_seconds'],(instant(a['expires_at'])-dt.datetime.now(dt.timezone.utc)).total_seconds())
                if timeout<=0:return 'expired'
                provider=getattr(args,'synthetic_provider',None);transport=load_transport()
                result=provider(prefix,body,timeout) if provider else transport.invoke(body,timeout,expected_model=LUNA,payload_limit=cap['input_bytes'])
                value,validation=validate_review_output(result.get('value'),schema,binding,transport,helper,result['receipt'])
                save(op,{'intent_hash':digest(intent),'receipt':result['receipt'],'value':value,'validation':validation,'resource_stop':result.get('resource_stop') if result.get('resource_stop') in ('resource_overrun','unknown_usage') else None})
                checkpoint('review_outcome')
            outcome=read(op)
            if outcome['intent_hash']!=digest(intent):raise ValueError('review outcome binding mismatch')
            receipt=outcome['receipt']
            if outcome.get('resource_stop'):return outcome['resource_stop']
            if receipt.get('status')!='settled' or receipt.get('usage_status')!='reported':return 'usage_unknown'
            if receipt.get('usd') is None:return 'cost_unknown'
            usage=receipt['usage']
            if usage['completion_tokens']>cap['completion_tokens'] or usage['prompt_tokens']>cap['input_bytes']:return 'resource_limit'
            if outcome['value'] is None:return 'invalid_report'
            if current()['state_root']!=review_state['state_root']:return 'state_drift'
            return helper.validate(outcome['value'],binding)
        def applied_reviews(number,phase,count):
            if not a.get('runtime_review'):return {}
            kinds=([phase] if phase in ('relational','assessment') else [])+(['post_action'] if phase=='assessment' and count==1 else [])
            links={}
            for kind in kinds:
                prefix=f'review-{number}-{kind}';op=out/f'{prefix}.outcome.json';value=read(op)
                binding=read(out/f'{prefix}.binding.json');subject=read(out/f'{prefix}.subject.json');rs=read(out/f'{prefix}.state.json')
                intent=read(out/f'intent-{number}.json');generator=read(out/f'outcome-{number}.json')
                if (subject['generator_outcome_sha256']!=digest(generator) or subject['generator_request']!=number
                        or subject['candidate']!=read(out/f'apply-value-{number}.json')):raise ValueError('review candidate lineage drift')
                if kind=='post_action':
                    if subject['controller_receipt']!=read(out/f'controller-{number}.json'):raise ValueError('review receipt lineage drift')
                elif rs['state_root']!=intent['state_root']:raise ValueError('review source lineage drift')
                if value.get('resource_stop') or value['receipt'].get('status')!='settled' or value['receipt'].get('usage_status')!='reported' or load_reviewer().validate(value['value'],binding)!='accept':raise ValueError('applied stage lacks accepted runtime gate')
                links[kind]=digest(value)
            return links
        def recovery_record(number,kind,verdict,candidate,cr=None):
            if not a.get('review_recovery') or verdict not in ('reject','unknown'):return None
            if list(out.glob('recovery-*.json')):return None
            if kind=='post_action' and not current()['relational_request']['due']:return None
            prefix=f'review-{number}-{kind}';op=read(out/f'{prefix}.outcome.json');binding=read(out/f'{prefix}.binding.json')
            helper=load_reviewer()
            record={'request':number,'kind':kind,'verdict':verdict,'review_sha256':digest(op),
                'generator_sha256':digest(read(out/f'outcome-{number}.json')),'before_root':read(out/f'intent-{number}.json')['state_root'],
                'after_root':current()['state_root'],'controller_sha256':digest(cr) if cr else None,
                'action_count':cr['action_count'] if cr else 0,'fingerprint':helper.candidate_fingerprint(kind,candidate),
                'feedback':helper.semantic_feedback(op['value'],binding)}
            save(out/f'recovery-{number}.json',record);checkpoint('semantic_recovery');return record
        def verify_recovery(path):
            record=read(path);n=record['request'];kind=record['kind'];prefix=f'review-{n}-{kind}'
            op=read(out/f'{prefix}.outcome.json');binding=read(out/f'{prefix}.binding.json');subject=read(out/f'{prefix}.subject.json');helper=load_reviewer()
            if (not a.get('review_recovery') or record['verdict'] not in ('reject','unknown') or helper.validate(op['value'],binding)!=record['verdict']
                or digest(op)!=record['review_sha256'] or digest(read(out/f'outcome-{n}.json'))!=record['generator_sha256']
                or record['feedback']!=helper.semantic_feedback(op['value'],binding) or record['fingerprint']!=helper.candidate_fingerprint(kind,subject['candidate'])
                or op['receipt'].get('status')!='settled' or op['receipt'].get('usage_status')!='reported' or op['receipt'].get('usd') is None or op.get('resource_stop')):raise ValueError('recovery provenance drift')
            intent=read(out/f'intent-{n}.json');generator=read(out/f'outcome-{n}.json');rs=read(out/f'{prefix}.state.json')
            if (intent['admission_hash']!=digest(a) or intent['payload_hash']!=digest(read(out/f'request-{n}.json'))
                or intent['schema_hash']!=digest(read(out/f'schema-{n}.json')) or generator['intent_hash']!=digest(intent)
                or subject['generator_request']!=n or subject['generator_outcome_sha256']!=digest(generator)
                or rs['state_root']!=(record['after_root'] if kind=='post_action' else record['before_root'])
                or intent['phase']!=('assessment' if kind=='post_action' else kind)):raise ValueError('recovery generator lineage drift')
            cr=read(out/f'controller-{n}.json') if record['controller_sha256'] else None
            if cr:
                apply=read(out/f'apply-intent-{n}.json');candidate=read(out/f'apply-value-{n}.json')
                if (kind!='post_action' or subject['controller_receipt']!=cr or subject['candidate']!=candidate
                    or cr['before_root']!=record['before_root'] or cr['outcome_hash']!=digest(generator)
                    or cr['apply_hash']!=digest(apply) or apply['outcome_hash']!=digest(generator)
                    or apply['value_hash']!=digest(candidate) or apply['state_root']!=record['before_root']):raise ValueError('recovery action lineage drift')
            elif kind=='post_action' or (out/f'apply-intent-{n}.json').exists():raise ValueError('recovery unexpected application')
            if record['before_root']!=read(out/f'intent-{n}.json')['state_root'] or (cr and (digest(cr)!=record['controller_sha256'] or cr['after_root']!=record['after_root'] or cr['action_count']!=record['action_count'])) or (not cr and (record['before_root']!=record['after_root'] or record['action_count']!=0)):raise ValueError('recovery receipt drift')
            return record
        recoveries=list(out.glob('recovery-*.json'))
        if len(recoveries)>1:raise ValueError('multiple semantic recoveries')
        for rp in recoveries:verify_recovery(rp)
        verify_reviews()
        if a.get('runtime_review'):
            for path in out.glob('applied-*.json'):
                n=int(path.stem.split('-')[-1]);record=read(path);phase=read(out/f'intent-{n}.json')['phase']
                if record.get('review_outcomes')!=applied_reviews(n,phase,record['action_count']):raise ValueError('applied review receipt drift')
        if a.get('autonomy_profile') and (out/'operator-boundary-receipt.json').exists():
            bi=read(out/'operator-boundary-intent.json');br=read(out/'operator-boundary-receipt.json')
            boundary_root=read(out/'applied-4.json')['after_root'];b=a['operator_boundary']
            if bi!={'state_root':boundary_root,'preference':b['preference'],'status':b['status']} or br['intent_hash']!=digest(bi) or br['before_root']!=boundary_root:
                raise ValueError('operator boundary history drift')
        if (out/'terminal.json').exists():
            terminal = read(out/'terminal.json')
            if terminal['status'] == 'controller_refused':
                binding = terminal['controller_refusal']
                if not re.fullmatch(r'controller-[1-9][0-9]*\.json',binding['file']): raise ValueError('invalid terminal refusal path')
                cr = read(out/binding['file'])
                if digest(cr) != binding['sha256'] or cr.get('result',{}).get('status') != 'controller_refused':
                    raise ValueError('terminal refusal evidence missing or changed')
            verify_refusals()
            return terminal
        transport = load_transport()
        pending = None; expected = initial['state_root']; actions = 0; reserved = 0; wall = 0; recovery = None
        for number in range(1,a['calls']+2):
            if getattr(args,'expire_remaining',None): return finish('withdrawn')
            if dt.datetime.now(dt.timezone.utc) >= instant(a['expires_at']): return finish('expired')
            stop = getattr(args,'stop_file',None)
            if (out/'STOP').exists() or (stop is not None and stop.exists()): return {'status':'operator_stop','state_root':current()['state_root']}
            if a.get('autonomy_profile') and number==5:
                b=a['operator_boundary'];bp=out/'operator-boundary-intent.json';br=out/'operator-boundary-receipt.json'
                if actions!=1 or pending is not None:raise ValueError('operator boundary phase mismatch')
                if br.exists():
                    bi=read(bp);receipt=read(br)
                    if (bi!={'state_root':expected,'preference':b['preference'],'status':b['status']} or receipt['intent_hash']!=digest(bi)):raise ValueError('operator boundary replay mismatch')
                    expected=receipt['after_root']
                elif bp.exists():
                    return {'status':'uncertain_operator_application','reservation_retained':True,'state_root':current()['state_root']}
                else:
                    prior=current()
                    if prior['state_root']!=expected or prior['relational_request']['due']:return finish('state_drift_or_pending_review')
                    update={'state_root':expected,'preference':b['preference'],'status':b['status']}
                    save(bp,update);checkpoint('operator_intent')
                    try:changed=d.cli(directory,'direction-status','--update',str(bp))
                    except Exception:return {'status':'uncertain_operator_application','reservation_retained':True,'state_root':current()['state_root']}
                    changed=current()
                    if changed['active_obligation_ids']!=[prior['obligation_ids'][0]] or changed['standing_desires']!=prior['standing_desires'] or changed['authority']!=prior['authority']:raise ValueError('operator withdrawal unexpected effect')
                    save(br,{'intent_hash':digest(update),'before_root':expected,'after_root':changed['state_root']});checkpoint('operator_receipt')
                    expected=changed['state_root']

            intent_path = out/f'intent-{number}.json'; outcome_path = out/f'outcome-{number}.json'
            applied_path = out/f'applied-{number}.json'; apply_path = out/f'apply-intent-{number}.json'; receipt_path = out/f'controller-{number}.json'
            rp=out/f'recovery-{number}.json'
            if rp.exists():
                recovery=verify_recovery(rp);ri=read(intent_path)
                if recovery['before_root']!=expected:raise ValueError('recovery prefix drift')
                expected=recovery['after_root'];actions+=recovery['action_count'];pending=None
                reserved+=ri['completion_reserved'];wall+=ri['wall_reserved'];continue
            if applied_path.exists():
                record = read(applied_path); intent = read(intent_path); outcome = read(outcome_path)
                if record['intent_hash'] != digest(intent) or record['outcome_hash'] != digest(outcome) or intent['state_root'] != expected:
                    raise ValueError('applied provenance mismatch')
                if record.get('controller_hash') and digest(read(receipt_path)) != record['controller_hash']: raise ValueError('controller receipt drift')
                expected = record['after_root']; actions += record['action_count']; pending = record['pending']
                reserved += intent['completion_reserved']; wall += intent['wall_reserved']
                if recovery and recovery['kind']=='post_action' and intent['phase']=='relational':return finish('semantic_recovery_reflection_complete')
                if intent['phase'] == 'possibilities_final':
                    if current()['state_root'] != expected or actions != a['actions']: raise ValueError('final proposal replay drift')
                    return finish('proposal_only_complete')
                continue
            state = current(); scope(state,d,a.get('scenario','archive'))
            if apply_path.exists() and not receipt_path.exists():
                return {'status':'uncertain_application','request':number,'state_root':state['state_root'],
                        'meaning':'Inspect controller history; never automatically replay an unacknowledged application.'}
            if receipt_path.exists():
                cr = read(receipt_path)
                if cr['after_root'] != state['state_root'] or cr['before_root'] != expected:
                    raise ValueError('controller receipt/current source mismatch')
            elif state['state_root'] != expected:
                return finish('state_drift_or_withdrawal')
            if intent_path.exists():
                intent = read(intent_path)
                if intent['state_root'] != expected or intent['admission_hash'] != digest(a): raise ValueError('intent binding mismatch')
                request = read(out/f'request-{number}.json')
                if digest(request) != intent['payload_hash']: raise ValueError('request drift')
                if not outcome_path.exists():
                    return {'status':'uncertain_generation','request':number,'reservation_retained':True,'state_root':state['state_root']}
                phase = intent['phase']; schema = read(out/f'schema-{number}.json')
                if digest(schema) != intent['schema_hash']: raise ValueError('schema drift')
            else:
                if (a.get('autonomy_profile') or a.get('reviewed_multi_profile')) and not state['relational_request']['due'] and state['status']!='deliberate':return finish(state['status'])
                if state['status'] == 'inactive': return finish('withdrawn_desires')
                if number > a['calls']: return finish('call_budget')
                due = state.get('relational_request',{}).get('due') is True
                if due: phase = 'relational'
                elif actions >= a['actions']:
                    if a.get('proposal_only_tail') and actions == a['actions'] and state['status'] == 'deliberate': phase = 'possibilities_final'
                    else: return finish('action_budget_review_complete')
                elif state['status'] != 'deliberate': return finish(state['status'])
                else: phase = 'assessment' if pending is not None else 'possibilities'
                cap = a['stages']['possibilities' if phase == 'possibilities_final' else phase]
                if reserved+cap['completion_tokens'] > a['max_reserved_completion_tokens'] or wall+cap['timeout_seconds'] > a['max_transport_seconds']:
                    return finish('reservation_budget')
                body,schema = payload(state,phase,pending,a,d,recovery['feedback'] if recovery else None)
                save(out/f'request-{number}.json',body); save(out/f'schema-{number}.json',schema)
                intent = {'phase':phase,'state_root':expected,'admission_hash':digest(a),'payload_hash':digest(body),'schema_hash':digest(schema),
                          'completion_reserved':cap['completion_tokens'],'input_reserved':a['reserved_input_tokens_per_call'],
                          'wall_reserved':cap['timeout_seconds'],'at':dt.datetime.now(dt.timezone.utc).isoformat()}
                save(intent_path,intent); checkpoint('intent')
                remaining = (instant(a['expires_at'])-dt.datetime.now(dt.timezone.utc)).total_seconds()
                if remaining <= 0: return finish('expired')
                invoke = getattr(args,'synthetic_provider',None)
                if invoke is not None: result = invoke(number,body,min(cap['timeout_seconds'],remaining))
                elif a['model']==LUNA: result = transport.invoke(body,min(cap['timeout_seconds'],remaining),expected_model=LUNA,payload_limit=a['max_payload_bytes'])
                else: result = transport.invoke(body,min(cap['timeout_seconds'],remaining))
                # Receipt is persisted even when schema validation fails; arbitrary/raw text is never stored.
                value = result.get('value'); receipt = result['receipt']
                try: transport.validate(value,schema)
                except (ValueError,TypeError,KeyError): value = None
                save(outcome_path,{'intent_hash':digest(intent),'receipt':receipt,'value':value,'resource_stop':result.get('resource_stop') if result.get('resource_stop') in ('resource_overrun','unknown_usage') else None}); checkpoint('outcome')
            outcome = read(outcome_path)
            if outcome['intent_hash'] != digest(intent): raise ValueError('outcome binding mismatch')
            if outcome.get('resource_stop'): return finish(outcome['resource_stop'])
            receipt = outcome['receipt']; value = outcome['value']
            if receipt.get('status') != 'settled': return finish('provider_unknown')
            if receipt.get('usage_status') != 'reported': return finish('usage_unknown')
            usage = receipt['usage']
            if usage['completion_tokens'] > intent['completion_reserved'] or usage['prompt_tokens'] > intent['input_reserved']:
                return finish('provider_resource_limit')
            if value is None: return finish('invalid_or_truncated_output')
            transport.validate(value,schema)
            if value.get('state_root') != expected: return finish('stale_output')
            if 'inability' in value: return finish('generation_inability')
            if (a.get('stop_empty_initial_aspiration') and phase == 'relational' and number == 1
                    and not value.get('proposals')): return finish('empty_initial_aspiration')
            # Stop/expiry after response: leave settled outcome replayable, never infer permission.
            if (out/'STOP').exists() or (stop is not None and stop.exists()): return {'status':'operator_stop','state_root':current()['state_root']}
            if dt.datetime.now(dt.timezone.utc) >= instant(a['expires_at']): return finish('expired')
            increment = 0; controller_hash = None
            if phase in ('possibilities','possibilities_final'):
                try: d.validate_possibilities(state,value,{'prior_rejected_offers':[]})
                except ValueError: return finish('invalid_possibilities')
                pending = value; after = expected
            else:
                if not receipt_path.exists():
                    if current()['state_root'] != expected: return finish('drift_before_apply')
                    try:
                        normalized = d.normalize_forecasts(state,d.complete_assessment(state,pending,value)) if phase == 'assessment' else d.validate_relational_development(state,value)
                    except ValueError: return finish('invalid_assessment' if phase == 'assessment' else 'invalid_relational_development')
                    if recovery and load_reviewer().candidate_fingerprint(phase,normalized)==recovery['fingerprint']:return finish('semantic_recovery_no_progress')
                    if phase == 'assessment' and actions >= a['actions']: return finish('action_budget')
                    if a.get('runtime_review'):
                        pre=context(state,d,a.get('scenario','archive'));stable(out/f'review-state-{number}.json',pre)
                        subject={'candidate':normalized,'generator_outcome_sha256':digest(outcome),'generator_request':number}
                        verdict=review_gate(phase,number,pre,subject)
                        if verdict!='accept':
                            recovered=recovery_record(number,phase,verdict,normalized)
                            if recovered:
                                recovery=recovered;pending=None;reserved+=intent['completion_reserved'];wall+=intent['wall_reserved'];continue
                            return finish('review_'+verdict)
                        if current()['state_root']!=expected:return finish('state_drift_after_review')
                        if (out/'STOP').exists() or (stop is not None and stop.exists()):return finish('operator_stop')
                        if dt.datetime.now(dt.timezone.utc)>=instant(a['expires_at']):return finish('expired')
                    save(out/f'apply-value-{number}.json',normalized)
                    save(apply_path,{'state_root':expected,'outcome_hash':digest(outcome),'value_hash':digest(normalized),
                                     'storage_before':storage_inventory(directory)})
                    checkpoint('apply_intent')
                    try:
                        def apply(): return d.cli(directory,'deliberate' if phase == 'assessment' else 'direction-relate','--response',str(out/f'apply-value-{number}.json'))
                        guard = getattr(args,'synthetic_action',None)
                        result = guard(number,apply) if phase=='assessment' and guard is not None else apply()
                    except d.ControllerRefusal as error:
                        try:
                            after_state = current(); after_storage = storage_inventory(directory)
                            before_storage = read(apply_path)['storage_before']
                            if after_state['state_root'] != expected or before_storage != after_storage:
                                raise ValueError('refusal changed storage')
                            cr = {'before_root':expected,'after_root':expected,'outcome_hash':digest(outcome),
                                  'apply_hash':digest(read(apply_path)),'action_count':0,
                                  'storage_before':before_storage,'storage_after':after_storage,
                                  'result':{'status':'controller_refused','diagnostic':error.receipt}}
                            save(receipt_path,cr); checkpoint('controller_refusal')
                        except Exception:
                            save(out/f'application-failure-{number}.json',{'apply_hash':digest(read(apply_path)),
                                 'outcome_hash':digest(outcome),'diagnostic':error.receipt,
                                 'classification':'recognized_refusal_without_verified_unchanged_storage'})
                            return {'status':'uncertain_application','request':number,'reservation_retained':True,'state_root':current()['state_root']}
                        return finish('controller_refused')
                    except Exception as error:
                        diagnostic = getattr(error,'receipt',{'contract':'controller-failure@v1','kind':'unclassified_application_exception'})
                        save(out/f'application-failure-{number}.json',{'apply_hash':digest(read(apply_path)),
                             'outcome_hash':digest(outcome),'diagnostic':diagnostic})
                        return {'status':'uncertain_application','request':number,'reservation_retained':True,'state_root':current()['state_root']}
                    checkpoint('controller_returned')
                    after_state = current()
                    increment = after_state['cycles']-state['cycles']
                    if increment not in (0,1) or phase == 'relational' and increment != 0: raise ValueError('unexpected action count')
                    cr = {'before_root':expected,'after_root':after_state['state_root'],'outcome_hash':digest(outcome),
                          'apply_hash':digest(read(apply_path)),'result':result,'action_count':increment}
                    save(receipt_path,cr); checkpoint('controller_receipt')
                cr = read(receipt_path)
                if cr['outcome_hash'] != digest(outcome) or cr['apply_hash'] != digest(read(apply_path)):
                    raise ValueError('application receipt provenance mismatch')
                if cr.get('result',{}).get('status') == 'controller_refused':
                    verify_refusal(cr,read(apply_path),directory,expected)
                    return finish('controller_refused')
                if a.get('runtime_review') and phase=='assessment' and cr['action_count']==1:
                    after_context=context(current(),d,a.get('scenario','archive'))
                    native_turn=cr['result']['turn'];candidate=read(out/f'apply-value-{number}.json');before_context=read(out/f'review-state-{number}.json')
                    actual=native_turn['offered'][native_turn['chosen']]['direction']['action']
                    if native_turn['before']!=before_context['observation'] or native_turn['after']!=after_context['observation'] or actual not in [node['action'] for node in candidate['directions']]:raise ValueError('post review actual receipt mismatch')
                    subject={'candidate':candidate,'controller_receipt':cr,
                             'before':read(out/f'review-state-{number}.json'),'after':after_context,
                             'generator_outcome_sha256':digest(outcome),'generator_request':number}
                    verdict=review_gate('post_action',number,after_context,subject)
                    if verdict!='accept':
                        recovered=recovery_record(number,'post_action',verdict,candidate,cr)
                        if recovered:
                            recovery=recovered;pending=None;expected=cr['after_root'];actions+=cr['action_count'];reserved+=intent['completion_reserved'];wall+=intent['wall_reserved'];continue
                        return finish('review_'+verdict)
                controller_hash = digest(cr); after = cr['after_root']; increment = cr['action_count']; pending = None
            save(applied_path,{'intent_hash':digest(intent),'outcome_hash':digest(outcome),'controller_hash':controller_hash,
                              'after_root':after,'action_count':increment,'pending':pending,
                              **({'review_outcomes':applied_reviews(number,phase,increment)} if a.get('runtime_review') else {})})
            checkpoint('applied')
            expected = after; actions += increment; reserved += intent['completion_reserved']; wall += intent['wall_reserved']
            if recovery and recovery['kind']=='post_action' and phase=='relational':return finish('semantic_recovery_reflection_complete')
            if phase == 'possibilities_final': return finish('proposal_only_complete')
            if a.get('continuation') and phase=='relational' and actions==1:
                return finish('action_budget_review_complete')
        return finish('call_budget')
