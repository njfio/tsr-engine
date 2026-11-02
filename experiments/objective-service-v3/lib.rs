#[derive(Clone, Debug)]
pub struct Note { pub id: u32, pub tag: u8 }
pub fn shortlist(notes: &[Note], _previous: &[u32], capacity: usize) -> Vec<u32> {
    notes.iter().take(capacity).map(|n| n.id).collect()
}
