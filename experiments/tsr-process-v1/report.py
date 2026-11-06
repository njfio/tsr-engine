"""What one launch did, from its journals and the native reading view. Reads only; no provider calls.

Counts carry no claim about what they mean. `detected_wants` lists each want that is new after a
return (spec #12, rule 5); whether it is traceable to the task is the operator's call, with the list
in front of them. `person_return` says whether an operator's feedback exists for the final source.
"""
import importlib.util,json,os,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def journal(path):
 rows=[json.loads(line)['event'] for line in Path(path).read_text().splitlines()] if Path(path).exists() else []
 return [r for r in rows if r.get('kind')=='result']
def report(base,view=None):
 ab=load('process_report_abilities',ROOT/'scripts/engine-abilities.py');pr=load('process_report_process',ROOT/'scripts/engine-process.py')
 cycle=journal(base/'tsr/cycle/cycle.jsonl');adapter=journal(base/'tsr/adapter/adapter.jsonl')
 terminal=[r['value'] for r in cycle if r['name']=='terminal']
 value=ab.signals([r['name'] for r in cycle],terminal[-1]['status'] if terminal else None)
 value['tries']=sum(r['name'].startswith('try_evidence_') for r in adapter)
 value['tries_followed_by_rejection']=sum(r['name'].startswith('rejected_proposal_') and any(c['name']=='trial_run_'+r['name'].rsplit('_',1)[1] for c in cycle) for r in cycle)
 if view is None:
  native=load('process_report_driver',ROOT/'scripts/engine-b91-driver.py').Native(base/'tsr/native')
  view=native.cli('project','reading-view')
 bounded=view.get('bounded',view)
 value['detected_wants']=pr.detect(pr.timeline_from_view(bounded))
 value['readings_omitted_from_detection']=bounded.get('omitted',0)
 value['person_return']=any(r['name'].startswith('feedback_') or r['name'].startswith('person_return_') for r in adapter)
 return value
if __name__=='__main__':
 print(json.dumps(report(ROOT/('target/tsr-process-'+os.environ.get('TSR_LIVE_LABEL','r1'))),indent=2))
