"""Case-level evidence prerequisites, not natural-language truth checking.

Uses existing native relational receipts and supplemental finite outcomes. Never
parses candidate-controlled panic output into authoritative assertion telemetry.
"""
import hashlib
import json

def canonical(x):return json.dumps(x,sort_keys=True,separators=(',',':'),ensure_ascii=False,allow_nan=False).encode()
def identity(x):return hashlib.sha256(canonical(x)).hexdigest()
def digest(x):
    if not isinstance(x,str) or len(x)!=64 or any(c not in '0123456789abcdef' for c in x):raise ValueError('evidence identity')
    return x

def catalog(request):
    coding=request['coding'];inputs=coding['relational']['inputs'];snapshot=digest(inputs['snapshot'])
    facts=[];seen=set()
    def add(channel,receipt,case,outcome,exit_code,scope):
        digest(receipt)
        if not isinstance(case,str) or not case or len(case.encode())>256:raise ValueError('case identity')
        key=(channel,receipt,case)
        if key in seen:raise ValueError('duplicate evidence case')
        seen.add(key)
        fact={'channel':channel,'receipt':receipt,'case_id':case,'outcome':outcome,'exit_code':exit_code,'scope':scope}
        facts.append({'id':str(len(facts)),**fact})
    allowed={x['id'] for x in coding['cases']}
    for item in inputs.get('returns',[]):
        for o in item.get('observations',[]):
            case=o.get('case_id')
            if case is None:continue
            if case not in allowed:raise ValueError('foreign case')
            value=o.get('passed')
            if value is not None and type(value) is not bool:raise ValueError('invalid case outcome')
            code=o.get('exit')
            if code is not None and type(code) is not int:raise ValueError('invalid exit')
            if code is not None and not 0<=code<=255:code=None
            # Verdict failure/timeout is not permission to turn incomplete summaries into observations.
            outcome='unknown'
            # Native Cargo verdict already validates complete inventory/exit/truncation.
            # abbreviated describes the stored diagnostic excerpt, not missing case results.
            if (item['verdict'],code,value) in [('passed',0,True),('failed_tests',101,True)]:outcome='passed'
            elif (item['verdict'],code,value)==('failed_tests',101,False):outcome='failed'
            elif coding.get('kind')=='browser_project@v1' and (item['verdict'],code,value)==('failed_tests',1,False):outcome='failed'
            add('protected',item['receipt'],case,outcome,code,digest(item['commitment']))
    for item in inputs.get('supplemental',[]):
        if item['snapshot']!=snapshot:raise ValueError('stale supplemental source')
        for case,value in item['cases'].items():
            if value not in ('passed_finite_case','counterexample','unknown'):raise ValueError('supplemental outcome')
            # Imported legacy summaries do not carry numeric exit codes; never infer 101 from failure.
            add('supplemental',item['receipt'],case,{'passed_finite_case':'passed','counterexample':'failed','unknown':'unknown'}[value],None,digest(item['bundle_digest']))
    for item in inputs.get('information_returns',[]):
        for record in item['records']:
            add('information_access',item['receipt'],record['id'],'content_accessed',None,digest(record['digest']))
    for item in inputs.get('service_returns',[]):
        if type(item['complete']) is not bool or type(item['exit']) is not int:raise ValueError('service receipt facts')
        add('service_execution',item['receipt'],item['condition']['id'],'completed_invocation' if item['complete'] else 'incomplete_invocation',item['exit'],digest(item['source_snapshot']))
    sources=[{'path':x['path'],'digest':digest(x['source_digest'])} for x in coding['files'] if x.get('editable')]
    value={'version':'b91-return-evidence@v1','state_root':digest(request['state_root']),'snapshot':snapshot,'sources':sources,'cases':facts,
        'granularity':'Case results only. Assertion execution, early abort, later not-run status and returned values are unknown unless separately evidenced. A failed case does not say which assertion failed. Source/control-flow predictions are inferences, never observed results. Numeric exit is shared command exit, not an assertion result; null means unavailable.'}
    if inputs.get('information_returns'):
        value['granularity']='information_access facts prove exact bytes accessed/content identity only; fictional assertions are not established truth or program test outcomes. Other facts are finite program case outcomes; assertion execution and unrun checks remain unknown.'
    if inputs.get('service_returns'):
        value['granularity']+=' Service execution establishes exact invocation receipt facts only, never semantic correctness, repeatability or fulfillment; source snapshot is per receipt.'
    return {**value,'binding':identity(value)}

def prompt_catalog(request):
    view=catalog(request)
    # Stable indexes into the same displayed native return tables avoid duplicating
    # long receipt/snapshot identities within the unchanged request byte budget.
    inputs=request['coding']['relational']['inputs']
    tables={'protected':inputs.get('returns',[]),'supplemental':inputs.get('supplemental',[]),'information_access':inputs.get('information_returns',[]),'service_execution':inputs.get('service_returns',[])}
    cases=[]
    for x in view['cases']:
        index=next(i for i,r in enumerate(tables[x['channel']]) if r['receipt']==x['receipt'])
        cases.append({'id':x['id'],'channel':x['channel'],'return_index':index,'case_id':x['case_id'],
                      'outcome':x['outcome'],'exit_code':x['exit_code']})
    return {'binding':view['binding'],'sources':view['sources'],'cases':cases,
        'granularity':('information_access references information_returns by return_index and proves access/content identity only, never fictional claim truth or program success. Protected and supplemental entries remain finite case results. ' if inputs.get('information_returns') else '')+'Case results only for protected/supplemental; assertion execution/early abort/not-run/returned values are unknown. Source predictions are not observed. exit_code is shared command status, null if unavailable. protected references observed_returns; supplemental references supplemental_returns by return_index.'}

def contract(narrative_bytes=512):
    return {'binding':'copy return_evidence.binding',
        'observed':'return_evidence.cases[].id only. service_execution establishes bounded invocation identity and recorded output only, never correctness or repeatability. information_access facts establish bytes accessed/content identity only, never truth of asserted contents or a program test outcome.',
        'inferred':'Objects {path,digest,kind,statement}: current return_evidence.sources; kind=source_prediction; nonblank statement <='+str(narrative_bytes)+' UTF-8 bytes within aggregate envelope.',
        'unknown':'Nonblank strings <='+str(narrative_bytes)+' UTF-8 bytes within aggregate envelope.',
        'limits':'information_access is access/content identity only, not a test or assertion truth. Other observed refs establish case facts only, not assertion execution or values. Predictions are not observations. Independent prose review required; no automatic truth or authority.'}


def schema(request, *, narrative_bytes=512):
    """Publish the exact source-bound return basis shape; citations remain model choices."""
    view=catalog(request)
    text={'type':'string','minLength':1,'maxLength':narrative_bytes}
    source_options=[{'type':'object','additionalProperties':False,
        'required':['path','digest','kind','statement'],
        'properties':{'path':{'const':source['path']},'digest':{'const':source['digest']},
                      'kind':{'const':'source_prediction'},'statement':text}}
        for source in view['sources']]
    return {'type':'object','additionalProperties':False,
        'required':['binding','observed','inferred','unknown'],
        'properties':{'binding':{'const':view['binding']},
            'observed':{'type':'array','maxItems':8,'uniqueItems':True,
                        'items':({'enum':[fact['id'] for fact in view['cases']]} if view['cases'] else False)},
            'inferred':{'type':'array','maxItems':4,'items':({'oneOf':source_options} if source_options else False)},
            'unknown':{'type':'array','maxItems':4,'items':text}},
        'description':'Observed entries are catalog id strings, never case_id names. Empty arrays are valid. Text limits are UTF-8 bytes; whitespace-only text is invalid. This schema guides output; native validation remains authoritative.'}


def validate(basis,request,*,narrative_bytes=512):
    view=catalog(request)
    if not isinstance(basis,dict) or set(basis)!={'binding','observed','inferred','unknown'} or basis['binding']!=view['binding']:raise ValueError('stale or malformed return basis')
    refs=basis['observed'];facts={x['id']:x for x in view['cases']}
    if not isinstance(refs,list) or len(refs)>8 or any(not isinstance(x,str) or x not in facts for x in refs) or len(refs)!=len(set(refs)):raise ValueError('unobserved or duplicate fact')
    def short(x):return isinstance(x,str) and bool(x.strip()) and len(x.encode())<=narrative_bytes
    sources={(x['path'],x['digest']) for x in view['sources']}
    inferred=basis['inferred'];unknown=basis['unknown']
    if not isinstance(inferred,list) or len(inferred)>4:raise ValueError('inference bound')
    for x in inferred:
        if (not isinstance(x,dict) or set(x)!={'path','digest','kind','statement'} or not isinstance(x['path'],str) or not isinstance(x['digest'],str)
                or (x['path'],x['digest']) not in sources or x['kind']!='source_prediction' or not short(x['statement'])):raise ValueError('unbound source inference')
    if not isinstance(unknown,list) or len(unknown)>4 or not all(short(x) for x in unknown):raise ValueError('unknown bound')
    return {'basis':basis,'resolved_observations':[facts[x] for x in refs],'snapshot':view['snapshot'],
            'assertion_execution':'unknown','authority':'none','semantic_review_required':True}


def case_fact_summary(request):
    """Bounded full-catalog facts, not model interpretation or assertion telemetry.

    The prospective reflection profile asks the model to copy this exact text.
    Selected support references in return_basis remain a separate subset; this
    summary is bound to the entire catalog and does not fabricate missing results.
    """
    facts=catalog(request)['cases']
    outcomes='; '.join(x['id']+'='+x['outcome'] for x in facts if x['channel']!='information_access')
    accesses=[x for x in facts if x['channel']=='information_access']
    text=('Observed case outcomes: '+outcomes+'. ' if outcomes else 'No case outcomes are available. ')
    if accesses:text+='Authenticated content access only: '+', '.join(x['id'] for x in accesses)+'. Contents remain attributed assertions. '
    text+='Assertion execution details and returned values are unknown; finite cases are not universal proof.'
    if len(text.encode())>512:raise ValueError('case-fact summary exceeds existing bound')
    return text


def validate_case_fact_reflection(proposal,request):
    """Check a new observation field exactly; never repair text or judge free prose."""
    development=proposal.get('development')
    if development is None:return None
    expected=case_fact_summary(request)
    if not isinstance(development,dict) or development.get('return_reveals')!=expected:
        raise ValueError('return_reveals differs from published receipt-bound case facts')
    return {'summary':expected,'catalog_binding':catalog(request)['binding'],
            'scope':'Full case catalog, independent of selected support refs. Interpretation elsewhere still requires semantic review.'}
