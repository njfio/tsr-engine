"""Freeze a local native integration boundary; deliberately no transport or adoption."""
import importlib.util
import json
from pathlib import Path
s=importlib.util.spec_from_file_location('frozen',Path(__file__).with_name('engine-complementarity-local.py'));p=importlib.util.module_from_spec(s);s.loader.exec_module(p)
def packet():
    p.check();v=p.source()
    return {'version':'complementarity-native-preflight@v1','source_sha256':p.SOURCE_SHA,
      'provenance':v['provenance'],'parents':v['parents'],'records':v['records'],'observations':v['observations'],
      'constraints':v['limits'],'authority':{'external_calls':0,'executions':0,'adoptions':0},
      'task_acceptance':None,'generated_guidance':[],
      'native_mapping':{'parent_identity':'Must be minted by native append-only bootstrap; A/B are source labels, not native IDs.',
        'record_visibility':'Visible fictional assertions, not verified world truth or execution receipts.',
        'basis':'unknown; there is no action return',
        'acceptance':'No predicate measures these rich desires. Do not invent a proxy or code project.',
        'status':'general_native_bootstrap_available_external_export_not_admitted'},
      'future_envelope':{'approval':'pending','model':'gpt-6-luna','max_calls':6,'max_executions':0,'completion_tokens':8192,'request_bytes':32768,'seconds_per_call':240,'prospective_reservation_microusd':300000},
      'campaign_baseline':{'calls':191,'groups':57,'settled_microusd':160913,'max_calls':200,'max_groups':60,'cap_microusd':5000000}}
def bootstrap():
    """Exact fictional content mapped to the general native qualitative schema."""
    p.check();v=p.source()
    attribution=p.canonical(v['provenance']).decode()
    records=[{'label':r['id'],'content':p.canonical({k:r[k] for k in ('topic','assertion','assumption')}).decode(),'attribution':p.canonical({'source':r['attributed_source'],'provenance':r['provenance']}).decode(),'uncertainty':r['uncertainty']} for r in v['records']]
    records += [{'label':o['id'],'content':o['text'],'attribution':attribution,'uncertainty':'Fixture structure only; not an empirical action return.' if o['kind']=='fixture_structure' else 'Unknown, not negative evidence.'} for o in v['observations']]
    result={'parents':[{'label':x['id'],'meaning':x['meaning'],'attribution':attribution} for x in v['parents']], 'records':records,'constraints':v['limits'],'acceptance_references':[]}
    assert all(len(text.encode())<=512 for record in result['parents']+result['records'] for text in record.values())
    return result

def write(directory):
    out=Path(directory);out.mkdir(parents=True,exist_ok=False);v=packet();raw=p.canonical(v)+b'\n';(out/'packet.json').write_bytes(raw)
    (out/'native-bootstrap.json').write_bytes(p.canonical(bootstrap())+b'\n')
    import hashlib
    manifest={'packet_sha256':hashlib.sha256(raw).hexdigest(),'source_sha256':p.SOURCE_SHA,'native_ready':True,'dispatches':0,'adoptions':0,'executions':0}
    (out/'manifest.json').write_bytes(p.canonical(manifest)+b'\n');return manifest
if __name__=='__main__':
    import argparse
    a=argparse.ArgumentParser(description=__doc__);a.add_argument('--directory',required=True);print(json.dumps(write(a.parse_args().directory),indent=2))
