"""One recorded no-input probe per source: Start click, then read the displayed routes.
Route words derive from this record alone. A second probe on the same source is refused."""
import importlib.util,json,shutil,subprocess,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('pair_prepare',Path(__file__).with_name('prepare.py'));p=importlib.util.module_from_spec(s);s.loader.exec_module(p);a=p.a
def mapper_source(files):
 s=importlib.util.spec_from_file_location('discover_mapping',ROOT/'experiments/continuous-game-v4/mapping.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m.source(files)
SELECTORS={'route':'#route','shard_route':'#shard-route'}
def run():
 p.check_pins();home=ROOT/('target/continuous-game-live-'+p.LABEL)/'tsr';native=a.d.Native(home/'native');snapshot=native.request()['coding']['snapshot']
 out=home/('discovery-'+snapshot+'.json')
 if out.exists():raise ValueError('discovery already recorded for this source')
 ad=json.loads((home/'adapter-admission.json').read_text());files={k:(native.directory/'coding-work'/k).read_text() for k in ad['origin']['initial_sources']}
 probe={'version':'browser-probe@v1','assets':sorted(files),'entry':'index.html','actions':[{'kind':'click','value':'#start'}],'checks':[{'selector':v,'text':''} for v in SELECTORS.values()]}
 with tempfile.TemporaryDirectory(prefix='discovery-',dir=ROOT/'target') as tmp:
  t=Path(tmp);shutil.copyfile(ROOT/'scripts/engine-browser-harness.cjs',t/'browser-harness.cjs');(t/'probe.json').write_text(json.dumps(probe))
  for k,v in files.items():(t/k).write_text(v)
  cmd=['docker','run','--rm','--init','--pull=never','--network=none','--read-only','--cap-drop=ALL','--security-opt=no-new-privileges','--pids-limit=256','--memory=2g','--cpus=2','--user=501:20','--tmpfs=/tmp:rw,nosuid,nodev,exec,size=256m','--mount',f'type=bind,src={t},dst=/workspace,readonly','--workdir=/workspace','--entrypoint=node',a.IMAGE,'browser-harness.cjs','probe.json','--container-only']
  result=subprocess.run(cmd,capture_output=True,timeout=170)
 try:raw=json.loads(result.stdout)
 except ValueError:raw={'unparsed_stdout':result.stdout.decode(errors='replace')[:2000]}
 text={o.get('selector'):o.get('text','') for o in raw.get('observations',[]) if o.get('kind')=='text'}
 report={'source':mapper_source(files),'readouts':{k:text.get(v,'') for k,v in SELECTORS.items()}}
 full={**report,'exit':result.returncode,'stderr':result.stderr.decode(errors='replace')[:2000],'raw':raw,'snapshot':snapshot}
 with out.open('x') as f:json.dump(report,f,indent=2);f.write('\n')
 with (home/('discovery-raw-'+snapshot+'.json')).open('x') as f:json.dump(full,f,indent=2);f.write('\n')
 return {'path':str(out),'readouts':report['readouts'],'exit':result.returncode,'no_game_inputs':True}
if __name__=='__main__':print(json.dumps(run(),indent=2))
