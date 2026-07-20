//! Command-scoped GPT-5.6 Sol 1.05M catalog preparation and launch.

use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};

use crate::model_catalog::{
    CALIBRATED_AUTO_COMPACT_LIMIT, CALIBRATED_CHECKPOINT_LIMIT, CALIBRATED_EFFECTIVE_PERCENT,
    CALIBRATED_OPERATIONAL_INPUT_LIMIT, CALIBRATED_ROLLOVER_LIMIT, OfficialSolLimits,
    OverlayPolicy, ParsedCatalog, ResolvedCatalogPolicy, capture_installed_catalog,
};
use crate::{OFFICIAL_MAX_INPUT, OFFICIAL_MAX_OUTPUT, OFFICIAL_TOTAL_CONTEXT, REQUIRED_MODEL};

const CATALOG_FILE: &str = "sol-1m-models.json";
const MANIFEST_FILE: &str = "sol-1m-catalog-manifest.json";
const RESERVED_BELOW_MAX_INPUT: u64 = OFFICIAL_MAX_INPUT - CALIBRATED_OPERATIONAL_INPUT_LIMIT;
const UNMEASURED_TOOL_AND_WRAPPER_RESERVE: u64 = 16_000;
const CONSERVATIVE_BYTES_PER_TOKEN: u64 = 3;

/// One deterministic input-size/canary stage for the live catalog proof.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProbeStage {
    pub id: String,
    pub target_input_tokens: u64,
    pub canaries: Vec<String>,
}

/// Visible operating band for the Sol context meter.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MeterPhase {
    Normal,
    Checkpoint,
    Rollover,
    AdmissionBlocked,
    CompactionBoundary,
}

/// Claim-safe snapshot used by terminal and future plugin status surfaces.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SolMeter {
    pub model: String,
    pub used_input_tokens: u64,
    pub total_context_window: u64,
    pub effective_codex_budget: u64,
    pub maximum_input: u64,
    pub checkpoint_input_limit: u64,
    pub rollover_input_limit: u64,
    pub operational_input_limit: u64,
    pub auto_compact_token_limit: u64,
    pub phase: MeterPhase,
    pub bar: String,
}

/// Build a deterministic meter from an observed input-token count.
pub fn meter(used_input_tokens: u64) -> SolMeter {
    let phase = if used_input_tokens < CALIBRATED_CHECKPOINT_LIMIT {
        MeterPhase::Normal
    } else if used_input_tokens < CALIBRATED_ROLLOVER_LIMIT {
        MeterPhase::Checkpoint
    } else if used_input_tokens < CALIBRATED_OPERATIONAL_INPUT_LIMIT {
        MeterPhase::Rollover
    } else if used_input_tokens < CALIBRATED_AUTO_COMPACT_LIMIT {
        MeterPhase::AdmissionBlocked
    } else {
        MeterPhase::CompactionBoundary
    };
    let filled = used_input_tokens
        .min(CALIBRATED_AUTO_COMPACT_LIMIT)
        .saturating_mul(40)
        / CALIBRATED_AUTO_COMPACT_LIMIT;
    let filled = usize::try_from(filled).expect("meter width is at most 40");
    let bar = format!("[{}{}]", "#".repeat(filled), ".".repeat(40 - filled));
    SolMeter {
        model: REQUIRED_MODEL.to_owned(),
        used_input_tokens,
        total_context_window: OFFICIAL_TOTAL_CONTEXT,
        effective_codex_budget: OverlayPolicy::sol_1m_calibrated()
            .effective_budget()
            .expect("frozen Sol policy must fit u64"),
        maximum_input: OFFICIAL_MAX_INPUT,
        checkpoint_input_limit: CALIBRATED_CHECKPOINT_LIMIT,
        rollover_input_limit: CALIBRATED_ROLLOVER_LIMIT,
        operational_input_limit: CALIBRATED_OPERATIONAL_INPUT_LIMIT,
        auto_compact_token_limit: CALIBRATED_AUTO_COMPACT_LIMIT,
        phase,
        bar,
    }
}

/// Render the compact terminal meter shared by launch and status commands.
pub fn render_meter(snapshot: &SolMeter) -> String {
    format!(
        "Context Continuum | {} | phase={:?}\n{} {:>7} / {} input tokens\ncheckpoint={} rollover={} admit-until={} compaction={} | native={} effective={} max-input={}\n",
        snapshot.model,
        snapshot.phase,
        snapshot.bar,
        snapshot.used_input_tokens,
        snapshot.auto_compact_token_limit,
        snapshot.checkpoint_input_limit,
        snapshot.rollover_input_limit,
        snapshot.operational_input_limit,
        snapshot.auto_compact_token_limit,
        snapshot.total_context_window,
        snapshot.effective_codex_budget,
        snapshot.maximum_input
    )
}

/// Files and policy produced for one command-scoped Sol launcher.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PreparedSolCatalog {
    pub schema_version: u32,
    pub model: String,
    pub codex_version: String,
    pub catalog_path: PathBuf,
    pub manifest_path: PathBuf,
    pub catalog_sha256: String,
    pub total_context_window: u64,
    pub maximum_input: u64,
    pub maximum_output: u64,
    pub effective_context_window_percent: u64,
    pub effective_codex_budget: u64,
    pub measured_base_instructions_bytes: u64,
    pub measured_model_messages_bytes: u64,
    pub conservative_catalog_overhead_tokens: u64,
    pub unmeasured_tool_and_wrapper_reserve_tokens: u64,
    pub reserved_below_maximum_input_tokens: u64,
    pub checkpoint_input_limit: u64,
    pub rollover_input_limit: u64,
    pub operational_input_limit: u64,
    pub auto_compact_token_limit: u64,
    pub probe_stages: Vec<ProbeStage>,
    pub global_config_changed: bool,
}

/// Generate the live-version, exact-Sol 1.05M catalog into an explicit state directory.
pub fn prepare(
    codex_command: &str,
    state_dir: &Path,
) -> Result<PreparedSolCatalog, SolLaunchError> {
    validate_state_dir(state_dir)?;
    let installed = capture_installed_catalog(codex_command)
        .map_err(|error| SolLaunchError::new(error.to_string()))?;
    let parsed = ParsedCatalog::parse(&installed.json, &installed.codex_version)
        .map_err(|error| SolLaunchError::new(error.to_string()))?;
    let overhead = parsed
        .overhead_observation()
        .map_err(|error| SolLaunchError::new(error.to_string()))?;
    let conservative_catalog_overhead_tokens = overhead
        .base_instructions_bytes
        .checked_add(overhead.model_messages_bytes)
        .and_then(|bytes| bytes.checked_add(CONSERVATIVE_BYTES_PER_TOKEN - 1))
        .map(|bytes| bytes / CONSERVATIVE_BYTES_PER_TOKEN)
        .ok_or_else(|| SolLaunchError::new("catalog overhead measurement overflowed"))?;
    let required_reserve = conservative_catalog_overhead_tokens
        .checked_add(UNMEASURED_TOOL_AND_WRAPPER_RESERVE)
        .ok_or_else(|| SolLaunchError::new("input overhead reserve overflowed"))?;
    if required_reserve > RESERVED_BELOW_MAX_INPUT {
        return Err(SolLaunchError::new(format!(
            "live catalog overhead requires {required_reserve} reserved tokens, above the frozen {RESERVED_BELOW_MAX_INPUT}-token reserve"
        )));
    }
    let generated = parsed
        .generate(
            &OfficialSolLimits::pinned(),
            &OverlayPolicy::sol_1m_calibrated(),
        )
        .map_err(|error| SolLaunchError::new(error.to_string()))?;
    let manifest = generated
        .manifest_json()
        .map_err(|error| SolLaunchError::new(error.to_string()))?;

    fs::create_dir_all(state_dir)
        .map_err(|error| io_error("create launcher state directory", state_dir, error))?;
    reject_symlink(state_dir, "launcher state directory")?;
    let catalog_path = state_dir.join(CATALOG_FILE);
    let manifest_path = state_dir.join(MANIFEST_FILE);
    atomic_write(&catalog_path, &generated.catalog_json)?;
    atomic_write(&manifest_path, &manifest)?;

    Ok(PreparedSolCatalog {
        schema_version: 1,
        model: REQUIRED_MODEL.to_owned(),
        codex_version: installed.codex_version,
        catalog_path,
        manifest_path,
        catalog_sha256: generated.manifest.output_catalog_sha256,
        total_context_window: OFFICIAL_TOTAL_CONTEXT,
        maximum_input: OFFICIAL_MAX_INPUT,
        maximum_output: OFFICIAL_MAX_OUTPUT,
        effective_context_window_percent: CALIBRATED_EFFECTIVE_PERCENT,
        effective_codex_budget: OverlayPolicy::sol_1m_calibrated()
            .effective_budget()
            .map_err(|error| SolLaunchError::new(error.to_string()))?,
        measured_base_instructions_bytes: overhead.base_instructions_bytes,
        measured_model_messages_bytes: overhead.model_messages_bytes,
        conservative_catalog_overhead_tokens,
        unmeasured_tool_and_wrapper_reserve_tokens: UNMEASURED_TOOL_AND_WRAPPER_RESERVE,
        reserved_below_maximum_input_tokens: RESERVED_BELOW_MAX_INPUT,
        checkpoint_input_limit: CALIBRATED_CHECKPOINT_LIMIT,
        rollover_input_limit: CALIBRATED_ROLLOVER_LIMIT,
        operational_input_limit: CALIBRATED_OPERATIONAL_INPUT_LIMIT,
        auto_compact_token_limit: CALIBRATED_AUTO_COMPACT_LIMIT,
        probe_stages: probe_stages(),
        global_config_changed: false,
    })
}

fn probe_stages() -> Vec<ProbeStage> {
    [
        ("above_legacy_cap", 300_000),
        ("mid_window", 600_000),
        ("near_operational_limit", CALIBRATED_OPERATIONAL_INPUT_LIMIT),
    ]
    .into_iter()
    .map(|(id, target_input_tokens)| ProbeStage {
        id: id.to_owned(),
        target_input_tokens,
        canaries: ["early", "middle", "late"]
            .into_iter()
            .map(|position| format!("cctx-sol-1m-{id}-{position}-v1"))
            .collect(),
    })
    .collect()
}

/// Ask Codex itself to resolve the prepared catalog and validate its exact policy.
pub fn verify(
    codex_command: &str,
    prepared: &PreparedSolCatalog,
) -> Result<ResolvedCatalogPolicy, SolLaunchError> {
    let arguments = catalog_override_arguments(&prepared.catalog_path);
    let output = Command::new(platform_command(codex_command))
        .args(&arguments)
        .args(["debug", "models"])
        .stdin(Stdio::null())
        .output()
        .map_err(|error| SolLaunchError::new(format!("could not run Codex verifier: {error}")))?;
    if !output.status.success() {
        return Err(SolLaunchError::new(format!(
            "Codex catalog verification failed with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    let parsed = ParsedCatalog::parse(&output.stdout, &prepared.codex_version)
        .map_err(|error| SolLaunchError::new(error.to_string()))?;
    let policy = parsed
        .resolved_policy()
        .map_err(|error| SolLaunchError::new(error.to_string()))?;
    let expected = ResolvedCatalogPolicy {
        model: REQUIRED_MODEL.to_owned(),
        context_window: OFFICIAL_TOTAL_CONTEXT,
        max_context_window: OFFICIAL_TOTAL_CONTEXT,
        effective_context_window_percent: CALIBRATED_EFFECTIVE_PERCENT,
        auto_compact_token_limit: Some(CALIBRATED_AUTO_COMPACT_LIMIT),
    };
    if policy != expected {
        return Err(SolLaunchError::new(format!(
            "Codex resolved an unexpected catalog policy: {policy:?}"
        )));
    }
    Ok(policy)
}

/// Launch Codex with the prepared catalog and no global configuration mutation.
pub fn launch(
    codex_command: &str,
    prepared: &PreparedSolCatalog,
    codex_arguments: &[String],
) -> Result<ExitStatus, SolLaunchError> {
    let arguments = catalog_override_arguments(&prepared.catalog_path);
    eprintln!("{}", render_meter(&meter(0)).trim_end());
    eprintln!(
        "Codex's footer tracks live context remaining; use /status or /statusline inside the launched TUI."
    );
    Command::new(platform_command(codex_command))
        .args(&arguments)
        .args(codex_arguments)
        .status()
        .map_err(|error| SolLaunchError::new(format!("could not launch Codex: {error}")))
}

fn catalog_override_arguments(catalog_path: &Path) -> Vec<String> {
    vec![
        "-c".to_owned(),
        format!(
            "model_catalog_json={}",
            toml_edit::Value::from(catalog_path.to_string_lossy().as_ref())
        ),
        "-c".to_owned(),
        format!("model={}", toml_edit::Value::from(REQUIRED_MODEL)),
        "-c".to_owned(),
        format!("model_context_window={OFFICIAL_TOTAL_CONTEXT}"),
        "-c".to_owned(),
        format!("model_auto_compact_token_limit={CALIBRATED_AUTO_COMPACT_LIMIT}"),
        "-c".to_owned(),
        "model_auto_compact_token_limit_scope=\"total\"".to_owned(),
        "-c".to_owned(),
        "tui.status_line=[\"model\",\"context-remaining\",\"current-dir\"]".to_owned(),
    ]
}

fn platform_command(command: &str) -> String {
    if cfg!(windows) && command.eq_ignore_ascii_case("codex") {
        "codex.cmd".to_owned()
    } else {
        command.to_owned()
    }
}

fn validate_state_dir(state_dir: &Path) -> Result<(), SolLaunchError> {
    if !state_dir.is_absolute() {
        return Err(SolLaunchError::new(
            "Sol launcher state directory must be an explicit absolute path",
        ));
    }
    reject_symlink(state_dir, "launcher state directory")
}

fn reject_symlink(path: &Path, label: &str) -> Result<(), SolLaunchError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(SolLaunchError::new(format!(
            "refusing {label} symlink: {}",
            path.display()
        ))),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error("inspect path metadata", path, error)),
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), SolLaunchError> {
    reject_symlink(path, "launcher artifact")?;
    let mut file = AtomicWriteFile::open(path)
        .map_err(|error| io_error("open launcher artifact", path, error))?;
    file.write_all(bytes)
        .map_err(|error| io_error("write launcher artifact", path, error))?;
    file.commit()
        .map_err(|error| io_error("commit launcher artifact", path, error))
}

fn io_error(action: &str, path: &Path, error: std::io::Error) -> SolLaunchError {
    SolLaunchError::new(format!("could not {action} {}: {error}", path.display()))
}

/// Fail-closed launcher error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolLaunchError(String);

impl SolLaunchError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for SolLaunchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for SolLaunchError {}

#[cfg(test)]
mod tests {
    use super::{MeterPhase, catalog_override_arguments, meter, probe_stages, render_meter};
    use std::path::Path;

    #[test]
    fn arguments_are_sol_only_and_carry_the_frozen_policy() {
        let arguments = catalog_override_arguments(Path::new("C:\\safe path\\models.json"));
        let rendered = arguments.join(" ");
        assert!(rendered.contains("gpt-5.6-sol"));
        assert!(rendered.contains("1050000"));
        assert!(rendered.contains("900000"));
        assert!(rendered.contains("model_catalog_json"));
        assert!(rendered.contains("context-remaining"));
        assert!(!rendered.contains("terra"));
        assert!(!rendered.contains("luna"));
    }

    #[test]
    fn probe_plan_is_progressive_deterministic_and_below_compaction() {
        let first = probe_stages();
        assert_eq!(first, probe_stages());
        assert_eq!(
            first
                .iter()
                .map(|stage| stage.target_input_tokens)
                .collect::<Vec<_>>(),
            [300_000, 600_000, 880_000]
        );
        assert!(first.iter().all(|stage| stage.canaries.len() == 3));
        assert!(
            first
                .iter()
                .all(|stage| stage.target_input_tokens < 900_000)
        );
    }

    #[test]
    fn meter_changes_phase_at_each_frozen_boundary() {
        assert_eq!(meter(839_999).phase, MeterPhase::Normal);
        assert_eq!(meter(840_000).phase, MeterPhase::Checkpoint);
        assert_eq!(meter(860_000).phase, MeterPhase::Rollover);
        assert_eq!(meter(880_000).phase, MeterPhase::AdmissionBlocked);
        assert_eq!(meter(900_000).phase, MeterPhase::CompactionBoundary);
        let rendered = render_meter(&meter(600_000));
        assert!(rendered.contains("gpt-5.6-sol"));
        assert!(rendered.contains("checkpoint=840000"));
        assert!(rendered.contains("native=1050000"));
        assert!(rendered.contains("effective=1008000"));
    }
}
