"""Terminal read-only per-arm billing and action accounting; no provider access."""
from pathlib import Path
from decimal import Decimal as D
from datetime import datetime
from collections import Counter
import json,sqlite3
ROOT=Path(__file__).resolve().parents[1]

def main():
    state=ROOT/'target/m3-progress-validation-state'
    terminal=json.loads((state/'complete.json').read_text())
    plan=json.loads((state/'plan.json').read_text())
    baseline=json.loads((ROOT/plan['baseline']).read_text())
    raw=(ROOT/plan['shared_ledger']).read_bytes()
    import m3_smoke_budget as budget
    assert budget.freeze(raw[:baseline['bytes']])==baseline
    aggregate=budget.total(raw)
    events=[json.loads(line) for line in raw[baseline['bytes']:].splitlines()]
    reserved={e['id']:e for e in events if e['kind']=='reserve'}
    settled={e['id']:D(e['usd']) for e in events if e['kind']=='settled'}
    assert reserved.keys()==settled.keys()
    arms=[];assigned=set()
    for job in plan['jobs']:
        for mode in job['modes']:
            path=state/(job['task_id'][:12]+'-'+mode+'.json')
            if not path.exists():
                arms.append(dict(task=job['task_id'],mode=mode,status='not_attempted'));continue
            process=json.loads(path.read_text());start=datetime.fromisoformat(process['started']);end=datetime.fromisoformat(process['finished'])
            ids={ident for ident,e in reserved.items() if start.timestamp()<=e['at']<=end.timestamp()}
            assert not assigned & ids;assigned |= ids
            store=ROOT/job['input']/mode/'state'/job['task_id']/(mode+'.sqlite')
            # Original DB could use WAL; terminal read-only connection observes committed WAL.
            with sqlite3.connect('file:'+str(store)+'?mode=ro',uri=True) as db:
                events_in_store=[json.loads(row[0])['kind'] for row in db.execute('select payload from events order by seq')]
            actions=Counter(e['candidate']['kind'] for e in events_in_store if e['kind']=='proposed')
            closure=[e['reason'] for e in events_in_store if e['kind']=='task_closed']
            arms.append(dict(task=job['task_id'],mode=mode,status=process['status'],started=process['started'],finished=process['finished'],process_wall_seconds=(end-start).total_seconds(),settled_calls=len(ids),usd=str(sum((settled[k] for k in ids),D(0))),actions=dict(actions),closure_reasons=closure))
    assert assigned==reserved.keys()
    diagnostics=[json.loads(line) for line in (ROOT/'target/m3-provider-cost-diagnostics.jsonl').read_text().splitlines()]
    diagnostics=[e for e in diagnostics if e.get('reservation_id') in reserved]
    report=dict(launch=json.loads((ROOT/'docs/m3-progress-validation-launch.json').read_text())['started'],finished=terminal['finished'],controller_status=terminal['status'],arms=arms,total_calls=len(reserved),incremental_usd=str(sum(settled.values(),D(0))),aggregate_usd=str(aggregate),outstanding_usd='0',diagnostics=diagnostics)
    (ROOT/'docs/m3-progress-validation-accounting.json').write_text(json.dumps(report,indent=2)+'\n')
    print(json.dumps({k:v for k,v in report.items() if k!='diagnostics'},indent=2))
if __name__=='__main__':main()
