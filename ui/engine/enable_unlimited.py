"""Apply the exact recorded user policy. No inference, approval impersonation or game effect."""
import argparse,json
from server import Store,ROOT,save
from decisions import REQUEST,REVIEW,KEY

def enable(store,*,apply=False):
 campaign,m,ledger,_=store.ledger();before=ledger.status()
 if before.get('unlimited_policy_sha256'):return {'already_enabled':True,'status':before}
 if before['unknown_executions'] or before['unknown_calls']!=[KEY]:raise ValueError('Exact historical generation only; reconcile any other effect first')
 proof=ledger.quarantine_generation(ROOT/'target/pond-beautiful-live-v1/adapter',REQUEST,REVIEW['journal_sha256'],REVIEW,dry_run=not apply)
 with ledger.locked() as (_,rows):
  if not apply and not any(r['event']['kind']=='generation_quarantined' and r['event']['key']==KEY for r in rows):rows=rows+[ledger.row(rows,proof)]
  policy={'version':'browser-unlimited-policy@v1','manifest_sha256':ledger.hash,'ledger_prefix_head':rows[-1]['hash'],'provider':'CheaperInference','model':m.MODEL,'source_class':'newly-authored-nonprivate-browser-project','authority':m.UNLIMITED_AUTHORITY,'quarantined_unknown_keys':[KEY]}
  after=ledger.fold(rows+[ledger.row(rows,{'kind':'unlimited_policy','key':m.digest(policy),'value':policy})])[0]
 if apply:ledger.enable_unlimited(policy,m.digest(policy));after=ledger.status()
 return {'applied':apply,'policy':policy,'policy_sha256':m.digest(policy),'quarantine':proof,'before':before,'after':after}
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--apply',action='store_true');args=p.parse_args();print(json.dumps(enable(Store(),apply=args.apply),indent=2))
