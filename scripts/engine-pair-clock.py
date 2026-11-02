"""Equal active windows inside a shared outer expiry; no effects or clock resets."""
import time
def validate(seconds):
 if seconds is not None and (type(seconds) is not int or not 300<=seconds<=7200):raise ValueError('explicit active session window')
def start(admission,op,now,stamp):
 seconds=admission.get('active_window_seconds')
 if seconds is None:return None
 def capture():
  if now+seconds>stamp(admission['expires_at']):raise ValueError('full equal active window no longer fits outer deadline')
  return {'started_at':now,'deadline':now+seconds,'active_window_seconds':seconds}
 return op('session_clock',capture)
def deadline(admission,done,stamp):
 outer=float('inf') if admission.get('unlimited_policy_sha256') and admission['expires_at'] is None else stamp(admission['expires_at']);seconds=admission.get('active_window_seconds')
 if seconds is None:return outer
 clock=done.get('session_clock')
 if not clock or clock['active_window_seconds']!=seconds or clock['deadline']!=clock['started_at']+seconds:raise ValueError('missing or inconsistent durable active clock')
 return min(outer,clock['deadline'])
def require(admission,done,stamp,headroom,*,now=None):
 if (time.time() if now is None else now)+headroom>deadline(admission,done,stamp):raise ValueError('insufficient active or outer session deadline headroom')

def wire(admission,done,stamp,body,timeout,send,*,now=None):
 if admission.get('active_window_seconds') is None:return send(body,timeout)
 remaining=deadline(admission,done,stamp)-(time.time() if now is None else now)
 if remaining<=0:
  return {'value':None,'receipt':{'status':'settled','usd':'0.000000','usage_status':'local_not_dispatched','usage':{'prompt_tokens':0,'completion_tokens':0,'total_tokens':0},'elapsed_seconds':0,'response_request_ids':[],'local_failure':'active_or_outer_deadline_expired_before_network'}}
 return send(body,min(timeout,remaining))
