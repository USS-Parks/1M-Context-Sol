//! Fail-closed startup and prompt policy for exact GPT-5.6 Sol tasks.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use atomic_write_file::AtomicWriteFile;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use crate::doctor::{
    AuthLane, CatalogCompatibility, CompactionGuardState, DoctorReport, DoctorState,
    EXIT_POLICY_READY,
};
use crate::{OFFICIAL_MAX_INPUT, OFFICIAL_TOTAL_CONTEXT, REQUIRED_MODEL};

/// Version of the per-task startup-policy audit schema.
pub const STARTUP_POLICY_SCHEMA_VERSION: u32 = 1;
/// Maximum accepted hook envelope, including a near-limit user prompt.
pub const MAX_HOOK_INPUT_BYTES: usize = 8 * 1024 * 1024;

const MINIMUM_EFFECTIVE_BUDGET: u64 = 1_000_000;
const MAX_ID_BYTES: usize = 512;
const MAX_CWD_BYTES: usize = 32 * 1024;
const MAX_MODEL_BYTES: usize = 256;

/// The two lifecycle events governed by CAC-13.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StartupHookEvent {
    #[serde(rename = "SessionStart")]
    SessionStart,
    #[serde(rename = "UserPromptSubmit")]
    UserPromptSubmit,
}

impl StartupHookEvent {
    fn file_name(self) -> &'static str {
        match self {
            Self::SessionStart => "session-start",
            Self::UserPromptSubmit => "user-prompt-submit",
        }
    }
}

/// Documented Codex session-start sources.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StartSource {
    Startup,
    Resume,
    Clear,
    Compact,
}

impl StartSource {
    fn file_name(self) -> &'static str {
        match self {
            Self::Startup => "startup",
            Self::Resume => "resume",
            Self::Clear => "clear",
            Self::Compact => "compact",
        }
    }
}

/// Documented Codex permission modes carried by startup and prompt hooks.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PermissionMode {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "acceptEdits")]
    AcceptEdits,
    #[serde(rename = "plan")]
    Plan,
    #[serde(rename = "dontAsk")]
    DontAsk,
    #[serde(rename = "bypassPermissions")]
    BypassPermissions,
}

#[derive(Debug)]
struct DiscardedPrompt;

impl<'de> Deserialize<'de> for DiscardedPrompt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DiscardedPromptVisitor;

        impl<'de> Visitor<'de> for DiscardedPromptVisitor {
            type Value = DiscardedPrompt;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a user-prompt string")
            }

            fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(DiscardedPrompt)
            }

            fn visit_borrowed_str<E>(self, _value: &'de str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(DiscardedPrompt)
            }

            fn visit_string<E>(self, _value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(DiscardedPrompt)
            }
        }

        deserializer.deserialize_string(DiscardedPromptVisitor)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WireHookInput {
    session_id: String,
    transcript_path: Option<String>,
    cwd: String,
    hook_event_name: StartupHookEvent,
    model: String,
    permission_mode: PermissionMode,
    #[serde(default)]
    source: Option<StartSource>,
    #[serde(default)]
    turn_id: Option<String>,
    #[serde(default)]
    prompt: Option<DiscardedPrompt>,
}

/// Sanitized hook fields needed by startup policy. Prompt and transcript text are discarded.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StartupHookInput {
    pub session_id: String,
    pub cwd: String,
    pub event: StartupHookEvent,
    pub model: String,
    pub permission_mode: PermissionMode,
    pub source: Option<StartSource>,
    pub turn_id: Option<String>,
}

/// Whether the hook permits the task or prompt to proceed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyDecision {
    Allow,
    Block,
}

/// Deterministic result of comparing the active hook envelope with doctor output.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct StartupPolicyVerdict {
    pub decision: PolicyDecision,
    pub event: StartupHookEvent,
    pub active_model: String,
    pub configured_model: Option<String>,
    pub per_task_model_override_detected: bool,
    pub doctor_state: DoctorState,
    pub doctor_exit_code: u8,
    pub blocking_check_ids: Vec<String>,
    pub remediation_commands: Vec<String>,
    pub visible_message: String,
}

/// Official hook-protocol response for SessionStart or UserPromptSubmit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HookResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continue_: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Prompt-free, credential-free per-task enforcement record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StartupPolicyAudit {
    pub schema_version: u32,
    pub captured_at_unix_seconds: u64,
    pub event: StartupHookEvent,
    pub source: Option<StartSource>,
    pub permission_mode: PermissionMode,
    pub session_id_sha256: String,
    pub turn_id_sha256: Option<String>,
    pub cwd_sha256: String,
    pub active_model: String,
    pub configured_model: Option<String>,
    pub per_task_model_override_detected: bool,
    pub decision: PolicyDecision,
    pub doctor_state: DoctorState,
    pub doctor_exit_code: u8,
    pub doctor_policy_ready: bool,
    pub auth_lane: AuthLane,
    pub authenticated: bool,
    pub catalog_compatibility: CatalogCompatibility,
    pub catalog_sha256: Option<String>,
    pub resolved_context_window: Option<u64>,
    pub effective_codex_budget: Option<u64>,
    pub blocking_check_ids: Vec<String>,
    pub remediation_commands: Vec<String>,
    pub prompt_omitted: bool,
    pub transcript_path_omitted: bool,
    pub credentials_omitted: bool,
    pub live_native_window_proven: bool,
}

/// Response plus the audit result produced by one valid hook envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupPolicyOutcome {
    pub response: HookResponse,
    pub verdict: StartupPolicyVerdict,
    pub audit: Option<StartupPolicyAudit>,
    pub audit_path: Option<PathBuf>,
}

/// Input, validation, or audit persistence error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StartupPolicyError(String);

impl StartupPolicyError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for StartupPolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for StartupPolicyError {}

/// Read one hook envelope without allowing unbounded stdin allocation.
pub fn read_bounded_hook_input(reader: &mut impl Read) -> Result<Vec<u8>, StartupPolicyError> {
    let mut bytes = Vec::new();
    reader
        .take((MAX_HOOK_INPUT_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| StartupPolicyError::new(format!("could not read hook stdin: {error}")))?;
    if bytes.len() > MAX_HOOK_INPUT_BYTES {
        return Err(StartupPolicyError::new(format!(
            "hook input exceeds the {MAX_HOOK_INPUT_BYTES}-byte policy limit"
        )));
    }
    Ok(bytes)
}

/// Parse only the documented CAC-13 hook wire shapes and discard prompt content.
pub fn parse_hook_input(bytes: &[u8]) -> Result<StartupHookInput, StartupPolicyError> {
    let bytes = bytes.strip_prefix(b"\xef\xbb\xbf").unwrap_or(bytes);
    let wire: WireHookInput = serde_json::from_slice(bytes)
        .map_err(|error| StartupPolicyError::new(format!("invalid hook JSON: {error}")))?;
    validate_scalar("session_id", &wire.session_id, MAX_ID_BYTES)?;
    validate_scalar("cwd", &wire.cwd, MAX_CWD_BYTES)?;
    validate_scalar("model", &wire.model, MAX_MODEL_BYTES)?;
    if let Some(path) = &wire.transcript_path {
        validate_scalar("transcript_path", path, MAX_CWD_BYTES)?;
    }

    match wire.hook_event_name {
        StartupHookEvent::SessionStart => {
            if wire.source.is_none() || wire.turn_id.is_some() || wire.prompt.is_some() {
                return Err(StartupPolicyError::new(
                    "SessionStart requires source and forbids turn_id/prompt fields",
                ));
            }
        }
        StartupHookEvent::UserPromptSubmit => {
            let turn_id = wire
                .turn_id
                .as_deref()
                .ok_or_else(|| StartupPolicyError::new("UserPromptSubmit requires a turn_id"))?;
            validate_scalar("turn_id", turn_id, MAX_ID_BYTES)?;
            if wire.prompt.is_none() || wire.source.is_some() {
                return Err(StartupPolicyError::new(
                    "UserPromptSubmit requires a string prompt and forbids source",
                ));
            }
        }
    }

    Ok(StartupHookInput {
        session_id: wire.session_id,
        cwd: wire.cwd,
        event: wire.hook_event_name,
        model: wire.model,
        permission_mode: wire.permission_mode,
        source: wire.source,
        turn_id: wire.turn_id,
    })
}

/// Evaluate exact active-model identity and all doctor-owned policy invariants.
pub fn evaluate_startup_policy(
    input: &StartupHookInput,
    doctor: &DoctorReport,
) -> StartupPolicyVerdict {
    let mut blockers: BTreeSet<String> =
        doctor.summary.blocking_check_ids.iter().cloned().collect();
    let configured_model = doctor.model.configured_slug.clone();
    let override_detected = configured_model
        .as_deref()
        .is_some_and(|configured| configured != input.model);

    if input.model != REQUIRED_MODEL {
        blockers.insert("active_hook_model_exact_sol".to_owned());
    }
    if override_detected {
        blockers.insert("per_task_model_override".to_owned());
    }

    let doctor_claims_green = doctor.state == DoctorState::PolicyReady
        && doctor.exit_code == EXIT_POLICY_READY
        && doctor.configuration_policy_ready
        && doctor.summary.failed == 0;
    if !doctor_claims_green {
        blockers.insert("doctor_policy_not_green".to_owned());
    }
    if doctor_claims_green && !doctor_invariants_are_green(doctor) {
        blockers.insert("doctor_report_inconsistent".to_owned());
    }

    let decision = if blockers.is_empty() {
        PolicyDecision::Allow
    } else {
        PolicyDecision::Block
    };
    let blocking_check_ids = blockers.into_iter().collect::<Vec<_>>();
    let remediation_commands = remediation_commands(&blocking_check_ids);
    let visible_message = if decision == PolicyDecision::Block {
        render_block_message(input.event, &blocking_check_ids, &remediation_commands)
    } else {
        String::new()
    };

    StartupPolicyVerdict {
        decision,
        event: input.event,
        active_model: input.model.clone(),
        configured_model,
        per_task_model_override_detected: override_detected,
        doctor_state: doctor.state,
        doctor_exit_code: doctor.exit_code,
        blocking_check_ids,
        remediation_commands,
        visible_message,
    }
}

/// Evaluate, persist the per-task audit, and return a protocol-valid response.
pub fn enforce_and_audit(
    input: &StartupHookInput,
    doctor: &DoctorReport,
    audit_dir: &Path,
) -> StartupPolicyOutcome {
    let mut verdict = evaluate_startup_policy(input, doctor);
    let mut audit = build_audit(input, doctor, &verdict);
    match persist_audit(audit_dir, &audit) {
        Ok(path) => StartupPolicyOutcome {
            response: response_for_verdict(&verdict),
            verdict,
            audit: Some(audit),
            audit_path: Some(path),
        },
        Err(error) => {
            verdict.decision = PolicyDecision::Block;
            insert_sorted_unique(
                &mut verdict.blocking_check_ids,
                "startup_audit_write_failed".to_owned(),
            );
            verdict.remediation_commands = remediation_commands(&verdict.blocking_check_ids);
            verdict.visible_message = format!(
                "{}\nAudit failure: {error}",
                render_block_message(
                    input.event,
                    &verdict.blocking_check_ids,
                    &verdict.remediation_commands,
                )
            );
            audit = build_audit(input, doctor, &verdict);
            StartupPolicyOutcome {
                response: response_for_verdict(&verdict),
                verdict,
                audit: Some(audit),
                audit_path: None,
            }
        }
    }
}

/// Produce a common fail-closed response when the event shape cannot be trusted.
pub fn generic_fail_closed_response(reason: impl AsRef<str>) -> HookResponse {
    let message = format!(
        "CONTEXT CONTINUUM BLOCKED THIS HOOK: {}. Run `cctx doctor --json`; no ordinary prompt is authorized.",
        reason.as_ref()
    );
    HookResponse {
        continue_: Some(false),
        stop_reason: Some(message.clone()),
        system_message: Some(message),
        decision: None,
        reason: None,
    }
}

fn doctor_invariants_are_green(doctor: &DoctorReport) -> bool {
    let operational_threshold = doctor
        .dimensions
        .iter()
        .find(|dimension| dimension.id == "operational_input_threshold")
        .and_then(|dimension| dimension.observed_policy_tokens);
    doctor.model.required_slug == REQUIRED_MODEL
        && doctor.model.configured_slug.as_deref() == Some(REQUIRED_MODEL)
        && doctor.model.exact_match
        && !doctor.model.fallback_allowed
        && !doctor.model.alias_allowed
        && doctor.authentication.authenticated
        && matches!(
            doctor.authentication.lane,
            AuthLane::Chatgpt | AuthLane::ApiKey
        )
        && doctor.codex.compatible
        && doctor.configuration.valid
        && doctor.configuration.profile_model_override_count == 0
        && doctor.configuration.model_context_window == Some(OFFICIAL_TOTAL_CONTEXT)
        && doctor.configuration.model_catalog_json_configured
        && doctor.catalog.compatibility == CatalogCompatibility::Supported
        && doctor
            .catalog
            .normalized_sha256
            .as_deref()
            .is_some_and(is_sha256_hex)
        && doctor.catalog.model_count == Some(1)
        && doctor.catalog.slug.as_deref() == Some(REQUIRED_MODEL)
        && doctor.catalog.context_window == Some(OFFICIAL_TOTAL_CONTEXT)
        && doctor.catalog.max_context_window == Some(OFFICIAL_TOTAL_CONTEXT)
        && doctor
            .catalog
            .effective_codex_budget
            .is_some_and(|budget| budget >= MINIMUM_EFFECTIVE_BUDGET)
        && operational_threshold.is_some_and(|threshold| {
            threshold > 0
                && threshold < OFFICIAL_MAX_INPUT
                && doctor.compaction_guard.checkpoint_threshold == Some(threshold)
        })
        && doctor.compaction_guard.state == CompactionGuardState::ObservedBlocking
        && doctor.compaction_guard.strict_blocking_proven
}

fn remediation_commands(blockers: &[String]) -> Vec<String> {
    let has = |candidates: &[&str]| {
        blockers
            .iter()
            .any(|blocker| candidates.contains(&blocker.as_str()))
    };
    let mut commands = vec!["cctx doctor --json".to_owned()];
    if has(&["auth_access"]) {
        push_unique(&mut commands, "codex login".to_owned());
    }
    if has(&[
        "active_hook_model_exact_sol",
        "per_task_model_override",
        "exact_sol_model",
        "no_profile_model_override",
        "config_valid",
        "config_native_total",
        "catalog_path_configured",
    ]) {
        push_unique(
            &mut commands,
            "cctx config plan --config <ABS_CONFIG> --state-dir <ABS_STATE_DIR> --catalog <ABS_SOL_CATALOG> --cctx <ABS_CCTX>".to_owned(),
        );
    }
    if has(&[
        "catalog_compatibility",
        "catalog_exact_sol",
        "catalog_single_model",
        "catalog_native_total",
        "effective_codex_budget",
    ]) {
        push_unique(
            &mut commands,
            "cctx catalog generate --codex codex --output <ABS_SOL_CATALOG> --manifest <ABS_CATALOG_MANIFEST>".to_owned(),
        );
    }
    if has(&["startup_audit_write_failed"]) {
        push_unique(&mut commands, "cctx hook startup-policy --help".to_owned());
    }
    commands
}

fn render_block_message(
    event: StartupHookEvent,
    blockers: &[String],
    commands: &[String],
) -> String {
    let subject = match event {
        StartupHookEvent::SessionStart => "TASK START",
        StartupHookEvent::UserPromptSubmit => "USER PROMPT",
    };
    let mut message = format!(
        "CONTEXT CONTINUUM BLOCKED {subject}\nExact `{REQUIRED_MODEL}` with a green Sol-1M doctor policy is mandatory.\nBlocking checks: {}\nRemediation commands:",
        blockers.join(", ")
    );
    for command in commands {
        message.push_str("\n  `");
        message.push_str(command);
        message.push('`');
    }
    message.push_str("\nRemove any `--model`/`-m` or profile override, re-run doctor, and restart the task. No ordinary prompt was released to the model.");
    message
}

fn response_for_verdict(verdict: &StartupPolicyVerdict) -> HookResponse {
    match (verdict.event, verdict.decision) {
        (_, PolicyDecision::Allow) => HookResponse {
            continue_: Some(true),
            stop_reason: None,
            system_message: None,
            decision: None,
            reason: None,
        },
        (StartupHookEvent::SessionStart, PolicyDecision::Block) => HookResponse {
            continue_: Some(false),
            stop_reason: Some(verdict.visible_message.clone()),
            system_message: Some(verdict.visible_message.clone()),
            decision: None,
            reason: None,
        },
        (StartupHookEvent::UserPromptSubmit, PolicyDecision::Block) => HookResponse {
            continue_: None,
            stop_reason: None,
            system_message: Some(verdict.visible_message.clone()),
            decision: Some("block".to_owned()),
            reason: Some(verdict.visible_message.clone()),
        },
    }
}

fn build_audit(
    input: &StartupHookInput,
    doctor: &DoctorReport,
    verdict: &StartupPolicyVerdict,
) -> StartupPolicyAudit {
    StartupPolicyAudit {
        schema_version: STARTUP_POLICY_SCHEMA_VERSION,
        captured_at_unix_seconds: doctor.captured_at_unix_seconds,
        event: input.event,
        source: input.source,
        permission_mode: input.permission_mode,
        session_id_sha256: sha256_hex(input.session_id.as_bytes()),
        turn_id_sha256: input
            .turn_id
            .as_deref()
            .map(|turn_id| sha256_hex(turn_id.as_bytes())),
        cwd_sha256: sha256_hex(input.cwd.as_bytes()),
        active_model: input.model.clone(),
        configured_model: doctor.model.configured_slug.clone(),
        per_task_model_override_detected: verdict.per_task_model_override_detected,
        decision: verdict.decision,
        doctor_state: doctor.state,
        doctor_exit_code: doctor.exit_code,
        doctor_policy_ready: doctor.configuration_policy_ready,
        auth_lane: doctor.authentication.lane,
        authenticated: doctor.authentication.authenticated,
        catalog_compatibility: doctor.catalog.compatibility,
        catalog_sha256: doctor.catalog.normalized_sha256.clone(),
        resolved_context_window: doctor.catalog.context_window,
        effective_codex_budget: doctor.catalog.effective_codex_budget,
        blocking_check_ids: verdict.blocking_check_ids.clone(),
        remediation_commands: verdict.remediation_commands.clone(),
        prompt_omitted: true,
        transcript_path_omitted: true,
        credentials_omitted: true,
        live_native_window_proven: doctor.claim_safety.live_native_window_proven,
    }
}

fn persist_audit(
    audit_dir: &Path,
    audit: &StartupPolicyAudit,
) -> Result<PathBuf, StartupPolicyError> {
    if !audit_dir.is_absolute() {
        return Err(StartupPolicyError::new(
            "startup audit directory must be absolute",
        ));
    }
    ensure_plain_directory(audit_dir)?;
    let session_dir = audit_dir.join(format!("session-{}", &audit.session_id_sha256[..24]));
    ensure_plain_directory(&session_dir)?;

    let mut bytes = serde_json::to_vec_pretty(audit)
        .map_err(|error| StartupPolicyError::new(format!("could not serialize audit: {error}")))?;
    bytes.push(b'\n');
    let record_hash = sha256_hex(&bytes);
    let discriminator = audit.turn_id_sha256.as_deref().map_or_else(
        || {
            audit.source.map_or_else(
                || "event".to_owned(),
                |source| source.file_name().to_owned(),
            )
        },
        |turn| turn[..16].to_owned(),
    );
    let path = session_dir.join(format!(
        "{}-{}-{}-{}.json",
        audit.captured_at_unix_seconds,
        audit.event.file_name(),
        discriminator,
        &record_hash[..16]
    ));

    if path.exists() {
        let existing = fs::read(&path).map_err(|error| io_error("read audit", &path, error))?;
        if existing == bytes {
            return Ok(path);
        }
        return Err(StartupPolicyError::new(format!(
            "audit collision at {}",
            path.display()
        )));
    }
    reject_symlink_if_present(&path, "audit target")?;
    let mut file = AtomicWriteFile::open(&path)
        .map_err(|error| io_error("open audit target", &path, error))?;
    file.write_all(&bytes)
        .map_err(|error| io_error("write audit", &path, error))?;
    file.commit()
        .map_err(|error| io_error("commit audit", &path, error))?;
    Ok(path)
}

fn ensure_plain_directory(path: &Path) -> Result<(), StartupPolicyError> {
    if path.exists() {
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| io_error("inspect audit directory", path, error))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(StartupPolicyError::new(format!(
                "audit path is not a plain directory: {}",
                path.display()
            )));
        }
        return Ok(());
    }
    fs::create_dir_all(path).map_err(|error| io_error("create audit directory", path, error))?;
    reject_symlink_if_present(path, "audit directory")
}

fn reject_symlink_if_present(path: &Path, label: &str) -> Result<(), StartupPolicyError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(StartupPolicyError::new(format!(
            "{label} must not be a symbolic link: {}",
            path.display()
        ))),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error(&format!("inspect {label}"), path, error)),
    }
}

fn validate_scalar(label: &str, value: &str, maximum: usize) -> Result<(), StartupPolicyError> {
    if value.is_empty() || value.len() > maximum || value.contains('\0') {
        return Err(StartupPolicyError::new(format!(
            "{label} must contain 1..={maximum} non-NUL bytes"
        )));
    }
    Ok(())
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

fn insert_sorted_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
        values.sort();
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn io_error(action: &str, path: &Path, error: io::Error) -> StartupPolicyError {
    StartupPolicyError::new(format!("could not {action} {}: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_deserialization_discards_the_prompt() {
        let input = parse_hook_input(
            br#"{"session_id":"s","transcript_path":null,"cwd":"C:\\\\repo","hook_event_name":"UserPromptSubmit","model":"gpt-5.6-sol","permission_mode":"default","turn_id":"t","prompt":"do not retain me"}"#,
        )
        .unwrap();
        assert_eq!(input.event, StartupHookEvent::UserPromptSubmit);
        assert_eq!(input.turn_id.as_deref(), Some("t"));
    }

    #[test]
    fn accepts_a_single_utf8_bom_from_windows_command_transport() {
        let mut bytes = b"\xef\xbb\xbf".to_vec();
        bytes.extend_from_slice(
            br#"{"session_id":"s","transcript_path":null,"cwd":"C:\\repo","hook_event_name":"SessionStart","model":"gpt-5.6-sol","permission_mode":"default","source":"startup"}"#,
        );
        let input = parse_hook_input(&bytes).unwrap();
        assert_eq!(input.event, StartupHookEvent::SessionStart);
    }

    #[test]
    fn event_specific_fields_are_strict() {
        let error = parse_hook_input(
            br#"{"session_id":"s","transcript_path":null,"cwd":"/repo","hook_event_name":"SessionStart","model":"gpt-5.6-sol","permission_mode":"default","source":"startup","prompt":"not allowed"}"#,
        )
        .unwrap_err();
        assert!(error.to_string().contains("forbids turn_id/prompt"));
    }

    #[test]
    fn generic_failure_uses_only_common_blocking_fields() {
        let value = serde_json::to_value(generic_fail_closed_response("bad input")).unwrap();
        assert_eq!(value["continue"], false);
        assert!(value["stopReason"].as_str().unwrap().contains("bad input"));
        assert!(value.get("decision").is_none());
    }
}
