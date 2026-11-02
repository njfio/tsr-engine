"""Offline packet construction/check only. No grant, model or execution dispatch."""
import hashlib,json,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def projection(index):
 data=json.loads((HERE/'conditions.json').read_text());c=data['conditions'][index]
 return {'attribution':'authored synthetic environment; not user experience','wants':data['wants'],'obligations':data['obligations'],'condition':c,'control_contract':data['control_contract'],'decision_freedom':data['decision_freedom']}
def build():
 template=(HERE/'observer-template.cjs').read_text()
 (HERE/'observe.cjs').write_text(template)
 manifest={'version':'relational-observation-offline-packet@v1','implementation_commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'status':'offline_only_not_admitted','provider':'CheaperInference','model':'gpt-6-luna','settings':{'max_completion_tokens':32768,'stream':False,'response_format':{'type':'json_object'},'temperature':'omitted','reasoning_effort':'omitted'},'policy':{'cumulative_calls':None,'cumulative_dollars':None,'cumulative_time':None,'cumulative_operations':None,'stop':['three condition checkpoints complete','explicit abstention','human Stop','unsafe or unresolved external effect','substantive blocker'],'per_request_capacity_timeouts_and_bounded_backoff':'unchanged; no infinite failure retries'},'runtime_image':'sha256:58deb9d12b7291f6d2c697b335eb927557d707488df09d1503659d7abdd00883','files':{p.name:digest(p) for p in sorted(HERE.iterdir()) if p.is_file() and p.name!='manifest.json'},'implementation_files':{str(p.relative_to(ROOT)):digest(p) for p in sorted((ROOT/'scripts').glob('engine-*.py'))},'verification_files':{'scripts/test_relational_observation_packet.py':digest(ROOT/'scripts/test_relational_observation_packet.py')},'binary_sha256':digest(ROOT/'target/debug/tsr'),'criterion_profile':'public-criterion-return@v2','transient_recovery':{'version':'exact-failed-zero-recovery@v1','max_retries':1,'backoff_seconds':60},'fresh_admission_required':True}
 (HERE/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
def check():
 m=json.loads((HERE/'manifest.json').read_text())
 for n,h in m['files'].items():assert digest(HERE/n)==h,n
 for n,h in {**m['implementation_files'],**m['verification_files']}.items():assert digest(ROOT/n)==h,n
 assert digest(ROOT/'target/debug/tsr')==m['binary_sha256']
 assert all(v is None for k,v in m['policy'].items() if k.startswith('cumulative_'))
 for i in range(3):
  p=projection(i);assert p['condition']['id'] in ('opening','storm','reopening')
 assert (HERE/'observe.cjs').read_bytes()==(HERE/'observer-template.cjs').read_bytes()
 print(json.dumps({'packet_sha256':digest(HERE/'manifest.json'),'paid_calls':0,'admissions':0,'status':'offline_prepared'}))
if __name__=='__main__':
 import sys
 if sys.argv[1:] == ['--build']:build()
 check()
