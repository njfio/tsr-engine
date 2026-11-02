"""Independent freshness fold over canonically verified controller evidence.
No candidate execution, provider calls, or controller invalidation flags are used.
"""
import json
from pathlib import Path
class Freshness:
    def __init__(self):self.latest=None;self.state=None
    def issue(self,r):
        if self.state is not None:assert self.state==r['source_state'],'issuance state discontinuity'
        self.state=r['source_state'];self.latest=r
    def valid(self,proposal):
        r=self.latest
        return bool(r and proposal['inspection_handle']==r['handle'] and proposal['path']==r['path'] and r['source_state']==self.state)
    def transition(self,action,before,after):
        assert self.state in (None,before),'source checkpoint discontinuity'
        if action in ('edit','replace') or (action=='test' and before!=after):self.latest=None
        if action=='inspect':assert before==after,'inspection changed source'
        self.state=after

def replay(directory, terminal=None):
    d=Path(directory);events=json.loads((d/'events.json').read_text())
    rows=[json.loads(l) for l in (d/'boundary.jsonl').read_text().splitlines()] if (d/'boundary.jsonl').exists() else []
    assert [r['call'] for r in rows]==list(range(len(rows)))
    fold=Freshness();pending=None;cursor=0;checkpoints=0;edits=0
    for e in events:
        k=e['kind'];evidence=k.get('observation',{}).get('evidence',{});source=evidence.get('source','')
        if source.startswith('broker/inspection-handle-receipt@v1:'):
            fold.issue(json.loads(source.split(':',1)[1]))
        elif k['kind']=='proposed':
            assert pending is None
            r=rows[cursor];cursor+=1;proposal=r['proposal']
            assert fold.state in (None,r['source_state']),'request state differs from replay'
            fold.state=r['source_state']
            advertised=None if fold.latest is None else {'handle':fold.latest['handle'],'path':fold.latest['path']}
            assert r['inspection']==advertised,'request advertised stale/superseded handle'
            if proposal['action']=='edit':assert fold.valid(proposal),'accepted stale/foreign/path handle';edits+=1
            if proposal['action']!='stop':pending=(r,k['candidate']['id'])
        elif source.startswith('broker/source-checkpoint@v1:'):
            c=json.loads(source.split(':',1)[1]);assert pending is not None
            r,candidate=pending;assert (c['candidate'],c['call'],c['action'],c['before'])==(candidate,r['call'],r['proposal']['action'],r['source_state'])
            fold.transition(c['action'],c['before'],c['after']);pending=None;checkpoints+=1
    assert pending is None,'missing post-action checkpoint'
    refused=0
    for r in rows[cursor:]:
        assert r['source_state']==fold.state
        assert len(rows[cursor:])==1,'multiple non-dispatched proposals'
        action=r['proposal']['action']
        invalid=action=='edit' and not fold.valid(r['proposal'])
        ordinary=terminal and (terminal.get('failure') in ('edit-precondition','stale-digest-or-oversize') or any(reason.startswith('stop:budget:') for reason in terminal.get('closure_reasons',[])))
        assert invalid or ordinary,'unexplained non-dispatched proposal'
        if action=='edit' and not invalid:assert fold.valid(r['proposal'])
        refused+=int(invalid)
    return dict(source_checkpoints=checkpoints,fresh_edits=edits,independently_invalid_handles=refused)
if __name__=='__main__':
    import sys
    print(json.dumps(replay(sys.argv[1])))
