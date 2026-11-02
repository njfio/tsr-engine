//! Controller-owned issuance evidence. No source text, host path, or credentials.
use serde::{Deserialize, Serialize};
use tsr_domain::Hash;

const PREFIX: &str = "broker/inspection-handle-receipt@v1:";
const MAX_RECEIPT: usize = 16 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub run: Hash,
    pub task: String,
    pub call: u32,
    pub path: String,
    pub file_digest: Hash,
    pub source_state: String,
    pub inspection_event: String,
    pub handle: Hash,
}
impl Receipt {
    pub fn expected_handle(&self) -> Hash {
        Hash::of(&[
            b"inspection-handle@v1",
            &self.run.0,
            self.task.as_bytes(),
            &self.call.to_le_bytes(),
            self.path.as_bytes(),
            &self.file_digest.0,
            self.source_state.as_bytes(),
            self.inspection_event.as_bytes(),
        ])
    }
    pub fn seal(mut self) -> Result<(Self, String, Hash), &'static str> {
        self.handle = self.expected_handle();
        let source = format!(
            "{PREFIX}{}",
            serde_json::to_string(&self).map_err(|_| "receipt encoding")?
        );
        if source.len() > MAX_RECEIPT {
            return Err("receipt size");
        }
        let digest = Hash::of(&[source.as_bytes()]);
        Ok((self, source, digest))
    }
    /// Use only after verifying the containing Rules event and its append-only chain.
    /// This verifies evidence integrity; it never grants edit or policy authority.
    pub fn verify(source: &str, digest: Hash) -> Result<Self, &'static str> {
        if source.len() > MAX_RECEIPT || Hash::of(&[source.as_bytes()]) != digest {
            return Err("receipt digest or size");
        }
        let raw = source.strip_prefix(PREFIX).ok_or("receipt version")?;
        let receipt: Self = serde_json::from_str(raw).map_err(|_| "receipt shape")?;
        if receipt.handle != receipt.expected_handle() {
            return Err("receipt binding");
        }
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn receipt() -> Receipt {
        Receipt {
            run: Hash::of(&[b"run"]),
            task: "synthetic-task".into(),
            call: 3,
            path: "src/lib.rs".into(),
            file_digest: Hash::of(&[b"source"]),
            source_state: "workspace-state".into(),
            inspection_event: "event-id".into(),
            handle: Hash::of(&[]),
        }
    }
    #[test]
    fn persisted_roundtrip() {
        let (r, s, d) = receipt().seal().unwrap();
        let restored = Receipt::verify(&s, d).unwrap();
        assert_eq!(restored.handle, r.handle);
        assert_eq!(restored.inspection_event, "event-id");
        assert!(!s.contains("/Users/"));
    }
    #[test]
    fn every_binding_field_changes_handle() {
        let base = receipt();
        let expected = base.expected_handle();
        for field in 0..7 {
            let mut changed = base.clone();
            match field {
                0 => changed.run = Hash::of(&[b"other-arm"]),
                1 => changed.task.push('x'),
                2 => changed.call += 1,
                3 => changed.path.push('x'),
                4 => changed.file_digest = Hash::of(&[b"modified"]),
                5 => changed.source_state.push('x'),
                _ => changed.inspection_event.push('x'),
            }
            assert_ne!(expected, changed.expected_handle());
        }
    }
    #[test]
    fn modified_evidence_or_handle_rejected() {
        let (mut r, s, d) = receipt().seal().unwrap();
        assert!(Receipt::verify(&(s.clone() + " "), d).is_err());
        r.path = "src/other.rs".into();
        let forged = format!("{PREFIX}{}", serde_json::to_string(&r).unwrap());
        assert!(Receipt::verify(&forged, Hash::of(&[forged.as_bytes()])).is_err());
    }
    #[test]
    fn receipt_is_bounded_and_strict() {
        let mut r = receipt();
        r.path = "x".repeat(MAX_RECEIPT);
        assert!(r.seal().is_err());
        let (_, s, _) = receipt().seal().unwrap();
        let extra = s.replacen('{', "{\"unknown\":0,", 1);
        assert!(Receipt::verify(&extra, Hash::of(&[extra.as_bytes()])).is_err());
    }
}
