#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note { pub id: u32, pub perspective: u8, pub available: bool }
// Return a bounded repeatable starting set from currently available notes.
pub fn shortlist(notes: &[Note], _previous: &[u32], limit: usize) -> Vec<u32> {
    notes.iter().filter(|n| n.available).take(limit).map(|n| n.id).collect()
}
