use pursuit::{Note,shortlist};
use std::collections::BTreeSet;
fn valid(n:&[Note], k:usize, out:&[u32]) {
 assert_eq!(out,shortlist(n,k));
 assert_eq!(out.len(),k.min(n.len()));
 assert_eq!(out.iter().collect::<BTreeSet<_>>().len(),out.len());
 assert!(out.iter().all(|id|n.iter().any(|x|x.id==*id)));
}
fn notes(groups:&[u8])->Vec<Note>{groups.iter().enumerate().map(|(i,g)|Note{id:i as u32+10,perspective:*g}).collect()}
#[test] fn bounded_repeatable_existing_ids() {
 for groups in [vec![],vec![1],vec![1,1,2,3],vec![3,2,3,1,2]] {
  let n=notes(&groups);
  for k in [0,1,2,3,8,usize::MAX] {
   let out=shortlist(&n,k);
   valid(&n,k,&out);
  }
 }
}
#[test] fn perspective_coverage_when_capacity_allows() {
 for groups in [vec![],vec![1],vec![1,1,2,3],vec![3,2,3,1,2],vec![0,0,0,4,4,9]] {
  let n=notes(&groups);let distinct=groups.iter().copied().collect::<BTreeSet<_>>();
  for k in 0..=n.len()+1 {
   let out=shortlist(&n,k);
   valid(&n,k,&out);
   let seen=out.iter().filter_map(|id|n.iter().find(|x|x.id==*id).map(|x|x.perspective)).collect::<BTreeSet<_>>();
   assert_eq!(seen.len(),k.min(distinct.len()));
  }
 }
}
