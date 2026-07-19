use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::process::Command;

use context_continuum::claim_contract::{parse_and_validate, validate_public_wording};
use context_continuum::doctor::{
    AuthLane, CatalogCompatibility, DoctorObservation, DoctorReport, DoctorState,
    EXIT_INCOMPATIBLE, EXIT_NOT_READY, EXIT_POLICY_READY, EXIT_RUNTIME_ERROR, EXIT_USAGE,
    StatusReport, evaluate, render_doctor, render_status,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[derive(Debug, Deserialize)]
struct Scenario {
    id: String,
    observation: DoctorObservation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct GoldenExpectation {
    id: String,
    state: DoctorState,
    exit_code: u8,
    passed: u64,
    warned: u64,
    failed: u64,
    blocking_check_ids: Vec<String>,
    doctor_json_sha256: String,
    status_json_sha256: String,
    doctor_text_sha256: String,
    status_text_sha256: String,
}

fn scenarios() -> Vec<Scenario> {
    serde_json::from_str(include_str!("fixtures/doctor-scenarios.json")).unwrap()
}

fn golden() -> Vec<GoldenExpectation> {
    serde_json::from_str(include_str!("golden/doctor-scenarios.json")).unwrap()
}

fn expectation(scenario: &Scenario) -> GoldenExpectation {
    let doctor = evaluate(&scenario.observation);
    let status = StatusReport::from(&doctor);
    GoldenExpectation {
        id: scenario.id.clone(),
        state: doctor.state,
        exit_code: doctor.exit_code,
        passed: doctor.summary.passed,
        warned: doctor.summary.warned,
        failed: doctor.summary.failed,
        blocking_check_ids: doctor.summary.blocking_check_ids.clone(),
        doctor_json_sha256: sha256_hex(&pretty_json(&doctor)),
        status_json_sha256: sha256_hex(&pretty_json(&status)),
        doctor_text_sha256: sha256_hex(render_doctor(&doctor).as_bytes()),
        status_text_sha256: sha256_hex(render_status(&status).as_bytes()),
    }
}

fn reports() -> BTreeMap<String, DoctorReport> {
    scenarios()
        .into_iter()
        .map(|scenario| (scenario.id, evaluate(&scenario.observation)))
        .collect()
}

fn pretty_json(value: &impl Serialize) -> Vec<u8> {
    let mut bytes = serde_json::to_vec_pretty(value).unwrap();
    bytes.push(b'\n');
    bytes
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[test]
fn all_eight_scenarios_match_frozen_golden_outputs() {
    let actual = scenarios().iter().map(expectation).collect::<Vec<_>>();
    let expected = golden();
    assert_eq!(actual.len(), 8);
    assert_eq!(actual, expected);
}

#[test]
fn doctor_and_status_outputs_conform_to_draft_2020_12_schemas() {
    let doctor_schema: Value =
        serde_json::from_str(include_str!("../schemas/doctor-report.schema.json")).unwrap();
    let status_schema: Value =
        serde_json::from_str(include_str!("../schemas/status-report.schema.json")).unwrap();
    let doctor_validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&doctor_schema)
        .unwrap();
    let status_validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&status_schema)
        .unwrap();

    for scenario in scenarios() {
        let doctor = evaluate(&scenario.observation);
        let status = StatusReport::from(&doctor);
        let doctor_errors = doctor_validator
            .iter_errors(&serde_json::to_value(&doctor).unwrap())
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        let status_errors = status_validator
            .iter_errors(&serde_json::to_value(&status).unwrap())
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        assert!(
            doctor_errors.is_empty(),
            "{} doctor schema errors: {}",
            scenario.id,
            doctor_errors.join("\n")
        );
        assert!(
            status_errors.is_empty(),
            "{} status schema errors: {}",
            scenario.id,
            status_errors.join("\n")
        );
    }
}

#[test]
fn canonical_dimension_labels_and_claim_safe_wording_are_preserved() {
    let contract =
        parse_and_validate(include_str!("../contracts/capability-vocabulary.json")).unwrap();
    for scenario in scenarios() {
        let doctor = evaluate(&scenario.observation);
        assert_eq!(doctor.dimensions.len(), contract.dimensions.len());
        for (observed, canonical) in doctor.dimensions.iter().zip(&contract.dimensions) {
            assert_eq!(observed.id, canonical.id);
            assert_eq!(observed.label, canonical.label);
            assert_eq!(observed.unit, canonical.unit);
            assert_eq!(observed.authority, canonical.authority);
        }
        let status = StatusReport::from(&doctor);
        validate_public_wording(&contract, &render_doctor(&doctor)).unwrap();
        validate_public_wording(&contract, &render_status(&status)).unwrap();
        assert!(doctor.claim_safety.catalog_is_not_live_proof);
        assert!(!doctor.claim_safety.live_native_window_proven);
        assert!(!doctor.claim_safety.release_claim_ready);
    }
}

#[test]
fn scenario_verdicts_cover_every_cac_12_acceptance_case() {
    let reports = reports();
    let baseline = &reports["chatgpt_272k"];
    assert_eq!(baseline.state, DoctorState::NotReady);
    assert_eq!(baseline.catalog.context_window, Some(272_000));
    assert_eq!(baseline.catalog.effective_codex_budget, Some(258_400));

    let overlay = &reports["chatgpt_sol_1m_overlay"];
    assert_eq!(overlay.catalog.context_window, Some(1_050_000));
    assert_eq!(overlay.catalog.effective_codex_budget, Some(1_008_000));
    assert!(
        overlay
            .summary
            .blocking_check_ids
            .contains(&"compaction_guard".to_owned())
    );

    let api = &reports["api_key_sol_ready_policy"];
    assert_eq!(api.authentication.lane, AuthLane::ApiKey);
    assert_eq!(api.state, DoctorState::PolicyReady);
    assert!(!api.claim_safety.live_native_window_proven);

    let non_sol = &reports["non_sol_override"];
    assert!(!non_sol.model.exact_match);
    assert!(
        non_sol
            .summary
            .blocking_check_ids
            .contains(&"exact_sol_model".to_owned())
    );

    let missing_access = &reports["missing_access"];
    assert!(!missing_access.authentication.authenticated);
    assert!(
        missing_access
            .summary
            .blocking_check_ids
            .contains(&"auth_access".to_owned())
    );

    let stale = &reports["stale_catalog"];
    assert_eq!(stale.catalog.compatibility, CatalogCompatibility::Stale);
    assert_eq!(stale.state, DoctorState::Incompatible);

    let invalid = &reports["invalid_config"];
    assert!(!invalid.configuration.valid);
    assert_eq!(invalid.state, DoctorState::Incompatible);

    let unsupported = &reports["unsupported_codex_version"];
    assert!(!unsupported.codex.compatible);
    assert_eq!(unsupported.state, DoctorState::Incompatible);
}

#[test]
fn exit_code_contract_is_distinct_and_cli_usage_returns_64() {
    assert_eq!(
        BTreeSet::from([
            EXIT_POLICY_READY,
            EXIT_RUNTIME_ERROR,
            EXIT_NOT_READY,
            EXIT_INCOMPATIBLE,
            EXIT_USAGE,
        ])
        .len(),
        5
    );
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_cctx"));
    let output = Command::new(&binary)
        .args(["doctor", "--not-a-real-option"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(i32::from(EXIT_USAGE)));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown doctor option"));

    for command in ["doctor", "status"] {
        let output = Command::new(&binary)
            .args([command, "--help"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Configuration policy ready"));
        assert!(stdout.contains("not live native-window proof"));
    }
}
