#!/usr/bin/env python3
"""Prepare/check one historical synthetic guidance pair. No transport or native effects."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import sqlite3
import tempfile
import importlib.util
spec=importlib.util.spec_from_file_location('pair_envelope',Path(__file__).with_name('engine-b91-project-envelope.py'));e=importlib.util.module_from_spec(spec);spec.loader.exec_module(e)
ROOT=Path(__file__).resolve().parents[1]
PARENT=ROOT/'target/b91-live-trial11';CONTINUATION=ROOT/'target/b91-trial11-continuation';REVIEWS=ROOT/'target/b91-live-trial11-reviews'
RUBRIC={'purpose':'Does the reviewed elaboration alter the usefulness, specificity, or grounding of the next proposed possibility?',
 'criteria':['Preserves both original desires and immutable authority','Uses actual finite outcomes without inventing assertion execution or returned values','Offers a useful bounded next possibility or honest abstention','Distinguishes any proposal difference from evidence of benefit'],
 'limits':'One paired shadow comparison, not statistical evidence, richer emergence, general autonomy, or fulfillment. Initial guidance already prohibited dropped fields; development elaborates that distinction. No expected winner.'}

def sha(x):return hashlib.sha256(x).hexdigest()
def read(path,pins):
    path=Path(path)
    if path.is_symlink():raise ValueError('source symlink')
    data=e.bounded_file(path,16*1024*1024);pins[str(path.relative_to(ROOT))]=sha(data);return data

def journal(path,pins,sequence):
    rows=[e.strict(x) for x in read(path,pins).splitlines()]
    for i,row in enumerate(rows):
        content={sequence:i,'previous':e.sha(rows[i-1]) if i else None,'event':row['event']}
        if row!={**content,'hash':e.sha(content)}:raise ValueError('historical journal integrity')
    return [x['event'] for x in rows]

def native_records(project,pins):
    # SQLite reads operate on a disposable byte-copy, including its WAL. Historical
    # DB/SHM files are never opened by SQLite or checkpointed by this preparer.
    with tempfile.TemporaryDirectory() as tmp:
        for name in ('events.sqlite','events.sqlite-wal'):
            path=project/name
            if path.exists():(Path(tmp)/name).write_bytes(read(path,pins))
        connection=sqlite3.connect('file:'+str(Path(tmp)/'events.sqlite')+'?mode=ro',uri=True)
        try:events={i:e.strict(payload) for i,payload in connection.execute('select id,payload from events')}
        finally:connection.close()
    def get(identity):
        event=events.get(identity)
        if not event or event['id']!=identity:raise ValueError('missing actual native record')
        evidence=event['kind']['observation']['evidence']
        if evidence['source']!='configured-cycle@v1':raise ValueError('foreign native record source')
        return event,e.strict(read(project/'cas'/evidence['digest'],pins))
    return get

def material():
    pins={};continuation=journal(CONTINUATION/'driver.jsonl',pins,'seq');prefix=continuation[0]['value']['prefix']
    if Path(prefix['parent'])!=PARENT:raise ValueError('unexpected historical parent')
    for name,expected in prefix['files'].items():
        if sha(read(PARENT/name,pins))!=expected:raise ValueError('historical prefix changed')
    provider=journal(PARENT/'provider.jsonl',pins,'sequence');driver=journal(PARENT/'driver.jsonl',pins,'seq')
    origin=e.strict(read(PARENT/'origin.json',pins));e.validate_origin(origin)
    copied=journal(CONTINUATION/'provider.jsonl',pins,'sequence')
    done={x['name']:x['value'] for x in driver if x['kind']=='result'}
    native=native_records(PARENT/'project',pins)
    pairs=[]
    for stage in ('relation_after_seed','relation_after_experiment'):
        dispatch=next(x for x in provider if x['kind']=='dispatch_intent' and x['stage']==stage);ident=dispatch['id']
        outcome=next(x for x in provider if x['kind']=='provider_outcome' and x['id']==ident)
        event=next(x for x in provider if x['kind']=='validated_proposal' and x['id']==ident)
        stored=next(x for x in provider if x['kind']=='review_only_stored' and x['id']==ident)
        if event not in copied or outcome['status']!='completed' or outcome['usage_status']!='reported' or outcome['usd'] is None:raise ValueError('generated lineage incomplete')
        response=event['response'];proposal=response['proposals'][0]
        if len(response['proposals'])!=1 or proposal['kind']!='relational' or done['generate_'+stage]['response']!=response:raise ValueError('generated response mismatch')
        record=stored['proposal_record'];native_event,native_proposal=native(record)
        if native_event['source']!='generator' or native_proposal!={'fact':'proposal_received','response':response}:raise ValueError('native proposal mismatch')
        admission=done['admit_'+stage];admit_event,admit=native(admission)
        if admit_event['source']!='operator' or admit!={'fact':'coding','record':{'phase':'relational_admission','proposal':record,'index':0,'binding':proposal['binding']}}:raise ValueError('native admission mismatch')
        review=done['review_'+stage]
        if review.get('approved') is not True:raise ValueError('missing independent qualitative review')
        matches=[]
        for path in REVIEWS.glob('*.request.json'):
            request=e.strict(path.read_bytes())
            if request.get('subject',{}).get('proposal_record')==record:matches.append(path)
        if len(matches)!=1:raise ValueError('missing exact review request')
        path=matches[0];request=e.strict(read(path,pins));review_file=e.strict(read(path.with_name(path.name.replace('.request.','.review.')),pins))
        if (path.stem.removesuffix('.request')!=e.sha(request) or request['subject_sha256']!=e.sha(request['subject'])
            or request['subject']['proposal']!=proposal or review_file!=review or review['subject_sha256']!=request['subject_sha256'] or review['state_root']!=request['state_root']):raise ValueError('review subject/source mismatch')
        pairs.append({'request':ident,'admission':admission,'proposal':proposal['proposal'],'snapshot':request['source_snapshot'],'review_sha256':e.sha(review)})
    old,new=pairs
    if new['proposal']['revises']!=old['admission'] or new['proposal']['development']['prior_revision']!=old['admission'] or new['proposal']['development']['original_revision']!=old['admission'] or old['proposal']['parents']!=new['proposal']['parents'] or old['snapshot']!=new['snapshot']:raise ValueError('not same-source same-parent lineage')
    if old['proposal']['parents']!=[p['id'] for p in origin['parents']]:raise ValueError('original parents drift')
    receipt=done['execute_seed']['status']['goal_completion']['test_receipt'];receipt_event,raw=native(receipt);raw=raw['record']
    if receipt_event['source']!='rules' or done['execute_seed']['fixture_only'] is not False:raise ValueError('non-actual protected receipt')
    if raw['phase']!='receipt' or raw['source_after']!=new['snapshot'] or raw['verdict']!='failed_tests':raise ValueError('protected return missing')
    cases=[{'case_id':x['case_id'],'passed':x['passed'],'exit':x['exit']} for x in raw['observations'] if x.get('case_id')]
    if {x['case_id'] for x in cases}!={x['id'] for x in e.expected_configuration()['cases']}:raise ValueError('case scope drift')
    import_event,imported=native(done['import_check_before'])
    if import_event['source']!='rules':raise ValueError('supplemental import provenance')
    observation=imported['record']['event']['observation']
    if observation['snapshot']!=new['snapshot'] or observation['requirements_revision']!=origin['requirements_revision']:raise ValueError('supplemental source drift')
    check=native_records(PARENT/'checks-before',pins);actual_event,actual=check(observation['receipt']);actual=actual['record']
    if actual_event['source']!='rules' or done['execute_check_before']['fixture_only'] is not False:raise ValueError('non-actual supplemental receipt')
    if actual['phase']!='receipt' or actual['source_after']!=observation['checked_snapshot'] or actual['verdict']!='failed_tests' or done['execute_check_before']['status']['goal_completion']['test_receipt']!=observation['receipt']:raise ValueError('actual supplemental receipt missing')
    basis=done['generate_relation_after_experiment']['return_basis'];facts=basis['resolved_observations']
    expected_facts=[]
    for case in cases:
        expected_facts.append({'id':str(len(expected_facts)),'channel':'protected','receipt':receipt,'case_id':case['case_id'],'outcome':'passed' if case['passed'] is True else 'failed' if case['passed'] is False else 'unknown','exit_code':case['exit'],'scope':raw['commitment']})
    actual_cases={x['case_id']:x for x in actual['observations'] if x.get('case_id')}
    for case,outcome in observation['cases'].items():
        seen=actual_cases.get(case)
        if not seen or {'passed_finite_case':True,'counterexample':False,'unknown':None}.get(outcome)!=seen['passed']:raise ValueError('supplemental case outcome mismatch')
        expected_facts.append({'id':str(len(expected_facts)),'channel':'supplemental','receipt':observation['receipt'],'case_id':case,'outcome':{'passed_finite_case':'passed','counterexample':'failed','unknown':'unknown'}[outcome],'exit_code':None,'scope':observation['bundle_digest']})
    if facts!=expected_facts:raise ValueError('revision evidence case/outcome mismatch')
    source=done['evaluator_seed_response']['proposals'][0]['patches'][0]['new_text']
    if source!=e.prepare.FAILED:raise ValueError('source not frozen synthetic seed')
    common={'scenario':'Frozen evaluator-authored synthetic parser; historical post-experiment state', 'parents':origin['parents'],
      'source':{'path':'src/lib.rs','text':source,'snapshot':new['snapshot']},
      'evidence':{'protected':{'receipt':receipt,'verdict':raw['verdict'],'cases':cases},'supplemental':{k:observation[k] for k in ('receipt','snapshot','checked_snapshot','cases','findings')},'case_facts':facts,
        'limits':'Finite case outcomes only; assertion execution and returned values unknown. Source predictions are inferences.'},
      'lineage_provenance':[{'admission':p['admission'],'revises':p['proposal'].get('revises'),'parents':p['proposal']['parents']} for p in pairs],
      'authority':{'provider_calls_authorized':0,'executions_authorized':0,'adoption_authorized':False,'protected_tests':'withheld and immutable','editable_paths':['src/lib.rs'],'historical_bindings_are_current_authority':False},
      'output_contract':{'type':'object','required':['proposed_next_choice','reason','epistemic_limits'],'additionalProperties':False,
        'properties':{'proposed_next_choice':{'description':'Bounded descriptive shadow possibility, NOT a native Response or executable patch. Null means abstention.','oneOf':[{'type':'null'},{'type':'object','additionalProperties':False,'required':['kind','description'],'properties':{'kind':{'type':'string','enum':['qualitative_revision','supplemental_experiment','source_edit','verify_current_source']},'description':{'type':'string','maxLength':1024}}}]},'reason':{'type':'string','maxLength':512},'epistemic_limits':{'type':'string','maxLength':512}}}}
    system='This is an output-only controlled historical shadow comparison. Only active_guidance varies; neither arm claims current native admission. Preserve both parents and supplied authority. Propose one useful next possibility or honest abstention; no prescribed winner or action. Treat guidance as a revisable hypothesis, not new requirements. Distinguish source inference, finite observed cases and unknowns. Return strict JSON matching output_contract. No actual execution, adoption or fulfillment.'
    bodies=[]
    for pair in pairs:
        context={**common,'active_guidance':copy.deepcopy(pair['proposal'])}
        body={'model':'gpt-6-luna','stream':False,'max_completion_tokens':8192,'messages':[{'role':'system','content':system},{'role':'user','content':e.canonical(context).decode()}]}
        if len(json.dumps(body,ensure_ascii=False,allow_nan=False).encode())>32768:raise ValueError('paired request size bound')
        bodies.append(body)
    return pins,pairs,bodies

def prepare(out):
    out=Path(out).resolve()
    if any(out==p or out.is_relative_to(p) or p.is_relative_to(out) for p in (PARENT,CONTINUATION,REVIEWS)):raise ValueError('output overlaps immutable history')
    pins,pairs,bodies=material()
    manifest={'version':'historical-guidance-pair@v1','implementation':{name:sha((ROOT/'scripts'/name).read_bytes()) for name in ('engine-guidance-pair-export.py','engine-b91-project-envelope.py','engine-b91-prepare.py','engine-b91-evidence.py','engine-local-model.py','engine-run.py','m3-cheaper-broker.py','engine-provider-envelope.py','engine-response-recovery.py')},'sources':pins,
      'order':['X','Y'],'local_arm_mapping':{'X':{'meaning':'historical prior counterfactual',**{k:v for k,v in pairs[0].items() if k!='proposal'}},'Y':{'meaning':'reviewed elaboration',**{k:v for k,v in pairs[1].items() if k!='proposal'}}},
      'payloads':{label:e.sha(body) for label,body in zip(('X','Y'),bodies)},'rubric':RUBRIC,
      'calls_authorized':0,'executions_authorized':0,'adoption_authorized':False,'limits':{'input_bytes':32768,'completion_tokens':8192,'prospective_deadline_seconds':240},
      'integrity_limit':'SHA-256 byte pins and explicit event/CAS joins; this tool does not reimplement native BLAKE3 event validation. Qualitative review prose remains local.'}
    expected={'manifest.json':manifest,'X.json':bodies[0],'Y.json':bodies[1]}
    if out.exists():
        if {x.name for x in out.iterdir()}!=set(expected) or any(e.strict((out/name).read_bytes())!=value for name,value in expected.items()):raise ValueError('existing paired export differs')
    else:
        out.mkdir(parents=True)
        for name,value in expected.items():e.private_save(out/name,value)
    return e.sha(manifest)

def check(out,expected_hash):
    out=Path(out);manifest=e.strict(e.bounded_file(out/'manifest.json'))
    if e.sha(manifest)!=expected_hash:raise ValueError('manifest identity drift')
    actual=prepare(out)
    if actual!=expected_hash:raise ValueError('historical source or payload drift')
    bodies=[e.strict(e.bounded_file(out/(label+'.json'))) for label in ('X','Y')]
    contexts=[e.strict(x['messages'][1]['content']) for x in bodies]
    selected=[x.pop('active_guidance') for x in contexts]
    if contexts[0]!=contexts[1] or selected[0]==selected[1]:raise ValueError('not exactly one changed guidance field')
    return {'status':'checked_output_only','manifest_sha256':actual,'payload_bytes':[len(json.dumps(x,ensure_ascii=False).encode()) for x in bodies],'provider_calls':0,'executions':0,'adoptions':0}

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('mode',choices=('prepare','check'));parser.add_argument('--out',type=Path,required=True);parser.add_argument('--expected-sha256');args=parser.parse_args()
    if args.mode=='prepare':print(prepare(args.out))
    else:
        if not args.expected_sha256:parser.error('check requires --expected-sha256')
        print(json.dumps(check(args.out,args.expected_sha256),indent=2))
