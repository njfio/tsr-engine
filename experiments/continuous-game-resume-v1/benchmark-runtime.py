"""Offline exact-state runtime comparison; no model or action dispatch."""
import hashlib,json,shutil,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'target/continuous-game-resume-v1/runtime-comparison-v2'
OUT.mkdir(exist_ok=False)
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def tree(p):return {str(x.relative_to(p)):digest(x) for x in sorted(p.rglob('*')) if x.is_file()}
def save(name,value):(OUT/name).write_text(json.dumps(value,indent=2)+'\n')
results=[]
for label,source in [('six',ROOT/'target/continuous-game-live-v1/migration-rehearsal/native'),('eighteen',ROOT/'target/continuous-game-resume-v1/tsr/native')]:
    state=OUT/label;original=tree(source)
    values={}
    for profile in ('debug','release'):
        shutil.copytree(source,state);before=tree(state)
        binary=ROOT/'target'/profile/'tsr';started=time.monotonic()
        try:
            result=subprocess.run([str(binary),'engine','--directory',str(state),'project','cycle','request'],capture_output=True,timeout=120)
            (OUT/(label+'-'+profile+'.stdout')).write_bytes(result.stdout)
            (OUT/(label+'-'+profile+'.stderr')).write_bytes(result.stderr)
            record={'state':label,'profile':profile,'seconds':time.monotonic()-started,'returncode':result.returncode,'binary_sha256':digest(binary)}
            if result.returncode==0:values[profile]=json.loads(result.stdout)
        except subprocess.TimeoutExpired as error:
            record={'state':label,'profile':profile,'seconds':time.monotonic()-started,'timeout':120,'binary_sha256':digest(binary)}
            (OUT/(label+'-'+profile+'.stderr')).write_bytes(error.stderr or b'')
        after=tree(state)
        changes=[k for k in before.keys()|after.keys() if before.get(k)!=after.get(k)]
        record['derived_projection_changes']=changes
        assert all(k.startswith('shadow/') for k in changes) and tree(source)==original,'authoritative state changed'
        state.rename(OUT/(label+'-'+profile+'-state'))
        results.append(record);save('progress.json',results)
    if len(values)==2:
        assert values['debug']==values['release'],'runtime outputs differ: inspect exact saved outputs'
        results.append({'state':label,'exact_json_equivalent':True,'normalization':None})
    else:results.append({'state':label,'exact_json_equivalent':None,'reason':'not both outputs completed'})
save('result.json',results)
print(json.dumps(results,indent=2))
