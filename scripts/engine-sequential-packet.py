"""Frozen public-only packet; no model calls, corpus reads, target execution or authority."""
import importlib.util,json
from pathlib import Path
spec=importlib.util.spec_from_file_location('pursuit_packet_base',Path(__file__).with_name('engine-pursuit-packet.py'))
base=importlib.util.module_from_spec(spec);spec.loader.exec_module(base)
class Scenario(base.Scenario):
 scenario_id='sequential-pursuit-v1'
 packet=base.ROOT/'experiments/sequential-pursuit-v1'
 pin='689f5db8fd2632d852d7458e88d3cd65d647d16f9c3e30f943c721a95c6c86a0'
 public_fields=base.Scenario.public_fields|{'record_disclosure'}
 def information_manifest(self,root=None):
  # Constructor and projection revalidate exact packet bytes. No arbitrary public labeling.
  current=type(self)()
  records=json.loads((current.packet/'information-records.json').read_bytes())['records']
  return {'root':str(Path(root).resolve() if root else current.packet.resolve()),'records':records}
 def artifacts(self):
  files=super().artifacts()
  for r in json.loads((self.packet/'information-records.json').read_bytes())['records']:
   files['information/'+r['path']]=(self.packet/r['path']).read_bytes()
  return files
