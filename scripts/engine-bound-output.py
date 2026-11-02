"""Local invocation provenance, not provider-signed understanding or model authority.

Only singleton request metadata is host-owned. Model-selected evidence/lineage
references remain part of the semantic response and are validated unchanged.
"""
import copy,hashlib,json
VERSION='bound-semantic-output@v1'
PROFILE='strict-json-bound@v2'
HOST_REVIEW_FIELDS=frozenset({'review_id','response_digest','binding','receipt'})
def digest(v):return hashlib.sha256(json.dumps(v,sort_keys=True,separators=(',',':'),ensure_ascii=False,allow_nan=False).encode()).hexdigest()
def review_contract(schema):
    metadata={k:copy.deepcopy(v['const']) for k,v in schema['properties'].items() if k in HOST_REVIEW_FIELDS and set(v)=={'const'}}
    semantic=copy.deepcopy(schema)
    semantic['properties']={k:v for k,v in semantic['properties'].items() if k not in metadata}
    semantic['required']=[k for k in semantic['required'] if k not in metadata]
    return semantic,metadata

REFERENCE_PROFILE='review-evidence-aliases@v1'
def alias_review_payload(payload):
    value=copy.deepcopy(payload);ids=value['binding']['evidence_ids']
    if len(ids)!=len(set(ids)) or set(ids)!=set(value['evidence']):raise ValueError('evidence catalog mismatch')
    aliases={'e'+str(i):key for i,key in enumerate(ids)};reverse={v:k for k,v in aliases.items()}
    catalog={'version':REFERENCE_PROFILE,'aliases':aliases,'binding_sha256':digest(value['binding'])}
    value['reference_catalog']=catalog
    value['evidence']={reverse[k]:v for k,v in value['evidence'].items()}
    if 'coverage' in value:
        coverage=value['coverage']
        if not isinstance(coverage,dict) or any(not isinstance(refs,list) or len(refs)!=len(set(refs)) or any(x not in reverse for x in refs) for refs in coverage.values()):raise ValueError('coverage evidence catalog mismatch')
        value['coverage']={key:[reverse[x] for x in refs] for key,refs in coverage.items()}
    for key in ('evidence_ids','observed_receipts','information_receipts'):
        if key in value['binding']:value['binding'][key]=[reverse[x] for x in value['binding'][key]]
    def rewrite(node):
        if isinstance(node,dict):
            if node.get('enum')==ids:node['enum']=list(aliases)
            for child in node.values():rewrite(child)
        elif isinstance(node,list):
            for child in node:rewrite(child)
    rewrite(value['response_schema'])
    return value

def expand_review(content,wire):
    payload=json.loads(wire['messages'][1]['content']);catalog=payload['reference_catalog']
    if catalog.get('version')!=REFERENCE_PROFILE or set(catalog)!={'version','aliases','binding_sha256'}:raise ValueError('reference profile')
    aliases=catalog['aliases']
    if set(aliases)!={'e'+str(i) for i in range(len(aliases))} or payload['binding']['evidence_ids']!=['e'+str(i) for i in range(len(aliases))] or len(set(aliases.values()))!=len(aliases) or set(payload['evidence'])!=set(aliases):raise ValueError('non-bijective reference catalog')
    if 'coverage' in payload and (not isinstance(payload['coverage'],dict) or any(not isinstance(refs,list) or len(refs)!=len(set(refs)) or any(x not in aliases for x in refs) for refs in payload['coverage'].values())):raise ValueError('invalid coverage aliases')
    original=copy.deepcopy(payload['binding'])
    for key in ('evidence_ids','observed_receipts','information_receipts'):
        if key in original:original[key]=[aliases[x] for x in original[key]]
    if digest(original)!=catalog['binding_sha256']:raise ValueError('reference binding mismatch')
    value=copy.deepcopy(content)
    # Interpretation generation has no self-review findings. Review schemas still
    # require them; do not turn a malformed reviewer response into an empty review.
    findings=value['findings'] if 'findings' in payload['response_schema']['properties'] else {}
    for finding in findings.values():
        refs=finding['evidence']
        if not isinstance(refs,list) or len(refs)!=len(set(refs)) or any(x not in aliases for x in refs):raise ValueError('unknown/duplicate reference alias')
        finding['evidence']=[aliases[x] for x in refs]
    return value

def associate(*,admission,kind,index,wire,ident,content,dispatch,settled,source,current_source):
    expected=digest({'admission':digest(admission),'kind':kind,'index':index,'body':digest(wire)})
    if ident!=expected or source!=current_source:raise ValueError('foreign or stale invocation association')
    if dispatch.get('candidate_sha256')!=digest(content) or settled is None:raise ValueError('unsettled or substituted response content')
    if dispatch.get('receipt',{}).get('usage_status')!='reported' or settled.get('stopped') or settled.get('unknown_usage',0) or settled.get('unsettled_requests',0) or settled.get('unknown_compute_requests',0) or settled.get('unknown_external_cost_requests',0):raise ValueError('unknown response association')
    return {'version':VERSION,'request':ident,'kind':kind,'index':index,'request_body_sha256':digest(wire),'admission_sha256':digest(admission),'source':copy.deepcopy(source),'model_content_sha256':digest(content),'receipt_sha256':digest(dispatch['receipt'])}

def attach_review(content,metadata):
    if not isinstance(content,dict):return content
    if set(content)&set(metadata):raise ValueError('model supplied trusted binding metadata; no repair')
    return {**copy.deepcopy(metadata),**copy.deepcopy(content)}

def verify_envelope(record,*,admission,ident,dispatch,settled,source):
    envelope=record['association'];content=record['model_content']
    expected=digest({'admission':digest(admission),'kind':envelope['kind'],'index':envelope['index'],'body':envelope['request_body_sha256']})
    if (envelope.get('version')!=VERSION or ident!=expected or envelope.get('request')!=ident
        or envelope.get('admission_sha256')!=digest(admission) or envelope.get('source')!=source
        or record['source']!=source or envelope.get('model_content_sha256')!=digest(content)
        or dispatch.get('candidate_sha256')!=digest(content)
        or envelope.get('receipt_sha256')!=digest(dispatch.get('receipt'))):
        raise ValueError('bound response provenance mismatch')
    if settled is None or dispatch.get('receipt',{}).get('usage_status')!='reported' or any(settled.get(k,0) for k in ('stopped','unknown_usage','unsettled_requests','unknown_compute_requests','unknown_external_cost_requests')):
        raise ValueError('unknown response association')
    if envelope['kind']=='review' or record.get('host_metadata'):
        native=attach_review(content,record['host_metadata'])
        if 'reference_request' in record:
            wire=record['reference_request']
            if digest(wire)!=envelope['request_body_sha256']:raise ValueError('substituted reference request')
            catalog=json.loads(wire['messages'][1]['content'])['reference_catalog']
            if envelope.get('reference_catalog_sha256')!=digest(catalog):raise ValueError('substituted reference map')
            native=expand_review(native,wire)
        elif 'reference_catalog_sha256' in envelope:raise ValueError('missing reference request')
        if digest(native)!=record['native_content_sha256']:raise ValueError('mixed review metadata')
    return record


def prepare_review_body(body, *, aliases=False):
    """One canonical host-owned-binding/alias contract for dispatch and audit."""
    original=copy.deepcopy(body);payload=json.loads(original['messages'][1]['content'])
    semantic_schema,metadata=review_contract(payload['response_schema']);payload['response_schema']=semantic_schema
    if aliases:
        payload=alias_review_payload(payload)
        original['messages'][0]['content']+=' Evidence citations must use only the exact short aliases in response_schema and reference_catalog, never reproduce long hashes. Aliases identify the same request-bound evidence; no new facts are implied.'
    payload['output_envelope']={'version':VERSION,'host_owned_fields':list(metadata),'instruction':'Return only the semantic fields in response_schema. Do not return host-owned binding fields. Evidence references remain exact model-selected citations.'}
    original['messages'][0]['content']+=' The versioned response_schema overrides any earlier example of response fields: host-owned binding fields must be omitted, never copied.'
    original['messages'][1]['content']=json.dumps(payload,sort_keys=True,separators=(',',':'),ensure_ascii=False,allow_nan=False)
    return original,metadata
