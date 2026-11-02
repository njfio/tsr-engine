"""Exact failed-zero reconciliation for a separately admitted patch recovery.

No dispatch, no admission migration, no change to pinned campaign/recovery files.
The join below follows Ledger.reconcile_failed_zero; only historical admission,
archive and origin readers differ. Append requires independent proof review.
"""
import importlib.util
import hashlib
import datetime
import decimal
from pathlib import Path
spec=importlib.util.spec_from_file_location('recovery_billing_campaign',Path(__file__).with_name('engine-b91-campaign.py'))
c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)
digest=c.digest
identity=c.identity
load_module=c.load_module
MODEL=c.MODEL
ENDPOINT=c.ENDPOINT

class RecoveryLedger(c.Ledger):
    def read_history(self,root):
        """Trusted code hook; never resolve a reader from provider/evidence content."""
        recovery=load_module('campaign_historical_recovery','engine-b91-patch-recovery.py');d=recovery.d
        raw=d.e.bounded_file(root/'driver.jsonl',8*1024*1024);native=[d.e.strict(line) for line in raw.splitlines()]
        a=native[0]['event']['value']
        recovery.validate(a,root,a['mode']=='fixture',expired=True,full=True)
        view=object.__new__(recovery.Recovery);view.root=root;view.file=root/'driver.jsonl';view.admission=a;view.fixture=a['mode']=='fixture'
        _,_,done=view.read()
        view.verify_archive()
        return d,raw,native,a,view,done,view.journal()

    def reconcile_failed_zero(self,trial_root,lookup_path,review,review_hash,*,dry_run=False):
        """Read-only exact historical joins, then one append. No provider lookup or retry."""
        if digest(review)!=review_hash or set(review)!={'reviewer','provider_endpoint','lookup_sha256','billing_semantics_sha256','decision'}:
            raise ValueError('exact independent reconciliation review required')
        if review['decision']!='exact_failed_zero' or not isinstance(review['reviewer'],str) or not 1<=len(review['reviewer'])<=256:
            raise ValueError('independent failed-zero review required')
        if not review['provider_endpoint'].startswith('https://api.cheaperinference.com/v1/usage/requests?request_ids='):raise ValueError('official exact request lookup required')
        identity(review['billing_semantics_sha256'])
        root=Path(trial_root).resolve()
        d,raw,native,a,view,done,journal=self.read_history(root)
        if raw!=d.e.bounded_file(root/'driver.jsonl',8*1024*1024):raise ValueError('historical reader bytes mismatch')
        if native!=[d.e.strict(line) for line in raw.splitlines()] or a!=native[0]['event']['value']:raise ValueError('historical reader admission mismatch')
        if view.root!=root or view.admission!=a or view.read()[2]!=done:raise ValueError('historical reader state mismatch')
        if a['model']!=MODEL or Path(a['trial']).resolve()!=root or view.fixture!=self.manifest['fixture_only']:raise ValueError('historical trial scope mismatch')
        trial=digest({'trial':str(root),'admission':d.e.sha(a)})
        events=journal.events();provider_raw=d.e.bounded_file(root/'provider.jsonl',8*1024*1024)
        lookup_raw=d.e.bounded_file(Path(lookup_path),262144)
        if hashlib.sha256(lookup_raw).hexdigest()!=review['lookup_sha256']:raise ValueError('lookup evidence changed')
        lookup=d.e.strict(lookup_raw);data=lookup.get('record',{}).get('data',[])
        if len(data)!=1 or lookup['record'].get('has_more') is not False:raise ValueError('ambiguous provider lookup')
        r=data[0];provider_id=lookup.get('request_id')
        if review['provider_endpoint']!='https://api.cheaperinference.com/v1/usage/requests?request_ids='+str(provider_id):raise ValueError('lookup query identity mismatch')
        if (r.get('request_id')!=provider_id or r.get('model')!=MODEL or r.get('status')!='failed' or r.get('currency')!='USD'
                or any(type(r.get(k)) is not int or r[k]!=0 for k in ('prompt_tokens','completion_tokens','total_tokens'))
                or r.get('billing',{}).get('status')!='failed' or decimal.Decimal(str(r.get('billing',{}).get('billed_cost_usd'))) != 0
                or (r.get('billed_cost_usd') is not None and decimal.Decimal(str(r['billed_cost_usd']))!=0)):
            raise ValueError('positive terminal failed zero-charge evidence required')
        candidates=[]
        for name,value in done.items():
            receipt=value.get('receipt',{}) if isinstance(value,dict) else {}
            if name.startswith('transport_') and receipt.get('response_request_ids')==[provider_id] and receipt.get('response_status')=='received_http_error' and receipt.get('usd') is None:
                request=name.removeprefix('transport_')
                intent=next((e for e in events if e['kind']=='dispatch_intent' and e['id']==request),None)
                if intent:candidates.append((request,intent,value))
        if len(candidates)!=1:raise ValueError('provider ID not uniquely bound to original native transport')
        request,intent,value=candidates[0]
        def timestamp(text):
            t=datetime.datetime.fromisoformat(text.replace('Z','+00:00'))
            if t.tzinfo is None:raise ValueError('timezone-aware reconciliation chronology required')
            return t.timestamp()
        native_intent=next(row['event'] for row in native if row['event'].get('kind')=='intent' and row['event'].get('name')=='transport_'+request)
        dispatched=timestamp(native_intent['at']);created=timestamp(r['created_at']);retrieved=timestamp(lookup['retrieved_at'])
        # At most five seconds of provider/host clock skew, never an arbitrary older request.
        if not dispatched-5<=created<=retrieved or retrieved<dispatched:raise ValueError('reconciliation chronology mismatch')
        key=digest({'trial':trial,'request':request,'kind':'call'})
        with self.locked() as (f,rows):
            _,trials,calls,_=self.fold(rows)
            if trial not in trials or trials[trial]['admission']!=d.e.sha(a) or key not in calls or calls[key]['binding']!=intent['body_sha256']:raise ValueError('campaign/native request mismatch')
            bill=next((row for row in rows if row['event'].get('kind')=='bill' and row['event']['key']==key and row['event']['microusd'] is None),None)
            if bill is None:raise ValueError('original unknown bill missing')
            event={'kind':'bill_reconciled','key':key,'original_bill_row':bill['hash'],'microusd':0,'basis':'exact_provider_failed_zero',
                'provider_request_id':provider_id,'lookup_sha256':review['lookup_sha256'],'native_journal_sha256':hashlib.sha256(raw).hexdigest(),
                'provider_journal_sha256':hashlib.sha256(provider_raw).hexdigest(),'native_result_sha256':digest(value),'review_sha256':review_hash,
                'billing_semantics_sha256':review['billing_semantics_sha256']}
            prior=next((row['event'] for row in rows if row['event'].get('kind')=='bill_reconciled' and row['event']['key']==key),None)
            if prior is not None:
                if prior!=event:raise ValueError('reconciliation cannot be rewritten')
                return self.fold(rows)[0]
            result=self.fold(rows+[self.row(rows,event)])[0]
            if dry_run:return {'proposed_event':event,'status_after':result,'applied':False}
            self.append_locked(f,rows,event)
            return result
