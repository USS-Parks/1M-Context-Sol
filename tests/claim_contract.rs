use std::collections::BTreeSet;

use context_continuum::{
    OFFICIAL_MAX_INPUT, OFFICIAL_MAX_OUTPUT, OFFICIAL_TOTAL_CONTEXT, REQUIRED_MODEL,
    claim_contract::{
        ClaimContract, ReleaseEvidence, parse_and_validate, validate_contract,
        validate_public_wording, validate_release_evidence,
    },
};

fn contract() -> ClaimContract {
    parse_and_validate(include_str!("../contracts/capability-vocabulary.json"))
        .expect("checked-in claim contract must validate")
}

fn passing_evidence() -> ReleaseEvidence {
    ReleaseEvidence {
        model_slug: REQUIRED_MODEL.to_owned(),
        native_total_context: OFFICIAL_TOTAL_CONTEXT,
        native_max_input: OFFICIAL_MAX_INPUT,
        native_max_output: OFFICIAL_MAX_OUTPUT,
        codex_effective_budget: 1_000_000,
        operational_input_threshold: 880_000,
        durable_reservoir_capacity: 1_000_000,
        passed_gates: ["G2", "G3", "G4", "G5"]
            .into_iter()
            .map(str::to_owned)
            .collect(),
    }
}

#[test]
fn checked_in_vocabulary_and_schema_are_valid_json() {
    let contract = contract();
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/capability-vocabulary.schema.json"))
            .expect("checked-in claim schema must be valid JSON");

    assert_eq!(contract.schema_version, 1);
    assert_eq!(
        schema
            .pointer("/properties/model_contract/properties/required_slug/const")
            .and_then(serde_json::Value::as_str),
        Some(REQUIRED_MODEL)
    );
}

#[test]
fn all_capacity_dimensions_are_distinct_and_non_substitutable() {
    let contract = contract();
    let ids: BTreeSet<_> = contract
        .dimensions
        .iter()
        .map(|dimension| dimension.id.as_str())
        .collect();
    assert_eq!(ids.len(), 6);
    assert!(
        contract
            .dimensions
            .iter()
            .all(|dimension| dimension.substitutes_for.is_empty())
    );
}

#[test]
fn every_public_claim_has_a_named_gate_and_is_disabled_before_it() {
    let contract = contract();
    assert_eq!(contract.public_claims.len(), 11);
    assert!(contract.public_claims.iter().all(|claim| {
        !claim.required_gates.is_empty()
            && claim
                .required_gates
                .iter()
                .all(|gate| gate.starts_with('G'))
            && !claim.enabled_before_gates_pass
    }));
}

#[test]
fn truthful_headline_evidence_passes_only_when_dimensions_and_gates_are_separate() {
    let contract = contract();
    validate_release_evidence(&contract, &passing_evidence())
        .expect("complete separated evidence should pass");
}

#[test]
fn a_non_sol_model_fails_compliance() {
    let contract = contract();
    let mut evidence = passing_evidence();
    evidence.model_slug = "not-sol".to_owned();
    let error =
        validate_release_evidence(&contract, &evidence).expect_err("non-Sol evidence must fail");
    assert!(error.to_string().contains("not exact gpt-5.6-sol"));
}

#[test]
fn total_input_output_conflation_fails() {
    let contract = contract();
    let mut evidence = passing_evidence();
    evidence.native_total_context = 1_000_000;
    assert!(validate_release_evidence(&contract, &evidence).is_err());

    let mut evidence = passing_evidence();
    evidence.native_max_input = evidence.native_total_context;
    assert!(validate_release_evidence(&contract, &evidence).is_err());
}

#[test]
fn effective_operational_and_durable_dimensions_cannot_substitute() {
    let contract = contract();

    let mut evidence = passing_evidence();
    evidence.codex_effective_budget = 999_999;
    assert!(validate_release_evidence(&contract, &evidence).is_err());

    let mut evidence = passing_evidence();
    evidence.operational_input_threshold = OFFICIAL_MAX_INPUT;
    assert!(validate_release_evidence(&contract, &evidence).is_err());

    let mut evidence = passing_evidence();
    evidence.durable_reservoir_capacity = 999_999;
    assert!(validate_release_evidence(&contract, &evidence).is_err());
}

#[test]
fn missing_headline_gate_fails() {
    let contract = contract();
    let mut evidence = passing_evidence();
    evidence.passed_gates.remove("G4");
    let error =
        validate_release_evidence(&contract, &evidence).expect_err("missing gate must fail");
    assert!(error.to_string().contains("missing required gate G4"));
}

#[test]
fn ambiguous_public_wording_fails_and_normative_wording_passes() {
    let contract = contract();
    validate_public_wording(
        &contract,
        "GPT-5.6 Sol has 1.05M total context and a separate 922k maximum input.",
    )
    .expect("separated wording should pass");

    for wording in [
        "The product provides one million input tokens.",
        "The product provides unlimited context.",
        "The product works with any model.",
    ] {
        assert!(
            validate_public_wording(&contract, wording).is_err(),
            "ambiguous wording should fail: {wording}"
        );
    }
}

#[test]
fn checked_in_readme_uses_permitted_wording() {
    let contract = contract();
    validate_public_wording(&contract, include_str!("../README.md"))
        .expect("README wording must remain within the claim contract");
}

#[test]
fn mutating_an_authoritative_number_breaks_the_contract() {
    let mut contract = contract();
    let total = contract
        .dimensions
        .iter_mut()
        .find(|dimension| dimension.id == "native_total_context")
        .expect("native total dimension must exist");
    total.exact_tokens = Some(1_000_000);
    assert!(validate_contract(&contract).is_err());
}
