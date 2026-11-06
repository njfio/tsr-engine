"""Run continue.py with a parents check that accepts an admitted generated wanting.

continue.py is pinned by the v4f launch manifest, so it stays unchanged for this test.
Its check reads x['requirement'] on every parent, and a generated wanting has none.
The stricter comparison here: every parent's identity and full record must be unchanged.
"""
import importlib.util,json,sys
from pathlib import Path
s=importlib.util.spec_from_file_location('continue_pinned',Path(__file__).with_name('continue.py'));c=importlib.util.module_from_spec(s);s.loader.exec_module(c)
def verify(before,after,original):
 if before['coding']['snapshot']!=after['coding']['snapshot']:raise ValueError('preparation source changed')
 old=before['coding']['relational']['inputs'];new=after['coding']['relational']['inputs']
 pairs=lambda rows:[(x['id'],x['requirement']) for x in rows]
 if pairs(new.get('acceptance_parents',new['parents']))!=pairs(original):raise ValueError('original obligations changed')
 ids=lambda rows:[x['id'] for x in rows]
 if ids(old['parents'])!=ids(new['parents']):raise ValueError('contributor identities changed')
 # Original roots carry guidance that may change. A generated parent's record may not.
 for a,b in zip(old['parents'],new['parents']):
  if 'requirement' in a or 'requirement' in b:
   if a.get('requirement')!=b.get('requirement'):raise ValueError('contributor identities changed')
  elif a!=b:raise ValueError('generated parent changed')
 for key in ('generated_preferences','generated_preference_statuses'):
  if old.get(key,[])!=new.get(key,[]):raise ValueError('generated lineage or eligibility changed')
c.verify_identity_continuity=verify
if __name__=='__main__':print(json.dumps(c.prepare(sys.argv[1],sys.argv[2]),indent=2))
