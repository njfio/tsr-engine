"""Reviewed, one-shot recovery of an exact terminal failed review.

Not an HTTP retry policy. An operator-reviewed immutable ticket authenticates
one failed invocation, its zero-use provider record and retained callback. All
ordinary journals remain append-only. Accounting does not itself grant retry.
"""
import copy, decimal, hashlib, importlib.util, json, contextlib, fcntl, os
from pathlib import Path

S = Path(__file__).parent
spec = importlib.util.spec_from_file_location('failed_review_ordinary', S/'engine-browser-ordinary.py')
ordinary = importlib.util.module_from_spec(spec); spec.loader.exec_module(ordinary)
a = ordinary.b; e = a.e; d = a.d
history_module = a.load('failed_review_history', 'engine-browser-review-continuation.py')
campaign = a.load('failed_review_campaign', 'engine-b91-campaign.py')

def read(path): return e.strict(Path(path).read_bytes())
def file_sha(path): return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def require(ok, reason):
    if not ok: raise ValueError(reason)

def failed_zero(receipt, lookup, earliest):
    """Positive exact evidence, independent of automatic HTTP503 policy."""
    ids = receipt.get('response_request_ids', [])
    require(receipt.get('response_status') == 'received_http_error' and receipt.get('http_status') in (400, 503) and len(ids) == 1, 'not exact supported terminal HTTP failure')
    pid = ids[0]; endpoint = 'https://api.cheaperinference.com/v1/usage/requests?request_ids='+pid
    require(lookup.get('request_id') == pid and lookup.get('endpoint') == endpoint, 'provider identity mismatch')
    page = lookup.get('record', {})
    require(page.get('has_more') is False and len(page.get('data', [])) == 1 and not page.get('missing_request_ids'), 'ambiguous provider record')
    row = page['data'][0]
    require(row.get('request_id') == pid and row.get('model') == 'gpt-6-luna' and row.get('endpoint') == '/v1/chat/completions' and row.get('status') == 'failed' and row.get('currency') == 'USD', 'provider terminal scope mismatch')
    require(all(type(row.get(k)) is int and row[k] == 0 for k in ('prompt_tokens','completion_tokens','total_tokens')), 'partial or unknown usage')
    charge = decimal.Decimal(str(row.get('billing', {}).get('billed_cost_usd')))
    require(row.get('billing', {}).get('status') == 'failed' and charge.is_finite() and charge == 0, 'charge not positively zero')
    require(row.get('billed_cost_usd') is None or decimal.Decimal(str(row['billed_cost_usd'])) == 0, 'conflicting charge')
    require(earliest-5 <= d.stamp(row['created_at']) <= d.stamp(row['completed_at']) <= d.stamp(lookup['retrieved_at']), 'provider chronology mismatch')
    return pid

def inspect(parent, lookup_name="provider-exact-lookup.json"):
    require(Path(lookup_name).name == lookup_name and lookup_name.endswith(".json"), "local lookup filename required")
    parent = Path(parent).resolve(); ad = read(parent/'adapter-admission.json'); ca = read(parent/'cycle-admission.json')
    ar, ap, done = history_module.history(parent/'adapter/adapter.jsonl')
    cr, cp, cd = history_module.history(parent/'cycle/cycle.jsonl')
    require(ar[0]['event']['value'] == ad and cr[0]['event']['value'] == ca and ad.get('process_profile') == 'ordinary-memory@v1', 'ordinary admission mismatch')
    require(not ap and len(cp) == 1, 'unknown nested effect')
    stage = next(iter(cp)); require(stage.startswith('review_') and stage[7:].isdigit(), 'not missing independent review')
    callback = cd.get('callback_input_'+stage); require(callback and callback['kind'] == 'review', 'missing retained review callback')
    calls = [(k[9:],v) for k,v in done.items() if k.startswith('dispatch_')]
    require(bool(calls), 'no failed request'); ident, failed = calls[-1]
    require(failed.get('candidate') is None and failed.get('candidate_sha256') == e.sha(None) and failed.get('resource_stop') is True, 'response or nontechnical result exists')
    require(not any(k.startswith(('execution_','reviewed_','review_rejection_')) for k in done), 'execution or judgment already exists')
    require(ar[-1]['event'].get('name') == 'settle_'+ident, 'failure not final settled adapter operation')
    wire = done['request_body_'+ident]
    require(wire['kind'] == 'review' and wire['body_sha256'] == e.sha(wire['body']), 'wrong retained request')
    expected = e.sha({'admission':e.sha(ad),'kind':'review','index':len(calls)-1,'body':e.sha(wire['body'])})
    require(ident == expected, 'review request association mismatch')
    intent = next(r['event'] for r in ar if r['event'].get('name') == 'dispatch_'+ident and r['event']['kind'] == 'intent')
    lookup = read(parent/lookup_name); pid = failed_zero(failed['receipt'], lookup, d.stamp(intent['at']))
    header=done.get('response_identity_'+ident)
    require(header is not None, 'missing retained response identity')
    if header is not None:
        require(header['local_request']==ident and header['admission_sha256']==e.sha(ad) and header['canonical_body_sha256']==e.sha(wire['body']) and header['identity']['response_request_ids']==[pid] and header['identity']['http_status']==failed['receipt']['http_status'], 'response header association mismatch')
    source = {k:callback['context'][k] for k in ('state_root','source_snapshot','scope')}
    require(wire['source'] == source, 'review source mismatch')
    candidates = [(k[10:],v) for k,v in done.items() if k.startswith('generated_') and v['response'] == callback['response']]
    require(len(candidates) == 1, 'retained proposal missing or ambiguous'); generated, proposal = candidates[0]
    bound = a.base.bound_output.verify_envelope(done['bound_output_'+generated], admission=ad, ident=generated, dispatch=done['dispatch_'+generated], settled=done['settle_'+generated], source=source)
    require(bound['decoded_sha256'] == e.sha([callback['response'],proposal['return_basis']]), 'proposal association mismatch')
    trial = e.sha({'trial':ad['directory'],'admission':e.sha(ad)})
    return {'version':'exact-failed-review-proof@v1','parent':str(parent),'admission':e.sha(ad),'trial':trial,'request':ident,'call_key':e.sha({'trial':trial,'request':ident,'kind':'call'}),'provider_request_id':pid,'http_status':failed['receipt']['http_status'],'source':source,'body':wire['body'],'callback':callback,'stage':stage,'failed':failed,'adapter_rows':len(ar),'adapter_head':ar[-1]['hash'],'cycle_rows':len(cr),'cycle_head':cr[-1]['hash'],'adapter_sha256':file_sha(parent/'adapter/adapter.jsonl'),'cycle_sha256':file_sha(parent/'cycle/cycle.jsonl'),'lookup_sha256':file_sha(parent/lookup_name),'lookup_name':lookup_name,'prior':{'request_id':ident,'prompt_tokens':None,'output_tokens':None,'duration_ns':int(failed['receipt']['elapsed_seconds']*1e9),'external_cost_microusd':None,'outcome':'unknown'}}

def verify_prefix(proof):
    p = Path(proof['parent'])
    require(file_sha(p/proof.get('lookup_name','provider-exact-lookup.json')) == proof['lookup_sha256'], 'lookup changed')
    for stem in ('adapter','cycle'):
        rows,_,_ = history_module.history(p/stem/(stem+'.jsonl')); n = proof[stem+'_rows']
        require(len(rows) >= n and rows[n-1]['hash'] == proof[stem+'_head'], 'historical prefix changed')
    require(e.sha(read(p/'adapter-admission.json')) == proof['admission'], 'admission changed')

def check_stop(proof):
    p=Path(proof['parent'])
    require(not any((p/x/'STOP').exists() for x in ('','adapter','cycle')), 'human Stop blocks recovery')

def reviewed(proof, review, *, retry=False):
    require(review.get('version') == 'failed-review-review@v1' and review.get('proof_sha256') == e.sha(proof) and review.get('decision') == 'exact_failed_zero' and bool(review.get('reviewer')), 'independent exact proof review required')
    require(review.get('implementation_sha256')==file_sha(__file__), 'reviewed recovery implementation changed')
    if retry: require(review.get('retry') == 'one-identical-review-only', 'one-shot retry authority missing')

@contextlib.contextmanager
def exclusive(proof):
    with contextlib.ExitStack() as stack:
        for stem,name in (('adapter','launch.lock'),('cycle','run.lock')):
            path=Path(proof['parent'])/stem/name
            handle=stack.enter_context(os.fdopen(os.open(path,os.O_WRONLY|os.O_CREAT|os.O_NOFOLLOW,0o600),'a'))
            fcntl.flock(handle,fcntl.LOCK_EX|fcntl.LOCK_NB)
        yield

def reconcile(proof, review, ledger, native, *, dry_run=False):
    with exclusive(proof):return _reconcile(proof,review,ledger,native,dry_run=dry_run)

def _reconcile(proof, review, ledger, native, *, dry_run=False):
    reviewed(proof,review); verify_prefix(proof); check_stop(proof)
    request=native.cli('project','cycle','request')
    require({k:request[k] for k in proof['source']} == proof['source'] and (request['branch']=='propose' or (request['branch']=='reconcile' and request.get('reason')=='uncertain_effect_or_resource_reservation')), 'stale native source')
    budget=native.budget()
    require(not budget['exceeded'] and not any(budget.get(k,0) for k in ('unsettled_requests','unresolved_tool_requests','unknown_compute_requests')), 'unknown nonbilling effect')
    event=None
    with ledger.locked() as (_,rows):
        status,trials,calls,executions=ledger.fold(rows)
        require(not status['overrun'] and not status['blocking_unknown_executions'] and set(status['blocking_unknown_calls']) <= {proof['call_key']}, 'unrelated campaign blocker')
        require(trials[proof['trial']]['admission']==proof['admission'] and calls[proof['call_key']]['binding']==campaign.digest(proof['body']), 'campaign request mismatch')
        require(not any(v['trial']==proof['trial'] for v in executions.values()), 'unexpected execution')
        bills=[r for r in rows if r['event'].get('kind')=='bill' and r['event'].get('key')==proof['call_key']]
        require(len(bills)==1 and bills[0]['event']['microusd'] is None, 'original unknown bill missing')
        event={'kind':'bill_reconciled','key':proof['call_key'],'original_bill_row':bills[0]['hash'],'microusd':0,'basis':'exact_provider_failed_zero','provider_request_id':proof['provider_request_id'],'lookup_sha256':proof['lookup_sha256'],'native_journal_sha256':proof['cycle_sha256'],'provider_journal_sha256':proof['adapter_sha256'],'native_result_sha256':e.sha(proof['failed']),'review_sha256':e.sha(review),'billing_semantics_sha256':e.sha({'terminal_status':'failed','tokens':0,'billing_status':'failed','billed_cost_usd':'0.000000'})}
        prior=[r['event'] for r in rows if r['event'].get('kind')=='bill_reconciled' and r['event'].get('key')==proof['call_key']]
        require(not prior or prior==[event], 'reconciliation changed')
        resolved=ledger.fold(rows if prior else rows+[ledger.row(rows,event)])[0]
        require(not resolved['stopped'], 'reconciling this exact bill cannot clear remaining blocker')
        head=rows[-1]['hash']
    native_proof={'state_root':proof['source']['state_root'],'prior':proof['prior'],'proof':e.sha(proof),'review':e.sha(review)}
    if dry_run:return {'event':event,'native_proof':native_proof,'applied':False}
    # Native command and campaign append are separately idempotent. A restart
    # repeats only this exact local accounting proof, never a provider request.
    check_stop(proof);native.json_cli(('project-resource-reconcile-failed-zero',),native_proof,'--proof')
    current=native.budget()
    require(not current['stopped'] and not any(current.get(k,0) for k in ('unknown_usage','unknown_external_cost_requests','unsettled_requests','unresolved_tool_requests','unknown_compute_requests')), 'native reconciliation left a blocker')
    with ledger.locked() as (f,rows):
        prior=[r['event'] for r in rows if r['event'].get('kind')=='bill_reconciled' and r['event'].get('key')==proof['call_key']]
        require(not prior or prior==[event], 'reconciliation drift')
        if not prior:
            require(rows[-1]['hash']==head, 'campaign changed during reconciliation; native proof retained, revalidate before append')
            ledger.fold(rows+[ledger.row(rows,event)]);ledger.append_locked(f,rows,event)
    return {'event':event,'native_proof':native_proof,'applied':True,'status':ledger.status()}

def retry_state(proof, review, rows, pending, done, source):
    """Durable one-request allowance. A valid semantic result is never retried."""
    reviewed(proof,review,retry=True)
    require(not pending and source==proof['source'], 'pending effect or stale retry source')
    # Only dispatches after the frozen failure count against this ticket.
    newer=[(r['event']['name'][9:],r['event']['value']) for r in rows[proof['adapter_rows']:] if r['event']['kind']=='result' and r['event'].get('name','').startswith('dispatch_')]
    require(len(newer)<=1, 'one review allowance exceeded')
    if not newer:return 'fresh'
    ident,value=newer[0]
    require(done.get('request_body_'+ident,{}).get('kind')=='review' and done['request_body_'+ident]['body']==proof['body'], 'retry request changed')
    meter=value.get('receipt',{})
    require(not value.get('resource_stop') and value.get('candidate') is not None and meter.get('usage_status')=='reported' and meter.get('usd') is not None and 'settle_'+ident in done, 'fresh review failed or unknown; no further retry')
    return ident

class Adapter(ordinary.Adapter):
    """Same admitted ordinary controller; a reviewed ticket resumes one callback."""
    def __init__(self,*args,proof,review,**kwargs):
        self.failed_review_proof=proof;self.failed_review_review=review
        reviewed(proof,review,retry=True)
        super().__init__(*args,**kwargs)
    def ticket_state(self):
        p=self.failed_review_proof;verify_prefix(p);check_stop(p)
        status=self.ledger.status()
        require(not status['stopped'] and not status['blocking_unknown_calls'] and not status['blocking_unknown_executions'], 'unreconciled campaign or Stop')
        source=self.recovery_source();rows,pending,done=self.read()
        return retry_state(p,self.failed_review_review,rows,pending,done,source)
    def recovery_ready(self):
        p=self.failed_review_proof;_,_,done=history_module.history(Path(p['parent'])/'cycle/cycle.jsonl')
        if p['stage'] in done:return super().recovery_ready()
        return {'ready':True,'ticket':e.sha(self.failed_review_review),'state':self.ticket_state()}
    def native(self,kind,value):
        if kind=='provider_recovery_ready':return self.recovery_ready()
        return super().native(kind,value)
    def replay_checked(self,kind,body):
        p=self.failed_review_proof
        # Once the original review has a durable outer result the ticket is spent
        # and ordinary future choices use the unmodified controller behavior.
        _,_,cd=history_module.history(Path(p['parent'])/'cycle/cycle.jsonl')
        if p['stage'] in cd:return super().replay_checked(kind,body)
        state=self.ticket_state()
        require(kind=='review' and body==p['body'], 'one-shot review wire changed')
        if state=='fresh':return None
        return super().replay_checked(kind,body)
    def call(self,kind,body):
        p=self.failed_review_proof;_,_,cd=history_module.history(Path(p['parent'])/'cycle/cycle.jsonl')
        if p['stage'] in cd:return super().call(kind,body)
        require(self.ticket_state()=='fresh' and kind=='review' and body==p['body'], 'fresh review already consumed or changed')
        claim={'proof':e.sha(p),'review':e.sha(self.failed_review_review),'body_sha256':e.sha(body),'original_request':p['request']}
        require(self.op('failed_review_retry_claim',lambda:claim)==claim, 'one-shot claim cannot be rebound')
        result=super().call(kind,body)
        require(result[1] is not None, 'review returned no content; no second attempt')
        return result
    def retry_terminal_failure(self,ident,receipt):
        p=self.failed_review_proof;_,_,cd=history_module.history(Path(p['parent'])/'cycle/cycle.jsonl')
        if p['stage'] not in cd:return False
        return super().retry_terminal_failure(ident,receipt)
    def review(self,view,response):
        p=self.failed_review_proof;_,_,cd=history_module.history(Path(p['parent'])/'cycle/cycle.jsonl')
        if p['stage'] in cd:return super().review(view,response)
        require({'kind':'review','context':view,'response':response}==p['callback'], 'retained review callback changed')
        state=self.ticket_state();done=self.read()[2]
        if state!='fresh' and 'reviewed_'+state in done:
            # Close an interrupted local outer callback using its actual retained
            # reviewed result; never invoke a second reviewer for the same ticket.
            record=done['reviewed_'+state];verdict=record['report']['verdict']
            judgment={'decision':'approve' if verdict=='accept' else verdict,'response_digest':view['response_digest'],'reviewer':'separate-runtime-invocation:'+state,'generator':'gpt-6-luna'}
            self.verify_owned_review(judgment,response)
            return {'judgment':judgment,'receipt':{**done['dispatch_'+state]['receipt'],'mode':'fixture' if self.fixture else 'accounted','status':'settled'}}
        return super().review(view,response)

def main():
    import argparse
    parser=argparse.ArgumentParser()
    parser.add_argument('mode',choices=('check','reconcile','run'))
    parser.add_argument('--parent',type=Path,required=True)
    parser.add_argument('--review',type=Path,required=True)
    parser.add_argument('--review-sha256',required=True)
    args=parser.parse_args();parent=args.parent.resolve();proof=read(parent/'recovery-proof.json');review=read(args.review)
    require(e.sha(review)==args.review_sha256 and proof['parent']==str(parent), 'review/parent binding mismatch')
    reviewed(proof,review,retry=args.mode!='reconcile');verify_prefix(proof);check_stop(proof)
    man=read(e.ROOT/'target/tsr-integrated-campaign-1/manifest.json');up=read(e.ROOT/'target/game-wants-live-v3/campaign-upgrade.json')
    ledger=campaign.Ledger(man['ledger_path'],man,campaign.digest(man),upgrade=up,upgrade_hash=campaign.digest(up))
    ad=read(parent/'adapter-admission.json');ca=read(parent/'cycle-admission.json');native=d.Native(ad['origin']['directory'])
    if args.mode=='reconcile':
        print(json.dumps(reconcile(proof,review,ledger,native),indent=2));return
    launch=a.load('failed_review_launch','engine-native-cycle.py')
    class Captured(Exception):pass
    class CheckAdapter(Adapter):
        def checked(self,kind,body,validate,review=False):
            require(kind=='review' and review,'only exact review preflight')
            wire,_=a.base.bound_output.prepare_review_body(body,aliases=getattr(self,'review_reference_aliases',False))
            require(wire==proof['body'], 'prospective review request differs from failed request')
            raise Captured()
    cls=CheckAdapter if args.mode=='check' else Adapter
    adapter=cls(ad['directory'],ad,native,ledger,proof=proof,review=review,before_call=lambda:launch.refresh_price(ledger,campaign))
    with os.fdopen(os.open(adapter.root/'launch.lock',os.O_WRONLY|os.O_NOFOLLOW),'a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB);adapter.launch_lock_owned=True
        adapter.validate_current_admission();check_stop(proof)
        if args.mode=='check':
            require(adapter.ticket_state()=='fresh','only unconsumed ticket preflight')
            try:adapter.review(proof['callback']['context'],proof['callback']['response'])
            except Captured:
                print(json.dumps({'ready':True,'same_review_body_sha256':e.sha(proof['body']),'provider_calls':0,'target_executions':0}));return
            raise ValueError('review preflight did not capture exact wire')
        adapter.recover_predispatch();adapter.recover_session_transition()
        cycle=ordinary.Cycle(ca['journal'],ca,adapter.native,adapter.proposal,adapter.review)
        result=cycle.run()
        print(json.dumps({'result':result,'campaign':ledger.status()},indent=2))

if __name__=='__main__':main()
