use tsr_domain::{Event, Hash};
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let events: Vec<Event> = serde_json::from_slice(&std::fs::read(&args[1]).unwrap()).unwrap();
    for (i, e) in events.iter().enumerate() {
        assert_eq!(e.schema_version, Event::SCHEMA_VERSION);
        assert_eq!(e.task, events[0].task);
        assert!(e.observed_at.is_none());
        let expected = Event::new(e.task, e.parents.clone(), e.source, e.kind.clone(), e.recorded_at.clone());
        assert_eq!(e.id, expected.id, "canonical content identity");
        assert_eq!(e.parents, if i == 0 {vec![]} else {vec![events[i-1].id]}, "linear parents");
    }
    let ids: Vec<String> = events.iter().map(|e|e.id.to_string()).collect();
    let root = Hash::of(&ids.iter().map(|s|s.as_bytes()).collect::<Vec<_>>());
    assert_eq!(root.to_string(), args[2]);
    println!("{}", serde_json::json!({"canonical_events":events.len(),"root":root.to_string()}));
}
