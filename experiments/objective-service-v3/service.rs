// Immutable independent fixture entry. No winning output or implementation.
include!("lib.rs");
use std::collections::BTreeSet;
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() { return; } // ABI smoke only, never a service outcome.
    assert_eq!(args.len(), 2, "exact condition and prior-output arguments");
    let (cap, body) = args[0].split_once('|').expect("condition delimiter");
    let cap: usize = cap.parse().expect("capacity");
    let notes: Vec<Note> = if body.is_empty() { vec![] } else {
        body.split(',').map(|s| {
            let (id, tag) = s.split_once(':').expect("record delimiter");
            Note { id: id.parse().expect("id"), tag: tag.parse().expect("tag") }
        }).collect()
    };
    let previous: Vec<u32> = if args[1].trim().is_empty() { vec![] } else {
        args[1].trim().split(',').map(|s| s.parse().expect("prior actual output")).collect()
    };
    assert!(notes.len() < 20, "bounded independent enumeration");
    assert_eq!(notes.iter().map(|n|n.id).collect::<BTreeSet<_>>().len(), notes.len());
    let output = shortlist(&notes, &previous, cap);
    // Record the actual candidate before reporting finite oracle failure.
    println!("{}", output.iter().map(u32::to_string).collect::<Vec<_>>().join(","));
    assert_eq!(output, shortlist(&notes, &previous, cap), "repeatability");
    let k = cap.min(notes.len());
    let coverage = k.min(notes.iter().map(|n|n.tag).collect::<BTreeSet<_>>().len());
    let prior: BTreeSet<_> = previous.iter().copied().collect();
    let mut optimum = 0;
    for mask in 0u64..(1u64<<notes.len()) {
        if mask.count_ones() as usize != k { continue; }
        let subset: Vec<_> = notes.iter().enumerate().filter(|(i,_)|mask&(1<<i)!=0).map(|(_,n)|n).collect();
        if subset.iter().map(|n|n.tag).collect::<BTreeSet<_>>().len() != coverage { continue; }
        optimum = optimum.max(subset.iter().filter(|n|prior.contains(&n.id)).count());
    }
    assert_eq!(output.len(), k, "cardinality");
    assert_eq!(output.iter().collect::<BTreeSet<_>>().len(), k, "unique IDs");
    assert!(output.iter().all(|id|notes.iter().any(|n|n.id==*id)), "available IDs");
    assert_eq!(output.iter().map(|id|notes.iter().find(|n|n.id==*id).unwrap().tag).collect::<BTreeSet<_>>().len(), coverage, "coverage");
    assert_eq!(output.iter().filter(|id|prior.contains(id)).count(), optimum, "retention subject to coverage");
}
