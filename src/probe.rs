//! Read-only inspection of the installed Codex runtime.

use std::collections::BTreeMap;
use std::env;
use std::fmt::{self, Write as _};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::{OFFICIAL_MAX_INPUT, OFFICIAL_MAX_OUTPUT, OFFICIAL_TOTAL_CONTEXT, REQUIRED_MODEL};

const DEFAULT_EFFECTIVE_CONTEXT_PERCENT: u64 = 95;

/// Inputs for a read-only capability capture.
#[derive(Debug, Clone)]
pub struct ProbeOptions {
    /// Codex executable or launcher name.
    pub codex_command: String,
    /// Config file to inspect. Defaults to the active Codex home.
    pub config_path: Option<PathBuf>,
}

impl Default for ProbeOptions {
    fn default() -> Self {
        Self {
            codex_command: "codex".to_owned(),
            config_path: default_config_path(),
        }
    }
}

/// Sanitized capability evidence produced by `cctx probe`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityReport {
    pub schema_version: u32,
    pub captured_at_unix_seconds: u64,
    pub host: HostEvidence,
    pub codex: CodexEvidence,
    pub catalogs: CatalogEvidence,
    pub official_sol_limits: OfficialSolLimits,
    pub comparison: WindowComparison,
    pub compliance: ComplianceEvidence,
    pub sources: Vec<SourceReference>,
    pub sanitization: SanitizationEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostEvidence {
    pub os: String,
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexEvidence {
    pub version: String,
    pub auth_lane: String,
    pub authenticated: bool,
    pub configured_model: Option<String>,
    pub config: ConfigEvidence,
    pub runtime_features: RuntimeFeatureEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEvidence {
    pub location: String,
    pub present: bool,
    pub model_context_window: Option<u64>,
    pub model_auto_compact_token_limit: Option<u64>,
    pub model_catalog_json_configured: bool,
    pub profile_model_override_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeFeatureEvidence {
    pub hooks: FeatureState,
    pub plugins: FeatureState,
    pub plugin_command_present: bool,
    pub mcp_command_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureState {
    pub available: bool,
    pub stage: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEvidence {
    pub bundled: CatalogSummary,
    pub resolved: CatalogSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogSummary {
    pub normalized_sha256: String,
    pub model_count: u64,
    pub slug: String,
    pub context_window: u64,
    pub max_context_window: u64,
    pub effective_context_window_percent: u64,
    pub effective_budget: u64,
    pub auto_compact_token_limit: Option<u64>,
    pub use_responses_lite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfficialSolLimits {
    pub slug: String,
    pub total_context_window: u64,
    pub max_input: u64,
    pub max_output: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowComparison {
    pub bundled_total_shortfall: u64,
    pub resolved_total_shortfall: u64,
    pub resolved_effective_shortfall_from_one_million: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceEvidence {
    pub exact_sol_configured: bool,
    pub resolved_catalog_is_exact_sol: bool,
    pub resolved_catalog_has_one_million_internal_budget: bool,
    pub native_one_million_gate_ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceReference {
    pub subject: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SanitizationEvidence {
    pub arbitrary_config_keys_omitted: bool,
    pub credentials_omitted: bool,
    pub model_instructions_omitted: bool,
    pub model_request_sent: bool,
}

/// Error returned when the local capability evidence cannot be trusted.
#[derive(Debug, Clone)]
pub struct ProbeError(String);

impl ProbeError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ProbeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ProbeError {}

/// Capture the installed Codex state without changing config or sending a model request.
pub fn capture(options: &ProbeOptions) -> Result<CapabilityReport, ProbeError> {
    let captured_at_unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| ProbeError::new(format!("system clock is before Unix epoch: {error}")))?
        .as_secs();

    let version = run_required(&options.codex_command, &["--version"])?
        .trim()
        .to_owned();
    let command_help = run_required(&options.codex_command, &["--help"])?;
    let feature_output = run_required(&options.codex_command, &["features", "list"])?;
    let auth_output = run_allow_failure(&options.codex_command, &["login", "status"])?;

    let bundled_raw = run_required(&options.codex_command, &["debug", "models", "--bundled"])?;
    let resolved_raw = run_required(&options.codex_command, &["debug", "models"])?;
    let bundled = parse_catalog(&bundled_raw)?;
    let resolved = parse_catalog(&resolved_raw)?;

    let config = inspect_config(options.config_path.as_deref())?;
    let configured_model = config.model.clone();
    let feature_states = parse_feature_states(&feature_output);
    let hooks = feature_states
        .get("hooks")
        .cloned()
        .unwrap_or_else(FeatureState::missing);
    let plugins = feature_states
        .get("plugins")
        .cloned()
        .unwrap_or_else(FeatureState::missing);

    let exact_sol_configured = configured_model.as_deref() == Some(REQUIRED_MODEL);
    let resolved_catalog_is_exact_sol = resolved.slug == REQUIRED_MODEL;
    let resolved_catalog_has_one_million_internal_budget = resolved.effective_budget >= 1_000_000;
    let native_one_million_gate_ready = exact_sol_configured
        && resolved_catalog_is_exact_sol
        && resolved.context_window == OFFICIAL_TOTAL_CONTEXT
        && resolved.max_context_window == OFFICIAL_TOTAL_CONTEXT
        && resolved_catalog_has_one_million_internal_budget;

    Ok(CapabilityReport {
        schema_version: 1,
        captured_at_unix_seconds,
        host: HostEvidence {
            os: env::consts::OS.to_owned(),
            architecture: env::consts::ARCH.to_owned(),
        },
        codex: CodexEvidence {
            version,
            auth_lane: parse_auth_lane(&auth_output.text),
            authenticated: auth_output.success,
            configured_model,
            config: ConfigEvidence {
                location: config.location,
                present: config.present,
                model_context_window: config.model_context_window,
                model_auto_compact_token_limit: config.model_auto_compact_token_limit,
                model_catalog_json_configured: config.model_catalog_json_configured,
                profile_model_override_count: config.profile_model_override_count,
            },
            runtime_features: RuntimeFeatureEvidence {
                hooks,
                plugins,
                plugin_command_present: help_has_command(&command_help, "plugin"),
                mcp_command_present: help_has_command(&command_help, "mcp"),
            },
        },
        catalogs: CatalogEvidence {
            bundled: bundled.clone(),
            resolved: resolved.clone(),
        },
        official_sol_limits: OfficialSolLimits {
            slug: REQUIRED_MODEL.to_owned(),
            total_context_window: OFFICIAL_TOTAL_CONTEXT,
            max_input: OFFICIAL_MAX_INPUT,
            max_output: OFFICIAL_MAX_OUTPUT,
        },
        comparison: WindowComparison {
            bundled_total_shortfall: OFFICIAL_TOTAL_CONTEXT.saturating_sub(bundled.context_window),
            resolved_total_shortfall: OFFICIAL_TOTAL_CONTEXT
                .saturating_sub(resolved.context_window),
            resolved_effective_shortfall_from_one_million: 1_000_000_u64
                .saturating_sub(resolved.effective_budget),
        },
        compliance: ComplianceEvidence {
            exact_sol_configured,
            resolved_catalog_is_exact_sol,
            resolved_catalog_has_one_million_internal_budget,
            native_one_million_gate_ready,
        },
        sources: vec![
            SourceReference {
                subject: "GPT-5.6 Sol model limits".to_owned(),
                url: "https://developers.openai.com/api/docs/models/gpt-5.6-sol".to_owned(),
            },
            SourceReference {
                subject: "Codex configuration reference".to_owned(),
                url: "https://learn.chatgpt.com/docs/config-file/config-reference".to_owned(),
            },
            SourceReference {
                subject: "Codex hooks".to_owned(),
                url: "https://learn.chatgpt.com/docs/hooks".to_owned(),
            },
            SourceReference {
                subject: "Codex plugins".to_owned(),
                url: "https://learn.chatgpt.com/docs/build-plugins".to_owned(),
            },
            SourceReference {
                subject: "Open-source Codex model catalog".to_owned(),
                url:
                    "https://github.com/openai/codex/blob/main/codex-rs/models-manager/models.json"
                        .to_owned(),
            },
        ],
        sanitization: SanitizationEvidence {
            arbitrary_config_keys_omitted: true,
            credentials_omitted: true,
            model_instructions_omitted: true,
            model_request_sent: false,
        },
    })
}

impl FeatureState {
    fn missing() -> Self {
        Self {
            available: false,
            stage: None,
            enabled: None,
        }
    }
}

#[derive(Debug)]
struct CommandEvidence {
    success: bool,
    text: String,
}

fn run_required(command: &str, arguments: &[&str]) -> Result<String, ProbeError> {
    let output = run_command(command, arguments)?;
    if !output.status.success() {
        return Err(command_error(command, arguments, &output));
    }

    String::from_utf8(output.stdout)
        .map_err(|error| ProbeError::new(format!("`{command}` returned non-UTF-8 output: {error}")))
}

fn run_allow_failure(command: &str, arguments: &[&str]) -> Result<CommandEvidence, ProbeError> {
    let output = run_command(command, arguments)?;
    let mut bytes = output.stdout;
    bytes.extend_from_slice(&output.stderr);
    let text = String::from_utf8(bytes).map_err(|error| {
        ProbeError::new(format!("`{command}` returned non-UTF-8 output: {error}"))
    })?;
    Ok(CommandEvidence {
        success: output.status.success(),
        text,
    })
}

fn run_command(command: &str, arguments: &[&str]) -> Result<Output, ProbeError> {
    let executable = platform_command(command);
    Command::new(executable)
        .args(arguments)
        .output()
        .map_err(|error| {
            ProbeError::new(format!(
                "could not run `{} {}`: {error}",
                command,
                arguments.join(" ")
            ))
        })
}

fn platform_command(command: &str) -> &str {
    if cfg!(windows) && command.eq_ignore_ascii_case("codex") {
        "codex.cmd"
    } else {
        command
    }
}

fn command_error(command: &str, arguments: &[&str], output: &Output) -> ProbeError {
    let stderr = String::from_utf8_lossy(&output.stderr);
    ProbeError::new(format!(
        "`{} {}` failed with {}: {}",
        command,
        arguments.join(" "),
        output.status,
        stderr.trim()
    ))
}

fn parse_catalog(raw: &str) -> Result<CatalogSummary, ProbeError> {
    let root: Value = serde_json::from_str(raw).map_err(|error| {
        ProbeError::new(format!("Codex model catalog is invalid JSON: {error}"))
    })?;
    let models = root
        .get("models")
        .and_then(Value::as_array)
        .ok_or_else(|| ProbeError::new("Codex model catalog does not contain a `models` array"))?;
    let sol = models
        .iter()
        .find(|model| model.get("slug").and_then(Value::as_str) == Some(REQUIRED_MODEL))
        .ok_or_else(|| {
            ProbeError::new(format!(
                "Codex model catalog does not contain exact slug `{REQUIRED_MODEL}`"
            ))
        })?;

    let context_window = required_u64(sol, "context_window")?;
    let max_context_window = required_u64(sol, "max_context_window")?;
    let effective_context_window_percent = sol
        .get("effective_context_window_percent")
        .and_then(Value::as_u64)
        .unwrap_or(DEFAULT_EFFECTIVE_CONTEXT_PERCENT);
    if !(1..=100).contains(&effective_context_window_percent) {
        return Err(ProbeError::new(
            "`effective_context_window_percent` must be between 1 and 100",
        ));
    }

    let normalized = serde_json::to_vec(&root).map_err(|error| {
        ProbeError::new(format!("could not normalize Codex model catalog: {error}"))
    })?;
    let effective_budget = context_window.saturating_mul(effective_context_window_percent) / 100;

    Ok(CatalogSummary {
        normalized_sha256: sha256_hex(&normalized),
        model_count: u64::try_from(models.len())
            .map_err(|_| ProbeError::new("model count does not fit in u64"))?,
        slug: REQUIRED_MODEL.to_owned(),
        context_window,
        max_context_window,
        effective_context_window_percent,
        effective_budget,
        auto_compact_token_limit: sol.get("auto_compact_token_limit").and_then(Value::as_u64),
        use_responses_lite: sol
            .get("use_responses_lite")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    })
}

fn required_u64(value: &Value, field: &str) -> Result<u64, ProbeError> {
    value.get(field).and_then(Value::as_u64).ok_or_else(|| {
        ProbeError::new(format!(
            "Sol catalog entry is missing numeric field `{field}`"
        ))
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

fn parse_auth_lane(output: &str) -> String {
    let normalized = output.to_ascii_lowercase();
    if normalized.contains("chatgpt") {
        "chatgpt".to_owned()
    } else if normalized.contains("api key") || normalized.contains("api-key") {
        "api_key".to_owned()
    } else if normalized.contains("not logged in") {
        "unauthenticated".to_owned()
    } else {
        "unknown".to_owned()
    }
}

fn parse_feature_states(output: &str) -> BTreeMap<String, FeatureState> {
    let mut states = BTreeMap::new();
    for line in output.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        if fields.len() < 3 {
            continue;
        }
        let Some(enabled) = fields.last().and_then(|value| value.parse::<bool>().ok()) else {
            continue;
        };
        states.insert(
            fields[0].to_owned(),
            FeatureState {
                available: true,
                stage: Some(fields[1..fields.len() - 1].join(" ")),
                enabled: Some(enabled),
            },
        );
    }
    states
}

fn help_has_command(help: &str, command: &str) -> bool {
    help.lines().any(|line| {
        line.trim_start()
            .strip_prefix(command)
            .is_some_and(|suffix| suffix.starts_with(char::is_whitespace))
    })
}

#[derive(Debug)]
struct InspectedConfig {
    location: String,
    present: bool,
    model: Option<String>,
    model_context_window: Option<u64>,
    model_auto_compact_token_limit: Option<u64>,
    model_catalog_json_configured: bool,
    profile_model_override_count: u64,
}

fn inspect_config(path: Option<&Path>) -> Result<InspectedConfig, ProbeError> {
    let Some(path) = path else {
        return Ok(InspectedConfig::missing());
    };
    if !path.exists() {
        return Ok(InspectedConfig::missing());
    }
    let content = fs::read_to_string(path).map_err(|error| {
        ProbeError::new(format!(
            "could not read Codex config at {}: {error}",
            path.display()
        ))
    })?;
    Ok(inspect_config_text(&content, "default Codex config"))
}

impl InspectedConfig {
    fn missing() -> Self {
        Self {
            location: "default Codex config".to_owned(),
            present: false,
            model: None,
            model_context_window: None,
            model_auto_compact_token_limit: None,
            model_catalog_json_configured: false,
            profile_model_override_count: 0,
        }
    }
}

fn inspect_config_text(content: &str, location: &str) -> InspectedConfig {
    let mut evidence = InspectedConfig {
        location: location.to_owned(),
        present: true,
        model: None,
        model_context_window: None,
        model_auto_compact_token_limit: None,
        model_catalog_json_configured: false,
        profile_model_override_count: 0,
    };
    let mut in_root = true;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            in_root = false;
            continue;
        }
        let Some((key, raw_value)) = line.split_once('=') else {
            continue;
        };
        let key = key.trim();
        let value = raw_value.trim();
        if !in_root {
            if key == "model" {
                evidence.profile_model_override_count =
                    evidence.profile_model_override_count.saturating_add(1);
            }
            continue;
        }

        match key {
            "model" => evidence.model = parse_toml_string(value),
            "model_context_window" => {
                evidence.model_context_window = parse_toml_u64(value);
            }
            "model_auto_compact_token_limit" => {
                evidence.model_auto_compact_token_limit = parse_toml_u64(value);
            }
            "model_catalog_json" => evidence.model_catalog_json_configured = true,
            _ => {}
        }
    }

    evidence
}

fn parse_toml_string(value: &str) -> Option<String> {
    let value = value.split('#').next()?.trim();
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        Some(value[1..value.len() - 1].to_owned())
    } else if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn parse_toml_u64(value: &str) -> Option<u64> {
    value
        .split('#')
        .next()?
        .trim()
        .replace('_', "")
        .parse()
        .ok()
}

fn default_config_path() -> Option<PathBuf> {
    if let Some(codex_home) = env::var_os("CODEX_HOME") {
        return Some(PathBuf::from(codex_home).join("config.toml"));
    }
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(|home| PathBuf::from(home).join(".codex").join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_current_resolved_catalog() {
        let report = parse_catalog(include_str!(
            "../tests/fixtures/models-current-resolved-272k.json"
        ))
        .expect("current catalog should parse");
        assert_eq!(report.slug, REQUIRED_MODEL);
        assert_eq!(report.context_window, 272_000);
        assert_eq!(report.effective_budget, 258_400);
        assert!(report.use_responses_lite);
    }

    #[test]
    fn parses_bundled_catalog_drift_separately() {
        let report = parse_catalog(include_str!("../tests/fixtures/models-bundled-372k.json"))
            .expect("bundled catalog should parse");
        assert_eq!(report.context_window, 372_000);
        assert_eq!(report.effective_budget, 353_400);
    }

    #[test]
    fn applies_documented_default_to_older_shape() {
        let report = parse_catalog(include_str!(
            "../tests/fixtures/models-older-known-shape.json"
        ))
        .expect("known older catalog should parse");
        assert_eq!(
            report.effective_context_window_percent,
            DEFAULT_EFFECTIVE_CONTEXT_PERCENT
        );
    }

    #[test]
    fn rejects_unknown_catalog_shape() {
        let error = parse_catalog(include_str!("../tests/fixtures/models-unknown-schema.json"))
            .expect_err("unknown catalog should fail closed");
        assert!(error.to_string().contains("`models` array"));
    }

    #[test]
    fn config_probe_omits_arbitrary_keys_and_secrets() {
        let config = inspect_config_text(
            r#"
model = "gpt-5.6-sol"
model_context_window = 1_050_000
model_catalog_json = "C:\\private\\catalog.json"
api_key = "must-not-appear"
[profiles.other]
model = "gpt-5.6-sol"
"#,
            "fixture",
        );
        assert_eq!(config.model.as_deref(), Some(REQUIRED_MODEL));
        assert_eq!(config.model_context_window, Some(1_050_000));
        assert!(config.model_catalog_json_configured);
        assert_eq!(config.profile_model_override_count, 1);
        let serialized = serde_json::to_string(&ConfigEvidence {
            location: config.location,
            present: config.present,
            model_context_window: config.model_context_window,
            model_auto_compact_token_limit: config.model_auto_compact_token_limit,
            model_catalog_json_configured: config.model_catalog_json_configured,
            profile_model_override_count: config.profile_model_override_count,
        })
        .expect("config evidence should serialize");
        assert!(!serialized.contains("must-not-appear"));
        assert!(!serialized.contains("private"));
    }

    #[test]
    fn detects_auth_lanes_without_retaining_status_text() {
        assert_eq!(parse_auth_lane("Logged in using ChatGPT"), "chatgpt");
        assert_eq!(parse_auth_lane("Logged in using API key"), "api_key");
        assert_eq!(parse_auth_lane("Not logged in"), "unauthenticated");
    }

    #[test]
    fn detects_command_rows_without_substring_false_positives() {
        let help = "Commands:\n  plugin  Manage plugins\n  mcp     Manage MCP\n";
        assert!(help_has_command(help, "plugin"));
        assert!(help_has_command(help, "mcp"));
        assert!(!help_has_command(help, "app"));
    }

    #[test]
    fn uses_executable_codex_shim_on_windows() {
        if cfg!(windows) {
            assert_eq!(platform_command("codex"), "codex.cmd");
        } else {
            assert_eq!(platform_command("codex"), "codex");
        }
        assert_eq!(platform_command("custom-codex"), "custom-codex");
    }
}
