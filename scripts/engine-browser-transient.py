"""Admission-governed failed-provider recovery; never retries uncertain effects."""
import copy
import decimal
import time

def validate(policy):
    if policy is None:return
    if not isinstance(policy,dict) or set(policy)!={'version','max_retries','backoff_seconds'} or policy['version']!='exact-failed-zero-recovery@v1' or type(policy['max_retries']) is not int or not 1<=policy['max_retries']<=10 or type(policy['backoff_seconds']) is not int or not 60<=policy['backoff_seconds']<=600:
        raise ValueError('explicit bounded provider recovery policy required')

class Recovery:
    def recovery_deadline(self):
        return float('inf') if self.admission.get('unlimited_policy_sha256') and self.admission['expires_at'] is None else self.recovery_stamp(self.admission['expires_at'])
    def recovery_modules(self):
        from pathlib import Path
        # Existing independently reviewed exact billing reader and validator.
        return self.recovery_load('browser_failure_proof','engine-objective-continuation.py')
    def recovery_source(self):
        request=self.native('request',None)
        return {k:request[k] for k in ('scope','state_root','source_snapshot')}
    def _wire(self,body,timeout):
        start=time.time();result=super()._wire(body,timeout)
        policy=self.admission.get('transient_recovery');receipt=result.get('receipt',{})
        if not policy or receipt.get('http_status')!=503 or result.get('value') is not None:return result
        try:
            proof=self.recovery_modules();ids=receipt['response_request_ids']
            if len(ids)!=1 or time.time()-start>timeout-25:return result
            used={pid for k,v in self.read()[2].items() if k.startswith('dispatch_') for pid in v.get('receipt',{}).get('response_request_ids',[])}
            if ids[0] in used:return result
            lookup=getattr(self,'failure_lookup',proof.lookup_failed)(ids[0])
            proof.failed_zero(receipt,lookup,earliest=start)
            if time.time()-start>timeout:return result
        except (ValueError,KeyError,TypeError,OSError,decimal.InvalidOperation):return result
        self.op('provider_zero_'+self.recovery_ident,lambda:{'original_receipt':copy.deepcopy(receipt),'lookup':lookup,'body_sha256':self.recovery_sha(body),'wire':copy.deepcopy(body),'original_sha256':getattr(self,'recovery_original_digest',None),'source':self.recovery_source()})
        return {**result,'receipt':{**receipt,'status':'settled','usd':'0.000000','usage':{'prompt_tokens':0,'completion_tokens':0,'total_tokens':0},'usage_status':'reported','elapsed_seconds':time.time()-start}}
    def recovery_ticket(self):
        _,pending,done=self.read()
        if pending or self.ledger.status()['stopped']:raise ValueError('unresolved external or adapter effect; no retry')
        dispatches=[k[len('dispatch_'):] for k in done if k.startswith('dispatch_')]
        if not dispatches:return None
        ident=dispatches[-1];proof=done.get('provider_zero_'+ident)
        if proof is None:return None
        if 'settle_'+ident not in done:raise ValueError('failed request settlement missing')
        policy=self.admission.get('transient_recovery');validate(policy)
        if policy is None:raise ValueError('provider retry authority absent')
        if proof['source']!=self.recovery_source():raise ValueError('retry source changed')
        name='provider_retry_'+ident
        if name not in done:
            if sum(k.startswith('provider_retry_') for k in done)>=policy['max_retries']:raise ValueError('provider retry allowance exhausted')
            if (self.admission['max_calls'] is not None and len(dispatches)>=self.admission['max_calls']) or time.time()+policy['backoff_seconds']+240>=self.recovery_deadline():raise ValueError('provider retry cannot fit remaining envelope')
            self.op(name,lambda:{'failed_request':ident,'proof_sha256':self.recovery_sha(proof),'body_sha256':proof['body_sha256'],'source':proof['source'],'not_before':time.time()+policy['backoff_seconds']})
        return self.read()[2][name]
    def recovery_wire(self,original,wire):
        self.recovery_original_digest=self.recovery_sha(original)
        if not self.admission.get('transient_recovery'):return wire
        ticket=self.recovery_ticket()
        if ticket is None:return wire
        proof=self.read()[2]['provider_zero_'+ticket['failed_request']]
        if proof['original_sha256']!=self.recovery_sha(original) or self.recovery_sha(proof['wire'])!=proof['body_sha256']:raise ValueError('retained retry task/wire mismatch')
        return copy.deepcopy(proof['wire'])
    def call(self,kind,body):
        ticket=self.recovery_ticket() if self.admission.get('transient_recovery') else None
        if ticket:
            if ticket['body_sha256']!=self.recovery_sha(body):raise ValueError('retry request changed; no dispatch')
            while time.time()<ticket['not_before']:
                if time.time()+240>=self.recovery_deadline():raise ValueError('retry deadline expired')
                time.sleep(min(30,ticket['not_before']-time.time()))
            if ticket['source']!=self.recovery_source():raise ValueError('retry source changed during backoff')
        count=sum(k.startswith(('dispatch_','not_dispatched_')) for k in self.read()[2])
        self.recovery_ident=self.recovery_sha({'admission':self.recovery_sha(self.admission),'kind':kind,'index':count,'body':self.recovery_sha(body)})
        return super().call(kind,body)
    def retry_terminal_failure(self,ident,receipt):
        if 'provider_zero_'+ident not in self.read()[2]:return super().retry_terminal_failure(ident,receipt)
        self.recovery_ticket()
        return True
    def recovery_ready(self):
        self.validate_current_admission()
        ticket=self.recovery_ticket()
        if ticket is None:raise ValueError('no exact settled failed request eligible for resume')
        return {'ready':True,'ticket_sha256':self.recovery_sha(ticket)}
