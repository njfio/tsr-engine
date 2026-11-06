"""Counts from one launch's cycle journal. Reads only; no provider calls. No claim about what they mean."""
import importlib.util,json,os,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('want_loop_abilities',ROOT/'scripts/engine-abilities.py');ab=importlib.util.module_from_spec(s);s.loader.exec_module(ab)
def report(path):
 rows=[json.loads(line)['event'] for line in Path(path).read_text().splitlines()]
 done=[r for r in rows if r.get('kind')=='result']
 terminal=[r['value'] for r in done if r['name']=='terminal']
 return ab.signals([r['name'] for r in done],terminal[-1]['status'] if terminal else None)
if __name__=='__main__':
 print(json.dumps(report(ROOT/('target/want-loop-'+os.environ.get('TSR_LIVE_LABEL','r1'))/'tsr/cycle/cycle.jsonl'),indent=2))
