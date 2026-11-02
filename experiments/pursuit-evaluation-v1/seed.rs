#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note { pub id: u32, pub perspective: u8 }
// Return a bounded, repeatable shortlist of existing note IDs.
pub fn shortlist(notes: &[Note], limit: usize) -> Vec<u32> {
    notes.iter().take(limit).map(|n| n.id).collect()
}
