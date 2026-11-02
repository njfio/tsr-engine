from pathlib import Path
import json,decimal,shutil
r=Path(__file__).resolve().parents[2];s=r/'target/m3-handle-live-amendment-v1-state';b=json.loads((r/'docs/m3-handle-live-budget-baseline.json').read_text());raw=(r/'target/m3-provider-cost.jsonl').read_bytes()[b['bytes']:]
if raw and not raw.endswith(b'\n'):print('ledger write in progress');raise SystemExit
rows=[json.loads(l) for l in raw.splitlines()];reserve={x['id']:x for x in rows if x['kind']=='reserve'};settled={x['id']:x for x in rows if x['kind']=='settled'}
decisions=[json.loads((r/'target/m3-handle-live-state/0/decision.json').read_text())]+[json.loads(p.read_text()) for p in sorted(s.glob('*/decision.json'),key=lambda p:int(p.parent.name))];active=[]
for p in s.glob('*/*-record.json'):
 v=json.loads(p.read_text())
 if v['status']=='running':active.append(dict(index=int(p.parent.name),status=v['status'],heartbeat=v.get('heartbeat')))
paid=sum((decimal.Decimal(x['usd']) for x in settled.values()),decimal.Decimal(0));pending=sum((decimal.Decimal(x['usd']) for i,x in reserve.items() if i not in settled),decimal.Decimal(0))
print(json.dumps(dict(active=active,completed=len(decisions),latest={k:decisions[-1].get(k) for k in ('index','interface','outcome','final_oracle_completed','final_oracle_success','requests')} if decisions else None,calls=len(reserve),settled_calls=len(settled),incremental_settled_usd=str(paid),cumulative_settled_usd=str(paid+decimal.Decimal(b['settled_usd'])),outstanding_reserved_usd=str(pending),tokens=sum(x.get('usage',{}).get('total_tokens') or 0 for x in settled.values()),free_gib=round(shutil.disk_usage(r).free/2**30,2),terminal=json.loads((s/'terminal.json').read_text()).get('status') if (s/'terminal.json').exists() else None)))
