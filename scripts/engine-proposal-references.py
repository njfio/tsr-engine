"""Prospective request-local proposal references; no output repair or authority.

project returns a copied wire context containing proposal_references. expand takes
that exact catalog plus the captured native request; callers must additionally
verify the live source and authenticated request/response association. Only typed
reference fields are translated. Code, narrative, judgments and evidence choices
are never rewritten. Native validation and independent review still follow.
"""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path

PROFILE='browser-proposal-references@v1'
_spec=importlib.util.spec_from_file_location('proposal_native_envelope',Path(__file__).with_name('engine-b91-project-envelope.py'))
native=importlib.util.module_from_spec(_spec);_spec.loader.exec_module(native)

def digest(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False,allow_nan=False).encode()).hexdigest()

def catalog(request):
    contract=native.relational_contract(request)
    parents=contract['parent_ids']
    history=request['coding']['relational'].get('history',[])
    revisions=[item['admission'] for item in history]
    sources=native.return_evidence.catalog(request)['sources']
    paths=[item['path'] for item in sources]
    if len(parents)!=len(set(parents)) or len(revisions)!=len(set(revisions)) or len(paths)!=len(set(paths)):
        raise ValueError('ambiguous proposal reference inventory')
    return {'version':PROFILE,'request_sha256':digest(request),
            **({'successes':{'s0':request['coding']['relational']['inputs']['current_success']['id']}} if request['coding']['relational']['inputs'].get('current_success') else {}),
            'contrasts':({'c0':request['coding']['relational']['inputs']['current_contrast']['id']} if request['coding']['relational']['inputs'].get('current_contrast') else {}),
            'parents':{'p'+str(i):value for i,value in enumerate(parents)},
            'revisions':{'r'+str(i):value for i,value in enumerate(revisions)},
            'meaning_judgments':({'j0':request['coding']['relational']['inputs']['semantic_return']['review']} if request['coding']['relational']['inputs'].get('semantic_return',{}).get('applicable') is True else {}),'desire_revisions':{'d'+str(i):p['revision'] for i,p in enumerate(request['coding']['relational']['inputs']['parents']) if p.get('revision')},'observation_receipts':{'o'+str(i):r['receipt'] for i,r in enumerate(request['coding']['relational']['inputs'].get('returns',[]))},'inference_paths':paths,'host_owned':['return_basis.binding','return_basis.inferred[].digest'],
            'instruction':'Choose exact request-local parent/revision aliases, never long identities. Omit return_basis.binding and inferred[].digest; trusted transport binds them to this request and selected current source path. This is representation only, not evidence or new authority.'}

def _translate(value,mapping,field):
    if not isinstance(value,str) or value not in mapping:raise ValueError('unknown or stale '+field+' reference')
    return mapping[value]

def _relation(value,parents,revisions):
    if not isinstance(value,dict):raise ValueError('relational proposal object required')
    if 'binding' in value:raise ValueError('model supplied host relational binding')
    result=copy.deepcopy(value)
    ids=result.get('parents')
    if not isinstance(ids,list) or any(not isinstance(x,str) for x in ids) or len(ids)!=len(set(ids)):raise ValueError('distinct parent reference array required')
    result['parents']=[_translate(x,parents,'parent') for x in ids]
    if 'catalysts' in result:
        if not isinstance(result['catalysts'],list) or any(not isinstance(x,str) for x in result['catalysts']) or len(set(result['catalysts']))!=len(result['catalysts']):raise ValueError('distinct catalyst references required')
        result['catalysts']=[_translate(x,revisions,'catalyst') for x in result['catalysts']]
    if result.get('revises') is not None:result['revises']=_translate(result['revises'],revisions,'revision')
    development=result.get('development')
    if development is not None:
        if not isinstance(development,dict):raise ValueError('development object required')
        for field in ('original_revision','prior_revision'):
            if field in development:development[field]=_translate(development[field],revisions,field)
    return result

LEGACY_HOST_BRIEF_SHA256='2981fccadca017caf54f89bcf2562b45940d88ebfe75735f43e4f6abbe198a7b'
LEGACY_OPTIONS='Edits, initial or developed relationships, supported Contrast-derived preferences, retention and abstention are valid.'

# Forms that develop meaning rather than keep or change the artifact.
DEVELOPMENT_FORMS=('relationship_initial','relationship_revision','meaning_development','provisional_preference','contrast_preference','success_preference')
RETAIN_PLAIN='Keep the current artifact and meaning unchanged for this checkpoint: {retain:reason}. Not fulfillment or acceptance.'
RETAIN_WITH_DECISION=('Keep the current artifact and meaning unchanged for this checkpoint: {retain:reason, development_declined:reason}. '
    'development_declined names the development forms listed here that you considered and why none is worth proposing now; '
    'it is recorded with the retention. Not fulfillment or acceptance.')

def retain_form(forms):
    """Retention costs the same decision as development whenever development is offered."""
    return RETAIN_WITH_DECISION if any(k in forms for k in DEVELOPMENT_FORMS) else RETAIN_PLAIN

def development_forms(summary):
    """Development forms a wire choice_summary offers (empty when absent or not a dict)."""
    forms=summary.get('forms') if isinstance(summary,dict) else None
    return sorted(k for k in (forms or {}) if k in DEVELOPMENT_FORMS)

def choice_summary(value,request,references):
    """Describe proposal forms, not a second semantic admission decision."""
    contracts=value['response_contracts'];relation=value['relational_contract'];unavailable={};eligible={}
    if value.get('advisory_reconsideration') and value.get('version')!='public-browser-project@v1':
        forms={'abstain': 'Stop this reconsideration without changing desires.',
               'relationship_revision': 'Only the supplied existing lineage; source-bound return and independent review required.'}
    else:
        forms={'abstain':'Stop this session without changing desires or granting acceptance.',
               'retain':'Keep the current artifact and meaning unchanged for this checkpoint; not fulfillment or acceptance.'}
        if 'edits_or_abstention' in contracts and not value.get('advisory_reconsideration'):forms['edit']='Propose an incremental artifact change; native preview, independent review and execution authorization still required.'
        if value.get('files') and not value.get('advisory_reconsideration'):forms['inspect']='Read another admitted source window; no new access.'
        if value.get('existing_verification_actions') and not value.get('advisory_reconsideration'):forms['verify']='Propose one listed existing verification action under its current authorization.'
        if 'relational' in contracts and len(relation['parent_ids'])>=2:forms['relationship_initial']='Propose how current contributors relate and an aspiration; independent review, no new identity or permission.'
        if relation['allowed_lineage']:forms['relationship_revision']='Develop a listed lineage using its return contract; preserve contributors; independent review required.'
        if 'public_desire' in contracts and value.get('public_meaning_references',{}).get('eligible_contributors',value.get('public_meaning_references',{}).get('eligible_originals')):forms['meaning_development']='Propose clarification/development of an eligible contributor identity; preserve history; independent review, not a new identity.'
        if 'provisional_preference' in contracts and relation['parent_ids']:forms['provisional_preference']='Optionally propose generated wanting from current contributors, with honest uncertain fulfillment. Independent review; no success/Contrast prerequisite, invented user preference or action authority.'
        for kind,evidence in (('contrast_preference','current_contrast'),('success_preference','current_success')):
            if kind in contracts and value.get(evidence):
                choices=request['coding']['relational'].get('identity_choices',{}).get(kind,[])
                eligible[kind]=[a for a,i in references['revisions'].items() if i in choices]
                if eligible[kind]:forms[kind]='Propose a new identity from one of eligible_identity_aspirations and the named evidence. Independent semantic review remains required; structural eligibility is not admission or authority.'
                else:unavailable[kind]='No current reviewed aspiration passes this identity route’s contributor, evidence-scope and duplicate checks. Existing evidence remains available; a relationship alone cannot supply missing success or Contrast.'
    if 'deliberation' in contracts:forms['deliberation']='Artifact retention (or an edit during ordinary choice) plus optional independently reviewed development; explicit dependency and separate review/admission.'
    # Say why a development route is absent, so a run without one is diagnosable
    # rather than read as the model declining it.
    unavailable_development={}
    if not value.get('advisory_reconsideration'):
        parents=relation['parent_ids']
        if 'provisional_preference' not in forms:
            unavailable_development['provisional_preference']=('No current contributor (parents) is supplied, so generated wanting has nothing to connect to.' if not parents
                else 'The provisional wanting contract is not supplied for this request.')
        if 'relationship_initial' not in forms and 'relational' in contracts:
            unavailable_development['relationship_initial']='A relationship needs at least two current contributors; '+str(len(parents))+' supplied.'
    if 'retain' in forms:forms['retain']=retain_form(forms)
    # Native guidance is already a source-bound fold. Merely having a reference
    # is not proof of contributor coverage, novelty or identity admissibility.
    current={x['admission'] for x in request['coding']['relational'].get('guidance',[])}
    return {'version':'browser-choice-summary@v1','source_request_sha256':references['request_sha256'],
            'forms':dict(sorted(forms.items())),
            'eligible_identity_aspirations':eligible,'unavailable_identity_routes':unavailable,
            'unavailable_development_routes':unavailable_development,
            'reviewed_aspiration_references':[a for a,i in references['revisions'].items() if i in current],
            'interpretation':'Available proposal forms, not approved outcomes or a ranking. Artifact changes, reviewed meaning/relationship development and new identity are distinct. Success permits consideration, never requires novelty. Retention and uncertainty remain valid. Native validation and independent review are authoritative.'}

def project(context,request):
    """Return a new public wire context; original native/context values untouched."""
    references=catalog(request)
    expected=native.relational_contract(request)
    evidence=native.return_evidence.catalog(request)
    if context.get('relational_contract')!=expected or context.get('return_evidence')!=evidence:
        raise ValueError('proposal context differs from exact native contract/evidence')
    inputs=request['coding']['relational']['inputs']
    for key in ('current_success','current_contrast'):
        if context.get(key)!=inputs.get(key):raise ValueError('choice evidence differs from native request')
    value=copy.deepcopy(context)
    p={identity:alias for alias,identity in references['parents'].items()}
    r={identity:alias for alias,identity in references['revisions'].items()}
    for parent in value['parents']:parent['id']=_translate(parent['id'],p,'context parent')
    relation=value['relational_contract'];relation['parent_ids']=[p[x] for x in expected['parent_ids']]
    for lineage in relation['allowed_lineage']:
        for field in ('revises','original_revision','prior_revision'):lineage[field]=_translate(lineage[field],r,field)
        lineage['parents']=[_translate(x,p,'lineage parent') for x in lineage['parents']]
    for catalyst in relation.get('catalyst_references',[]):
        catalyst['admission']=_translate(catalyst['admission'],r,'catalyst admission')
        catalyst['parents']=[_translate(x,p,'catalyst parent') for x in catalyst['parents']]
    relation['revision']['development']='Required object; use the exact short aliases in allowed_lineage, retain the parent set. Native ancestry checks remain unchanged.'
    for entry in value.get('relational_history',[]):
        entry['admission']=_translate(entry['admission'],r,'historical revision')
        entry['proposal']=_relation(entry['proposal'],p,r)
    if 'advisory_reconsideration' in value:
        item=value['advisory_reconsideration']
        item['lineage']=_translate(item['lineage'],r,'advisory lineage')
    value['return_evidence'].pop('binding')
    for source in value['return_evidence']['sources']:source.pop('digest')
    if 'pursuit_expectation_contract' in value:
        value['pursuit_expectation_contract']=value['pursuit_expectation_contract'].replace('parents:[contributing IDs]','parents:[current request-local p aliases]')+' Use only aliases from proposal_references.parents for expectation.parents; never copy full IDs from retained history or the reference catalog targets.'
    # The wire schema and prose must name the same request-local vocabulary.
    # Native history and source content remain untouched; expansion still rejects
    # full, foreign and stale identities rather than repairing a model response.
    def expectation_aliases(node):
        if isinstance(node,dict):
            expectation=node.get('properties',{}).get('expectation')
            if isinstance(expectation,dict):expectation['properties']['parents']['items']={'type':'string','enum':list(references['parents'])}
            for child in node.values():expectation_aliases(child)
        elif isinstance(node,list):
            for child in node:expectation_aliases(child)
    expectation_aliases(value['response_contracts'])
    contracts=value['response_contracts'];basis=contracts['return_basis']
    basis['semantics'].pop('binding',None)
    basis['semantics']['inferred']='Objects {path,kind,statement}: choose a current inference_paths path; kind=source_prediction. Omit digest; transport binds exact current bytes. Statements remain hypotheses, never observations.'
    schema=basis['schema'];schema['required'].remove('binding');schema['properties'].pop('binding')
    options=schema['properties']['inferred']['items']
    if isinstance(options,dict):
        for option in options['oneOf']:
            option['required'].remove('digest');option['properties'].pop('digest')
    relation_fields=contracts['relational']
    relation_fields['required']['parents']='array of 2..8 distinct request-local p aliases from relational_contract.parent_ids'
    relation_fields['required']['revises']='null for initial, otherwise exact r alias from relational_contract.allowed_lineage'
    relation_fields['development']['required']['original_revision']='exact r alias from selected allowed_lineage.original_revision'
    relation_fields['development']['required']['prior_revision']='exact r alias from selected allowed_lineage.prior_revision'
    # Same request-local catalog and immutable native request own meaning references.
    spec=importlib.util.spec_from_file_location('browser_desire_references',Path(__file__).with_name('engine-browser-desire.py'));helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
    helper.wire(value,request,references)
    if value.get('current_success'):value['current_success']['id']='s0'
    if value.get('current_contrast'):value['current_contrast']['id']='c0'
    value['proposal_references']=references
    value['choice_summary']=choice_summary(value,request,references)
    brief=value.get('public_brief')
    if isinstance(brief,str) and hashlib.sha256(brief.encode()).hexdigest()==LEGACY_HOST_BRIEF_SHA256:
        if brief.count(LEGACY_OPTIONS)!=1:raise ValueError('host capability wording mismatch')
        value['public_brief']=brief.replace(LEGACY_OPTIONS,'Current proposal forms and prerequisites are in choice_summary.')
        value['brief_presentation']={'original_sha256':LEGACY_HOST_BRIEF_SHA256,'source':'exact frozen host-authored synthetic noticeboard brief','replaced_host_capability_text':LEGACY_OPTIONS,'original_preserved':'unchanged admitted origin; no task facts, user wording, evidence or obligations removed'}
    return value

DECLINED_PREFIX='\n\nDevelopment considered and declined: '

def fold_retention(value):
    """Native retention keeps one reason string; the development decision travels inside it, labeled."""
    for key in ('retain','development_declined'):
        if not isinstance(value.get(key),str) or not value[key].strip():raise ValueError(key+' reason required')
    if DECLINED_PREFIX in value['retain']:raise ValueError('retention reason must not forge the development decision label')
    return value['retain']+DECLINED_PREFIX+value['development_declined']

def expand(value,references,request):
    """Translate exact typed aliases, then inject only request-determined metadata."""
    if references!=catalog(request):raise ValueError('stale or substituted proposal reference catalog')
    if isinstance(value,dict) and set(value)=={'deliberation'}:
        spec=importlib.util.spec_from_file_location('deliberation_refs',Path(__file__).with_name('engine-browser-deliberation.py'));helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
        return helper.expand(value,references,request,expand)
    if isinstance(value,dict) and set(value)=={'retain','development_declined'}:
        return {'retain':fold_retention(value)}
    result=copy.deepcopy(value)
    if not isinstance(result,dict):return result  # Shape diagnostics belong to native contract.
    if set(result)&{'protocol','state_root','attention_binding','binding'}:raise ValueError('model supplied host proposal metadata')
    alternatives=result.get('alternatives')
    if not isinstance(alternatives,list):return result
    evidence=native.return_evidence.catalog(request)
    sources={item['path']:item['digest'] for item in evidence['sources']}
    for item in alternatives:
        if not isinstance(item,dict):continue
        if 'expectation' in item:
            x=item['expectation']
            if not isinstance(x,dict) or not isinstance(x.get('parents'),list):raise ValueError('expectation contributor references')
            x['parents']=[_translate(v,references['parents'],'expectation parent') for v in x['parents']]
        if 'provisional_preference' in item:
            x=item['provisional_preference']
            if not isinstance(x,dict) or set(x)!={'parents','meaning','rationale'} or not isinstance(x['parents'],list):raise ValueError('provisional preference model fields; no host binding')
            item['provisional_preference']={**copy.deepcopy(x),'parents':[_translate(p,references['parents'],'provisional contributor') for p in x['parents']],'binding':request['coding']['relational']['binding']}
        if 'success_preference' in item:
            x=item['success_preference']
            if not isinstance(x,dict) or set(x)!={'aspiration','success'}:raise ValueError('success preference model fields; no host binding')
            item['success_preference']={'aspiration':_translate(x['aspiration'],references['revisions'],'aspiration'),'success':_translate(x['success'],references.get('successes',{}),'success'),'binding':request['coding']['relational']['binding']}
        if 'contrast_preference' in item:
            x=item['contrast_preference']
            if not isinstance(x,dict) or set(x)!={'aspiration','contrast'}:raise ValueError('contrast preference model fields; no host binding')
            item['contrast_preference']={'aspiration':_translate(x['aspiration'],references['revisions'],'aspiration'),'contrast':_translate(x['contrast'],references['contrasts'],'contrast'),'binding':request['coding']['relational']['binding']}
        if 'public_desire' in item:
            spec=importlib.util.spec_from_file_location('browser_desire_references',Path(__file__).with_name('engine-browser-desire.py'));helper=importlib.util.module_from_spec(spec);spec.loader.exec_module(helper)
            item['public_desire']=helper.expand(item['public_desire'],request,references)
        if 'relational' in item:item['relational']=_relation(item['relational'],references['parents'],references['revisions'])
        if 'return_basis' in item:
            basis=item['return_basis']
            if not isinstance(basis,dict):raise ValueError('return_basis object required')
            if 'binding' in basis:raise ValueError('model supplied host return binding')
            inferred=basis.get('inferred')
            if not isinstance(inferred,list):raise ValueError('return_basis.inferred array required')
            for inference in inferred:
                if not isinstance(inference,dict):raise ValueError('source inference object required')
                if 'digest' in inference:raise ValueError('model supplied host source digest')
                path=inference.get('path')
                if not isinstance(path,str) or path not in sources:raise ValueError('unadmitted source inference path')
                inference['digest']=sources[path]
            basis['binding']=evidence['binding']
    return result


def compatible_generation_request(captured,current):
    """Only counted sunk tokens may advance during generation; all semantics exact."""
    normalized=copy.deepcopy(current)
    before=captured.get('estimates',[]);after=normalized.get('estimates',[])
    if not isinstance(before,list) or not isinstance(after,list) or len(before)!=len(after):return False
    for a,b in zip(before,after):
        if not isinstance(a,dict) or not isinstance(b,dict):return False
        old=a.get('resource_cost');new=b.get('resource_cost')
        if isinstance(old,dict) and isinstance(new,dict) and 'sunk_tokens_charged' in old and 'sunk_tokens_charged' in new:
            x=old['sunk_tokens_charged'];y=new['sunk_tokens_charged']
            if type(x) is not int or type(y) is not int or not 0<=x<=y:return False
            new['sunk_tokens_charged']=x
    return captured==normalized

def review_provenance(request,record,wire,response,basis,*,captured=None):
    """Explain an already authenticated decode; never admit or repair a response."""
    association=record['association'];raw=record['model_content']
    if digest(wire)!=association['request_body_sha256'] or digest(raw)!=association['model_content_sha256']:
        raise ValueError('review representation request/content substitution')
    if digest([response,basis])!=record['decoded_sha256']:
        raise ValueError('review representation candidate substitution')
    references=json.loads(wire['messages'][1]['content'])['proposal_references']
    generation=request if captured is None else captured
    if captured is not None:
        if digest(captured)!=references['request_sha256']:raise ValueError('captured proposal request substitution')
        current=catalog(request)
        if {k:v for k,v in current.items() if k!='request_sha256'}!={k:v for k,v in references.items() if k!='request_sha256'}:
            raise ValueError('stale current proposal reference inventory')
        # Only the measured sunk-token counter may advance; source_prefix, option
        # order, predictions, action evidence and all other state remain exact.
        if not compatible_generation_request(captured,request):
            raise ValueError('stale captured proposal state')
    expanded=expand(raw,references,generation)
    expectations={str(i):item['expectation'] for i,item in enumerate(expanded.get('alternatives',[])) if 'expectation' in item}
    if expectations!=response.get('pursuit_expectations',{}):
        raise ValueError('review representation expectation substitution')
    return {'version':'authenticated-proposal-representation@v1',
            'request':association['request'],'source':copy.deepcopy(association['source']),
            'wire_request_sha256':association['request_body_sha256'],
            'model_submission':copy.deepcopy(raw),'reference_catalog':copy.deepcopy(references),
            'native_candidate_sha256':digest(response),'decoded_pair_sha256':record['decoded_sha256'],
            'instruction':'The model_submission is the original authenticated wire response. Request-local aliases apply there. The candidate evidence is the validated native representation after transport expansion: full IDs there are expected, not model-supplied wire IDs. Historical criticism applies to its named failed response, not automatically to this candidate. This verifies representation only; independently assess all semantic and epistemic criteria. It grants no execution, acceptance or fulfillment.'}
