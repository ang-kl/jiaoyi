//! The frozen pre-registration (owner, 24-10-2026), embedded in the binary.
//! Its SHA-256 is computed at run time from the embedded text; the runner
//! refuses any job registered under a different hash.

use sha2::{Digest, Sha256};

/// The file, relative to the repository root.
pub const PREREG_FILE: &str = "docs/preregistration-2026-10-24.md";

/// The file's content, compiled in.
pub const PREREG_TEXT: &str = include_str!("../../../../docs/preregistration-2026-10-24.md");

/// The hash of the merged file (#7, main 5797cf3d). A change to any rule is
/// a new pre-registration in a new file, never an edit of this one.
pub const PREREG_SHA256: &str = "9105c5eb812ff6b3d267627a53e64d924018c799753d70e598830381a909098f";

/// SHA-256 of the embedded text, hex.
pub fn hash() -> String {
    Sha256::digest(PREREG_TEXT.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_text_is_the_frozen_file() {
        assert_eq!(hash(), PREREG_SHA256);
        // And the file on disk is the one embedded.
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../docs/preregistration-2026-10-24.md"
        );
        let disk = std::fs::read(path).unwrap();
        let disk_hash: String = Sha256::digest(&disk)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        assert_eq!(disk_hash, PREREG_SHA256);
        assert!(PREREG_TEXT.contains("Held-out period:** 2025-10-24 → 2026-10-23"));
    }
}
