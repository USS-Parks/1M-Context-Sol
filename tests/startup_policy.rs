use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use context_continuum::doctor::{DoctorObservation, DoctorReport, evaluate};
use context_continuum::startup_policy::{
    MAX_HOOK_INPUT_BYTES, PermissionMode, PolicyDecision, StartSource, StartupHookEvent,
    StartupHookInput, enforce_and_audit, evaluate_startup_policy, generic_fail_closed_response,
    parse_hook_input, read_bounded_hook_input,
};
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize)]
struct DoctorScenario {
    id: String,
    observation: DoctorObservation,
}

#[derive(Debug, Deserialize)]
struct PolicyCase {
    id: String,
    doctor_scenario: String,
    event: StartupHookEvent,
    active_model: String,
    expected_decision: PolicyDecision,
    expected_blocker: Option<String>,
    override_expected: bool,
}

fn reports() -> BTreeMap<String, DoctorReport> {
    serde_json::from_str::<Vec<DoctorScenario>>(include_str!("fixtures/doctor-scenarios.json"))
        .unwrap()
        .into_iter()
        .map(|scenario| (scenario.id, evaluate(&scenario.observation)))
        .collect()
}

fn cases() -> Vec<PolicyCase> {
    serde_json::from_str(include_str!("fixtures/startup-policy-cases.json")).unwrap()
}

fn input_for(event: StartupHookEvent, active_model: &str, suffix: &str) -> StartupHookInput {
    StartupHookInput {
        session_id: format!("session-{suffix}"),
        cwd: format!("C:\\workspace\\{suffix}"),
        event,
        model: active_model.to_owned(),
        permission_mode: PermissionMode::Default,
        source: (event == StartupHookEvent::SessionStart).then_some(StartSource::Startup),
        turn_id: (event == StartupHookEvent::UserPromptSubmit).then(|| format!("turn-{suffix}")),
    }
}

fn temporary_directory(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("cctx-cac13-{label}-{}-{nanos}", std::process::id()))
}

#[test]
fn acceptance_matrix_blocks_every_noncompliant_case_and_allows_only_green_doctor() {
    let reports = reports();
    let cases = cases();
    assert_eq!(cases.len(), 6);
    for case in cases {
        let doctor = reports.get(&case.doctor_scenario).unwrap();
        let input = input_for(case.event, &case.active_model, &case.id);
        let verdict = evaluate_startup_policy(&input, doctor);
        assert_eq!(verdict.decision, case.expected_decision, "{}", case.id);
        assert_eq!(
            verdict.per_task_model_override_detected, case.override_expected,
            "{}",
            case.id
        );
        if let Some(blocker) = case.expected_blocker {
            assert!(
                verdict.blocking_check_ids.contains(&blocker),
                "{} missing {blocker:?}: {:?}",
                case.id,
                verdict.blocking_check_ids
            );
            assert!(
                verdict
                    .visible_message
                    .contains("CONTEXT CONTINUUM BLOCKED")
            );
            assert!(verdict.visible_message.contains("cctx doctor --json"));
            assert!(
                verdict
                    .visible_message
                    .contains("No ordinary prompt was released")
            );
        } else {
            assert!(verdict.blocking_check_ids.is_empty(), "{}", case.id);
            assert!(verdict.visible_message.is_empty(), "{}", case.id);
        }
    }
}

#[test]
fn override_audit_is_prompt_free_hash_scoped_and_schema_valid() {
    let reports = reports();
    let doctor = &reports["api_key_sol_ready_policy"];
    let secret_prompt = "TOP-SECRET-CAC13-PROMPT-MUST-NOT-PERSIST";
    let raw = serde_json::to_vec(&json!({
        "session_id": "raw-session-id-must-not-persist",
        "transcript_path": "C:\\private\\transcript.jsonl",
        "cwd": "C:\\private\\workspace",
        "hook_event_name": "UserPromptSubmit",
        "model": "gpt-5.6-terra",
        "permission_mode": "default",
        "turn_id": "raw-turn-id-must-not-persist",
        "prompt": secret_prompt
    }))
    .unwrap();
    let input = parse_hook_input(&raw).unwrap();
    let audit_dir = temporary_directory("audit");
    let outcome = enforce_and_audit(&input, doctor, &audit_dir);

    assert_eq!(outcome.verdict.decision, PolicyDecision::Block);
    assert!(outcome.verdict.per_task_model_override_detected);
    let audit = outcome.audit.as_ref().unwrap();
    let audit_path = outcome.audit_path.as_ref().unwrap();
    let audit_bytes = fs::read(audit_path).unwrap();
    let audit_text = String::from_utf8(audit_bytes).unwrap();
    for forbidden in [
        secret_prompt,
        "raw-session-id-must-not-persist",
        "raw-turn-id-must-not-persist",
        "C:\\private\\workspace",
        "C:\\private\\transcript.jsonl",
    ] {
        assert!(!audit_text.contains(forbidden), "retained {forbidden}");
        assert!(!audit_path.to_string_lossy().contains(forbidden));
    }
    assert_eq!(audit.session_id_sha256.len(), 64);
    assert_eq!(audit.turn_id_sha256.as_deref().unwrap().len(), 64);
    assert!(audit.prompt_omitted);
    assert!(audit.transcript_path_omitted);
    assert!(audit.credentials_omitted);
    assert!(!audit.live_native_window_proven);

    let schema: Value =
        serde_json::from_str(include_str!("../schemas/startup-policy-audit.schema.json")).unwrap();
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema)
        .unwrap();
    let document: Value = serde_json::from_str(&audit_text).unwrap();
    let errors = validator
        .iter_errors(&document)
        .map(|error| error.to_string())
        .collect::<Vec<_>>();
    assert!(errors.is_empty(), "{}", errors.join("\n"));

    fs::remove_dir_all(&audit_dir).unwrap();
}

#[test]
fn responses_use_the_documented_event_specific_block_shapes() {
    let reports = reports();
    let audit_dir = temporary_directory("responses");

    let green_session = enforce_and_audit(
        &input_for(
            StartupHookEvent::SessionStart,
            "gpt-5.6-sol",
            "green-session",
        ),
        &reports["api_key_sol_ready_policy"],
        &audit_dir,
    );
    let green_json = serde_json::to_value(&green_session.response).unwrap();
    assert_eq!(green_json, json!({"continue": true}));

    let blocked_session = enforce_and_audit(
        &input_for(
            StartupHookEvent::SessionStart,
            "gpt-5.6-sol",
            "blocked-session",
        ),
        &reports["chatgpt_272k"],
        &audit_dir,
    );
    let session_json = serde_json::to_value(&blocked_session.response).unwrap();
    assert_eq!(session_json["continue"], false);
    assert!(session_json["stopReason"].is_string());
    assert!(session_json["systemMessage"].is_string());
    assert!(session_json.get("decision").is_none());

    let blocked_prompt = enforce_and_audit(
        &input_for(
            StartupHookEvent::UserPromptSubmit,
            "gpt-5.6-sol",
            "blocked-prompt",
        ),
        &reports["missing_access"],
        &audit_dir,
    );
    let prompt_json = serde_json::to_value(&blocked_prompt.response).unwrap();
    assert_eq!(prompt_json["decision"], "block");
    assert!(prompt_json["reason"].is_string());
    assert!(prompt_json["systemMessage"].is_string());
    assert!(prompt_json.get("continue").is_none());

    fs::remove_dir_all(&audit_dir).unwrap();
}

#[test]
fn malformed_unknown_and_oversized_inputs_fail_closed() {
    let unknown_field = br#"{"session_id":"s","transcript_path":null,"cwd":"/repo","hook_event_name":"SessionStart","model":"gpt-5.6-sol","permission_mode":"default","source":"startup","unexpected":true}"#;
    assert!(parse_hook_input(unknown_field).is_err());

    let wrong_shape = br#"{"session_id":"s","transcript_path":null,"cwd":"/repo","hook_event_name":"UserPromptSubmit","model":"gpt-5.6-sol","permission_mode":"default","turn_id":"t"}"#;
    assert!(parse_hook_input(wrong_shape).is_err());

    let response = serde_json::to_value(generic_fail_closed_response("invalid hook JSON")).unwrap();
    assert_eq!(response["continue"], false);
    assert!(
        response["systemMessage"]
            .as_str()
            .unwrap()
            .contains("BLOCKED")
    );

    let mut oversized = std::io::repeat(b'x').take((MAX_HOOK_INPUT_BYTES + 1) as u64);
    let error = read_bounded_hook_input(&mut oversized).unwrap_err();
    assert!(error.to_string().contains("exceeds"));
}

#[test]
fn an_unusable_audit_path_converts_an_allow_to_a_block() {
    let reports = reports();
    let input = input_for(
        StartupHookEvent::SessionStart,
        "gpt-5.6-sol",
        "relative-audit",
    );
    let outcome = enforce_and_audit(
        &input,
        &reports["api_key_sol_ready_policy"],
        PathBuf::from("relative-audit-dir").as_path(),
    );
    assert_eq!(outcome.verdict.decision, PolicyDecision::Block);
    assert!(
        outcome
            .verdict
            .blocking_check_ids
            .contains(&"startup_audit_write_failed".to_owned())
    );
    assert!(outcome.audit_path.is_none());
    assert_eq!(outcome.response.continue_, Some(false));
}

#[test]
fn a_green_label_without_a_catalog_hash_is_rejected_as_inconsistent() {
    let mut reports = reports();
    let doctor = reports.get_mut("api_key_sol_ready_policy").unwrap();
    doctor.catalog.normalized_sha256 = None;
    let input = input_for(
        StartupHookEvent::SessionStart,
        "gpt-5.6-sol",
        "missing-catalog-hash",
    );
    let verdict = evaluate_startup_policy(&input, doctor);
    assert_eq!(verdict.decision, PolicyDecision::Block);
    assert!(
        verdict
            .blocking_check_ids
            .contains(&"doctor_report_inconsistent".to_owned())
    );
}

#[test]
fn cli_help_exposes_the_strict_hook_contract() {
    let output = Command::new(env!("CARGO_BIN_EXE_cctx"))
        .args(["hook", "startup-policy", "--help"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("SessionStart or UserPromptSubmit"));
    assert!(stdout.contains("--audit-dir <ABSOLUTE_DIR>"));
    assert!(stdout.contains("not live native-window proof"));

    let missing_audit = Command::new(env!("CARGO_BIN_EXE_cctx"))
        .args(["hook", "startup-policy"])
        .output()
        .unwrap();
    assert!(missing_audit.status.success());
    let response: Value = serde_json::from_slice(&missing_audit.stdout).unwrap();
    assert_eq!(response["continue"], false);
    assert!(
        response["systemMessage"]
            .as_str()
            .unwrap()
            .contains("requires --audit-dir")
    );
}
