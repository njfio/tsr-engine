"""Pinned public preparation data for native project callbacks. No dispatch or solution."""
import hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
PACKET=ROOT/'experiments/pursuit-evaluation-v1'
PIN='17989a0cac61f573ac1fca2e7bf1cf8a2cc6ec84db775a4ae72a2ba30cacd13d'
def encoded(v):return (json.dumps(v,sort_keys=True,indent=2)+'\n').encode()
def digest(v):return hashlib.sha256(v).hexdigest()
class Scenario:
 scenario_id='pursuit-v1'
 packet=PACKET
 pin=PIN
 public_fields={'version','provenance','parents','input_contract','instructions','source','export_exclusions'}
 def __init__(self):
  if digest((self.packet/'manifest.json').read_bytes())!=self.pin:raise ValueError('packet manifest drift')
  m=json.loads((self.packet/'manifest.json').read_bytes())
  for name,h in m['files'].items():
   p=self.packet/name
   if p.is_symlink() or digest(p.read_bytes())!=h:raise ValueError('packet bytes drift')
  self.spec=json.loads((self.packet/'project-spec.json').read_bytes());self.public=json.loads((self.packet/'public.json').read_bytes())
  self.SOURCE=self.spec['editable']['src/lib.rs']
  self.REQUIREMENTS={p['id']:p['meaning']+' '+p['operational_contract'] for p in self.public['parents']}
 def public_task_context(self):
  # Explicit public artifact only: never serialize ProjectSpec/protected bodies.
  # Revalidate the frozen bytes on each projection; object mutation is not authority.
  current=type(self)()
  fields=self.public_fields
  if set(self.public)!=fields or self.public!=current.public or self.spec!=current.spec:
   raise ValueError('public task contract drift')
  return {'version':'public-task-context@1',
          'provenance_status':'Historical frozen task provenance, not current run or evaluation status.',
          'packet_manifest_sha256':self.pin,
          'public_file_sha256':digest((self.packet/'public.json').read_bytes()),
          'goal':self.spec['goal'],'task':json.loads(json.dumps(self.public))}
 def artifacts(self):
  # The prepared image name is non-immutable. Pin the already established runtime
  # digest and existing non-root worker identity. This prospective runtime
  # derivation is explicit in the new manifest; frozen packet bytes stay unchanged.
  # Availability/host mapping must be rechecked at prospective launch.
  scope={**self.spec,'image':'sha256:cff6dcceb8b9d203a818b7ff104b89a6aaa353f8576850ca4f58903539417945','user':'501:20'}
  scope.pop('probe_entry',None)
  files={'source/'+p:s.encode() for k in ('editable','protected') for p,s in scope[k].items()}
  scope.update(editable={p:'' for p in scope['editable']},protected={p:'' for p in scope['protected']})
  files.update({'scope.json':encoded(scope),'requirements.json':encoded(self.REQUIREMENTS),'public.json':encoded(self.public)})
  return files
 def manifest(self):return {'schema':'native-pursuit-source@v1','packet':self.pin,'files':{k:digest(v) for k,v in self.artifacts().items()}}
 def prepare(self,directory):
  p=Path(directory);p.mkdir(parents=True,exist_ok=False)
  for name,b in self.artifacts().items():
   q=p/name;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b)
  (p/'manifest.json').write_bytes(encoded(self.manifest()));return self.validate(p)
 def validate(self,directory):
  p=Path(directory);expected=self.artifacts()
  if json.loads((p/'manifest.json').read_bytes())!=self.manifest():raise ValueError('prepared packet drift')
  paths=list(p.rglob('*'))
  if any(x.is_symlink() for x in paths) or {x.relative_to(p).as_posix() for x in paths if x.is_file()}!=set(expected)|{'manifest.json'}:raise ValueError('packet inventory')
  for name,b in expected.items():
   if (p/name).read_bytes()!=b:raise ValueError('packet content')
  return {'scenario_sha256':digest(encoded(self.manifest())),'calls':0,'actions':0,'execution_authorized':False}
