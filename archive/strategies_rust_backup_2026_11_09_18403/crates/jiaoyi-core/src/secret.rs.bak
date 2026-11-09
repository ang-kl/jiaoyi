//! A string that must never reach a log line: OAuth tokens and client secrets.

use std::fmt;

/// Holds a secret value. `Debug` prints `[redacted]`; there is no `Display`.
/// The value is only reachable through [`Secret::expose`], which makes every
/// read of it searchable in the source.
#[derive(Clone, PartialEq, Eq)]
pub struct Secret(String);

impl Secret {
    pub fn new(value: impl Into<String>) -> Self {
        Secret(value.into())
    }

    pub fn expose(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The first 16 hex characters of the SHA-256 of the value: enough to
    /// tell two tokens apart in a record or a log line, never enough to use.
    pub fn fingerprint(&self) -> String {
        use sha2::{Digest, Sha256};
        Sha256::digest(self.0.as_bytes())
            .iter()
            .take(8)
            .map(|b| format!("{b:02x}"))
            .collect()
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret([redacted])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_output_never_contains_the_value() {
        let s = Secret::new("very-secret-token-123");
        let shown = format!("{s:?} {:?}", Some(&s));
        assert!(!shown.contains("very-secret-token-123"), "{shown}");
        assert!(shown.contains("[redacted]"));
        assert_eq!(s.expose(), "very-secret-token-123");
    }

    #[test]
    fn the_fingerprint_is_the_head_of_the_sha256() {
        // FIPS 180-2, appendix B.1: SHA-256("abc") = ba7816bf8f01cfea41...
        assert_eq!(Secret::new("abc").fingerprint(), "ba7816bf8f01cfea");
        let f = Secret::new("very-secret-token-123").fingerprint();
        assert_eq!(f.len(), 16);
        assert!(!f.contains("secret"));
    }
}
