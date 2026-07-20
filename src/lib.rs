#![forbid(unsafe_code)]

//! Shared library skeleton for Context Continuum.

pub mod claim_contract;
pub mod config_manager;
pub mod doctor;
pub mod model_catalog;
pub mod probe;
pub mod startup_policy;

/// The only model identifier permitted by the project contract.
pub const REQUIRED_MODEL: &str = "gpt-5.6-sol";

/// GPT-5.6 Sol's documented total context window.
pub const OFFICIAL_TOTAL_CONTEXT: u64 = 1_050_000;

/// GPT-5.6 Sol's documented maximum input.
pub const OFFICIAL_MAX_INPUT: u64 = 922_000;

/// GPT-5.6 Sol's documented maximum output.
pub const OFFICIAL_MAX_OUTPUT: u64 = 128_000;

#[cfg(test)]
mod tests {
    use super::{OFFICIAL_MAX_INPUT, OFFICIAL_MAX_OUTPUT, OFFICIAL_TOTAL_CONTEXT, REQUIRED_MODEL};

    #[test]
    fn skeleton_is_sol_only() {
        assert_eq!(REQUIRED_MODEL, "gpt-5.6-sol");
        assert_eq!(
            OFFICIAL_MAX_INPUT + OFFICIAL_MAX_OUTPUT,
            OFFICIAL_TOTAL_CONTEXT
        );
    }
}
