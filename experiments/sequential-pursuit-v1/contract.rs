use sequential_pursuit::{shortlist, Note};
use std::collections::BTreeSet;
fn evaluate(notes: &[Note], previous: &[u32], limit: usize) {
    let available: Vec<_> = notes.iter().filter(|n| n.available).collect();
    let k = limit.min(available.len());
    let tags: BTreeSet<_> = available.iter().map(|n| n.perspective).collect();
    let coverage = k.min(tags.len());
    let prior: BTreeSet<_> = previous.iter().copied().collect();
    let mut optimal = 0;
    // Independent exhaustive finite-set oracle; no preferred output or algorithm.
    for mask in 0u64..(1u64 << available.len()) {
        if mask.count_ones() as usize != k { continue; }
        let chosen: Vec<_> = available.iter().enumerate().filter(|(i,_)|mask & (1<<i)!=0).map(|(_,n)|*n).collect();
        if chosen.iter().map(|n|n.perspective).collect::<BTreeSet<_>>().len()!=coverage {continue;}
        optimal=optimal.max(chosen.iter().filter(|n|prior.contains(&n.id)).count());
    }
    let out=shortlist(notes,previous,limit);
    assert_eq!(out,shortlist(notes,previous,limit),"repeatability");
    assert_eq!(out.len(),k,"bounded cardinality");
    assert_eq!(out.iter().collect::<BTreeSet<_>>().len(),k,"unique IDs");
    assert!(out.iter().all(|id|available.iter().any(|n|n.id==*id)),"available existing IDs");
    assert_eq!(out.iter().map(|id|available.iter().find(|n|n.id==*id).unwrap().perspective).collect::<BTreeSet<_>>().len(),coverage,"perspective coverage");
    assert_eq!(out.iter().filter(|id|prior.contains(id)).count(),optimal,"continuity subject to coverage");
}
#[test]
fn bounded_available_coverage_and_continuity() {
    for groups in [vec![],vec![1],vec![1,1,2],vec![1,1,2,3,3,4],vec![0,2,0,2,0]] {
        for availability in 0..(1u64<<groups.len()) {
            let notes:Vec<_>=groups.iter().enumerate().map(|(i,g)|Note{id:10+i as u32,perspective:*g,available:availability&(1<<i)!=0}).collect();
            for previous in [vec![],vec![10,10,999],vec![11,13,15],vec![15,14,13,12,11,10]] {
                for limit in [0,1,2,3,6,usize::MAX] {evaluate(&notes,&previous,limit);}
            }
        }
    }
}
#[test]
fn unchanged_contract_across_available_conditions() {
    let first=vec![Note{id:11,perspective:1,available:true},Note{id:12,perspective:1,available:true},Note{id:21,perspective:2,available:true},Note{id:31,perspective:3,available:true},Note{id:32,perspective:3,available:true},Note{id:41,perspective:4,available:false}];
    let mut second=first.clone();second[0].available=false;second[5].available=true;
    for previous in [vec![],vec![11,21,31],vec![12,21,32],vec![41,41,999]] {
        for limit in [0,1,2,3,4,usize::MAX] {evaluate(&first,&previous,limit);evaluate(&second,&previous,limit);}
    }
}
