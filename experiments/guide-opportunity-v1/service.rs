// Protected entry: actual output only; independent receipt checker assesses routes.
include!("lib.rs");
fn pairs(s:&str)->Vec<(u32,u32)> {
    if s.trim().is_empty(){return vec![];}
    s.trim().split(',').map(|v|{let(a,b)=v.split_once('>').unwrap();(a.parse().unwrap(),b.parse().unwrap())}).collect()
}
fn main(){
    let a:Vec<String>=std::env::args().skip(1).collect();
    if a.is_empty(){return;} // ABI smoke, not an objective outcome.
    assert_eq!(a.len(),2);
    let p:Vec<&str>=a[0].split('|').collect();assert_eq!(p.len(),3);
    let budget:usize=p[0].parse().unwrap();
    let notes:Vec<Note>=p[1].split(',').map(|v|{let(i,t)=v.split_once(':').unwrap();Note{id:i.parse().unwrap(),tag:t.parse().unwrap()}}).collect();
    let queries=pairs(p[2]);let previous=pairs(&a[1]);
    let out=guide(&notes,&previous,&queries,budget);
    println!("{}",out.iter().map(|(a,b)|format!("{}>{}",a,b)).collect::<Vec<_>>().join(","));
    assert_eq!(out,guide(&notes,&previous,&queries,budget),"repeatability");
}
