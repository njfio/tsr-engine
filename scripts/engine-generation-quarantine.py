"""Generation-only disposal proof. Never identifies a provider response or settles cost.

This first policy deliberately cannot certify a financial ceiling from a discounted
quote. Financially unbounded quarantines continue to stop paid dispatch.
"""
import hashlib,json,os,fcntl,types,contextlib,sqlite3
from pathlib import Path

def inspect(ledger,root,request_id,expected_prefix,review):
    root=Path(root).resolve()
    locks=contextlib.ExitStack()
    try:
        # Both supported launchers lock one of these locations. Holding both
        # prevents disposal while either launcher can accept a late response.
        for path in (root.parent/'launch.lock',root/'launch.lock'):
            lock=locks.enter_context(os.fdopen(os.open(path,os.O_RDWR|os.O_CREAT|os.O_NOFOLLOW,0o600),'r+'))
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
        return _inspect_locked(ledger,root,request_id,expected_prefix,review),locks
    except BaseException:
        locks.close()
        raise

def _inspect_locked(ledger,root,request_id,expected_prefix,review):
    d=__import__('importlib.util',fromlist=['util'])
    spec=d.spec_from_file_location('quarantine_driver',Path(__file__).with_name('engine-b91-driver.py'));driver=d.module_from_spec(spec);spec.loader.exec_module(driver)
    root=Path(root).resolve();file=root/'adapter.jsonl'
    raw=driver.e.bounded_file(file,512*1024*1024)
    if hashlib.sha256(raw).hexdigest()!=expected_prefix:raise ValueError('quarantine journal prefix changed')
    rows=[driver.e.strict(x) for x in raw.splitlines()];ad=rows[0]['event']['value']
    holder=types.SimpleNamespace(root=root,file=file,admission=ad,fixture=bool(ledger.manifest['fixture_only']),row=driver.Driver.row,journal_limit=512*1024*1024)
    _,pending,done=driver.Driver.read(holder)
    if set(pending)!={'dispatch_'+request_id}:raise ValueError('only one exact pending generation may be quarantined')
    if ad.get('version')!='browser-project-cycle-adapter@v7' or ad.get('model')!='gpt-6-luna' or ad.get('directory')!=str(root):raise ValueError('unsupported generation transport/admission')
    record=done.get('request_body_'+request_id,{})
    body=record.get('body',{})
    if set(body)!={'model','messages','stream','max_completion_tokens','response_format'} or body.get('model')!='gpt-6-luna' or body.get('stream') is not False or body.get('max_completion_tokens')!=32768:raise ValueError('not the admitted tool-free generation shape')
    if not isinstance(body['messages'],list) or not body['messages'] or any(set(x)!={'role','content'} or x['role'] not in ('system','user','assistant') or not isinstance(x['content'],str) for x in body['messages']):raise ValueError('only plain text generation may be quarantined')
    if record.get('body_sha256')!=driver.e.sha(body) or set(record.get('source',{}))!={'state_root','source_snapshot','scope'}:raise ValueError('request/source binding mismatch')
    if 'reserve_'+request_id not in done:raise ValueError('native reservation missing')
    if review.get('decision')!='dispose_output_preserve_financial_unknown' or review.get('journal_sha256')!=expected_prefix or review.get('request_body_sha256')!=record['body_sha256'] or not isinstance(review.get('reviewer'),str) or not review['reviewer']:raise ValueError('exact independent disposal review required')
    native_proof(driver,ad,request_id,record)
    trial=driver.e.sha({'trial':str(root),'admission':driver.e.sha(ad)})
    # This is an adapter/campaign disposal classification only. It neither
    # settles native accounting nor certifies a safe native derivative; that
    # remains independently checked before any future admission.
    value={'version':'unknown-generation-quarantine@v1','trial':trial,'request_id':request_id,'admission_sha256':driver.e.sha(ad),'journal_sha256':expected_prefix,'request_body_sha256':record['body_sha256'],'campaign_binding':ledger_module_digest(body),'source':record['source'],'review_sha256':driver.e.sha(review),'financial_status':'unverified_ceiling','output_policy':'permanently_discard','directory':str(root)}
    return value

def native_proof(driver,ad,request_id,record):
    directory=Path(ad['origin']['directory']).resolve()
    c=driver.Native(directory);q=c.request();budget=c.budget()
    actual={'state_root':q['state_root'],'source_snapshot':q['coding']['snapshot'],'scope':q['next_attention']['scope']}
    if actual!=record['source']:raise ValueError('native source/root changed')
    if budget['unsettled_requests']!=1 or budget['unresolved_tool_requests']!=0:raise ValueError('native pending effect is not solely generation')
    # Native budget() authenticates resource evidence and CAS digests. Inspect
    # those same records read-only to bind the one outstanding reservation.
    db=directory/'shadow/events.sqlite';before=hashlib.sha256(db.read_bytes()).hexdigest();matches=[];settled=[]
    with sqlite3.connect('file:'+str(db)+'?mode=ro',uri=True) as conn:
        for (payload,) in conn.execute('SELECT payload FROM events ORDER BY seq'):
            event=json.loads(payload);evidence=event.get('kind',{}).get('observation',{}).get('evidence',{})
            if evidence.get('source')!='project-resource-accounting@v1':continue
            entry=json.loads((directory/'shadow/cas'/evidence['digest']).read_bytes());item=entry['record']
            if item['kind']=='reserve_unpriced' and item['request']['request_id']==request_id:
                if event['source']!='rules':raise ValueError('native reservation provenance')
                matches.append(item['request'])
            if item['kind']=='settle' and item['usage']['request_id']==request_id:settled.append(item)
    expected={'request_id':request_id,'state_root':record['source']['state_root'],'envelope':record['body_sha256'],'model':'gpt-6-luna','tokens_upper':len(json.dumps(record['body'],allow_nan=False,ensure_ascii=False).encode())+4096+record['body']['max_completion_tokens']}
    if matches!=[expected] or settled:raise ValueError('native reservation identity/settlement mismatch')
    if hashlib.sha256(db.read_bytes()).hexdigest()!=before or c.request()['state_root']!=actual['state_root']:raise ValueError('native state changed during disposal inspection')

def ledger_module_digest(value):
    return hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':'),allow_nan=False).encode()).hexdigest()
