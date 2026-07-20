//! Version-pinned GPT-5.6 Sol model-catalog parsing and overlay generation.

use std::collections::BTreeSet;
use std::fmt::{self, Write as _};
use std::process::{Command, Output};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::{OFFICIAL_MAX_INPUT, OFFICIAL_MAX_OUTPUT, OFFICIAL_TOTAL_CONTEXT, REQUIRED_MODEL};

/// Codex build whose resolved catalog shape is accepted by this parser profile.
pub const SUPPORTED_CODEX_VERSION: &str = "0.144.5";

/// Stable identifier for the only catalog schema profile currently supported.
pub const CATALOG_SCHEMA_ID: &str = "codex-model-catalog/0.144.5-v1";

/// Minimum internal effective budget required by the Sol-1M candidate policy.
pub const MINIMUM_EFFECTIVE_BUDGET: u64 = 1_000_000;

/// Effective-window percentage verified by Codex for the Sol-1M catalog.
pub const CALIBRATED_EFFECTIVE_PERCENT: u64 = 96;

/// Automatic-compaction boundary kept below Sol's separate maximum input.
pub const CALIBRATED_AUTO_COMPACT_LIMIT: u64 = 900_000;

/// Operational input boundary reserved below Sol's 922,000-token maximum.
pub const CALIBRATED_OPERATIONAL_INPUT_LIMIT: u64 = 880_000;

/// Proactive checkpoint boundary below rollover and request admission.
pub const CALIBRATED_CHECKPOINT_LIMIT: u64 = 840_000;

/// Successor-rollover boundary below the operational input ceiling.
pub const CALIBRATED_ROLLOVER_LIMIT: u64 = 860_000;

const POLICY_FIELDS: [&str; 4] = [
    "auto_compact_token_limit",
    "context_window",
    "effective_context_window_percent",
    "max_context_window",
];

const REQUIRED_SOL_FIELDS: &[FieldSpec] = &[
    FieldSpec::new("slug", JsonKind::String),
    FieldSpec::new("display_name", JsonKind::String),
    FieldSpec::new("description", JsonKind::StringOrNull),
    FieldSpec::new("default_reasoning_level", JsonKind::StringOrNull),
    FieldSpec::new("supported_reasoning_levels", JsonKind::Array),
    FieldSpec::new("shell_type", JsonKind::String),
    FieldSpec::new("visibility", JsonKind::String),
    FieldSpec::new("supported_in_api", JsonKind::Bool),
    FieldSpec::new("priority", JsonKind::Integer),
    FieldSpec::new("additional_speed_tiers", JsonKind::Array),
    FieldSpec::new("service_tiers", JsonKind::Array),
    FieldSpec::new("availability_nux", JsonKind::ObjectOrNull),
    FieldSpec::new("upgrade", JsonKind::ObjectOrNull),
    FieldSpec::new("base_instructions", JsonKind::String),
    FieldSpec::new("model_messages", JsonKind::ObjectOrNull),
    FieldSpec::new("include_skills_usage_instructions", JsonKind::Bool),
    FieldSpec::new("supports_reasoning_summaries", JsonKind::Bool),
    FieldSpec::new("default_reasoning_summary", JsonKind::String),
    FieldSpec::new("support_verbosity", JsonKind::Bool),
    FieldSpec::new("default_verbosity", JsonKind::StringOrNull),
    FieldSpec::new("apply_patch_tool_type", JsonKind::StringOrNull),
    FieldSpec::new("web_search_tool_type", JsonKind::String),
    FieldSpec::new("truncation_policy", JsonKind::Object),
    FieldSpec::new("supports_parallel_tool_calls", JsonKind::Bool),
    FieldSpec::new("supports_image_detail_original", JsonKind::Bool),
    FieldSpec::new("context_window", JsonKind::Integer),
    FieldSpec::new("max_context_window", JsonKind::Integer),
    FieldSpec::new("comp_hash", JsonKind::StringOrNull),
    FieldSpec::new("effective_context_window_percent", JsonKind::Integer),
    FieldSpec::new("experimental_supported_tools", JsonKind::Array),
    FieldSpec::new("input_modalities", JsonKind::Array),
    FieldSpec::new("supports_search_tool", JsonKind::Bool),
    FieldSpec::new("use_responses_lite", JsonKind::Bool),
    FieldSpec::new("tool_mode", JsonKind::StringOrNull),
    FieldSpec::new("multi_agent_version", JsonKind::StringOrNull),
];

const OPTIONAL_SOL_FIELDS: &[FieldSpec] = &[
    FieldSpec::new("default_service_tier", JsonKind::StringOrNull),
    FieldSpec::new("auto_compact_token_limit", JsonKind::IntegerOrNull),
    FieldSpec::new("auto_review_model_override", JsonKind::StringOrNull),
];

#[derive(Debug, Clone, Copy)]
struct FieldSpec {
    name: &'static str,
    kind: JsonKind,
}

impl FieldSpec {
    const fn new(name: &'static str, kind: JsonKind) -> Self {
        Self { name, kind }
    }
}

#[derive(Debug, Clone, Copy)]
enum JsonKind {
    Array,
    Bool,
    Integer,
    IntegerOrNull,
    Object,
    ObjectOrNull,
    String,
    StringOrNull,
}

impl JsonKind {
    fn accepts(self, value: &Value) -> bool {
        match self {
            Self::Array => value.is_array(),
            Self::Bool => value.is_boolean(),
            Self::Integer => value.as_i64().is_some() || value.as_u64().is_some(),
            Self::IntegerOrNull => {
                value.is_null() || value.as_i64().is_some() || value.as_u64().is_some()
            }
            Self::Object => value.is_object(),
            Self::ObjectOrNull => value.is_object() || value.is_null(),
            Self::String => value.is_string(),
            Self::StringOrNull => value.is_string() || value.is_null(),
        }
    }

    const fn label(self) -> &'static str {
        match self {
            Self::Array => "array",
            Self::Bool => "boolean",
            Self::Integer => "integer",
            Self::IntegerOrNull => "integer or null",
            Self::Object => "object",
            Self::ObjectOrNull => "object or null",
            Self::String => "string",
            Self::StringOrNull => "string or null",
        }
    }
}

/// Official Sol limits supplied to generation and frozen into its manifest.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OfficialSolLimits {
    pub slug: String,
    pub total_context_window: u64,
    pub max_input: u64,
    pub max_output: u64,
}

impl OfficialSolLimits {
    /// Return the official limits frozen by the claim contract.
    pub fn pinned() -> Self {
        Self {
            slug: REQUIRED_MODEL.to_owned(),
            total_context_window: OFFICIAL_TOTAL_CONTEXT,
            max_input: OFFICIAL_MAX_INPUT,
            max_output: OFFICIAL_MAX_OUTPUT,
        }
    }
}

/// Candidate catalog policy. CAC-14 remains responsible for calibrated thresholds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverlayPolicy {
    pub context_window: u64,
    pub max_context_window: u64,
    pub effective_context_window_percent: u64,
    pub auto_compact_token_limit: Option<u64>,
}

impl OverlayPolicy {
    /// Return the uninstalled Sol-1M candidate used by CAC-10.
    pub fn sol_1m_candidate() -> Self {
        Self {
            context_window: OFFICIAL_TOTAL_CONTEXT,
            max_context_window: OFFICIAL_TOTAL_CONTEXT,
            effective_context_window_percent: CALIBRATED_EFFECTIVE_PERCENT,
            auto_compact_token_limit: None,
        }
    }

    /// Return the parser-verified catalog policy used by the Sol launcher.
    pub fn sol_1m_calibrated() -> Self {
        Self {
            context_window: OFFICIAL_TOTAL_CONTEXT,
            max_context_window: OFFICIAL_TOTAL_CONTEXT,
            effective_context_window_percent: CALIBRATED_EFFECTIVE_PERCENT,
            auto_compact_token_limit: Some(CALIBRATED_AUTO_COMPACT_LIMIT),
        }
    }

    /// Effective Codex input budget represented by this policy.
    pub fn effective_budget(&self) -> Result<u64, CatalogError> {
        self.context_window
            .checked_mul(self.effective_context_window_percent)
            .map(|value| value / 100)
            .ok_or_else(|| CatalogError::new("effective context budget overflows u64"))
    }
}

/// Context policy read from one exact Sol entry after Codex resolves a catalog.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResolvedCatalogPolicy {
    pub model: String,
    pub context_window: u64,
    pub max_context_window: u64,
    pub effective_context_window_percent: u64,
    pub auto_compact_token_limit: Option<u64>,
}

/// Measured instruction metadata carried by the live resolved Sol catalog.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogOverheadObservation {
    pub base_instructions_bytes: u64,
    pub model_messages_bytes: u64,
}

/// Hash and compatibility evidence emitted beside an overlay.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogHashManifest {
    pub schema_version: u32,
    pub catalog_schema_id: String,
    pub supported_codex_version: String,
    pub model: String,
    pub source_catalog_sha256: String,
    pub source_normalized_sha256: String,
    pub source_sol_sha256: String,
    pub preserved_sol_sha256: String,
    pub output_catalog_sha256: String,
    pub output_sol_sha256: String,
    pub output_model_count: u64,
    pub changed_policy_fields: Vec<String>,
    pub official_sol_limits: OfficialSolLimits,
    pub overlay_policy: OverlayPolicy,
}

/// Deterministic catalog bytes and their hash manifest.
#[derive(Debug, Clone)]
pub struct CatalogGeneration {
    pub catalog_json: Vec<u8>,
    pub manifest: CatalogHashManifest,
}

impl CatalogGeneration {
    /// Serialize the manifest deterministically with a trailing newline.
    pub fn manifest_json(&self) -> Result<Vec<u8>, CatalogError> {
        pretty_json_bytes(&self.manifest)
    }
}

/// Resolved catalog captured from an installed Codex command.
#[derive(Debug, Clone)]
pub struct InstalledCatalog {
    pub codex_version: String,
    pub json: Vec<u8>,
}

/// Parsed, version-guarded source catalog.
#[derive(Debug, Clone)]
pub struct ParsedCatalog {
    codex_version: String,
    source_catalog_sha256: String,
    source_normalized_sha256: String,
    source_sol: Map<String, Value>,
}

impl ParsedCatalog {
    /// Parse the exact supported Codex catalog profile and locate one Sol entry.
    pub fn parse(raw: &[u8], codex_version: &str) -> Result<Self, CatalogError> {
        let codex_version = normalize_codex_version(codex_version)?;
        let root: Value = serde_json::from_slice(raw).map_err(|error| {
            CatalogError::new(format!("model catalog is invalid JSON: {error}"))
        })?;
        let root_object = root
            .as_object()
            .ok_or_else(|| CatalogError::new("model catalog root must be an object"))?;
        if root_object.len() != 1 || !root_object.contains_key("models") {
            return Err(CatalogError::new(
                "unknown model catalog root schema; expected only `models`",
            ));
        }
        let models = root_object
            .get("models")
            .and_then(Value::as_array)
            .ok_or_else(|| CatalogError::new("model catalog `models` must be an array"))?;

        let mut seen_slugs = BTreeSet::new();
        let mut source_sol = None;
        for model in models {
            let object = model
                .as_object()
                .ok_or_else(|| CatalogError::new("every model catalog entry must be an object"))?;
            let slug = object.get("slug").and_then(Value::as_str).ok_or_else(|| {
                CatalogError::new("every model catalog entry needs a string slug")
            })?;
            if !seen_slugs.insert(slug.to_owned()) {
                return Err(CatalogError::new(format!(
                    "model catalog contains duplicate slug `{slug}`"
                )));
            }
            if slug == REQUIRED_MODEL {
                validate_sol_schema(object)?;
                source_sol = Some(object.clone());
            }
        }
        let source_sol = source_sol.ok_or_else(|| {
            CatalogError::new(format!(
                "model catalog does not contain exact slug `{REQUIRED_MODEL}`"
            ))
        })?;

        let normalized = canonical_json_bytes(&root)?;
        Ok(Self {
            codex_version,
            source_catalog_sha256: sha256_hex(raw),
            source_normalized_sha256: sha256_hex(&normalized),
            source_sol,
        })
    }

    /// Generate an exact one-model Sol catalog while preserving non-policy metadata.
    pub fn generate(
        &self,
        limits: &OfficialSolLimits,
        policy: &OverlayPolicy,
    ) -> Result<CatalogGeneration, CatalogError> {
        validate_official_limits(limits)?;
        validate_policy(limits, policy)?;

        let source_sol_value = Value::Object(self.source_sol.clone());
        let source_sol_sha256 = sha256_hex(&canonical_json_bytes(&source_sol_value)?);
        let preserved_source = without_policy_fields(&self.source_sol);
        let preserved_sol_sha256 = sha256_hex(&canonical_json_bytes(&Value::Object(
            preserved_source.clone(),
        ))?);

        let mut output_sol = self.source_sol.clone();
        insert_u64(&mut output_sol, "context_window", policy.context_window);
        insert_u64(
            &mut output_sol,
            "max_context_window",
            policy.max_context_window,
        );
        insert_u64(
            &mut output_sol,
            "effective_context_window_percent",
            policy.effective_context_window_percent,
        );
        if let Some(limit) = policy.auto_compact_token_limit {
            insert_u64(&mut output_sol, "auto_compact_token_limit", limit);
        } else {
            output_sol.remove("auto_compact_token_limit");
        }

        if without_policy_fields(&output_sol) != preserved_source {
            return Err(CatalogError::new(
                "internal preservation check failed outside approved policy fields",
            ));
        }

        let changed_policy_fields = POLICY_FIELDS
            .iter()
            .filter(|field| self.source_sol.get(**field) != output_sol.get(**field))
            .map(|field| (*field).to_owned())
            .collect::<Vec<_>>();
        if changed_policy_fields.is_empty() {
            return Err(CatalogError::new(
                "overlay generation made no policy-field changes",
            ));
        }

        let output_sol_value = Value::Object(output_sol);
        let output_root = Value::Object(Map::from_iter([(
            "models".to_owned(),
            Value::Array(vec![output_sol_value.clone()]),
        )]));
        let catalog_json = pretty_json_bytes(&output_root)?;
        let output_catalog_sha256 = sha256_hex(&catalog_json);
        let output_sol_sha256 = sha256_hex(&canonical_json_bytes(&output_sol_value)?);

        Ok(CatalogGeneration {
            catalog_json,
            manifest: CatalogHashManifest {
                schema_version: 1,
                catalog_schema_id: CATALOG_SCHEMA_ID.to_owned(),
                supported_codex_version: self.codex_version.clone(),
                model: REQUIRED_MODEL.to_owned(),
                source_catalog_sha256: self.source_catalog_sha256.clone(),
                source_normalized_sha256: self.source_normalized_sha256.clone(),
                source_sol_sha256,
                preserved_sol_sha256,
                output_catalog_sha256,
                output_sol_sha256,
                output_model_count: 1,
                changed_policy_fields,
                official_sol_limits: limits.clone(),
                overlay_policy: policy.clone(),
            },
        })
    }

    /// Return the policy fields from the parsed exact-Sol entry.
    pub fn resolved_policy(&self) -> Result<ResolvedCatalogPolicy, CatalogError> {
        Ok(ResolvedCatalogPolicy {
            model: REQUIRED_MODEL.to_owned(),
            context_window: required_u64(&self.source_sol, "context_window")?,
            max_context_window: required_u64(&self.source_sol, "max_context_window")?,
            effective_context_window_percent: required_u64(
                &self.source_sol,
                "effective_context_window_percent",
            )?,
            auto_compact_token_limit: optional_u64(&self.source_sol, "auto_compact_token_limit")?,
        })
    }

    /// Measure the instruction-bearing fields preserved into the replacement catalog.
    pub fn overhead_observation(&self) -> Result<CatalogOverheadObservation, CatalogError> {
        let base_instructions_bytes = self
            .source_sol
            .get("base_instructions")
            .and_then(Value::as_str)
            .ok_or_else(|| CatalogError::new("Sol base instructions must be a string"))?
            .len();
        let model_messages_bytes = serde_json::to_vec(
            self.source_sol
                .get("model_messages")
                .ok_or_else(|| CatalogError::new("Sol model messages must exist"))?,
        )
        .map_err(|error| CatalogError::new(format!("could not measure model messages: {error}")))?
        .len();
        Ok(CatalogOverheadObservation {
            base_instructions_bytes: u64::try_from(base_instructions_bytes)
                .map_err(|_| CatalogError::new("base-instruction byte count exceeds u64"))?,
            model_messages_bytes: u64::try_from(model_messages_bytes)
                .map_err(|_| CatalogError::new("model-message byte count exceeds u64"))?,
        })
    }
}

/// Read the version and resolved catalog from an installed Codex without model traffic.
pub fn capture_installed_catalog(codex_command: &str) -> Result<InstalledCatalog, CatalogError> {
    let version_output = run_codex(codex_command, &["--version"])?;
    if !version_output.status.success() {
        return Err(command_error(
            codex_command,
            &["--version"],
            &version_output,
        ));
    }
    let version = String::from_utf8(version_output.stdout).map_err(|error| {
        CatalogError::new(format!("Codex version output is not UTF-8: {error}"))
    })?;
    let codex_version = normalize_codex_version(&version)?;

    let catalog_output = run_codex(codex_command, &["debug", "models"])?;
    if !catalog_output.status.success() {
        return Err(command_error(
            codex_command,
            &["debug", "models"],
            &catalog_output,
        ));
    }

    Ok(InstalledCatalog {
        codex_version,
        json: catalog_output.stdout,
    })
}

fn validate_sol_schema(sol: &Map<String, Value>) -> Result<(), CatalogError> {
    let allowed = REQUIRED_SOL_FIELDS
        .iter()
        .chain(OPTIONAL_SOL_FIELDS)
        .map(|spec| spec.name)
        .collect::<BTreeSet<_>>();
    if let Some(field) = sol.keys().find(|field| !allowed.contains(field.as_str())) {
        return Err(CatalogError::new(format!(
            "unknown Sol catalog field `{field}` for schema `{CATALOG_SCHEMA_ID}`"
        )));
    }

    for spec in REQUIRED_SOL_FIELDS {
        let value = sol.get(spec.name).ok_or_else(|| {
            CatalogError::new(format!(
                "Sol catalog schema `{CATALOG_SCHEMA_ID}` is missing `{}`",
                spec.name
            ))
        })?;
        validate_kind(spec, value)?;
    }
    for spec in OPTIONAL_SOL_FIELDS {
        if let Some(value) = sol.get(spec.name) {
            validate_kind(spec, value)?;
        }
    }

    if sol.get("slug").and_then(Value::as_str) != Some(REQUIRED_MODEL) {
        return Err(CatalogError::new(
            "Sol catalog slug changed during validation",
        ));
    }
    if sol
        .get("base_instructions")
        .and_then(Value::as_str)
        .is_none_or(str::is_empty)
    {
        return Err(CatalogError::new("Sol base instructions must not be empty"));
    }
    for field in ["context_window", "max_context_window"] {
        if sol
            .get(field)
            .and_then(Value::as_u64)
            .is_none_or(|value| value == 0)
        {
            return Err(CatalogError::new(format!(
                "Sol catalog field `{field}` must be a positive u64"
            )));
        }
    }
    let effective_percent = sol
        .get("effective_context_window_percent")
        .and_then(Value::as_u64)
        .ok_or_else(|| CatalogError::new("Sol effective percentage must be an unsigned integer"))?;
    if !(1..=100).contains(&effective_percent) {
        return Err(CatalogError::new(
            "Sol effective context percentage must be between 1 and 100",
        ));
    }
    validate_truncation_policy(sol)?;
    Ok(())
}

fn validate_kind(spec: &FieldSpec, value: &Value) -> Result<(), CatalogError> {
    if spec.kind.accepts(value) {
        Ok(())
    } else {
        Err(CatalogError::new(format!(
            "Sol catalog field `{}` must be {}",
            spec.name,
            spec.kind.label()
        )))
    }
}

fn validate_truncation_policy(sol: &Map<String, Value>) -> Result<(), CatalogError> {
    let policy = sol
        .get("truncation_policy")
        .and_then(Value::as_object)
        .ok_or_else(|| CatalogError::new("Sol truncation policy must be an object"))?;
    let expected = BTreeSet::from(["limit", "mode"]);
    let actual = policy.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(CatalogError::new(
            "unknown Sol truncation policy schema; expected `mode` and `limit`",
        ));
    }
    if policy.get("mode").and_then(Value::as_str).is_none()
        || policy.get("limit").and_then(Value::as_i64).is_none()
    {
        return Err(CatalogError::new(
            "Sol truncation policy needs a string mode and integer limit",
        ));
    }
    Ok(())
}

fn validate_official_limits(limits: &OfficialSolLimits) -> Result<(), CatalogError> {
    if limits.slug != REQUIRED_MODEL {
        return Err(CatalogError::new(format!(
            "official limits apply to `{}`, not required model `{REQUIRED_MODEL}`",
            limits.slug
        )));
    }
    if limits.total_context_window < OFFICIAL_TOTAL_CONTEXT
        || limits.max_input < OFFICIAL_MAX_INPUT
        || limits.max_output < OFFICIAL_MAX_OUTPUT
    {
        return Err(CatalogError::new(
            "official GPT-5.6 Sol limits regressed below the frozen claim contract",
        ));
    }
    if limits.max_input.checked_add(limits.max_output) != Some(limits.total_context_window) {
        return Err(CatalogError::new(
            "official Sol maximum input plus output must equal total context",
        ));
    }
    Ok(())
}

fn validate_policy(limits: &OfficialSolLimits, policy: &OverlayPolicy) -> Result<(), CatalogError> {
    if policy.context_window != OFFICIAL_TOTAL_CONTEXT
        || policy.max_context_window != OFFICIAL_TOTAL_CONTEXT
        || policy.context_window > limits.total_context_window
    {
        return Err(CatalogError::new(
            "candidate context and maximum must equal the frozen 1,050,000-token total",
        ));
    }
    if !(1..=100).contains(&policy.effective_context_window_percent) {
        return Err(CatalogError::new(
            "candidate effective context percentage must be between 1 and 100",
        ));
    }
    if policy.effective_budget()? < MINIMUM_EFFECTIVE_BUDGET {
        return Err(CatalogError::new(format!(
            "candidate effective budget must be at least {MINIMUM_EFFECTIVE_BUDGET} tokens"
        )));
    }
    if let Some(limit) = policy.auto_compact_token_limit {
        let codex_clamp = policy.context_window.saturating_mul(9) / 10;
        if limit == 0 || limit > codex_clamp {
            return Err(CatalogError::new(format!(
                "auto-compaction limit must be within 1..={codex_clamp}; Codex clamps at 90%"
            )));
        }
    }
    Ok(())
}

fn without_policy_fields(sol: &Map<String, Value>) -> Map<String, Value> {
    sol.iter()
        .filter(|(field, _)| !POLICY_FIELDS.contains(&field.as_str()))
        .map(|(field, value)| (field.clone(), value.clone()))
        .collect()
}

fn insert_u64(object: &mut Map<String, Value>, field: &str, value: u64) {
    object.insert(field.to_owned(), Value::from(value));
}

fn required_u64(object: &Map<String, Value>, field: &str) -> Result<u64, CatalogError> {
    object
        .get(field)
        .and_then(Value::as_u64)
        .ok_or_else(|| CatalogError::new(format!("Sol catalog field `{field}` must be a u64")))
}

fn optional_u64(object: &Map<String, Value>, field: &str) -> Result<Option<u64>, CatalogError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value.as_u64().map(Some).ok_or_else(|| {
            CatalogError::new(format!("Sol catalog field `{field}` must be a u64 or null"))
        }),
    }
}

fn normalize_codex_version(raw: &str) -> Result<String, CatalogError> {
    let trimmed = raw.trim();
    let version = trimmed.strip_prefix("codex-cli ").unwrap_or(trimmed);
    if version != SUPPORTED_CODEX_VERSION {
        return Err(CatalogError::new(format!(
            "unsupported Codex version `{version}`; supported version is `{SUPPORTED_CODEX_VERSION}`"
        )));
    }
    Ok(version.to_owned())
}

fn run_codex(command: &str, arguments: &[&str]) -> Result<Output, CatalogError> {
    let executable = if cfg!(windows) && command.eq_ignore_ascii_case("codex") {
        "codex.cmd"
    } else {
        command
    };
    Command::new(executable)
        .args(arguments)
        .output()
        .map_err(|error| {
            CatalogError::new(format!(
                "could not run `{} {}`: {error}",
                command,
                arguments.join(" ")
            ))
        })
}

fn command_error(command: &str, arguments: &[&str], output: &Output) -> CatalogError {
    CatalogError::new(format!(
        "`{} {}` failed with {}: {}",
        command,
        arguments.join(" "),
        output.status,
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}

fn canonical_json_bytes(value: &Value) -> Result<Vec<u8>, CatalogError> {
    serde_json::to_vec(value)
        .map_err(|error| CatalogError::new(format!("could not serialize catalog JSON: {error}")))
}

fn pretty_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, CatalogError> {
    let mut bytes = serde_json::to_vec_pretty(value)
        .map_err(|error| CatalogError::new(format!("could not serialize catalog JSON: {error}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("writing to a String cannot fail");
    }
    output
}

/// Error returned when a catalog cannot be trusted or generated safely.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogError(String);

impl CatalogError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for CatalogError {}
