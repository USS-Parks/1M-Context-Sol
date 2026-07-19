//! Claim-safe doctor and status evaluation for the exact GPT-5.6 Sol policy.

use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use toml_edit::{DocumentMut, Item};

use crate::model_catalog::SUPPORTED_CODEX_VERSION;
use crate::probe::{CapabilityReport, ProbeOptions};
use crate::{OFFICIAL_MAX_INPUT, OFFICIAL_MAX_OUTPUT, OFFICIAL_TOTAL_CONTEXT, REQUIRED_MODEL};

/// A successful command whose configuration policy is ready.
pub const EXIT_POLICY_READY: u8 = 0;
/// The runtime could not be inspected at all.
pub const EXIT_RUNTIME_ERROR: u8 = 1;
/// Inspection succeeded, but one or more fixable policy requirements failed.
pub const EXIT_NOT_READY: u8 = 2;
/// Inspection found an unsupported or untrustworthy input shape.
pub const EXIT_INCOMPATIBLE: u8 = 3;
/// Command-line usage was invalid.
pub const EXIT_USAGE: u8 = 64;

/// Version of both doctor and status report schemas.
pub const DOCTOR_SCHEMA_VERSION: u32 = 1;

const MINIMUM_EFFECTIVE_BUDGET: u64 = 1_000_000;
const MAX_AUTO_COMPACT_LIMIT: u64 = OFFICIAL_TOTAL_CONTEXT * 9 / 10;

/// Supported authentication lanes exposed without retaining credential material.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthLane {
    Chatgpt,
    ApiKey,
    Unauthenticated,
    Unknown,
}

impl AuthLane {
    fn from_probe(value: &str) -> Self {
        match value {
            "chatgpt" => Self::Chatgpt,
            "api_key" => Self::ApiKey,
            "unauthenticated" => Self::Unauthenticated,
            _ => Self::Unknown,
        }
    }

    fn human(self) -> &'static str {
        match self {
            Self::Chatgpt => "ChatGPT",
            Self::ApiKey => "API key",
            Self::Unauthenticated => "Unauthenticated",
            Self::Unknown => "Unknown",
        }
    }
}

/// Whether a catalog can be interpreted by the pinned compatibility profile.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CatalogCompatibility {
    Supported,
    Stale,
    UnknownSchema,
    Missing,
}

/// Strict compaction-guard observation available at inspection time.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompactionGuardState {
    NotInstalled,
    Configured,
    ObservedBlocking,
    Failed,
}

/// A paired automatic-compaction scope parsed from Codex configuration.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutoCompactScope {
    Total,
    BodyAfterPrefix,
}

/// Sanitized strict configuration observation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigObservation {
    pub present: bool,
    pub valid: bool,
    pub error_code: Option<String>,
    pub configured_model: Option<String>,
    pub model_context_window: Option<u64>,
    pub model_auto_compact_token_limit: Option<u64>,
    pub model_auto_compact_token_limit_scope: Option<AutoCompactScope>,
    pub model_catalog_json_configured: bool,
    pub profile_model_override_count: u64,
}

impl ConfigObservation {
    fn missing() -> Self {
        Self {
            present: false,
            valid: true,
            error_code: None,
            configured_model: None,
            model_context_window: None,
            model_auto_compact_token_limit: None,
            model_auto_compact_token_limit_scope: None,
            model_catalog_json_configured: false,
            profile_model_override_count: 0,
        }
    }

    fn invalid(error_code: &str) -> Self {
        Self {
            present: true,
            valid: false,
            error_code: Some(error_code.to_owned()),
            configured_model: None,
            model_context_window: None,
            model_auto_compact_token_limit: None,
            model_auto_compact_token_limit_scope: None,
            model_catalog_json_configured: false,
            profile_model_override_count: 0,
        }
    }
}

/// Sanitized resolved-catalog observation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogObservation {
    pub compatibility: CatalogCompatibility,
    pub normalized_sha256: Option<String>,
    pub model_count: Option<u64>,
    pub slug: Option<String>,
    pub context_window: Option<u64>,
    pub max_context_window: Option<u64>,
    pub effective_context_window_percent: Option<u64>,
    pub auto_compact_token_limit: Option<u64>,
}

impl CatalogObservation {
    fn unavailable(compatibility: CatalogCompatibility) -> Self {
        Self {
            compatibility,
            normalized_sha256: None,
            model_count: None,
            slug: None,
            context_window: None,
            max_context_window: None,
            effective_context_window_percent: None,
            auto_compact_token_limit: None,
        }
    }
}

/// Sanitized strict-guard observation supplied by the lifecycle subsystem.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactionGuardObservation {
    pub state: CompactionGuardState,
    pub checkpoint_threshold: Option<u64>,
    pub strict_blocking_proven: bool,
}

impl Default for CompactionGuardObservation {
    fn default() -> Self {
        Self {
            state: CompactionGuardState::NotInstalled,
            checkpoint_threshold: None,
            strict_blocking_proven: false,
        }
    }
}

/// Complete normalized input to deterministic doctor evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorObservation {
    pub captured_at_unix_seconds: u64,
    pub codex_version: String,
    pub auth_lane: AuthLane,
    pub authenticated: bool,
    pub config: ConfigObservation,
    pub catalog: CatalogObservation,
    pub operational_input_threshold: Option<u64>,
    pub compaction_guard: CompactionGuardObservation,
    pub capture_error_code: Option<String>,
}

/// Inputs for a live, read-only doctor capture.
#[derive(Debug, Clone, Default)]
pub struct DoctorOptions {
    pub probe: ProbeOptions,
    pub operational_input_threshold: Option<u64>,
    pub compaction_guard: CompactionGuardObservation,
}

/// Overall configuration-policy result. This is never a live G2 verdict.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DoctorState {
    PolicyReady,
    NotReady,
    Incompatible,
}

impl DoctorState {
    fn human(self) -> &'static str {
        match self {
            Self::PolicyReady => "POLICY READY",
            Self::NotReady => "NOT READY",
            Self::Incompatible => "INCOMPATIBLE",
        }
    }
}

/// Status of one deterministic diagnostic check.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Pass,
    Warn,
    Fail,
}

impl CheckStatus {
    fn human(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Warn => "WARN",
            Self::Fail => "FAIL",
        }
    }
}

/// One canonical capacity dimension with policy and observed values kept separate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DimensionReport {
    pub id: String,
    pub label: String,
    pub unit: String,
    pub authority: String,
    pub contract_exact_tokens: Option<u64>,
    pub contract_minimum_tokens: Option<u64>,
    pub contract_maximum_exclusive_tokens: Option<u64>,
    pub observed_policy_tokens: Option<u64>,
    pub observation_kind: String,
}

/// One inspectable diagnostic with actionable, claim-safe guidance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiagnosticCheck {
    pub id: String,
    pub label: String,
    pub status: CheckStatus,
    pub incompatible_when_failed: bool,
    pub observed: String,
    pub expected: String,
    pub guidance: String,
}

/// Counts and stable blocker IDs for automation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckSummary {
    pub passed: u64,
    pub warned: u64,
    pub failed: u64,
    pub blocking_check_ids: Vec<String>,
}

/// Model-identity evidence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelStatus {
    pub required_slug: String,
    pub configured_slug: Option<String>,
    pub exact_match: bool,
    pub fallback_allowed: bool,
    pub alias_allowed: bool,
}

/// Authentication evidence without credential material.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthenticationStatus {
    pub lane: AuthLane,
    pub authenticated: bool,
}

/// Codex compatibility evidence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexStatus {
    pub observed_version: String,
    pub supported_version: String,
    pub compatible: bool,
}

/// Sanitized configuration facts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigurationStatus {
    pub present: bool,
    pub valid: bool,
    pub error_code: Option<String>,
    pub model_context_window: Option<u64>,
    pub model_auto_compact_token_limit: Option<u64>,
    pub model_auto_compact_token_limit_scope: Option<AutoCompactScope>,
    pub model_catalog_json_configured: bool,
    pub profile_model_override_count: u64,
}

/// Sanitized catalog facts and checked arithmetic.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogStatus {
    pub compatibility: CatalogCompatibility,
    pub normalized_sha256: Option<String>,
    pub model_count: Option<u64>,
    pub slug: Option<String>,
    pub context_window: Option<u64>,
    pub max_context_window: Option<u64>,
    pub effective_context_window_percent: Option<u64>,
    pub effective_codex_budget: Option<u64>,
    pub auto_compact_token_limit: Option<u64>,
}

/// Compaction state shown without claiming an unproven block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactionStatus {
    pub state: CompactionGuardState,
    pub checkpoint_threshold: Option<u64>,
    pub effective_auto_compact_token_limit: Option<u64>,
    pub strict_blocking_proven: bool,
}

/// Explicit anti-overclaim state carried in every report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaimSafetyStatus {
    pub catalog_is_not_live_proof: bool,
    pub live_native_window_proven: bool,
    pub release_claim_ready: bool,
}

/// Detailed machine-readable doctor report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorReport {
    pub schema_version: u32,
    pub captured_at_unix_seconds: u64,
    pub state: DoctorState,
    pub exit_code: u8,
    pub configuration_policy_ready: bool,
    pub model: ModelStatus,
    pub authentication: AuthenticationStatus,
    pub codex: CodexStatus,
    pub dimensions: Vec<DimensionReport>,
    pub configuration: ConfigurationStatus,
    pub catalog: CatalogStatus,
    pub compaction_guard: CompactionStatus,
    pub checks: Vec<DiagnosticCheck>,
    pub summary: CheckSummary,
    pub claim_safety: ClaimSafetyStatus,
    pub sanitization: DoctorSanitization,
}

/// Compact automation report derived from the detailed report.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusReport {
    pub schema_version: u32,
    pub captured_at_unix_seconds: u64,
    pub state: DoctorState,
    pub exit_code: u8,
    pub configuration_policy_ready: bool,
    pub required_model: String,
    pub configured_model: Option<String>,
    pub auth_lane: AuthLane,
    pub authenticated: bool,
    pub codex_version: String,
    pub native_total_context: u64,
    pub native_max_input: u64,
    pub native_max_output: u64,
    pub resolved_context_window: Option<u64>,
    pub effective_codex_budget: Option<u64>,
    pub operational_input_threshold: Option<u64>,
    pub compaction_guard_state: CompactionGuardState,
    pub live_native_window_proven: bool,
    pub release_claim_ready: bool,
    pub blocking_check_ids: Vec<String>,
}

/// Sanitization assertions shared by human and JSON paths.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DoctorSanitization {
    pub credentials_omitted: bool,
    pub arbitrary_config_omitted: bool,
    pub model_instructions_omitted: bool,
    pub model_request_sent: bool,
}

/// Read-only doctor capture failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorError(String);

impl DoctorError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for DoctorError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for DoctorError {}

/// Capture the local Codex state read-only and produce a deterministic report.
pub fn capture_live(options: &DoctorOptions) -> Result<DoctorReport, DoctorError> {
    let captured_at_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| DoctorError::new(format!("system clock precedes Unix epoch: {error}")))?
        .as_secs();
    let config = inspect_config(options.probe.config_path.as_deref());

    let observation = match crate::probe::capture(&options.probe) {
        Ok(report) => observation_from_capability(
            captured_at_unix_seconds,
            report,
            config,
            options.operational_input_threshold,
            options.compaction_guard.clone(),
        ),
        Err(_) => DoctorObservation {
            captured_at_unix_seconds,
            codex_version: capture_version(&options.probe.codex_command)
                .unwrap_or_else(|| "unobserved".to_owned()),
            auth_lane: AuthLane::Unknown,
            authenticated: false,
            config,
            catalog: CatalogObservation::unavailable(CatalogCompatibility::UnknownSchema),
            operational_input_threshold: options.operational_input_threshold,
            compaction_guard: options.compaction_guard.clone(),
            capture_error_code: Some("capability_capture_failed".to_owned()),
        },
    };
    Ok(evaluate(&observation))
}

/// Evaluate a normalized observation without I/O.
pub fn evaluate(observation: &DoctorObservation) -> DoctorReport {
    let version_compatible = normalize_codex_version(&observation.codex_version)
        .is_some_and(|version| version == SUPPORTED_CODEX_VERSION);
    let exact_model = observation.config.configured_model.as_deref() == Some(REQUIRED_MODEL);
    let auth_ready = observation.authenticated
        && matches!(observation.auth_lane, AuthLane::Chatgpt | AuthLane::ApiKey);
    let effective_budget = calculate_effective_budget(&observation.catalog);
    let effective_auto_compact_token_limit = observation
        .config
        .model_auto_compact_token_limit
        .or(observation.catalog.auto_compact_token_limit);

    let operational_valid = observation
        .operational_input_threshold
        .is_some_and(|value| value > 0 && value < OFFICIAL_MAX_INPUT);
    let operational_before_compaction = match (
        observation.operational_input_threshold,
        effective_auto_compact_token_limit,
    ) {
        (Some(operational), Some(compaction)) => operational <= compaction,
        (Some(_), None) => true,
        (None, _) => false,
    };
    let guard_ready = observation.compaction_guard.state == CompactionGuardState::ObservedBlocking
        && observation.compaction_guard.strict_blocking_proven
        && observation
            .compaction_guard
            .checkpoint_threshold
            .is_some_and(|value| value > 0 && value < OFFICIAL_MAX_INPUT)
        && observation.compaction_guard.checkpoint_threshold
            == observation.operational_input_threshold;

    let mut checks = vec![
        diagnostic(
            "capture_complete",
            "Capability capture complete",
            observation.capture_error_code.is_none(),
            true,
            observation
                .capture_error_code
                .as_deref()
                .unwrap_or("complete"),
            "complete read-only Codex inspection",
            "Run `cctx probe` and resolve the reported local Codex diagnostic failure.",
        ),
        diagnostic(
            "codex_version_supported",
            "Codex version compatibility",
            version_compatible,
            true,
            &observation.codex_version,
            SUPPORTED_CODEX_VERSION,
            "Install the supported Codex build or add a reviewed compatibility profile.",
        ),
        diagnostic(
            "config_valid",
            "Codex configuration syntax",
            observation.config.valid,
            true,
            observation.config.error_code.as_deref().unwrap_or("valid"),
            "valid reviewed TOML",
            "Repair config.toml syntax and value types before applying Context Continuum.",
        ),
        diagnostic(
            "auth_access",
            "GPT-5.6 Sol access lane",
            auth_ready,
            false,
            &format!(
                "{}; authenticated={}",
                observation.auth_lane.human(),
                observation.authenticated
            ),
            "authenticated ChatGPT or API-key lane",
            "Authenticate Codex on a GPT-5.6 Sol-capable ChatGPT or API-key lane.",
        ),
        diagnostic(
            "exact_sol_model",
            "Exact GPT-5.6 Sol model",
            exact_model,
            false,
            observation
                .config
                .configured_model
                .as_deref()
                .unwrap_or("not configured"),
            REQUIRED_MODEL,
            "Set the root model to exact `gpt-5.6-sol`; aliases and fallback are not accepted.",
        ),
        diagnostic(
            "no_profile_model_override",
            "No profile model override",
            observation.config.profile_model_override_count == 0,
            false,
            &observation.config.profile_model_override_count.to_string(),
            "0",
            "Remove profile-level model overrides so no supported task can leave exact Sol.",
        ),
        diagnostic(
            "catalog_compatibility",
            "Resolved catalog compatibility",
            observation.catalog.compatibility == CatalogCompatibility::Supported,
            true,
            catalog_compatibility_text(observation.catalog.compatibility),
            "supported",
            "Regenerate the version-pinned catalog from the supported Codex build; do not reuse stale overlays.",
        ),
        diagnostic(
            "catalog_exact_sol",
            "Resolved catalog exact Sol identity",
            observation.catalog.slug.as_deref() == Some(REQUIRED_MODEL),
            false,
            observation
                .catalog
                .slug
                .as_deref()
                .unwrap_or("not observed"),
            REQUIRED_MODEL,
            "Regenerate a one-model catalog containing exact `gpt-5.6-sol` only.",
        ),
        diagnostic(
            "catalog_single_model",
            "No catalog fallback model",
            observation.catalog.model_count == Some(1),
            false,
            &observation
                .catalog
                .model_count
                .map_or_else(|| "not observed".to_owned(), |value| value.to_string()),
            "1 exact Sol model",
            "Regenerate the reviewed one-model Sol catalog; additional entries are fallback surface.",
        ),
        diagnostic(
            "catalog_native_total",
            "Resolved catalog native-total policy",
            observation.catalog.context_window == Some(OFFICIAL_TOTAL_CONTEXT)
                && observation.catalog.max_context_window == Some(OFFICIAL_TOTAL_CONTEXT),
            false,
            &format!(
                "context={}; max={}",
                optional_tokens(observation.catalog.context_window),
                optional_tokens(observation.catalog.max_context_window)
            ),
            "context=1,050,000; max=1,050,000",
            "Generate and select the reviewed Sol-1M catalog overlay. Catalog values alone are not live proof.",
        ),
        diagnostic(
            "config_native_total",
            "Configured Codex model window",
            observation.config.model_context_window == Some(OFFICIAL_TOTAL_CONTEXT),
            false,
            &optional_tokens(observation.config.model_context_window),
            "1,050,000 tokens",
            "Set `model_context_window = 1050000` through the ownership-aware config manager.",
        ),
        diagnostic(
            "catalog_path_configured",
            "Version-pinned catalog selected",
            observation.config.model_catalog_json_configured,
            false,
            if observation.config.model_catalog_json_configured {
                "configured"
            } else {
                "not configured"
            },
            "configured owned catalog path",
            "Select the generated version-pinned Sol catalog through `model_catalog_json`.",
        ),
        diagnostic(
            "effective_codex_budget",
            "Effective Codex budget",
            effective_budget.is_some_and(|value| value >= MINIMUM_EFFECTIVE_BUDGET),
            false,
            &optional_tokens(effective_budget),
            "at least 1,000,000 tokens",
            "Raise the reviewed effective catalog policy until the computed budget is at least 1,000,000 tokens.",
        ),
        diagnostic(
            "auto_compact_policy",
            "Automatic compaction threshold policy",
            effective_auto_compact_token_limit
                .is_none_or(|value| (1..=MAX_AUTO_COMPACT_LIMIT).contains(&value)),
            false,
            &optional_tokens(effective_auto_compact_token_limit),
            "absent or within 1..=945,000 tokens",
            "Remove the invalid override or keep the automatic-compaction threshold within Codex's 90% clamp.",
        ),
        diagnostic(
            "operational_input_threshold",
            "Operational input threshold",
            operational_valid && operational_before_compaction,
            false,
            &optional_tokens(observation.operational_input_threshold),
            "CAC-14 measured value below 922,000 and no later than automatic compaction",
            "Complete CAC-14 calibration and persist its measured operating threshold.",
        ),
        diagnostic(
            "compaction_guard",
            "Strict compaction guard",
            guard_ready,
            false,
            &format!(
                "state={}; blocking_proven={}; threshold={}",
                guard_state_text(observation.compaction_guard.state),
                observation.compaction_guard.strict_blocking_proven,
                optional_tokens(observation.compaction_guard.checkpoint_threshold)
            ),
            "observed blocking with a checkpoint threshold below 922,000",
            "Install and exercise the strict PreCompact guard; configured state without a blocking observation is not proof.",
        ),
        diagnostic(
            "catalog_not_live_proof",
            "Catalog/config separated from live native-window proof",
            true,
            false,
            "live proof not established by this command",
            "catalog and status values remain non-live policy evidence",
            "Use the separately budgeted G2 live probe before enabling a native-window release claim.",
        ),
    ];

    if effective_auto_compact_token_limit.is_none()
        && let Some(check) = checks
            .iter_mut()
            .find(|check| check.id == "auto_compact_policy")
    {
        check.status = CheckStatus::Warn;
        check.observed = "not explicitly configured".to_owned();
        check.guidance =
            "CAC-14 remains responsible for the measured automatic-compaction policy.".to_owned();
    }

    let incompatible = checks
        .iter()
        .any(|check| check.status == CheckStatus::Fail && check.incompatible_when_failed);
    let failed = checks.iter().any(|check| check.status == CheckStatus::Fail);
    let state = if incompatible {
        DoctorState::Incompatible
    } else if failed {
        DoctorState::NotReady
    } else {
        DoctorState::PolicyReady
    };
    let exit_code = match state {
        DoctorState::PolicyReady => EXIT_POLICY_READY,
        DoctorState::NotReady => EXIT_NOT_READY,
        DoctorState::Incompatible => EXIT_INCOMPATIBLE,
    };
    let summary = summarize(&checks);

    DoctorReport {
        schema_version: DOCTOR_SCHEMA_VERSION,
        captured_at_unix_seconds: observation.captured_at_unix_seconds,
        state,
        exit_code,
        configuration_policy_ready: state == DoctorState::PolicyReady,
        model: ModelStatus {
            required_slug: REQUIRED_MODEL.to_owned(),
            configured_slug: observation.config.configured_model.clone(),
            exact_match: exact_model,
            fallback_allowed: false,
            alias_allowed: false,
        },
        authentication: AuthenticationStatus {
            lane: observation.auth_lane,
            authenticated: observation.authenticated,
        },
        codex: CodexStatus {
            observed_version: observation.codex_version.clone(),
            supported_version: SUPPORTED_CODEX_VERSION.to_owned(),
            compatible: version_compatible,
        },
        dimensions: dimensions(observation, effective_budget),
        configuration: ConfigurationStatus {
            present: observation.config.present,
            valid: observation.config.valid,
            error_code: observation.config.error_code.clone(),
            model_context_window: observation.config.model_context_window,
            model_auto_compact_token_limit: observation.config.model_auto_compact_token_limit,
            model_auto_compact_token_limit_scope: observation
                .config
                .model_auto_compact_token_limit_scope,
            model_catalog_json_configured: observation.config.model_catalog_json_configured,
            profile_model_override_count: observation.config.profile_model_override_count,
        },
        catalog: CatalogStatus {
            compatibility: observation.catalog.compatibility,
            normalized_sha256: observation.catalog.normalized_sha256.clone(),
            model_count: observation.catalog.model_count,
            slug: observation.catalog.slug.clone(),
            context_window: observation.catalog.context_window,
            max_context_window: observation.catalog.max_context_window,
            effective_context_window_percent: observation.catalog.effective_context_window_percent,
            effective_codex_budget: effective_budget,
            auto_compact_token_limit: observation.catalog.auto_compact_token_limit,
        },
        compaction_guard: CompactionStatus {
            state: observation.compaction_guard.state,
            checkpoint_threshold: observation.compaction_guard.checkpoint_threshold,
            effective_auto_compact_token_limit,
            strict_blocking_proven: observation.compaction_guard.strict_blocking_proven,
        },
        checks,
        summary,
        claim_safety: ClaimSafetyStatus {
            catalog_is_not_live_proof: true,
            live_native_window_proven: false,
            release_claim_ready: false,
        },
        sanitization: DoctorSanitization {
            credentials_omitted: true,
            arbitrary_config_omitted: true,
            model_instructions_omitted: true,
            model_request_sent: false,
        },
    }
}

impl From<&DoctorReport> for StatusReport {
    fn from(report: &DoctorReport) -> Self {
        Self {
            schema_version: DOCTOR_SCHEMA_VERSION,
            captured_at_unix_seconds: report.captured_at_unix_seconds,
            state: report.state,
            exit_code: report.exit_code,
            configuration_policy_ready: report.configuration_policy_ready,
            required_model: report.model.required_slug.clone(),
            configured_model: report.model.configured_slug.clone(),
            auth_lane: report.authentication.lane,
            authenticated: report.authentication.authenticated,
            codex_version: report.codex.observed_version.clone(),
            native_total_context: OFFICIAL_TOTAL_CONTEXT,
            native_max_input: OFFICIAL_MAX_INPUT,
            native_max_output: OFFICIAL_MAX_OUTPUT,
            resolved_context_window: report.catalog.context_window,
            effective_codex_budget: report.catalog.effective_codex_budget,
            operational_input_threshold: report
                .dimensions
                .iter()
                .find(|dimension| dimension.id == "operational_input_threshold")
                .and_then(|dimension| dimension.observed_policy_tokens),
            compaction_guard_state: report.compaction_guard.state,
            live_native_window_proven: report.claim_safety.live_native_window_proven,
            release_claim_ready: report.claim_safety.release_claim_ready,
            blocking_check_ids: report.summary.blocking_check_ids.clone(),
        }
    }
}

/// Render the detailed report for a person without weakening claim labels.
pub fn render_doctor(report: &DoctorReport) -> String {
    let mut output = String::new();
    push_line(
        &mut output,
        format!(
            "Context Continuum doctor: {} (exit {})",
            report.state.human(),
            report.exit_code
        ),
    );
    push_line(
        &mut output,
        format!(
            "Model: configured {}; required {}",
            report
                .model
                .configured_slug
                .as_deref()
                .unwrap_or("not configured"),
            report.model.required_slug
        ),
    );
    push_line(
        &mut output,
        format!(
            "Authentication: {} (authenticated={})",
            report.authentication.lane.human(),
            report.authentication.authenticated
        ),
    );
    push_line(
        &mut output,
        format!(
            "Codex version: {} (supported {})",
            report.codex.observed_version, report.codex.supported_version
        ),
    );
    for dimension in &report.dimensions {
        let contract = if let Some(exact) = dimension.contract_exact_tokens {
            format!("contract {} tokens", comma(exact))
        } else {
            let minimum = dimension
                .contract_minimum_tokens
                .map_or_else(|| "not set".to_owned(), comma);
            let maximum = dimension
                .contract_maximum_exclusive_tokens
                .map_or_else(|| "none".to_owned(), comma);
            format!("minimum {minimum}; maximum exclusive {maximum}")
        };
        push_line(
            &mut output,
            format!(
                "{}: {}; observed policy value {}",
                dimension.label,
                contract,
                dimension.observed_policy_tokens.map_or_else(
                    || "not observed".to_owned(),
                    |value| { format!("{} tokens", comma(value)) }
                )
            ),
        );
    }
    push_line(
        &mut output,
        format!(
            "Compaction guard: {} (blocking proven={})",
            guard_state_text(report.compaction_guard.state),
            report.compaction_guard.strict_blocking_proven
        ),
    );
    push_line(&mut output, "Checks:".to_owned());
    for check in &report.checks {
        push_line(
            &mut output,
            format!(
                "  [{}] {} — observed {}; expected {}",
                check.status.human(),
                check.label,
                check.observed,
                check.expected
            ),
        );
        if check.status != CheckStatus::Pass {
            push_line(&mut output, format!("         {}", check.guidance));
        }
    }
    push_line(
        &mut output,
        "Live native-window proof: not established; catalog/config values are policy evidence, not a live G2 result."
            .to_owned(),
    );
    push_line(&mut output, "Release claim ready: false".to_owned());
    output
}

/// Render the compact report for a person.
pub fn render_status(report: &StatusReport) -> String {
    let blockers = if report.blocking_check_ids.is_empty() {
        "none".to_owned()
    } else {
        report.blocking_check_ids.join(", ")
    };
    format!(
        "Context Continuum: {} (exit {})\nModel: {} (required {})\nAuthentication: {} (authenticated={})\nNative total context window: {} tokens\nNative maximum input: {} tokens\nNative maximum output: {} tokens\nResolved Codex context window: {}\nEffective Codex budget: {}\nOperational input threshold: {}\nCompaction guard: {}\nLive native-window proof: false\nRelease claim ready: false\nBlocking checks: {}\n",
        report.state.human(),
        report.exit_code,
        report
            .configured_model
            .as_deref()
            .unwrap_or("not configured"),
        report.required_model,
        report.auth_lane.human(),
        report.authenticated,
        comma(report.native_total_context),
        comma(report.native_max_input),
        comma(report.native_max_output),
        human_optional_tokens(report.resolved_context_window),
        human_optional_tokens(report.effective_codex_budget),
        report.operational_input_threshold.map_or_else(
            || "pending CAC-14 calibration".to_owned(),
            |value| format!("{} tokens", comma(value))
        ),
        guard_state_text(report.compaction_guard_state),
        blockers
    )
}

fn observation_from_capability(
    captured_at_unix_seconds: u64,
    report: CapabilityReport,
    config: ConfigObservation,
    operational_input_threshold: Option<u64>,
    compaction_guard: CompactionGuardObservation,
) -> DoctorObservation {
    DoctorObservation {
        captured_at_unix_seconds,
        codex_version: report.codex.version,
        auth_lane: AuthLane::from_probe(&report.codex.auth_lane),
        authenticated: report.codex.authenticated,
        config,
        catalog: CatalogObservation {
            compatibility: CatalogCompatibility::Supported,
            normalized_sha256: Some(report.catalogs.resolved.normalized_sha256),
            model_count: Some(report.catalogs.resolved.model_count),
            slug: Some(report.catalogs.resolved.slug),
            context_window: Some(report.catalogs.resolved.context_window),
            max_context_window: Some(report.catalogs.resolved.max_context_window),
            effective_context_window_percent: Some(
                report.catalogs.resolved.effective_context_window_percent,
            ),
            auto_compact_token_limit: report.catalogs.resolved.auto_compact_token_limit,
        },
        operational_input_threshold,
        compaction_guard,
        capture_error_code: None,
    }
}

fn inspect_config(path: Option<&Path>) -> ConfigObservation {
    let Some(path) = path else {
        return ConfigObservation::missing();
    };
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return ConfigObservation::missing();
        }
        Err(_) => return ConfigObservation::invalid("config_unreadable"),
    };
    let text = match std::str::from_utf8(&bytes) {
        Ok(text) => text,
        Err(_) => return ConfigObservation::invalid("config_not_utf8"),
    };
    let document = match text.parse::<DocumentMut>() {
        Ok(document) => document,
        Err(_) => return ConfigObservation::invalid("invalid_toml"),
    };
    inspect_document(&document)
}

fn inspect_document(document: &DocumentMut) -> ConfigObservation {
    let table = document.as_table();
    let configured_model = match optional_string(table.get("model")) {
        Ok(value) => value,
        Err(code) => return ConfigObservation::invalid(code),
    };
    let model_context_window = match optional_positive_u64(table.get("model_context_window")) {
        Ok(value) => value,
        Err(code) => return ConfigObservation::invalid(code),
    };
    let model_auto_compact_token_limit =
        match optional_positive_u64(table.get("model_auto_compact_token_limit")) {
            Ok(value) => value,
            Err(code) => return ConfigObservation::invalid(code),
        };
    let model_auto_compact_token_limit_scope =
        match optional_scope(table.get("model_auto_compact_token_limit_scope")) {
            Ok(value) => value,
            Err(code) => return ConfigObservation::invalid(code),
        };
    if model_auto_compact_token_limit.is_some() != model_auto_compact_token_limit_scope.is_some() {
        return ConfigObservation::invalid("unpaired_auto_compact_policy");
    }
    let model_catalog_json_configured = match optional_string(table.get("model_catalog_json")) {
        Ok(Some(value)) if !value.is_empty() && Path::new(&value).is_absolute() => true,
        Ok(Some(value)) if !value.is_empty() => {
            return ConfigObservation::invalid("model_catalog_json_not_absolute");
        }
        Ok(None) => false,
        Ok(Some(_)) => return ConfigObservation::invalid("empty_model_catalog_json"),
        Err(_) => return ConfigObservation::invalid("invalid_model_catalog_json_type"),
    };

    let mut profile_model_override_count = 0_u64;
    if let Some(profiles_item) = table.get("profiles") {
        let Some(profiles) = profiles_item.as_table_like() else {
            return ConfigObservation::invalid("invalid_profiles_type");
        };
        for (_, profile_item) in profiles.iter() {
            let Some(profile) = profile_item.as_table_like() else {
                return ConfigObservation::invalid("invalid_profile_type");
            };
            if let Some(model) = profile.get("model") {
                if model.as_str().is_none() {
                    return ConfigObservation::invalid("invalid_profile_model_type");
                }
                profile_model_override_count = profile_model_override_count.saturating_add(1);
            }
        }
    }

    ConfigObservation {
        present: true,
        valid: true,
        error_code: None,
        configured_model,
        model_context_window,
        model_auto_compact_token_limit,
        model_auto_compact_token_limit_scope,
        model_catalog_json_configured,
        profile_model_override_count,
    }
}

fn optional_string(item: Option<&Item>) -> Result<Option<String>, &'static str> {
    item.map(|item| {
        item.as_str()
            .map(str::to_owned)
            .ok_or("invalid_string_value")
    })
    .transpose()
}

fn optional_positive_u64(item: Option<&Item>) -> Result<Option<u64>, &'static str> {
    item.map(|item| {
        item.as_integer()
            .and_then(|value| u64::try_from(value).ok())
            .filter(|value| *value > 0)
            .ok_or("invalid_positive_integer")
    })
    .transpose()
}

fn optional_scope(item: Option<&Item>) -> Result<Option<AutoCompactScope>, &'static str> {
    item.map(|item| match item.as_str() {
        Some("total") => Ok(AutoCompactScope::Total),
        Some("body_after_prefix") => Ok(AutoCompactScope::BodyAfterPrefix),
        _ => Err("invalid_auto_compact_scope"),
    })
    .transpose()
}

fn capture_version(command: &str) -> Option<String> {
    let executable = if cfg!(windows) && command.eq_ignore_ascii_case("codex") {
        PathBuf::from("codex.cmd")
    } else {
        PathBuf::from(command)
    };
    let output = Command::new(executable).arg("--version").output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn normalize_codex_version(raw: &str) -> Option<&str> {
    raw.split_whitespace().find(|field| {
        field
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_digit())
    })
}

fn calculate_effective_budget(catalog: &CatalogObservation) -> Option<u64> {
    let context = catalog.context_window?;
    let percent = catalog.effective_context_window_percent?;
    if !(1..=100).contains(&percent) {
        return None;
    }
    context.checked_mul(percent).map(|value| value / 100)
}

fn dimensions(
    observation: &DoctorObservation,
    effective_budget: Option<u64>,
) -> Vec<DimensionReport> {
    vec![
        dimension_exact(
            "native_total_context",
            "Native total context window",
            "OpenAI GPT-5.6 Sol model specification",
            OFFICIAL_TOTAL_CONTEXT,
            observation.catalog.context_window,
            "resolved_catalog_policy",
        ),
        dimension_exact(
            "native_max_input",
            "Native maximum input",
            "OpenAI GPT-5.6 Sol model specification",
            OFFICIAL_MAX_INPUT,
            None,
            "official_only_not_live_observed",
        ),
        dimension_exact(
            "native_max_output",
            "Native maximum output",
            "OpenAI GPT-5.6 Sol model specification",
            OFFICIAL_MAX_OUTPUT,
            None,
            "official_only_not_live_observed",
        ),
        dimension_minimum(
            "codex_effective_budget",
            "Effective Codex budget",
            "Resolved Codex runtime configuration and live probe",
            MINIMUM_EFFECTIVE_BUDGET,
            None,
            effective_budget,
            "resolved_catalog_arithmetic",
        ),
        dimension_minimum(
            "operational_input_threshold",
            "Operational input threshold",
            "CAC-14 measured calibration",
            1,
            Some(OFFICIAL_MAX_INPUT),
            observation.operational_input_threshold,
            if observation.operational_input_threshold.is_some() {
                "calibrated_policy"
            } else {
                "not_yet_calibrated"
            },
        ),
        dimension_minimum(
            "durable_reservoir_capacity",
            "Durable reservoir capacity",
            "G3 deterministic corpus benchmark",
            MINIMUM_EFFECTIVE_BUDGET,
            None,
            None,
            "not_implemented_at_cac_12",
        ),
    ]
}

fn dimension_exact(
    id: &str,
    label: &str,
    authority: &str,
    exact: u64,
    observed: Option<u64>,
    observation_kind: &str,
) -> DimensionReport {
    DimensionReport {
        id: id.to_owned(),
        label: label.to_owned(),
        unit: "tokens".to_owned(),
        authority: authority.to_owned(),
        contract_exact_tokens: Some(exact),
        contract_minimum_tokens: None,
        contract_maximum_exclusive_tokens: None,
        observed_policy_tokens: observed,
        observation_kind: observation_kind.to_owned(),
    }
}

#[allow(clippy::too_many_arguments)]
fn dimension_minimum(
    id: &str,
    label: &str,
    authority: &str,
    minimum: u64,
    maximum_exclusive: Option<u64>,
    observed: Option<u64>,
    observation_kind: &str,
) -> DimensionReport {
    DimensionReport {
        id: id.to_owned(),
        label: label.to_owned(),
        unit: "tokens".to_owned(),
        authority: authority.to_owned(),
        contract_exact_tokens: None,
        contract_minimum_tokens: Some(minimum),
        contract_maximum_exclusive_tokens: maximum_exclusive,
        observed_policy_tokens: observed,
        observation_kind: observation_kind.to_owned(),
    }
}

#[allow(clippy::too_many_arguments)]
fn diagnostic(
    id: &str,
    label: &str,
    passed: bool,
    incompatible_when_failed: bool,
    observed: &str,
    expected: &str,
    guidance: &str,
) -> DiagnosticCheck {
    DiagnosticCheck {
        id: id.to_owned(),
        label: label.to_owned(),
        status: if passed {
            CheckStatus::Pass
        } else {
            CheckStatus::Fail
        },
        incompatible_when_failed,
        observed: observed.to_owned(),
        expected: expected.to_owned(),
        guidance: guidance.to_owned(),
    }
}

fn summarize(checks: &[DiagnosticCheck]) -> CheckSummary {
    let mut passed = 0_u64;
    let mut warned = 0_u64;
    let mut failed = 0_u64;
    let mut blocking_check_ids = Vec::new();
    for check in checks {
        match check.status {
            CheckStatus::Pass => passed += 1,
            CheckStatus::Warn => warned += 1,
            CheckStatus::Fail => {
                failed += 1;
                blocking_check_ids.push(check.id.clone());
            }
        }
    }
    CheckSummary {
        passed,
        warned,
        failed,
        blocking_check_ids,
    }
}

fn optional_tokens(value: Option<u64>) -> String {
    value.map_or_else(
        || "not observed".to_owned(),
        |value| format!("{} tokens", comma(value)),
    )
}

fn human_optional_tokens(value: Option<u64>) -> String {
    value.map_or_else(
        || "not observed".to_owned(),
        |value| format!("{} tokens", comma(value)),
    )
}

fn catalog_compatibility_text(value: CatalogCompatibility) -> &'static str {
    match value {
        CatalogCompatibility::Supported => "supported",
        CatalogCompatibility::Stale => "stale",
        CatalogCompatibility::UnknownSchema => "unknown schema",
        CatalogCompatibility::Missing => "missing",
    }
}

fn guard_state_text(value: CompactionGuardState) -> &'static str {
    match value {
        CompactionGuardState::NotInstalled => "not installed",
        CompactionGuardState::Configured => "configured, not proven",
        CompactionGuardState::ObservedBlocking => "observed blocking",
        CompactionGuardState::Failed => "failed",
    }
}

fn comma(value: u64) -> String {
    let digits = value.to_string();
    let mut output = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, character) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            output.push(',');
        }
        output.push(character);
    }
    output
}

fn push_line(output: &mut String, line: String) {
    output.push_str(&line);
    output.push('\n');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_config_parser_reads_only_reviewed_fields() {
        let catalog_path = std::env::temp_dir().join("sol.json");
        let document = format!(
            r#"
model = "gpt-5.6-sol"
model_context_window = 1_050_000
model_auto_compact_token_limit = 900_000
model_auto_compact_token_limit_scope = "body_after_prefix"
model_catalog_json = '{}'
unrelated_secret = "must never enter the report"

[profiles.review]
model = "gpt-5.6-sol"
"#,
            catalog_path.display()
        )
        .parse::<DocumentMut>()
        .unwrap();
        let observed = inspect_document(&document);
        assert!(observed.valid);
        assert_eq!(observed.configured_model.as_deref(), Some(REQUIRED_MODEL));
        assert_eq!(observed.model_context_window, Some(OFFICIAL_TOTAL_CONTEXT));
        assert_eq!(observed.model_auto_compact_token_limit, Some(900_000));
        assert_eq!(
            observed.model_auto_compact_token_limit_scope,
            Some(AutoCompactScope::BodyAfterPrefix)
        );
        assert!(observed.model_catalog_json_configured);
        assert_eq!(observed.profile_model_override_count, 1);
        assert!(
            !serde_json::to_string(&observed)
                .unwrap()
                .contains("must never enter")
        );
    }

    #[test]
    fn strict_config_parser_rejects_bad_types_and_unpaired_policy() {
        let bad_type = "model_context_window = \"1050000\""
            .parse::<DocumentMut>()
            .unwrap();
        assert_eq!(
            inspect_document(&bad_type).error_code.as_deref(),
            Some("invalid_positive_integer")
        );

        let unpaired = "model_auto_compact_token_limit = 900000"
            .parse::<DocumentMut>()
            .unwrap();
        assert_eq!(
            inspect_document(&unpaired).error_code.as_deref(),
            Some("unpaired_auto_compact_policy")
        );
    }

    #[test]
    fn codex_version_and_effective_arithmetic_fail_closed() {
        assert_eq!(
            normalize_codex_version("codex-cli 0.144.5"),
            Some("0.144.5")
        );
        assert_eq!(normalize_codex_version("unobserved"), None);
        let invalid_percent = CatalogObservation {
            compatibility: CatalogCompatibility::Supported,
            normalized_sha256: None,
            model_count: Some(1),
            slug: Some(REQUIRED_MODEL.to_owned()),
            context_window: Some(OFFICIAL_TOTAL_CONTEXT),
            max_context_window: Some(OFFICIAL_TOTAL_CONTEXT),
            effective_context_window_percent: Some(101),
            auto_compact_token_limit: None,
        };
        assert_eq!(calculate_effective_budget(&invalid_percent), None);
    }
}
