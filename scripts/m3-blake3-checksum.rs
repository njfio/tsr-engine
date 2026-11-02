//! Read-only checksum helper for exported evidence; never executes corpus code.
use std::io::{self, Read};
fn main() -> io::Result<()> {
    let mut hasher = blake3::Hasher::new();
    let mut bytes = [0u8; 65536];
    let mut input = io::stdin().lock();
    loop {
        let count = input.read(&mut bytes)?;
        if count == 0 { break; }
        hasher.update(&bytes[..count]);
    }
    println!("{}", hasher.finalize().to_hex());
    Ok(())
}
