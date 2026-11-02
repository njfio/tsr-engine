"""Conservative M3 exclusivity with one exact, hash-pinned read-only observer."""
from pathlib import Path
import fcntl,hashlib,json,os,re,shlex,subprocess
ROOT=Path(__file__).resolve().parents[2]
TRUSTED_PYTHONS={
'/opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/bin/python3.14':'02a8df1726463faedb3adef289b3d1bdeabb7785c837fd8c174023aeeeb4d312',
'/opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python':'5ce0f1ab504ae7c1caefa42350a75f6e4e89dc9941723e67bb450713ef27fc9c'}
HISTORICAL_PID=29246
HISTORICAL_COMMAND='/opt/homebrew/Cellar/python@3.14/3.14.4_1/Frameworks/Python.framework/Versions/3.14/Resources/Python.app/Contents/MacOS/Python -B '+str(ROOT/'scripts/m3-following-native-expansion.py')
def role(command,observer,observer_hash):
    try:parts=shlex.split(command)
    except ValueError:return 'unparseable-m3' if 'm3-' in command else None
    dangerous=any(x in command for x in ('--approved-live','--approved-paired-handle-validation','--approved-continuation','m3-qualification-engine')) or bool(re.search(r'\b(?:task run|corpus (?:qualify|mutants))\b',command))
    if dangerous:return 'mutation-or-provider'
    # Only an exact interpreter + script invocation may be exempted. No -c/-m,
    # shell wrappers, extra arguments, alternative scripts or changed observer bytes.
    mentioned=str(observer) in command or any(observer.name in token for token in parts)
    interpreter=str(Path(parts[0]).resolve()) if parts else ''
    trusted=interpreter in TRUSTED_PYTHONS and hashlib.sha256(Path(interpreter).read_bytes()).hexdigest()==TRUSTED_PYTHONS[interpreter]
    exact=len(parts)==2 and trusted and Path(parts[1]).resolve()==observer.resolve()
    if exact:
        if observer.is_symlink() or hashlib.sha256(observer.read_bytes()).hexdigest()!=observer_hash:return 'changed-observer'
        return 'read-only-observer'
    if mentioned:return 'nonexact-observer'
    if re.search(r'/m3-[^ /]+\.py(?: |$)',command) or re.search(r'(?:^|[ /])experiments/m3-[^ ]+\.py(?: |$)',command):return 'm3-process'
    return None

def require_idle(observer,observer_hash,audit_path=None):
    rows=subprocess.check_output(['ps','-axo','pid=,stat=,command='],text=True,timeout=15).splitlines()
    seen=False;observed=[]
    for row in rows:
        p=row.strip().split(None,2)
        if len(p)!=3:continue
        pid,status,command=p;pid=int(pid)
        if pid==os.getpid():continue
        if pid==HISTORICAL_PID:
            seen=True
            start=subprocess.check_output(['ps','-p',str(pid),'-o','lstart='],text=True,timeout=15).strip()
            if 'T' not in status or command!=HISTORICAL_COMMAND or start!='Wed Oct 14 21:19:49 2026':raise ValueError('historical coordinator identity/status mismatch')
            continue
        category=role(command,observer,observer_hash)
        if category:
            observed.append(dict(pid=pid,status=status,role=category,command_sha256=hashlib.sha256(command.encode()).hexdigest()))
            if category!='read-only-observer':
                if audit_path:append(audit_path,dict(status='blocked',processes=observed))
                raise ValueError(f'competing M3 process pid={pid} role={category}')
    if not seen:raise ValueError('historical coordinator missing')
    names=subprocess.check_output(['docker','ps','--format','{{.Names}}'],text=True,timeout=15).splitlines()
    if any(n.startswith('tsr-m3-') for n in names):raise ValueError('M3 container remains')
    if audit_path:append(audit_path,dict(status='idle',processes=observed))

def append(path,value):
    with path.open('a') as f:f.write(json.dumps(value)+'\n');f.flush();os.fsync(f.fileno())

def lock():
    path=ROOT/'target/m3-active-controller.lock';fd=os.open(path,os.O_RDWR|os.O_CREAT|os.O_NOFOLLOW,0o600)
    try:fcntl.flock(fd,fcntl.LOCK_EX|fcntl.LOCK_NB)
    except BaseException:os.close(fd);raise
    os.ftruncate(fd,0);os.write(fd,str(os.getpid()).encode());os.fsync(fd)
    return fd
