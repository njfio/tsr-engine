"""Documented best-effort JSON Schema guidance for the two repair payload stages.

Not an admission, parser, or semantic validator. Native checks remain authoritative,
including UTF-8 byte bounds. strict=False preserves dynamic harness path selection
and optional fields that OpenAI strict structured output cannot directly express.
"""

def closed(properties,required=None):
    return {'type':'object','additionalProperties':False,'properties':properties,
            'required':list(properties) if required is None else required}

def response_format(stage, requirements):
    if stage not in ('experiment','boundary_patch'):return None
    if (not isinstance(requirements,(list,tuple)) or not 1<=len(requirements)<=8
            or any(not isinstance(x,str) or not x or len(x.encode())>256 for x in requirements)
            or len(set(requirements))!=len(requirements) or 'existing_behaviour' not in requirements):
        raise ValueError('published repair requirement IDs required')
    short={'type':'string','minLength':1,'maxLength':512,
           'description':'Nonblank, at most512 UTF-8 bytes, not512 Unicode characters. Be concise; the local byte validator remains authoritative.'}
    if stage=='experiment':
        case=closed({'id':{'type':'string','minLength':1,'maxLength':256},
                     'args':{'type':'array','maxItems':0,'items':{'type':'string'}},
                     'expected_stdout':{'type':'string','enum':['']}})
        mapping=closed({'cases':{'type':'array','minItems':1,'maxItems':2,'items':{'type':'string'}},
                        'rationale':short,'limits':short})
        experiment=closed({'harness':{'type':'object','minProperties':1,'maxProperties':2,
            'propertyNames':{'pattern':r'^tests/tsr_supplemental_[A-Za-z0-9_]+\.rs$'},
            'additionalProperties':{'type':'string','minLength':1,'maxLength':4096},
            'description':'Choose new test paths and full Rust test source;4096 UTF-8 bytes total. Do not modify existing files.'},
            'cases':{'type':'array','minItems':1,'maxItems':2,'items':case},
            'mapping':closed({key:mapping for key in requirements},['existing_behaviour'])})
        schema=closed({'experiment':experiment})
    else:
        edit=closed({'path':{'type':'string','enum':['src/lib.rs']},
                     'replacement':{'type':'string','minLength':1,'maxLength':2048,
                                    'description':'Full proposed source, at most2048 UTF-8 bytes. No reference repair is supplied.'}})
        schema=closed({'edits':{'type':'array','minItems':1,'maxItems':1,'items':edit},
                       'rationale':short,'common_event_probability':{'type':'number','minimum':0,'maximum':1}},['edits','rationale'])
    return {'type':'json_schema','json_schema':{'name':'tsr_'+stage,'strict':False,'schema':schema}}
