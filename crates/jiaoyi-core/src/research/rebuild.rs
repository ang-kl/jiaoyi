//! Versioned research authority. It never gates live quotes or position protection.
//! Reopening collection requires a reviewed RFC transition, not an environment flag.
pub const RFC_STATE: &str = "rebuild-offline-18368-v1";
pub const COLLECTION_REFUSAL: &str = "research_collection_paused_rfc_18368";

pub fn collection_authority() -> Result<(), &'static str> {
    Err(COLLECTION_REFUSAL)
}

#[cfg(test)]
mod tests {
    #[test]
    fn offline_state_has_no_collection_authority() {
        assert_eq!(
            super::collection_authority(),
            Err(super::COLLECTION_REFUSAL)
        );
    }
}
