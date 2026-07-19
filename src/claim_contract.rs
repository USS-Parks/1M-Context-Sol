//! Machine-enforced vocabulary and release-claim validation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{OFFICIAL_MAX_INPUT, OFFICIAL_MAX_OUTPUT, OFFICIAL_TOTAL_CONTEXT, REQUIRED_MODEL};

const REQUIRED_DIMENSIONS: [&str; 6] = [
    "native_total_context",
    "native_max_input",
    "native_max_output",
    "codex_effective_budget",
    "operational_input_threshold",
    "durable_reservoir_capacity",
];

const REQUIRED_CLAIMS: [(&str, &[&str]); 11] = [
    ("headline", &["G2", "G3", "G4", "G5"]),
    ("sol_only", &["G2", "G5"]),
    ("native_window", &["G2"]),
    ("maximum_input", &["G2"]),
    ("maximum_output", &["G2"]),
    ("effective_budget", &["G2"]),
    ("durable_capacity", &["G3"]),
    ("no_compaction", &["G4"]),
    ("fresh_task_enrollment", &["G5"]),
    ("local_private_default", &["G6"]),
    ("release_installable", &["G8", "G9"]),
];

const HEADLINE: &str = "GPT-5.6 Sol with its native 1.05M window. Compaction blocked. Durable continuity beyond the window.";

/// Complete machine-readable vocabulary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaimContract {
    pub schema_version: u32,
    pub model_contract: ModelContract,
    pub dimensions: Vec<Dimension>,
    pub public_claims: Vec<PublicClaim>,
    pub limitations: Vec<Limitation>,
    pub forbidden_public_phrases: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelContract {
    pub required_slug: String,
    pub fallback_allowed: bool,
    pub alias_allowed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    pub id: String,
    pub label: String,
    pub unit: String,
    pub definition: String,
    pub authority: String,
    pub gate: String,
    pub exact_tokens: Option<u64>,
    pub minimum_tokens: Option<u64>,
    pub maximum_exclusive_tokens: Option<u64>,
    pub substitutes_for: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicClaim {
    pub id: String,
    pub text: String,
    pub required_gates: Vec<String>,
    pub enabled_before_gates_pass: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Limitation {
    pub id: String,
    pub text: String,
}

/// Evidence required to enable the combined release headline.
#[derive(Debug, Clone)]
pub struct ReleaseEvidence {
    pub model_slug: String,
    pub native_total_context: u64,
    pub native_max_input: u64,
    pub native_max_output: u64,
    pub codex_effective_budget: u64,
    pub operational_input_threshold: u64,
    pub durable_reservoir_capacity: u64,
    pub passed_gates: BTreeSet<String>,
}

/// Fail-closed claim-validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimContractError(String);

impl ClaimContractError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ClaimContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ClaimContractError {}

/// Parse and validate the checked-in vocabulary.
pub fn parse_and_validate(input: &str) -> Result<ClaimContract, ClaimContractError> {
    let contract: ClaimContract = serde_json::from_str(input).map_err(|error| {
        ClaimContractError::new(format!("invalid claim-contract JSON: {error}"))
    })?;
    validate_contract(&contract)?;
    Ok(contract)
}

/// Validate contract structure, numeric authority, and gate mappings.
pub fn validate_contract(contract: &ClaimContract) -> Result<(), ClaimContractError> {
    if contract.schema_version != 1 {
        return Err(ClaimContractError::new(
            "unsupported claim-contract schema version",
        ));
    }
    if contract.model_contract.required_slug != REQUIRED_MODEL
        || contract.model_contract.fallback_allowed
        || contract.model_contract.alias_allowed
    {
        return Err(ClaimContractError::new(
            "model contract must require exact gpt-5.6-sol with aliases and fallback disabled",
        ));
    }

    let dimensions = unique_dimensions(contract)?;
    validate_dimensions(&dimensions)?;
    validate_claims(contract)?;
    validate_limitations(contract)?;

    if contract.forbidden_public_phrases.is_empty() {
        return Err(ClaimContractError::new(
            "forbidden public wording list must not be empty",
        ));
    }
    let normalized: BTreeSet<_> = contract
        .forbidden_public_phrases
        .iter()
        .map(|phrase| phrase.to_ascii_lowercase())
        .collect();
    if normalized.len() != contract.forbidden_public_phrases.len() {
        return Err(ClaimContractError::new(
            "forbidden public wording list contains duplicates",
        ));
    }

    Ok(())
}

/// Reject wording that would overstate or conflate a capability.
pub fn validate_public_wording(
    contract: &ClaimContract,
    wording: &str,
) -> Result<(), ClaimContractError> {
    let normalized = wording.to_ascii_lowercase();
    for phrase in &contract.forbidden_public_phrases {
        if normalized.contains(&phrase.to_ascii_lowercase()) {
            return Err(ClaimContractError::new(format!(
                "public wording contains forbidden phrase: {phrase}"
            )));
        }
    }
    Ok(())
}

/// Validate evidence before enabling the combined headline.
pub fn validate_release_evidence(
    contract: &ClaimContract,
    evidence: &ReleaseEvidence,
) -> Result<(), ClaimContractError> {
    if evidence.model_slug != REQUIRED_MODEL {
        return Err(ClaimContractError::new(
            "release evidence is not exact gpt-5.6-sol",
        ));
    }
    if evidence.native_total_context != OFFICIAL_TOTAL_CONTEXT
        || evidence.native_max_input != OFFICIAL_MAX_INPUT
        || evidence.native_max_output != OFFICIAL_MAX_OUTPUT
    {
        return Err(ClaimContractError::new(
            "native total, maximum input, and maximum output do not match the Sol contract",
        ));
    }
    if evidence
        .native_max_input
        .saturating_add(evidence.native_max_output)
        != evidence.native_total_context
    {
        return Err(ClaimContractError::new(
            "maximum input plus maximum output must equal native total context",
        ));
    }
    if evidence.codex_effective_budget < 1_000_000 {
        return Err(ClaimContractError::new(
            "effective Codex budget is below one million tokens",
        ));
    }
    if evidence.operational_input_threshold == 0
        || evidence.operational_input_threshold >= evidence.native_max_input
    {
        return Err(ClaimContractError::new(
            "operational input threshold must be positive and below native maximum input",
        ));
    }
    if evidence.durable_reservoir_capacity < 1_000_000 {
        return Err(ClaimContractError::new(
            "durable reservoir capacity is below one million tokens",
        ));
    }

    let headline = contract
        .public_claims
        .iter()
        .find(|claim| claim.id == "headline")
        .ok_or_else(|| ClaimContractError::new("headline claim is missing"))?;
    for gate in &headline.required_gates {
        if !evidence.passed_gates.contains(gate) {
            return Err(ClaimContractError::new(format!(
                "headline evidence is missing required gate {gate}"
            )));
        }
    }

    Ok(())
}

fn unique_dimensions(
    contract: &ClaimContract,
) -> Result<BTreeMap<&str, &Dimension>, ClaimContractError> {
    let mut dimensions = BTreeMap::new();
    for dimension in &contract.dimensions {
        if dimensions
            .insert(dimension.id.as_str(), dimension)
            .is_some()
        {
            return Err(ClaimContractError::new(format!(
                "duplicate capacity dimension {}",
                dimension.id
            )));
        }
    }
    if dimensions.len() != REQUIRED_DIMENSIONS.len()
        || REQUIRED_DIMENSIONS
            .iter()
            .any(|required| !dimensions.contains_key(required))
    {
        return Err(ClaimContractError::new(
            "claim contract must contain each of the six capacity dimensions exactly once",
        ));
    }
    Ok(dimensions)
}

fn validate_dimensions(dimensions: &BTreeMap<&str, &Dimension>) -> Result<(), ClaimContractError> {
    for dimension in dimensions.values() {
        if dimension.unit != "tokens"
            || dimension.label.is_empty()
            || dimension.definition.is_empty()
            || dimension.authority.is_empty()
            || !valid_gate(&dimension.gate)
            || !dimension.substitutes_for.is_empty()
        {
            return Err(ClaimContractError::new(format!(
                "capacity dimension {} has an invalid authority, unit, gate, or substitution",
                dimension.id
            )));
        }
    }

    require_exact(
        dimensions["native_total_context"],
        OFFICIAL_TOTAL_CONTEXT,
        "G2",
    )?;
    require_exact(dimensions["native_max_input"], OFFICIAL_MAX_INPUT, "G2")?;
    require_exact(dimensions["native_max_output"], OFFICIAL_MAX_OUTPUT, "G2")?;
    require_minimum(dimensions["codex_effective_budget"], 1_000_000, None, "G2")?;
    require_minimum(
        dimensions["operational_input_threshold"],
        1,
        Some(OFFICIAL_MAX_INPUT),
        "G2",
    )?;
    require_minimum(
        dimensions["durable_reservoir_capacity"],
        1_000_000,
        None,
        "G3",
    )?;

    if OFFICIAL_MAX_INPUT + OFFICIAL_MAX_OUTPUT != OFFICIAL_TOTAL_CONTEXT {
        return Err(ClaimContractError::new(
            "official maximum input plus output does not equal total context",
        ));
    }
    Ok(())
}

fn require_exact(dimension: &Dimension, tokens: u64, gate: &str) -> Result<(), ClaimContractError> {
    if dimension.exact_tokens != Some(tokens)
        || dimension.minimum_tokens.is_some()
        || dimension.maximum_exclusive_tokens.is_some()
        || dimension.gate != gate
    {
        return Err(ClaimContractError::new(format!(
            "capacity dimension {} does not have the required exact contract",
            dimension.id
        )));
    }
    Ok(())
}

fn require_minimum(
    dimension: &Dimension,
    minimum: u64,
    maximum_exclusive: Option<u64>,
    gate: &str,
) -> Result<(), ClaimContractError> {
    if dimension.exact_tokens.is_some()
        || dimension.minimum_tokens != Some(minimum)
        || dimension.maximum_exclusive_tokens != maximum_exclusive
        || dimension.gate != gate
    {
        return Err(ClaimContractError::new(format!(
            "capacity dimension {} does not have the required bounded contract",
            dimension.id
        )));
    }
    Ok(())
}

fn validate_claims(contract: &ClaimContract) -> Result<(), ClaimContractError> {
    let mut claims = BTreeMap::new();
    for claim in &contract.public_claims {
        if claims.insert(claim.id.as_str(), claim).is_some() {
            return Err(ClaimContractError::new(format!(
                "duplicate public claim {}",
                claim.id
            )));
        }
        if claim.text.is_empty()
            || claim.required_gates.is_empty()
            || claim.required_gates.iter().any(|gate| !valid_gate(gate))
            || claim.enabled_before_gates_pass
        {
            return Err(ClaimContractError::new(format!(
                "public claim {} has invalid wording, gates, or pre-gate state",
                claim.id
            )));
        }
    }

    if claims.len() != REQUIRED_CLAIMS.len() {
        return Err(ClaimContractError::new(
            "claim contract does not contain the complete public claim map",
        ));
    }
    for (id, expected_gates) in REQUIRED_CLAIMS {
        let claim = claims
            .get(id)
            .ok_or_else(|| ClaimContractError::new(format!("missing public claim {id}")))?;
        let actual: BTreeSet<_> = claim.required_gates.iter().map(String::as_str).collect();
        let expected: BTreeSet<_> = expected_gates.iter().copied().collect();
        if actual != expected {
            return Err(ClaimContractError::new(format!(
                "public claim {id} does not map to its required gates"
            )));
        }
    }
    if claims["headline"].text != HEADLINE {
        return Err(ClaimContractError::new(
            "headline wording does not match the normative contract",
        ));
    }
    Ok(())
}

fn validate_limitations(contract: &ClaimContract) -> Result<(), ClaimContractError> {
    let mut ids = BTreeSet::new();
    for limitation in &contract.limitations {
        if limitation.id.is_empty()
            || limitation.text.is_empty()
            || !ids.insert(limitation.id.as_str())
        {
            return Err(ClaimContractError::new(
                "limitations must have unique non-empty IDs and wording",
            ));
        }
    }
    if ids.is_empty() {
        return Err(ClaimContractError::new(
            "claim contract must record at least one limitation",
        ));
    }
    Ok(())
}

fn valid_gate(gate: &str) -> bool {
    let bytes = gate.as_bytes();
    bytes.len() == 2 && bytes[0] == b'G' && bytes[1].is_ascii_digit()
}
