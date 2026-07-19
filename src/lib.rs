#![forbid(unsafe_code)]

//! Shared library skeleton for Context Continuum.

/// The only model identifier permitted by the project contract.
pub const REQUIRED_MODEL: &str = "gpt-5.6-sol";

#[cfg(test)]
mod tests {
    use super::REQUIRED_MODEL;

    #[test]
    fn skeleton_is_sol_only() {
        assert_eq!(REQUIRED_MODEL, "gpt-5.6-sol");
    }
}
