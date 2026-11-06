#!/usr/bin/env python3
"""Bounded static-project projection/binding for the existing native kernel.

No network, execution, admission mutation, automatic acceptance, or live launcher.
The origin and source snapshots must be independently admitted by the caller. An
origin is an export allowlist, never proof that arbitrary local data is public.
"""
import argparse
import hashlib
import copy
import importlib.util
import json
from pathlib import Path

def load(name, filename):
    s=importlib.util.spec_from_file_location(name,Path(__file__).with_name(filename));m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
base=load('web_shared_adapter','engine-project-cycle-adapter.py')
e=base.e

# Shared by generator and independent reviewer; describes existing orchestration,
# not new authority or a preferred design. Applicable to initial and later sessions.
SPAN_LIMIT=1048576  # prospective snapshot memory resource budget, not a task-specific edit cap
NATIVE_RESPONSE_LIMIT=131072
RESPONSIBILITIES={
    'version':'proposal-responsibilities@v2',
    'generator':'Return proposed alternatives, not executed results. You may propose an untested initial design from the public brief, active wants and current source. Prior test returns or player feedback are not prerequisites for an initial edit proposal. Optional provisional wanting may arise from current contributors and possibilities without a corrective return; explain its connection and keep fulfillment unknown. Do not attribute generated wanting to the user. You do not invoke native preview yourself.',
    'controller':'After you return a candidate, the controller automatically validates and previews it against the current native state, obtains separate semantic review, and performs execution only within existing authorization. A candidate is not acceptance or execution authority.',
    'review_stage':'Independent proposal review assesses whether a complete incremental edit is useful and safe for the current stage; it need not finish every unresolved aim in one proposal. Review evaluates advisory development separately: provisional wanting needs a coherent contributor connection, preserved obligations and honest uncertainty, not demonstrated improvement or a witnessed defect. Retaining the artifact does not rule out development; retaining both remains valid. Review must still reject misleading claims, regressions or unsupported actions. Proposal approval is not final acceptance; unresolved aims remain pending.',
    'evidence':'Use supplied observations and attributed feedback when present. Distinguish design hypotheses from observed behavior. Do not invent tests, player feedback, satisfaction, or successful execution. Relationship revisions still require their supplied source-bound return contract.',
    'abstention':'Abstention remains valid when no useful permitted proposal is justified or necessary information is unavailable; no particular design, action, novelty, or relationship is required.'}

def narrative_capacity(request):
    value=request['coding']['relational'].get('narrative_capacity')
    if value!={'profile':'browser-aggregate-text@v1','aggregate_response_bytes':NATIVE_RESPONSE_LIMIT,'per_field':'nonblank; no additional field-length cap'}:raise ValueError('native browser narrative profile mismatch')
    return value['aggregate_response_bytes']

def owns_relationship_history(record, admitted_hashes):
    """Match immutable imported provenance; forward native-derived status as-is.

    Current/superseded/withdrawn are computed by the trusted native controller.
    Owning a historical record does not make it eligible guidance or authority.
    Admission, binding, proposal and all other fields must match exactly.
    """
    if e.sha(record) in admitted_hashes:return True
    flags=[k for k in ('current','superseded','withdrawn') if k in record]
    if not flags or any(type(record[k]) is not bool for k in flags):return False
    import itertools
    return any(e.sha({**record,**dict(zip(flags,values))}) in admitted_hashes
               for values in itertools.product((False,True),repeat=len(flags)))

def evidence_applicability(snapshot, returns, history):
    """Source identities are facts; historical narrative is never a new test receipt."""
    return {'version':'browser-evidence-epochs@v1','current_source_snapshot':snapshot,
        'native_returns':[{'receipt':r.get('receipt'),'source_snapshot':r.get('snapshot'),
            'applicability':'current_source' if r.get('snapshot')==snapshot else 'historical_source' if r.get('snapshot') else 'source_unknown'} for r in returns],
        'relationship_records':[{'admission':h.get('admission'),'binding':h.get('binding'),
            'current_guidance':bool(h.get('current')),'observation_authority':False,
            'embedded_test_claims':'historical_attribution_only; exact tested source must come from the original receipt or assessment, never this record being current'} for h in history],
        'rule':'A relationship revision carries meaning and contributing history, not fresh test results. Its current flag does not make embedded assessments current. Do not say historical checks passed on this source. Name the tested source explicitly; if unavailable say historical source unspecified. Only matching current-source receipts establish current execution. A code change invalidates transfer of test results, even if small. Reviewer: reject unsupported current-source passing claims; use unknown for ambiguous source attribution. Retain history without upgrading its evidence.'}

def project_assessment_brief(brief, coding):
    """Refresh host-owned assessment metadata, never its attributed report text.

    Older UI admissions embedded JSON records in the public brief. Keep that wire
    format compatible; decode complete JSON values, and change only marked
    assessment records. The frozen admission and original report remain intact.
    """
    import hashlib
    files={f['path']:hashlib.sha256(f['source'].encode()).hexdigest()
           for f in coding['files'] if f['editable']}
    def visit(value):
        if isinstance(value,list):return [visit(v) for v in value]
        if not isinstance(value,dict):return value
        if value.get('record_type')=='independent_assessment':
            text=value.get('text');binding=value.get('source_snapshot') or {}
            if not isinstance(text,str) or hashlib.sha256(text.encode()).hexdigest()!=value.get('report_sha256'):
                raise ValueError('Independent assessment report digest mismatch')
            snapshot=binding.get('native_snapshot')
            matched=snapshot==coding['snapshot']
            if binding.get('editable_sha256') is not None:matched=matched and binding['editable_sha256']==files
            if binding.get('artifact_sha256') is not None:matched=matched and binding['artifact_sha256']==files
            if binding.get('artifact_matches_native') is False:matched=False
            return {**value,'applicability':'current_source' if matched else 'historical_source_not_current_validation' if snapshot else 'source_unknown',
                    'applicability_checked_against':coding['snapshot']}
        return {k:visit(v) for k,v in value.items()}
    # Only the UI's serialized event arrays are metadata. JSON examples inside
    # user prose or report text are not interpreted as assessment records.
    marker='Recorded input:\n';decoder=json.JSONDecoder();out=[];start=0;i=0
    while True:
        marker_at=brief.find(marker,i)
        if marker_at<0:break
        i=marker_at+len(marker)
        try:value,end=decoder.raw_decode(brief,i)
        except json.JSONDecodeError:continue
        if not isinstance(value,list):i=end;continue
        updated=[visit(v) if isinstance(v,dict) and v.get('record_type')=='independent_assessment' else v for v in value]
        if updated!=value:
            out.extend((brief[start:i],json.dumps(updated,ensure_ascii=False)));start=end
        i=end
    out.append(brief[start:]);return ''.join(out)

def project(request, origin, source_view=None, context_budget=base.capacity.PAYLOAD_BYTES-65536):
    """Same projection for generator and reviewer; no clipping to fit a request."""
    if set(origin)-{'public_pursuits','public_meanings','public_preferences','public_supplemental'}!={'version','scope','requirements_revision','parents','protected','editable','public_brief','source_snapshots','public_relationships','public_returns'} or origin['version']!='public-browser-origin@v1':raise ValueError('exact reviewed public origin required')
    coding=request['coding'];inputs=coding['relational']['inputs']
    if coding['kind']!='browser_project@v1' or coding.get('supporting_context') or coding.get('information') or inputs.get('service_returns') or inputs.get('information_returns'):raise ValueError('unsupported context')
    if inputs['requirements_revision']!=origin['requirements_revision'] or inputs.get('acceptance_parents',inputs['parents'])!=origin['parents'] or request['next_attention']['scope']!=origin['scope']:raise ValueError('scope/parent drift')
    if coding['snapshot'] not in origin['source_snapshots']:raise ValueError('source snapshot not admitted/generated in this journal')
    for key in ('state_root','source_snapshot'):
        expected=request['state_root'] if key=='state_root' else coding['snapshot']
        if request['next_attention'][key]!=expected:raise ValueError('stale attention')
    isolation=coding.get('browser_isolation')
    if not isinstance(isolation,dict) or set(isolation)!={'chromium_sandbox','outer_container_required'} or type(isolation['chromium_sandbox']) is not bool or isolation['outer_container_required'] is not True:raise ValueError('explicit browser isolation required')
    files=coding['files'];editable=[f for f in files if f['editable']]
    if not 1<=len(editable)<=8 or sorted(f['path'] for f in editable)!=sorted(origin['editable']) or len({f['path'] for f in files})!=len(files):raise ValueError('file inventory')
    if {f['path']:f['source_digest'] for f in files if not f['editable']}!=origin['protected']:raise ValueError('protected identity changed')
    if any('source' in f for f in files if not f['editable']):raise ValueError('protected source leak')
    if any(not isinstance(f.get('source'),str) or not 0<len(f['source'].encode())<=1048576 for f in editable) or sum(len(f['source'].encode()) for f in editable)>1048576:raise ValueError('visible source bound')
    if not isinstance(origin['public_brief'],str) or not 0<len(origin['public_brief'].encode())<=base.capacity.PAYLOAD_BYTES:raise ValueError('public brief exceeds provider input capacity')
    if any(e.sha(r) not in origin['public_returns'] for r in inputs.get('returns',[])):raise ValueError('unadmitted receipt export')
    if any(e.sha(r) not in origin.get('public_supplemental',[]) for r in inputs.get('supplemental',[])):raise ValueError('unadmitted supplemental receipt export')
    development=load('browser_desire','engine-browser-desire.py')
    meanings=development.history_snapshot(inputs)
    if any(e.sha(h) not in origin.get('public_meanings',[]) for h in (meanings['history'] if meanings is not None else [])):raise ValueError('unadmitted public meaning export')
    if any(e.sha(h) not in origin.get('public_preferences',[]) for h in inputs.get('generated_preferences',[])):raise ValueError('unadmitted generated preference export')
    history=coding['relational'].get('history',[])
    if any(e.sha(r) not in origin['public_relationships'] for r in history):raise ValueError('unadmitted relationship export')
    context={'proposal_responsibilities':copy.deepcopy(RESPONSIBILITIES),'browser_isolation':isolation,'relational_history':history,'browser_returns':[{k:r[k] for k in ('receipt','snapshot','verdict','observations') if k in r} for r in inputs.get('returns',[])],'version':'public-browser-project@v1','public_brief':project_assessment_brief(origin['public_brief'],coding),'parents':inputs['parents'],'acceptance_parents':origin['parents'],
        'files':[{k:f[k] for k in ('path','source_digest','inspection_handle')}|{'bytes':len(f['source'].encode()),'source_sha256':hashlib.sha256(f['source'].encode()).hexdigest(),'source_omitted':True} for f in editable],
        'binding':{'state_root':request['state_root'],'source_snapshot':coding['snapshot'],'attention':request['next_attention']['binding']},
        'case_ids':[c['id'] for c in coding['cases']],
        'constraints':{'protected_source':'withheld, immutable','edits':'1..4 exact spans in distinct admitted files. No additional application edit-size cap. Browser snapshot memory budget is 1048576 bytes total; provider requests follow the explicitly admitted model-context/token estimate; the admitted completion allocation is reported separately and includes reasoning. Use scoped inspect and incremental edits; full code is never truncated or rewritten; controller performs native preview after receiving the proposal','execution':'protected sealed browser harness only, no shell or package authority','observations':'DOM checks are bounded functionality evidence, not player satisfaction. Render hashes are not visual review.'},
        'relational_contract':e.relational_contract(request),'return_evidence':e.return_evidence.catalog(request),
        'response_envelope':{'instruction':'The generator response is {alternatives:[candidate,...]}, {deliberation:{artifact:one artifact candidate or {retain:reason},development:one advisory candidate or null,artifact_dependency:independent|requires_development}}, {retain:reason} (optionally with development_declined:reason, recorded when given, never required), {abstain:reason}, or {inspect:{path,offset}}. Independent reviewers return their separate judgment contract, never inspect. edits/rationale are candidate fields, never top-level fields. Each candidate matches one of response_contracts. Byte limits count UTF-8, not characters. resulting_file in diagnostics means the complete file after applying that candidate, not a response field.','max_alternatives':4,'retention':'{retain:reason} is a current-condition no-change choice for artifact and meaning (optionally with development_declined:reason, never required); {abstain:reason} stops further work. Neither proves fulfillment or acceptance.'},
        'response_contracts':{'edits_or_abstention':response_schema(origin['editable']),'relational':e.payload_contract(request,'final_proposal_only',narrative_bytes=narrative_capacity(request),relational_bytes=NATIVE_RESPONSE_LIMIT)['relational'],'relational_envelope':'{relational: proposal, return_basis: required for revision; initial may omit}','return_basis':{'semantics':e.return_evidence.contract(narrative_bytes=narrative_capacity(request)),'schema':e.return_evidence.schema(request,narrative_bytes=narrative_capacity(request))},'edit_layout':'At most one exact old_text/new_text span per path in one proposal. To change several parts of a file, generate one encompassing exact span (the complete current file is allowed), or separate complete incremental proposals. Do not repeat a file path.', 'existing':'{existing:{action: exact listed verification ID, rationale: nonblank narrative within aggregate response envelope}, expectation?: same optional qualitative expectation as edit candidates}'},'existing_verification_actions':verification_actions(request)}
    context['public_desire_history']=development.public_history(inputs)
    context['response_contracts']['public_desire']=development.contract()
    context['response_contracts']['deliberation']=load('browser_deliberation_contract','engine-browser-deliberation.py').contract()
    if inputs['parents']:context['response_contracts']['provisional_preference']=load('browser_provisional_expansion','engine-browser-expansion.py').provisional_contract()
    context['generated_preferences']=copy.deepcopy(inputs.get('generated_preferences',[]))
    statuses=inputs.get('generated_preference_statuses',[])
    if not isinstance(statuses,list):raise ValueError('generated preference status projection')
    if statuses:
        ids=[x['id'] for x in inputs.get('generated_preferences',[])]
        allowed={'open','fulfilled','superseded','withdrawn','overlapping','out_of_scope','not_admitted'}
        if not isinstance(statuses,list) or any(not isinstance(x,dict) or set(x)!={'id','status'} or not isinstance(x['status'],str) or x['status'] not in allowed for x in statuses) or [x['id'] for x in statuses]!=ids:raise ValueError('generated preference status projection')
    context['generated_preference_statuses']=copy.deepcopy(statuses)

    if inputs.get('current_success'):
        context['current_success']=copy.deepcopy(inputs['current_success'])
        context['response_contracts']['success_preference']=load('browser_success_expansion','engine-browser-expansion.py').success_contract()
    context['current_contrast']=copy.deepcopy(inputs.get('current_contrast'))
    if inputs.get('current_contrast'):context['response_contracts']['contrast_preference']=load('browser_expansion','engine-browser-expansion.py').contract()
    context['pursuit_expectation_contract']='An executable candidate may include expectation:{parents:[contributing IDs],anticipated:observable consequence of this pursuit,grounds:why this outcome is expected,uncertainty:what remains unknown}. This freezes a hypothesis before action; it is neither observed evidence nor a utility score. Reviewed return interpretations will be supplied to later choices. Retention, uncertainty and unchanged interpretations remain valid; no new desire or relationship is required.'
    context['response_contracts']['relational']['required']['catalysts']='Optional distinct revision aliases from relational_contract.catalyst_references (at most 4), or []. Preserve cited parents. Historical inspiration only; independent review required, no new identity or authority.'
    context['source_identity_contract']='source_sha256 is raw SHA-256 of the FULL UTF-8 file, including omitted windows. source_digest is native BLAKE3(length-prefixed UTF-8). Compare report SHA-256 only to source_sha256. Identity is not test success or user acceptance.'
    if inputs.get('supplemental'):context['supplemental_returns']=copy.deepcopy(inputs['supplemental'])
    context['evidence_applicability']=evidence_applicability(coding['snapshot'],inputs.get('returns',[]),history)
    context['source_access']={'instruction':'File contents are explicit UTF-8 byte windows, not complete files unless complete=true. To inspect another window return only {inspect:{path:one editable path,offset:UTF-8 byte offset}}. This uses a counted model request, not execution authority. Paths/protected content remain fixed. Old text must match exactly; never infer omitted code. Reviewer: identities do not reveal omitted source; return unknown with limitations if unseen code is needed for a criterion.','current_view':source_view}
    ordered=list(editable)
    if source_view:
        if set(source_view)!={'path','offset'} or source_view['path'] not in origin['editable'] or type(source_view['offset']) is not int:raise ValueError('unadmitted source view')
        ordered.sort(key=lambda f:f['path']!=source_view['path'])
    # The view budget is transport headroom, not a file/edit limit. Every omission
    # has byte ranges; the model may request a different range without code repair.
    for f in ordered:
        raw=f['source'].encode();offset=source_view['offset'] if source_view and f['path']==source_view['path'] else 0
        if not 0<=offset<=len(raw):raise ValueError('source offset outside file')
        try:raw[:offset].decode()
        except UnicodeDecodeError:raise ValueError('source offset is not a UTF-8 boundary')
        target=next(x for x in context['files'] if x['path']==f['path'])
        lo=0;hi=len(raw)-offset
        while lo<hi:
            n=(lo+hi+1)//2;piece=raw[offset:offset+n].decode('utf-8','ignore');end=offset+len(piece.encode())
            target.update(source=piece,source_omitted=False,window={'start':offset,'end':end,'complete':offset==0 and end==len(raw),'next_offset':end if end<len(raw) else None})
            if len(e.canonical(context))<=context_budget:lo=n
            else:hi=n-1
        piece=raw[offset:offset+lo].decode('utf-8','ignore');end=offset+len(piece.encode())
        if piece:target.update(source=piece,source_omitted=False,window={'start':offset,'end':end,'complete':offset==0 and end==len(raw),'next_offset':end if end<len(raw) else None})
        else:
            for key in ('source','window'):target.pop(key,None)
            target['source_omitted']=True
    # Existing relational context is included only through the shared allowlisted contract.
    # Review history and full native state are not copied by this projection.
    if len(e.canonical(context))>max(28672,context_budget):raise ValueError('complete public context exceeds request headroom; no truncation')
    return context

def verification_actions(request):
    return [x['id'] for x in request.get('actions',[]) if x.get('id','').startswith('verify-') and x.get('description')=='Verify the admitted current source without editing it']

def response_schema(paths):
    text={'type':'string','minLength':1,'maxLength':NATIVE_RESPONSE_LIMIT}
    edit={'type':'object','additionalProperties':False,'required':['path','old_text','new_text'],'properties':{'path':{'type':'string','enum':paths},'old_text':{'type':'string','minLength':1,'maxLength':SPAN_LIMIT},'new_text':{'type':'string','maxLength':SPAN_LIMIT}}}
    expectation={'type':'object','additionalProperties':False,'required':['parents','anticipated','grounds','uncertainty'],'properties':{'parents':{'type':'array','minItems':1,'uniqueItems':True,'items':text},'anticipated':text,'grounds':text,'uncertainty':text}}
    return {'oneOf':[{'type':'object','additionalProperties':False,'required':['abstain'],'properties':{'abstain':text}}, {'type':'object','additionalProperties':False,'required':['edits','rationale'],'properties':{'edits':{'type':'array','minItems':1,'maxItems':4,'items':edit},'rationale':text,'expectation':expectation}}]}

def bind_edit(request, value, preview):
    """No rewriting of model code: bind immutable metadata then native validation."""
    shape=load('web_shape','engine-project-output.py')
    limit=narrative_capacity(request)
    def aggregate():
        actual=len(e.canonical(value))
        if actual>NATIVE_RESPONSE_LIMIT:shape.fail('$','representation_size',max_utf8_bytes=NATIVE_RESPONSE_LIMIT,actual_utf8_bytes=actual)
    files={f['path']:f for f in request['coding']['files'] if f['editable']}
    if isinstance(value,dict) and 'expectation' in value:
        if not ('edits' in value or 'existing' in value):raise ValueError('pursuit expectation requires an executable possibility')
        x=value['expectation'];schema=response_schema(list(files))['oneOf'][1]['properties']['expectation']
        shape.check(x,schema,'$.expectation')
        parents={p['id'] for p in request['coding']['relational']['inputs']['parents']}
        if not set(x['parents'])<=parents:raise ValueError('unknown expectation contributor')
        part=bind_edit(request,{k:v for k,v in value.items() if k!='expectation'},lambda _:None)
        part['pursuit_expectations']={'0':copy.deepcopy(x)}
        aggregate();preview(part);return part

    if isinstance(value,dict) and set(value)=={'provisional_preference'}:
        response=load('browser_provisional_expansion','engine-browser-expansion.py').bind_provisional(request,value['provisional_preference'])
        aggregate();preview(response);return response
    if isinstance(value,dict) and set(value)=={'success_preference'}:
        response=load('browser_success_expansion','engine-browser-expansion.py').bind_success(request,value['success_preference'])
        aggregate();preview(response);return response
    if isinstance(value,dict) and set(value)=={'contrast_preference'}:
        response=load('browser_expansion','engine-browser-expansion.py').bind(request,value['contrast_preference'])
        aggregate();preview(response);return response
    if isinstance(value,dict) and set(value)=={'public_desire'}:
        response=load('browser_desire','engine-browser-desire.py').bind(request,value['public_desire'])
        aggregate();preview(response);return response
    if isinstance(value,dict) and set(value)=={'existing'}:
        item=value['existing']
        if not isinstance(item,dict) or set(item)!={'action','rationale'} or item['action'] not in verification_actions(request) or not isinstance(item['rationale'],str) or not 0<len(item['rationale'].encode())<=limit:raise ValueError('unadmitted verification')
        response={'protocol':request['protocol'],'state_root':request['state_root'],'attention_binding':request['next_attention']['binding'],'proposals':[{'kind':'existing',**item}]}
        aggregate();preview(response);return response
    if isinstance(value,dict) and 'relational' in value:
        if set(value)-{'relational','return_basis'}:raise ValueError('relational envelope')
        e.validate_relational_lineage(request,value['relational'],'open_choice')
        if value['relational'].get('revises') is not None or 'return_basis' in value:e.return_evidence.validate(value.get('return_basis'),request,narrative_bytes=limit)
        # Same published native project contract; expose only structural text errors.
        # Check lineage/return/catalyst authority first, never coach a forged binding.
        texts={field:value['relational'].get(field) for field in ('relationship','aspiration','discriminator')}
        development=value['relational'].get('development')
        if isinstance(development,dict):
            texts.update({f'development.{k}':development.get(k) for k in ('retained_meaning','return_reveals','unresolved')})
            if development.get('proposed_development') is not None:texts['development.proposed_development']=development['proposed_development']
        for field,text in texts.items():
            if isinstance(text,str) and len(text.encode())>limit:shape.fail('$.relational.'+field,'text_bound',max_utf8_bytes=limit,actual_utf8_bytes=len(text.encode()),nonblank=True)
            shape.check(text,{'type':'string','maxLength':limit},'$.relational.'+field)
        response=e.local.bind_proposal(request,json.dumps({'relational':value['relational']}))
        aggregate();preview(response);return response
    schema=response_schema(list(files))['oneOf'][0 if isinstance(value,dict) and set(value)=={'abstain'} else 1]
    checked=copy.deepcopy(schema)
    if 'edits' in checked.get('properties',{}):
        for key in ('old_text','new_text'):checked['properties']['edits']['items']['properties'][key]={}
    shape.check(value,checked)
    if 'edits' in value and any(not isinstance(p[k],str) for p in value['edits'] for k in ('old_text','new_text')):raise ValueError('patch text must be strings')
    if 'abstain' in value:
        aggregate();return value
    edits=value['edits'];patches=[]
    if len(value['rationale'].encode())>limit:raise ValueError('aggregate narrative bound')
    # Validate identity/exact matching before allowing any size regeneration.
    if len({p['path'] for p in edits})!=len(edits):shape.fail('$.edits','unique_file_edits')
    missing=[]
    for p in edits:
        source=files[p['path']]
        if not p['old_text'] or p['old_text']==p['new_text']:raise ValueError('nonempty changed edit required')
        matches=sum(source['source'].startswith(p['old_text'],j) for j in range(len(source['source'])))
        if matches>1:raise ValueError('ambiguous edit span; no fresh-choice recovery')
        if matches==0:missing.append({'path':p['path'],'current_source_digest':source['source_digest'],'inspection_handle':source['inspection_handle'],'old_text_sha256':e.sha(p['old_text']),'match_count':0})
        patches.append({**p,'source_digest':source['source_digest'],'inspection_handle':source['inspection_handle']})
    if missing:
        error=e.StageValidationError('proposal','stale_edit_span');error.details=missing;raise error
    sizes=[]
    for i,p in enumerate(edits):
        for key in ('old_text','new_text'):
            sizes.append((f'$.edits[{i}].{key}',len(p[key].encode()),SPAN_LIMIT))
        result=files[p['path']]['source'].replace(p['old_text'],p['new_text'],1)
        sizes.append((f'$.edits[{i}].resulting_file',len(result.encode()),1048576))
    response={'protocol':request['protocol'],'state_root':request['state_root'],'attention_binding':request['next_attention']['binding'],'proposals':[{'kind':'edit_set','patches':patches,'rationale':value['rationale']}]}
    sizes.append(('$',len(e.canonical(response)),NATIVE_RESPONSE_LIMIT))
    for path,actual,limit in sizes:
        if actual>limit:shape.fail(path,'representation_size',max_utf8_bytes=limit,actual_utf8_bytes=actual)
    aggregate();preview(response)
    return response

def checkpoint(request, completion, budget):
    if request.get('pending') or budget.get('unsettled_requests') or budget.get('unresolved_tool_requests'):raise ValueError('uncertain or pending effects; do not checkpoint for replay')
    return {'version':'browser-checkpoint@v1','state_root':request['state_root'],'snapshot':request['coding']['snapshot'],
        'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],
        'budget_sha256':e.sha(budget),'authority':'unchanged; closed run allowances never reopen','next':'wait_for_actual_user_feedback_or_fresh_admission'}

def feedback_draft(checkpoint, request, completion, message_id, guidance, *, attribution="verified_user_input"):
    # Operator intake only. message_id is attribution, not cryptographic authentication.
    if attribution not in ('verified_user_input','synthetic_harness_verification'):raise ValueError('feedback attribution')
    if not isinstance(message_id,str) or not 0<len(message_id.encode())<=256:raise ValueError('actual user message attribution required')
    if checkpoint['state_root']!=request['state_root'] or checkpoint['snapshot']!=request['coding']['snapshot'] or checkpoint['requirements_revision']!=completion['requirements_revision'] or checkpoint['decision_record']!=completion['decision_record']:raise ValueError('stale checkpoint')
    inputs=request['coding']['relational']['inputs']
    authority=inputs.get('acceptance_parents',inputs['parents'])
    allowed={p['requirement'] for p in authority}
    if not isinstance(guidance,dict) or not guidance or not set(guidance)<=allowed or any(not isinstance(v,str) or not 0<len(v.encode())<=narrative_capacity(request) for v in guidance.values()):raise ValueError('explicit public current-aim guidance required')
    retained={p['requirement']:p['meaning'] for p in authority}
    retained.update(guidance)
    if any(not isinstance(v,str) or not 0<len(v.encode())<=narrative_capacity(request) for v in retained.values()):raise ValueError('current public meanings exceed feedback bounds')
    return {'retained_contributors':sorted(allowed-set(guidance)),'updated_contributors':sorted(guidance),'attribution':{'source':'user' if attribution=='verified_user_input' else 'authored_synthetic_scenario','message_id':message_id,'verification_required':'Verify exact admitted attribution before publication; synthetic guidance is not user testimony or observed behavior. This draft is not authentication'},
        'native_feedback':{'requirements_revision':completion['requirements_revision'],'decision_record':completion['decision_record'],'guidance':retained},
        'effect':'draft only; publish through existing native acceptance feedback API. No acceptance, desire deletion, criteria or authority change.'}

def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('request',type=Path);p.add_argument('origin',type=Path);a=p.parse_args()
    print(json.dumps(project(e.strict(e.bounded_file(a.request)),e.strict(e.bounded_file(a.origin))),indent=2))
if __name__=='__main__':main()
