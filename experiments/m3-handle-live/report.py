"""Predeclared six-pair viability decision; never replaces or excludes missing slots."""
import json
from pathlib import Path
EXPECTED=['old','new','new','old','old','new','new','old','old','new','new','old']
def evaluate(terminal):
    rows=terminal['results'];assert [r['index'] for r in rows]==list(range(len(rows))) and len(rows)<=12
    assert all(r['interface']==EXPECTED[r['index']] for r in rows)
    stats={}
    for interface in ('old','new'):
        group=[r for r in rows if r['interface']==interface]
        stats[interface]=dict(scheduled=6,terminal=len(group),missing=6-len(group),oracle_completed=sum(bool(r['final_oracle_completed']) for r in group),verified_accepted=sum(bool(r['final_oracle_success']) for r in group),malformed_or_stale=sum(r['outcome'] in ('schema-or-response-failure','handle-precondition','stale-digest-or-oversize') for r in group),multi_edit_verified=sum((r.get('freshness') or {}).get('result',{}).get('fresh_edits',0)>=2 for r in group))
    old,new=stats['old'],stats['new']
    checks=dict(complete=terminal['status']=='completed' and len(rows)==12,new_oracle_at_least_five=new['oracle_completed']>=5,new_acceptance_at_least_three=new['verified_accepted']>=3,no_new_malformed_or_stale=new['malformed_or_stale']==0,multi_edit_at_least_three=new['multi_edit_verified']>=3,no_lower_completion=new['oracle_completed']>=old['oracle_completed'],no_lower_acceptance=new['verified_accepted']>=old['verified_accepted'])
    return dict(decision='go-to-separately-approved-TSR-comparison' if all(checks.values()) else 'no-go-or-inconclusive',checks=checks,arms=stats,limits='Six pairs on one reused constructed mutation task; no statistical superiority or M3 completion claim. Missing slots stay in six-per-interface denominators.')
if __name__=='__main__':
    import sys
    print(json.dumps(evaluate(json.loads(Path(sys.argv[1]).read_text())),indent=2))
