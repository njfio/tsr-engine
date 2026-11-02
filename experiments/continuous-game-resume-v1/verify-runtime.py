"""Finish preserved benchmark after SQLite sidecar cleanup assertion; no effects."""
import hashlib,json,shutil,subprocess,time
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];BASE=ROOT/'target/continuous-game-resume-v1';OLD=BASE/'runtime-comparison-v2';OUT=BASE/'runtime-comparison-final';OUT.mkdir(exist_ok=False)
def digest(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def tree(p):return {str(x.relative_to(p)):digest(x) for x in sorted(p.rglob('*')) if x.is_file()}
def stable(rows):return {k:v for k,v in rows.items() if not k.startswith('shadow/') and not k.endswith(('-shm','-wal'))}
records=json.loads((OLD/'progress.json').read_text());assert records[-1]['state']=='eighteen' and records[-1]['timeout']==120
for r in records:
    if 'profile' in r:assert r['binary_sha256']==digest(ROOT/'target'/r['profile']/'tsr')
assert json.loads((OLD/'six-debug.stdout').read_text())==json.loads((OLD/'six-release.stdout').read_text())
source=BASE/'tsr/native';original=tree(source);assert stable(tree(OLD/'eighteen-debug-state'))==stable(original)
state=OUT/'eighteen';shutil.copytree(source,state);before=tree(state);started=time.monotonic()
r=subprocess.run([str(ROOT/'target/release/tsr'),'engine','--directory',str(state),'project','cycle','request'],capture_output=True,timeout=120)
seconds=time.monotonic()-started;(OUT/'eighteen-release.stdout').write_bytes(r.stdout);(OUT/'eighteen-release.stderr').write_bytes(r.stderr)
assert r.returncode==0 and tree(source)==original
after=tree(state);assert stable(before)==stable(after)
changes=[k for k in before.keys()|after.keys() if before.get(k)!=after.get(k)]
for k in changes:
    if k.startswith('shadow/'):continue
    assert k in ('events.sqlite-shm','events.sqlite-wal') and k not in after
    if k.endswith('-wal'):assert before[k]==hashlib.sha256(b'').hexdigest()
record={'state':'eighteen','profile':'release','seconds':seconds,'returncode':0,'binary_sha256':digest(ROOT/'target/release/tsr'),'derived_projection_changes':changes,'authoritative_files':stable(original),'stdout_sha256':digest(OUT/'eighteen-release.stdout'),'source_unchanged':True,'exact_json_equivalent':None,'limit':'Debug exceeded unchanged120-second deadline; full-state equivalence not demonstrated. Six-bundle JSON exact match.'}
records.append(record);(OUT/'result.json').write_text(json.dumps(records,indent=2)+'\n');print(json.dumps({'seconds':seconds,'unchanged':True,'changes':changes},indent=2))
