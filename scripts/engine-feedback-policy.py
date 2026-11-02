"""Prospectively admitted feedback continuation. No dispatch or authority creation."""
LEGACY='authorized-feedback-cycle@v1'
PURSUIT='authorized-feedback-cycle@v2'
def validate(policy,decisions):
 if not isinstance(policy,dict) or set(policy)!={'version','authorization_id','max_continuations'}:raise ValueError('finite attributed feedback-cycle authority required')
 if policy['version'] not in (LEGACY,PURSUIT) or not isinstance(policy['authorization_id'],str) or not 0<len(policy['authorization_id'])<=200:raise ValueError('feedback-cycle authority identity')
 ceiling=4 if policy['version']==LEGACY else decisions
 if not (policy['version']==PURSUIT and decisions is None and policy['max_continuations'] is None) and (type(policy['max_continuations']) is not int or ceiling is None or not 1<=policy['max_continuations']<=ceiling):raise ValueError('feedback continuation exceeds decision allowance')
def ongoing(policy):return bool(policy and policy.get('version')==PURSUIT)

# None is an explicit absent cumulative ceiling, never an absent authority.
def unlimited(admission):
 value=admission.get('unlimited_policy_sha256')
 if value is None:return False
 if not isinstance(value,str) or len(value)!=64 or any(c not in '0123456789abcdef' for c in value):raise ValueError('unlimited policy identity required')
 return True
def reached(value,ceiling):return ceiling is not None and value>=ceiling
def deadline(admission,stamp):
 if unlimited(admission):
  if admission.get('expires_at') is not None:raise ValueError('unlimited policy cannot hide a session expiry')
  return float('inf')
 return stamp(admission['expires_at'])
