#[derive(Clone,Debug)]
pub struct Note { pub id:u32, pub tag:u8 }
// Neutral executable starting organization, not a deliberately broken seed.
pub fn guide(notes:&[Note], _previous:&[(u32,u32)], _queries:&[(u32,u32)], budget:usize)->Vec<(u32,u32)> {
    if notes.len()<2 { return vec![]; }
    (0..notes.len()).take(budget).map(|i|(notes[i].id,notes[(i+1)%notes.len()].id)).collect()
}
