"""Offline historical-binary equivalence on identical completed-state copies.

No production timeout, admission, provider, or original episode is modified.
Both invocations use the same canonical directory; no JSON field normalization.
"""
import hashlib,json,shutil,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'target/continuous-game-resume-v1'
OUT=ROOT/'target/full-state-runtime-equivalence';OUT.mkdir(exist_ok=False)
SOURCE=BASE/'tsr/native';STATE=OUT/'same-path-state'
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def tree(p):return {str(x.relative_to(p)):digest(x) for x in sorted(p.rglob('*')) if x.is_file()}
def save(name,v):(OUT/name).write_text(json.dumps(v,indent=2)+'\n')
def stable(rows):return {k:v for k,v in rows.items() if not k.startswith('shadow/') and not k.endswith(('-shm','-wal'))}
provenance=json.loads((BASE/'runtime-build-provenance.json').read_text())
binaries={'debug':BASE/'pre-optimization-debug-tsr','optimized':ROOT/'target/release/tsr'}
assert digest(binaries['debug'])==provenance['debug_sha256']
assert digest(binaries['optimized'])==provenance['optimized_sha256']
original=tree(SOURCE)
assert not (SOURCE/'events.sqlite-wal').exists() or (SOURCE/'events.sqlite-wal').stat().st_size==0
protected=[ROOT/'target/continuous-game-campaign-v1/ledger.jsonl',ROOT.parent/'tsr-engine-m3/target/tsr-integrated-campaign-1/ledger.jsonl']
for parent in [BASE/'tsr',ROOT/'target/continuous-game-live-v1/tsr']:
 protected.extend(parent.rglob('adapter.jsonl'));protected.extend(parent.rglob('cycle.jsonl'))
protected_hashes={str(p):digest(p) for p in protected}
save('input.json',{'source':str(SOURCE),'source_files':original,'protected':protected_hashes,'build_provenance':provenance,'same_path':str(STATE),'deadline':'No additional offline subprocess deadline; production120seconds unchanged','order':['debug','optimized'],'cache_limit':'Sequential process runs; OS cache effects not controlled. Not a cold-cache speed claim.'})
results=[];outputs={}
for label,binary in binaries.items():
 shutil.copytree(SOURCE,STATE);assert tree(STATE)==original
 before=tree(STATE);started=time.monotonic()
 with (OUT/(label+'.stdout')).open('xb') as stdout,(OUT/(label+'.stderr')).open('xb') as stderr:
  child=subprocess.Popen([str(binary),'engine','--directory',str(STATE),'project','cycle','request'],stdout=stdout,stderr=stderr)
  save('active.json',{'profile':label,'pid':child.pid,'started_unix':time.time(),'completed_profiles':results})
  code=child.wait()
 seconds=time.monotonic()-started;after=tree(STATE)
 changes=[k for k in before.keys()|after.keys() if before.get(k)!=after.get(k)]
 assert stable(before)==stable(after),'authoritative files changed'
 for k in changes:
  if k.startswith('shadow/'):continue
  assert k in ('events.sqlite-shm','events.sqlite-wal') and k not in after
  if k.endswith('-wal'):assert before[k]==hashlib.sha256(b'').hexdigest()
 assert tree(SOURCE)==original and all(digest(Path(k))==v for k,v in protected_hashes.items())
 result={'profile':label,'binary_sha256':digest(binary),'seconds':seconds,'returncode':code,'stdout_sha256':digest(OUT/(label+'.stdout')),'changes':changes,'authoritative_unchanged':True,'original_and_ledgers_unchanged':True}
 results.append(result);save('progress.json',results);save(label+'-after-files.json',after)
 STATE.rename(OUT/(label+'-completed-state'))
 assert code==0,'request failed; retain diagnostics'
 outputs[label]=(OUT/(label+'.stdout')).read_bytes()
same_json=json.loads(outputs['debug'])==json.loads(outputs['optimized'])
save('result.json',{'profiles':results,'complete_json_equal':same_json,'byte_equal':outputs['debug']==outputs['optimized'],'normalization':None,'fields_dropped':[],'source_files_unchanged':tree(SOURCE)==original,'protected_unchanged':all(digest(Path(k))==v for k,v in protected_hashes.items()),'new_provider_calls':0,'coverage':'One completed eighteen-bundle state, request construction only; no universal runtime equivalence claim.'})
save('active.json',{'terminal':True,'profiles':results})
assert same_json,'complete request JSON differs; inspect saved outputs'
print((OUT/'result.json').read_text())
