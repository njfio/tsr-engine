"""Opt-in local quarantine, never model context, logging, admission or accounting.

Exact model-content UTF-8 bytes can be retained even when JSON parsing fails.
Wire bytes are retained only with a parseable, allowlisted envelope containing no
reasoning/private fields. Malformed wrappers are hashed, not retained: privacy
cannot safely be separated there. No historical response is reconstructed.
"""
import hashlib,json,os,re
from pathlib import Path
LIMIT=65536
SECRET=re.compile(rb'(?:ci_live_|sk-)[A-Za-z0-9_-]{8,}|Bearer\s+\S+|-----BEGIN [A-Z ]*PRIVATE KEY-----',re.I)
COLON=re.compile(rb'\s*:')
SENSITIVE=re.compile(rb'"(?:authorization|api[_-]?key|access[_-]?token|password|secret|reasoning(?:_content)?|thinking)"\s*:',re.I)
def private_bytes(data,secret):
 if data is None:return False
 secret_bytes=secret.encode() if isinstance(secret,str) and secret else None
 def blocked(value):return bool((secret_bytes and secret_bytes in value) or SECRET.search(value) or SENSITIVE.search(value))
 if blocked(data):return True
 # Decode individual JSON string tokens even when the enclosing JSON is malformed.
 # This prevents escaped credentials/field names from bypassing withholding.
 for token in re.finditer(rb'"(?:[^"\\]|\\.)*"',data):
  try:
   value=json.loads(token.group()).encode('utf-8')
   if blocked(value):return True
   if COLON.match(data,token.end()) and SENSITIVE.search(b'"'+value+b'":'):return True
  except (ValueError,UnicodeError):continue
 return False
class FailureEvidence:
 def __init__(self,parent,name,limit=LIMIT):
  if type(limit) is not int or not LIMIT<=limit<=1048576:raise ValueError('retention allocation')
  self.limit=limit
  parent=Path(parent)
  if parent.is_symlink() or not parent.is_dir() or not re.fullmatch(r'failed-response-[1-8]',name):raise ValueError('local evidence destination')
  self.parent=parent;self.name=name;self.parts={};self.saved=None
  self.parent_fd=os.open(parent,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW)
 def capture(self,raw,content,secret,wrapper_safe):
  """Keep only bounded eligible bytes in memory; discard known secrets first."""
  for name,data,eligible in [('wire',raw,wrapper_safe),('content',content,True)]:
   if data is None:continue
   blocked=not eligible or private_bytes(data,secret) or (name=='wire' and private_bytes(content,secret))
   self.parts[name]={'sha256':hashlib.sha256(data).hexdigest(),'observed_bytes':len(data),
     'withheld':bool(blocked),'truncated':len(data)>self.limit,'data':None if blocked else data[:self.limit]}
 def retain(self,stage):
  if self.saved is not None:return self.saved
  # Fixed local names, no symlink following or overwrite. Failure never changes billing.
  parent=os.dup(self.parent_fd)
  directory=None
  try:
   os.mkdir(self.name,0o700,dir_fd=parent)
   directory=os.open(self.name,os.O_RDONLY|os.O_DIRECTORY|os.O_NOFOLLOW,dir_fd=parent)
   meta={'version':'failed-response-evidence@v1','stage':stage,'local_only':True,'not_authoritative':True,'parts':{}}
   for name,part in self.parts.items():
    data=part['data'];info={k:v for k,v in part.items() if k!='data'}
    if data is not None:
     filename=name+'.bin';self.write(directory,filename,data);info.update(file=filename,retained_bytes=len(data))
    meta['parts'][name]=info
   self.write(directory,'metadata.json',json.dumps(meta,sort_keys=True).encode())
   self.saved={'directory':self.name,'metadata_sha256':hashlib.sha256(json.dumps(meta,sort_keys=True).encode()).hexdigest(),'parts':meta['parts']}
   return self.saved
  finally:
   if directory is not None:os.close(directory)
   os.close(parent)
 def clear(self):
  self.parts={}
  if self.parent_fd is not None:os.close(self.parent_fd);self.parent_fd=None
 def __del__(self):
  fd=getattr(self,'parent_fd',None)
  if fd is not None:os.close(fd);self.parent_fd=None
 @staticmethod
 def write(directory,name,data):
  fd=os.open(name,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600,dir_fd=directory)
  with os.fdopen(fd,'wb') as f:f.write(data);f.flush();os.fsync(f.fileno())

def safe_wire(parsed):
 if not isinstance(parsed,dict) or set(parsed)-{'id','object','created','model','request_id','choices','usage','billing','system_fingerprint','service_tier'}:return False
 for name,allowed in [('usage',{'prompt_tokens','completion_tokens','total_tokens','prompt_tokens_details','completion_tokens_details'}),('billing',{'status','currency','billed_cost_usd'})]:
  item=parsed.get(name,{})
  if not isinstance(item,dict) or set(item)-allowed:return False
 for name in ('id','object','model','request_id','system_fingerprint','service_tier'):
  if name in parsed and parsed[name] is not None and not isinstance(parsed[name],str):return False
 if 'created' in parsed and type(parsed['created']) is not int:return False
 billing=parsed.get('billing',{})
 if any(type(v) not in (str,int,float) for v in billing.values()):return False
 usage=parsed.get('usage',{})
 if any(k not in ('prompt_tokens_details','completion_tokens_details') and type(v) is not int for k,v in usage.items()):return False
 for name,allowed in [('prompt_tokens_details',{'cached_tokens','cache_write_tokens'}),('completion_tokens_details',{'reasoning_tokens','accepted_prediction_tokens','rejected_prediction_tokens'})]:
  detail=usage.get(name,{})
  if not isinstance(detail,dict) or set(detail)-allowed or any(type(v) is not int for v in detail.values()):return False
 choices=parsed.get('choices')
 if not isinstance(choices,list):return False
 return all(isinstance(c,dict) and not set(c)-{'index','finish_reason','message'} and
  isinstance(c.get('message'),dict) and not set(c['message'])-{'role','content','refusal'} and
  all(v is None or isinstance(v,str) for v in c['message'].values()) and
  ('index' not in c or type(c['index']) is int) and ('finish_reason' not in c or isinstance(c['finish_reason'],str)) for c in choices)
