"""One-shot approved14-build subset. Never operate on Docker-held six paths."""
import importlib.util,json,os,shutil,subprocess,sys,hashlib
from datetime import datetime,timezone
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
DEST=ROOT/'docs/m3-fixed12-approved-cleanup/subset14'
DEPENDENCIES=['scripts/m3-fixed12-approved-cleanup.py','docs/m3-fixed12-approved-cleanup/before.json','docs/m3-fixed12-approved-cleanup/open-reference-diagnosis.json']
def dependency_hashes():return {p:hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in DEPENDENCIES}
if sys.argv[1:]==['delete-reviewed-subset']:
    assert dependency_hashes()==json.loads((DEST/'before.json').read_text())['dependencies'], 'reviewed dependency changed'
spec=importlib.util.spec_from_file_location('base',Path(__file__).with_name('m3-fixed12-approved-cleanup.py'));base=importlib.util.module_from_spec(spec);spec.loader.exec_module(base)
ROOT=base.ROOT;DEST=base.DEST/'subset14'
def identity(p):return [p.stat().st_dev,p.stat().st_ino]
def scope():
    allpaths=base.targets()
    diagnosis=json.loads((base.DEST/'open-reference-diagnosis.json').read_text())
    held={ROOT/r['target'] for r in diagnosis['open_refs']}
    assert len(held)==6 and held <= set(allpaths)
    safe=[p for p in allpaths if p not in held]
    assert len(safe)==14
    for p in safe:
        for current,dirs,files in os.walk(p,followlinks=False):
            for n in dirs+files:assert not (Path(current)/n).is_symlink(),str(Path(current)/n)
    return allpaths,safe,sorted(held)
def active(paths):
    result=subprocess.run(['lsof','-nP','-Fpn'],capture_output=True,text=True)
    assert result.returncode==0,result.stderr
    hits=[];pid=None
    for line in result.stdout.splitlines():
        if line.startswith('p'):pid=line[1:]
        elif line.startswith('n'):
            name=line[1:].removesuffix(' (deleted)')
            if any(name==str(p) or name.startswith(str(p)+'/') for p in paths):hits.append([pid,name])
    ids=subprocess.check_output(['docker','ps','-aq'],text=True).split();mounts=[]
    for cid in ids:
        row=json.loads(subprocess.check_output(['docker','inspect','--format','{"id":{{json .Id}},"state":{{json .State.Status}},"mounts":{{json .Mounts}}}',cid],text=True))
        for m in row['mounts']:
            src=m.get('Source','').rstrip('/')
            if src and any(str(p)==src or str(p).startswith(src+'/') or src.startswith(str(p)+'/') for p in paths):mounts.append(row)
    return dict(at=datetime.now(timezone.utc).isoformat(),targets=[str(p.relative_to(ROOT)) for p in paths],open_refs=hits,container_overlaps=mounts,containers_checked=len(ids),safe=not hits and not mounts)
def main():
    allpaths,paths,held=scope();names=[str(p.relative_to(ROOT)) for p in paths]
    if sys.argv[1:]==['prepare']:
        DEST.mkdir()
        baseline=json.loads((base.DEST/'before.json').read_text())
        assert base.protected(allpaths)==baseline['protected']
        base.save(DEST/'before.json',dict(targets=names,held=[str(p.relative_to(ROOT)) for p in held],identities={str(p.relative_to(ROOT)):identity(p) for p in allpaths},free_bytes=shutil.disk_usage(ROOT).free,script_sha256=base.digest(Path(__file__)),dependencies=dependency_hashes(),protected_count=len(baseline['protected'])))
        print('Prepared14only; zero deletion.')
    elif sys.argv[1:]==['delete-reviewed-subset']:
        before=json.loads((DEST/'before.json').read_text());review=json.loads((DEST/'review.json').read_text())
        assert before['targets']==names and before['script_sha256']==base.digest(Path(__file__))
        assert review['dependencies']==before['dependencies']
        assert review['safe'] is True and review['approved_scope']==names and review['script_sha256']==before['script_sha256']
        assert not (DEST/'intent.json').exists()
        baseline=json.loads((base.DEST/'before.json').read_text())
        assert base.protected(allpaths)==baseline['protected']
        assert before['identities']=={str(p.relative_to(ROOT)):identity(p) for p in allpaths}
        check=active(paths);base.save(DEST/'active-use.json',check)
        assert check['safe'],'active use: no deletion'
        assert (datetime.now(timezone.utc)-datetime.fromisoformat(check['at'])).total_seconds()<60
        free=shutil.disk_usage(ROOT).free
        base.save(DEST/'intent.json',dict(targets=names,free_bytes=free))
        for p in paths:
            assert identity(p)==before['identities'][str(p.relative_to(ROOT))] and not p.is_symlink()
            shutil.rmtree(p)
        assert all(not p.exists() for p in paths)
        assert all(identity(p)==before['identities'][str(p.relative_to(ROOT))] for p in held)
        assert base.protected(allpaths)==baseline['protected']
        after=shutil.disk_usage(ROOT).free
        result=dict(removed=names,held_untouched=before['held'],protected_hashes_unchanged=len(baseline['protected']),free_bytes_before=free,free_bytes_after=after,recovered_bytes=after-free)
        base.save(DEST/'after.json',result);print(json.dumps(result))
    else:raise ValueError('prepare or delete-reviewed-subset required')
if __name__=='__main__':main()
