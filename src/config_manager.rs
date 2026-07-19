//! Reversible, ownership-aware management of Context Continuum Codex settings.

use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use atomic_write_file::AtomicWriteFile;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use toml_edit::{Array, DocumentMut, Item, Table, TableLike, value};

use crate::{OFFICIAL_TOTAL_CONTEXT, REQUIRED_MODEL};

/// Version of the ownership manifest emitted by this module.
pub const OWNERSHIP_SCHEMA_VERSION: u32 = 1;

const OWNERSHIP_FILE: &str = "ownership.json";
const LOCK_FILE: &str = "config-manager.lock";
const BACKUP_DIR: &str = "backups";
const SNAPSHOT_DIR: &str = "snapshots";
const MCP_SERVER_ID: &str = "context_continuum";
const MAX_AUTO_COMPACT_LIMIT: u64 = OFFICIAL_TOTAL_CONTEXT * 9 / 10;

const OWNED_PATHS: [&str; 13] = [
    "model",
    "model_context_window",
    "model_auto_compact_token_limit",
    "model_auto_compact_token_limit_scope",
    "model_catalog_json",
    "features.hooks",
    "features.plugins",
    "mcp_servers.context_continuum.command",
    "mcp_servers.context_continuum.args",
    "mcp_servers.context_continuum.enabled",
    "mcp_servers.context_continuum.required",
    "mcp_servers.context_continuum.startup_timeout_sec",
    "mcp_servers.context_continuum",
];

const DIFF_PATHS: [(&str, &[&str]); 12] = [
    ("model", &["model"]),
    ("model_context_window", &["model_context_window"]),
    (
        "model_auto_compact_token_limit",
        &["model_auto_compact_token_limit"],
    ),
    (
        "model_auto_compact_token_limit_scope",
        &["model_auto_compact_token_limit_scope"],
    ),
    ("model_catalog_json", &["model_catalog_json"]),
    ("features.hooks", &["features", "hooks"]),
    ("features.plugins", &["features", "plugins"]),
    (
        "mcp_servers.context_continuum.command",
        &["mcp_servers", MCP_SERVER_ID, "command"],
    ),
    (
        "mcp_servers.context_continuum.args",
        &["mcp_servers", MCP_SERVER_ID, "args"],
    ),
    (
        "mcp_servers.context_continuum.enabled",
        &["mcp_servers", MCP_SERVER_ID, "enabled"],
    ),
    (
        "mcp_servers.context_continuum.required",
        &["mcp_servers", MCP_SERVER_ID, "required"],
    ),
    (
        "mcp_servers.context_continuum.startup_timeout_sec",
        &["mcp_servers", MCP_SERVER_ID, "startup_timeout_sec"],
    ),
];

/// Scope accepted by Codex for an explicit automatic-compaction limit.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AutoCompactScope {
    Total,
    BodyAfterPrefix,
}

impl AutoCompactScope {
    fn as_str(self) -> &'static str {
        match self {
            Self::Total => "total",
            Self::BodyAfterPrefix => "body_after_prefix",
        }
    }
}

/// Context Continuum-owned MCP server settings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OwnedMcpConfig {
    pub command: String,
    pub args: Vec<String>,
    pub enabled: bool,
    pub required: bool,
    pub startup_timeout_sec: Option<u64>,
}

/// Exact Codex settings owned by one Context Continuum installation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OwnedConfig {
    pub model: String,
    pub model_context_window: u64,
    pub model_auto_compact_token_limit: Option<u64>,
    pub model_auto_compact_token_limit_scope: Option<AutoCompactScope>,
    pub model_catalog_json: String,
    pub hooks_enabled: bool,
    pub plugins_enabled: bool,
    pub mcp: OwnedMcpConfig,
}

impl OwnedConfig {
    /// Build the uninstalled CAC-11 candidate for exact GPT-5.6 Sol.
    pub fn candidate(catalog_path: &Path, cctx_command: &Path) -> Self {
        Self {
            model: REQUIRED_MODEL.to_owned(),
            model_context_window: OFFICIAL_TOTAL_CONTEXT,
            model_auto_compact_token_limit: None,
            model_auto_compact_token_limit_scope: None,
            model_catalog_json: catalog_path.to_string_lossy().into_owned(),
            hooks_enabled: true,
            plugins_enabled: true,
            mcp: OwnedMcpConfig {
                command: cctx_command.to_string_lossy().into_owned(),
                args: vec!["mcp".to_owned(), "serve".to_owned()],
                enabled: true,
                required: true,
                startup_timeout_sec: Some(10),
            },
        }
    }

    /// Validate the Sol-only and path-safety contract before a plan is created.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.model != REQUIRED_MODEL {
            return Err(ConfigError::new(format!(
                "config model must be exact `{REQUIRED_MODEL}`"
            )));
        }
        if self.model_context_window != OFFICIAL_TOTAL_CONTEXT {
            return Err(ConfigError::new(format!(
                "config context window must equal {OFFICIAL_TOTAL_CONTEXT}"
            )));
        }
        match (
            self.model_auto_compact_token_limit,
            self.model_auto_compact_token_limit_scope,
        ) {
            (None, None) => {}
            (Some(limit), Some(_)) if (1..=MAX_AUTO_COMPACT_LIMIT).contains(&limit) => {}
            (Some(_), None) => {
                return Err(ConfigError::new(
                    "an auto-compaction limit requires an explicit scope",
                ));
            }
            (None, Some(_)) => {
                return Err(ConfigError::new(
                    "an auto-compaction scope requires an explicit limit",
                ));
            }
            (Some(_), Some(_)) => {
                return Err(ConfigError::new(format!(
                    "auto-compaction limit must be within 1..={MAX_AUTO_COMPACT_LIMIT}"
                )));
            }
        }
        validate_absolute_value("model_catalog_json", &self.model_catalog_json)?;
        if !self.hooks_enabled || !self.plugins_enabled {
            return Err(ConfigError::new(
                "Context Continuum requires both hooks and plugins to be enabled",
            ));
        }
        validate_absolute_value("MCP command", &self.mcp.command)?;
        if self.mcp.args != ["mcp", "serve"] {
            return Err(ConfigError::new(
                "Context Continuum MCP args must be exactly `mcp serve`",
            ));
        }
        if !self.mcp.enabled || !self.mcp.required {
            return Err(ConfigError::new(
                "Context Continuum MCP must be enabled and required",
            ));
        }
        if self.mcp.startup_timeout_sec == Some(0) {
            return Err(ConfigError::new(
                "MCP startup timeout must be positive when present",
            ));
        }
        for argument in &self.mcp.args {
            validate_scalar("MCP argument", argument)?;
        }
        Ok(())
    }
}

/// One safe, owned-field-only entry in a dry-run diff.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConfigChange {
    pub path: String,
    pub before: Option<String>,
    pub after: Option<String>,
}

/// Sanitized plan summary. It never contains unrelated user configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DryRunDiff {
    pub schema_version: u32,
    pub config_path: String,
    pub config_existed: bool,
    pub observed_sha256: Option<String>,
    pub candidate_sha256: String,
    pub managed_update: bool,
    pub changes: Vec<ConfigChange>,
}

/// Opaque plan carrying the exact observed bytes required by `apply`.
#[derive(Debug, Clone)]
pub struct ConfigPlan {
    config_path: PathBuf,
    state_dir: PathBuf,
    observed_exists: bool,
    observed_bytes: Vec<u8>,
    observed_manifest_bytes: Option<Vec<u8>>,
    candidate_bytes: Vec<u8>,
    desired: OwnedConfig,
    active_manifest: Option<OwnershipManifest>,
    diff: DryRunDiff,
}

impl ConfigPlan {
    /// Return the safe dry-run representation of this plan.
    pub fn diff(&self) -> &DryRunDiff {
        &self.diff
    }
}

/// Lifecycle state recorded before and after an atomic configuration write.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ManifestState {
    PendingInstall,
    Installed,
    Uninstalled,
}

/// Durable ownership record used for conflict detection and exact rollback.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OwnershipManifest {
    pub schema_version: u32,
    pub state: ManifestState,
    pub created_at: String,
    pub updated_at: String,
    pub uninstalled_at: Option<String>,
    pub config_path: String,
    pub config_existed_before: bool,
    pub backup_file: Option<String>,
    pub pre_install_sha256: Option<String>,
    pub installed_snapshot_file: String,
    pub installed_sha256: String,
    pub desired: OwnedConfig,
    pub owned_paths: Vec<String>,
}

/// Result of an apply operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApplyOutcome {
    pub changed: bool,
    pub config_path: String,
    pub installed_sha256: String,
    pub manifest_path: String,
}

/// Result of a restore or uninstall operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RestoreOutcome {
    pub restored_original: bool,
    pub removed_installed_file: bool,
    pub config_path: String,
    pub manifest_path: String,
}

/// Config manager failure with a fail-closed, user-readable reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    message: String,
}

impl ConfigError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ConfigError {}

/// Create a read-only plan without creating directories or changing files.
pub fn plan(
    config_path: &Path,
    state_dir: &Path,
    desired: OwnedConfig,
) -> Result<ConfigPlan, ConfigError> {
    validate_explicit_paths(config_path, state_dir)?;
    desired.validate()?;
    reject_symlink_if_present(config_path, "config file")?;
    reject_symlink_if_present(state_dir, "state directory")?;

    let (observed_exists, observed_bytes) = read_optional(config_path)?;
    let mut document = parse_config(&observed_bytes)?;
    let (manifest_exists, manifest_bytes) = read_optional(&state_dir.join(OWNERSHIP_FILE))?;
    let observed_manifest_bytes = manifest_exists.then_some(manifest_bytes);
    let manifest = observed_manifest_bytes
        .as_deref()
        .map(parse_manifest)
        .transpose()?;
    let active_manifest = validate_plan_ownership(
        manifest,
        config_path,
        state_dir,
        observed_exists,
        &observed_bytes,
    )?;

    if active_manifest.is_none()
        && item_at(document.as_item(), &["mcp_servers", MCP_SERVER_ID]).is_some()
    {
        return Err(ConfigError::new(format!(
            "ownership conflict: MCP server `{MCP_SERVER_ID}` already exists without an active Context Continuum manifest"
        )));
    }

    let before = document.clone();
    apply_owned_values(&mut document, &desired)?;
    let candidate_bytes = document.to_string().into_bytes();
    let changes = DIFF_PATHS
        .iter()
        .filter_map(|(label, path)| {
            let before_value = item_at(before.as_item(), path).map(render_item);
            let after_value = item_at(document.as_item(), path).map(render_item);
            (before_value != after_value).then(|| ConfigChange {
                path: (*label).to_owned(),
                before: before_value,
                after: after_value,
            })
        })
        .collect();

    let diff = DryRunDiff {
        schema_version: OWNERSHIP_SCHEMA_VERSION,
        config_path: display_path(config_path),
        config_existed: observed_exists,
        observed_sha256: observed_exists.then(|| sha256_hex(&observed_bytes)),
        candidate_sha256: sha256_hex(&candidate_bytes),
        managed_update: active_manifest.is_some(),
        changes,
    };

    Ok(ConfigPlan {
        config_path: config_path.to_path_buf(),
        state_dir: state_dir.to_path_buf(),
        observed_exists,
        observed_bytes,
        observed_manifest_bytes,
        candidate_bytes,
        desired,
        active_manifest,
        diff,
    })
}

/// Apply a previously created plan after rechecking all observed bytes under a lock.
pub fn apply(plan: &ConfigPlan) -> Result<ApplyOutcome, ConfigError> {
    ensure_parent(&plan.config_path)?;
    fs::create_dir_all(&plan.state_dir)
        .map_err(|error| io_error("create state directory", &plan.state_dir, error))?;
    reject_symlink_if_present(&plan.config_path, "config file")?;
    reject_symlink_if_present(&plan.state_dir, "state directory")?;
    let _lock = acquire_lock(&plan.state_dir)?;

    let (current_exists, current_bytes) = read_optional(&plan.config_path)?;
    if current_exists != plan.observed_exists || current_bytes != plan.observed_bytes {
        return Err(ConfigError::new(
            "concurrent config edit detected after planning; refusing to apply",
        ));
    }
    let manifest_path = plan.state_dir.join(OWNERSHIP_FILE);
    let (current_manifest_exists, current_manifest_bytes) = read_optional(&manifest_path)?;
    let expected_manifest_exists = plan.observed_manifest_bytes.is_some();
    if current_manifest_exists != expected_manifest_exists
        || plan
            .observed_manifest_bytes
            .as_deref()
            .is_some_and(|expected| expected != current_manifest_bytes)
    {
        return Err(ConfigError::new(
            "concurrent ownership-manifest edit detected after planning; refusing to apply",
        ));
    }
    let current_manifest = current_manifest_exists
        .then(|| parse_manifest(&current_manifest_bytes))
        .transpose()?;
    let active_now = validate_plan_ownership(
        current_manifest,
        &plan.config_path,
        &plan.state_dir,
        current_exists,
        &current_bytes,
    )?;
    if active_now.is_some() != plan.active_manifest.is_some() {
        return Err(ConfigError::new(
            "ownership lifecycle changed after planning; refusing to apply",
        ));
    }

    if plan.candidate_bytes == current_bytes
        && let Some(manifest) = &plan.active_manifest
    {
        return Ok(ApplyOutcome {
            changed: false,
            config_path: display_path(&plan.config_path),
            installed_sha256: manifest.installed_sha256.clone(),
            manifest_path: display_path(&manifest_path),
        });
    }

    let stamp = timestamp()?;
    let backup_dir = ensure_state_subdir(&plan.state_dir, BACKUP_DIR)?;
    let snapshot_dir = ensure_state_subdir(&plan.state_dir, SNAPSHOT_DIR)?;

    let installed_sha256 = sha256_hex(&plan.candidate_bytes);
    let snapshot_file = format!("installed-{stamp}-{}.toml", &installed_sha256[..12]);
    write_new_state_blob(&snapshot_dir.join(&snapshot_file), &plan.candidate_bytes)?;

    let (created_at, config_existed_before, backup_file, pre_install_sha256) =
        if let Some(previous) = &plan.active_manifest {
            (
                previous.created_at.clone(),
                previous.config_existed_before,
                previous.backup_file.clone(),
                previous.pre_install_sha256.clone(),
            )
        } else if plan.observed_exists {
            let pre_hash = sha256_hex(&plan.observed_bytes);
            let backup_file = format!("original-{stamp}-{}.toml", &pre_hash[..12]);
            write_new_state_blob(&backup_dir.join(&backup_file), &plan.observed_bytes)?;
            (stamp.clone(), true, Some(backup_file), Some(pre_hash))
        } else {
            (stamp.clone(), false, None, None)
        };

    let mut manifest = OwnershipManifest {
        schema_version: OWNERSHIP_SCHEMA_VERSION,
        state: ManifestState::PendingInstall,
        created_at,
        updated_at: stamp,
        uninstalled_at: None,
        config_path: display_path(&plan.config_path),
        config_existed_before,
        backup_file,
        pre_install_sha256,
        installed_snapshot_file: snapshot_file,
        installed_sha256: installed_sha256.clone(),
        desired: plan.desired.clone(),
        owned_paths: OWNED_PATHS.iter().map(|path| (*path).to_owned()).collect(),
    };
    validate_manifest(&manifest)?;
    atomic_write_json(&manifest_path, &manifest)?;
    atomic_write(&plan.config_path, &plan.candidate_bytes)?;
    manifest.state = ManifestState::Installed;
    atomic_write_json(&manifest_path, &manifest)?;

    Ok(ApplyOutcome {
        changed: true,
        config_path: display_path(&plan.config_path),
        installed_sha256,
        manifest_path: display_path(&manifest_path),
    })
}

/// Restore the exact pre-install bytes, refusing any config changed after apply.
pub fn restore(config_path: &Path, state_dir: &Path) -> Result<RestoreOutcome, ConfigError> {
    validate_explicit_paths(config_path, state_dir)?;
    reject_symlink_if_present(config_path, "config file")?;
    reject_symlink_if_present(state_dir, "state directory")?;
    if !state_dir.is_dir() {
        return Err(ConfigError::new(format!(
            "state directory does not exist: {}",
            state_dir.display()
        )));
    }
    let _lock = acquire_lock(state_dir)?;
    let manifest_path = state_dir.join(OWNERSHIP_FILE);
    let manifest_bytes = fs::read(&manifest_path)
        .map_err(|error| io_error("read ownership manifest", &manifest_path, error))?;
    let mut manifest = parse_manifest(&manifest_bytes)?;
    validate_manifest(&manifest)?;
    if manifest.state != ManifestState::Installed {
        return Err(ConfigError::new(match manifest.state {
            ManifestState::PendingInstall => {
                "ownership manifest is pending; manual recovery is required before restore"
            }
            ManifestState::Uninstalled => "installation is already uninstalled",
            ManifestState::Installed => unreachable!(),
        }));
    }
    if Path::new(&manifest.config_path) != config_path {
        return Err(ConfigError::new(
            "ownership manifest belongs to a different config path",
        ));
    }

    let snapshot_path =
        checked_state_file(state_dir, SNAPSHOT_DIR, &manifest.installed_snapshot_file)?;
    reject_symlink_if_present(&snapshot_path, "installed snapshot")?;
    let snapshot = fs::read(&snapshot_path)
        .map_err(|error| io_error("read installed snapshot", &snapshot_path, error))?;
    if sha256_hex(&snapshot) != manifest.installed_sha256 {
        return Err(ConfigError::new(
            "installed snapshot hash does not match ownership manifest",
        ));
    }
    let (current_exists, current_bytes) = read_optional(config_path)?;
    if !current_exists
        || current_bytes != snapshot
        || sha256_hex(&current_bytes) != manifest.installed_sha256
    {
        return Err(ConfigError::new(
            "config changed after Context Continuum apply; refusing to overwrite user changes",
        ));
    }

    let (restored_original, removed_installed_file) = if manifest.config_existed_before {
        let backup_file = manifest.backup_file.as_deref().ok_or_else(|| {
            ConfigError::new("manifest is missing the required original backup filename")
        })?;
        let expected_hash = manifest
            .pre_install_sha256
            .as_deref()
            .ok_or_else(|| ConfigError::new("manifest is missing the required pre-install hash"))?;
        let backup_path = checked_state_file(state_dir, BACKUP_DIR, backup_file)?;
        reject_symlink_if_present(&backup_path, "original backup")?;
        let original = fs::read(&backup_path)
            .map_err(|error| io_error("read original backup", &backup_path, error))?;
        if sha256_hex(&original) != expected_hash {
            return Err(ConfigError::new(
                "original backup hash does not match ownership manifest",
            ));
        }
        atomic_write(config_path, &original)?;
        (true, false)
    } else {
        if manifest.backup_file.is_some() || manifest.pre_install_sha256.is_some() {
            return Err(ConfigError::new(
                "manifest for a newly created config must not name an original backup",
            ));
        }
        fs::remove_file(config_path)
            .map_err(|error| io_error("remove installed config", config_path, error))?;
        (false, true)
    };

    let stamp = timestamp()?;
    manifest.state = ManifestState::Uninstalled;
    manifest.updated_at = stamp.clone();
    manifest.uninstalled_at = Some(stamp);
    atomic_write_json(&manifest_path, &manifest)?;

    Ok(RestoreOutcome {
        restored_original,
        removed_installed_file,
        config_path: display_path(config_path),
        manifest_path: display_path(&manifest_path),
    })
}

/// Uninstall is an explicit alias for exact-byte restore.
pub fn uninstall(config_path: &Path, state_dir: &Path) -> Result<RestoreOutcome, ConfigError> {
    restore(config_path, state_dir)
}

fn validate_plan_ownership(
    manifest: Option<OwnershipManifest>,
    config_path: &Path,
    state_dir: &Path,
    config_exists: bool,
    config_bytes: &[u8],
) -> Result<Option<OwnershipManifest>, ConfigError> {
    let Some(manifest) = manifest else {
        return Ok(None);
    };
    validate_manifest(&manifest)?;
    match manifest.state {
        ManifestState::PendingInstall => Err(ConfigError::new(
            "ownership manifest is pending; manual recovery is required before planning",
        )),
        ManifestState::Uninstalled => Ok(None),
        ManifestState::Installed => {
            if Path::new(&manifest.config_path) != config_path {
                return Err(ConfigError::new(
                    "ownership manifest belongs to a different config path",
                ));
            }
            if !config_exists {
                return Err(ConfigError::new(
                    "managed config is missing; refusing to recreate it implicitly",
                ));
            }
            let snapshot_path =
                checked_state_file(state_dir, SNAPSHOT_DIR, &manifest.installed_snapshot_file)?;
            reject_symlink_if_present(&snapshot_path, "installed snapshot")?;
            let snapshot = fs::read(&snapshot_path)
                .map_err(|error| io_error("read installed snapshot", &snapshot_path, error))?;
            if sha256_hex(&snapshot) != manifest.installed_sha256
                || snapshot != config_bytes
                || sha256_hex(config_bytes) != manifest.installed_sha256
            {
                return Err(ConfigError::new(
                    "managed config or installed snapshot changed; refusing an implicit update",
                ));
            }
            Ok(Some(manifest))
        }
    }
}

fn validate_manifest(manifest: &OwnershipManifest) -> Result<(), ConfigError> {
    if manifest.schema_version != OWNERSHIP_SCHEMA_VERSION {
        return Err(ConfigError::new(format!(
            "unsupported ownership manifest schema {}",
            manifest.schema_version
        )));
    }
    manifest.desired.validate()?;
    validate_scalar("ownership manifest config path", &manifest.config_path)?;
    if !Path::new(&manifest.config_path).is_absolute() {
        return Err(ConfigError::new(
            "ownership manifest config path must be absolute",
        ));
    }
    validate_timestamp("manifest created_at", &manifest.created_at)?;
    validate_timestamp("manifest updated_at", &manifest.updated_at)?;
    if let Some(value) = &manifest.uninstalled_at {
        validate_timestamp("manifest uninstalled_at", value)?;
    }
    if (manifest.state == ManifestState::Uninstalled) != manifest.uninstalled_at.is_some() {
        return Err(ConfigError::new(
            "manifest uninstalled timestamp does not match lifecycle state",
        ));
    }
    validate_hash("installed_sha256", &manifest.installed_sha256)?;
    validate_artifact_filename(
        "installed snapshot",
        &manifest.installed_snapshot_file,
        "installed-",
        &manifest.installed_sha256,
    )?;
    if let Some(hash) = &manifest.pre_install_sha256 {
        validate_hash("pre_install_sha256", hash)?;
    }
    if manifest.config_existed_before
        != (manifest.backup_file.is_some() && manifest.pre_install_sha256.is_some())
    {
        return Err(ConfigError::new(
            "manifest backup fields do not match whether config existed before install",
        ));
    }
    if let (Some(filename), Some(hash)) = (&manifest.backup_file, &manifest.pre_install_sha256) {
        validate_artifact_filename("original backup", filename, "original-", hash)?;
    }
    let expected = OWNED_PATHS.to_vec();
    let actual = manifest
        .owned_paths
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    if actual != expected {
        return Err(ConfigError::new(
            "manifest owned-path set is not the exact reviewed CAC-11 set",
        ));
    }
    Ok(())
}

fn apply_owned_values(
    document: &mut DocumentMut,
    desired: &OwnedConfig,
) -> Result<(), ConfigError> {
    set_path(document, &["model"], value(desired.model.as_str()))?;
    set_path(
        document,
        &["model_context_window"],
        value(to_i64(
            "model_context_window",
            desired.model_context_window,
        )?),
    )?;
    match desired.model_auto_compact_token_limit {
        Some(limit) => set_path(
            document,
            &["model_auto_compact_token_limit"],
            value(to_i64("model_auto_compact_token_limit", limit)?),
        )?,
        None => remove_path(document, &["model_auto_compact_token_limit"])?,
    }
    match desired.model_auto_compact_token_limit_scope {
        Some(scope) => set_path(
            document,
            &["model_auto_compact_token_limit_scope"],
            value(scope.as_str()),
        )?,
        None => remove_path(document, &["model_auto_compact_token_limit_scope"])?,
    }
    set_path(
        document,
        &["model_catalog_json"],
        value(desired.model_catalog_json.as_str()),
    )?;
    set_path(
        document,
        &["features", "hooks"],
        value(desired.hooks_enabled),
    )?;
    set_path(
        document,
        &["features", "plugins"],
        value(desired.plugins_enabled),
    )?;
    set_path(
        document,
        &["mcp_servers", MCP_SERVER_ID, "command"],
        value(desired.mcp.command.as_str()),
    )?;
    let mut arguments = Array::new();
    for argument in &desired.mcp.args {
        arguments.push(argument.as_str());
    }
    set_path(
        document,
        &["mcp_servers", MCP_SERVER_ID, "args"],
        value(arguments),
    )?;
    set_path(
        document,
        &["mcp_servers", MCP_SERVER_ID, "enabled"],
        value(desired.mcp.enabled),
    )?;
    set_path(
        document,
        &["mcp_servers", MCP_SERVER_ID, "required"],
        value(desired.mcp.required),
    )?;
    match desired.mcp.startup_timeout_sec {
        Some(timeout) => set_path(
            document,
            &["mcp_servers", MCP_SERVER_ID, "startup_timeout_sec"],
            value(to_i64("MCP startup timeout", timeout)?),
        )?,
        None => remove_path(
            document,
            &["mcp_servers", MCP_SERVER_ID, "startup_timeout_sec"],
        )?,
    }
    Ok(())
}

fn set_path(document: &mut DocumentMut, path: &[&str], item: Item) -> Result<(), ConfigError> {
    let (leaf, parents) = path
        .split_last()
        .ok_or_else(|| ConfigError::new("owned config path must not be empty"))?;
    let mut table: &mut dyn TableLike = document.as_table_mut();
    for segment in parents {
        if !table.contains_key(segment) {
            table.insert(segment, Item::Table(Table::new()));
        }
        let next = table
            .get_mut(segment)
            .and_then(Item::as_table_like_mut)
            .ok_or_else(|| {
                ConfigError::new(format!(
                    "cannot set `{}` because `{segment}` is not a table",
                    path.join(".")
                ))
            })?;
        table = next;
    }
    table.insert(leaf, item);
    Ok(())
}

fn remove_path(document: &mut DocumentMut, path: &[&str]) -> Result<(), ConfigError> {
    let (leaf, parents) = path
        .split_last()
        .ok_or_else(|| ConfigError::new("owned config path must not be empty"))?;
    let mut table: &mut dyn TableLike = document.as_table_mut();
    for segment in parents {
        let Some(item) = table.get_mut(segment) else {
            return Ok(());
        };
        table = item.as_table_like_mut().ok_or_else(|| {
            ConfigError::new(format!(
                "cannot remove `{}` because `{segment}` is not a table",
                path.join(".")
            ))
        })?;
    }
    table.remove(leaf);
    Ok(())
}

fn item_at<'a>(root: &'a Item, path: &[&str]) -> Option<&'a Item> {
    let mut item = root;
    for segment in path {
        item = item.get(segment)?;
    }
    Some(item)
}

fn render_item(item: &Item) -> String {
    item.to_string().trim().to_owned()
}

fn parse_config(bytes: &[u8]) -> Result<DocumentMut, ConfigError> {
    if bytes.is_empty() {
        return Ok(DocumentMut::new());
    }
    let text = std::str::from_utf8(bytes)
        .map_err(|error| ConfigError::new(format!("Codex config is not UTF-8: {error}")))?;
    text.parse::<DocumentMut>()
        .map_err(|error| ConfigError::new(format!("Codex config is invalid TOML: {error}")))
}

fn parse_manifest(bytes: &[u8]) -> Result<OwnershipManifest, ConfigError> {
    serde_json::from_slice(bytes)
        .map_err(|error| ConfigError::new(format!("ownership manifest is invalid JSON: {error}")))
}

fn read_optional(path: &Path) -> Result<(bool, Vec<u8>), ConfigError> {
    match fs::read(path) {
        Ok(bytes) => Ok((true, bytes)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok((false, Vec::new())),
        Err(error) => Err(io_error("read", path, error)),
    }
}

fn acquire_lock(state_dir: &Path) -> Result<File, ConfigError> {
    let lock_path = state_dir.join(LOCK_FILE);
    reject_symlink_if_present(&lock_path, "config-manager lock")?;
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(|error| io_error("open config-manager lock", &lock_path, error))?;
    lock.try_lock().map_err(|error| {
        ConfigError::new(format!(
            "another config-manager operation holds {}: {error}",
            lock_path.display()
        ))
    })?;
    Ok(lock)
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), ConfigError> {
    reject_symlink_if_present(path, "atomic-write target")?;
    let mut file = AtomicWriteFile::open(path)
        .map_err(|error| io_error("open atomic-write target", path, error))?;
    file.write_all(bytes)
        .map_err(|error| io_error("write atomic replacement", path, error))?;
    file.commit()
        .map_err(|error| io_error("commit atomic replacement", path, error))
}

fn atomic_write_json(path: &Path, value: &OwnershipManifest) -> Result<(), ConfigError> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| {
        ConfigError::new(format!("could not serialize ownership manifest: {error}"))
    })?;
    bytes.push(b'\n');
    atomic_write(path, &bytes)
}

fn write_new_state_blob(path: &Path, bytes: &[u8]) -> Result<(), ConfigError> {
    if path.exists() {
        return Err(ConfigError::new(format!(
            "refusing to replace existing state artifact {}",
            path.display()
        )));
    }
    atomic_write(path, bytes)
}

fn ensure_parent(path: &Path) -> Result<(), ConfigError> {
    let parent = path.parent().ok_or_else(|| {
        ConfigError::new(format!("config path has no parent: {}", path.display()))
    })?;
    fs::create_dir_all(parent).map_err(|error| io_error("create config parent", parent, error))
}

fn ensure_state_subdir(state_dir: &Path, name: &str) -> Result<PathBuf, ConfigError> {
    let path = state_dir.join(name);
    reject_symlink_if_present(&path, "state subdirectory")?;
    fs::create_dir_all(&path)
        .map_err(|error| io_error("create state subdirectory", &path, error))?;
    reject_symlink_if_present(&path, "state subdirectory")?;
    Ok(path)
}

fn checked_state_file(
    state_dir: &Path,
    category: &str,
    filename: &str,
) -> Result<PathBuf, ConfigError> {
    validate_state_filename(filename)?;
    let category_path = state_dir.join(category);
    reject_symlink_if_present(&category_path, "state subdirectory")?;
    Ok(category_path.join(filename))
}

fn validate_explicit_paths(config_path: &Path, state_dir: &Path) -> Result<(), ConfigError> {
    if !config_path.is_absolute() || !state_dir.is_absolute() {
        return Err(ConfigError::new(
            "config and state-directory paths must both be explicit absolute paths",
        ));
    }
    if config_path == state_dir || config_path.starts_with(state_dir) {
        return Err(ConfigError::new(
            "config file must not be inside the private ownership state directory",
        ));
    }
    Ok(())
}

fn validate_absolute_value(label: &str, value: &str) -> Result<(), ConfigError> {
    validate_scalar(label, value)?;
    if !Path::new(value).is_absolute() {
        return Err(ConfigError::new(format!(
            "{label} must be an absolute path"
        )));
    }
    Ok(())
}

fn validate_scalar(label: &str, value: &str) -> Result<(), ConfigError> {
    if value.is_empty() || value.contains(['\0', '\r', '\n']) {
        return Err(ConfigError::new(format!(
            "{label} must be a nonempty single-line value"
        )));
    }
    Ok(())
}

fn validate_state_filename(filename: &str) -> Result<(), ConfigError> {
    validate_scalar("state filename", filename)?;
    let path = Path::new(filename);
    if path.components().count() != 1 || filename == "." || filename == ".." {
        return Err(ConfigError::new(
            "state artifact filename must be one safe path component",
        ));
    }
    Ok(())
}

fn validate_artifact_filename(
    label: &str,
    filename: &str,
    prefix: &str,
    full_hash: &str,
) -> Result<(), ConfigError> {
    validate_state_filename(filename)?;
    let body = filename
        .strip_prefix(prefix)
        .and_then(|value| value.strip_suffix(".toml"))
        .ok_or_else(|| ConfigError::new(format!("{label} filename has an invalid shape")))?;
    let (stamp, hash_fragment) = body
        .rsplit_once('-')
        .ok_or_else(|| ConfigError::new(format!("{label} filename has no hash suffix")))?;
    validate_timestamp(&format!("{label} timestamp"), stamp)?;
    if hash_fragment != &full_hash[..12] {
        return Err(ConfigError::new(format!(
            "{label} filename hash does not match its manifest SHA-256"
        )));
    }
    Ok(())
}

fn validate_timestamp(label: &str, timestamp: &str) -> Result<(), ConfigError> {
    let Some(body) = timestamp.strip_prefix("unix-") else {
        return Err(ConfigError::new(format!(
            "{label} must use the unix-seconds-nanoseconds format"
        )));
    };
    let Some((seconds, nanos)) = body.split_once('-') else {
        return Err(ConfigError::new(format!(
            "{label} must use the unix-seconds-nanoseconds format"
        )));
    };
    if seconds.is_empty()
        || !seconds.bytes().all(|byte| byte.is_ascii_digit())
        || nanos.len() != 9
        || !nanos.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(ConfigError::new(format!(
            "{label} must use the unix-seconds-nanoseconds format"
        )));
    }
    Ok(())
}

fn validate_hash(label: &str, hash: &str) -> Result<(), ConfigError> {
    if hash.len() != 64
        || !hash
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(ConfigError::new(format!(
            "{label} must be a 64-character hexadecimal SHA-256"
        )));
    }
    Ok(())
}

fn reject_symlink_if_present(path: &Path, label: &str) -> Result<(), ConfigError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(ConfigError::new(format!(
            "refusing {label} symlink: {}",
            path.display()
        ))),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_error("inspect path metadata", path, error)),
    }
}

fn to_i64(label: &str, value: u64) -> Result<i64, ConfigError> {
    i64::try_from(value)
        .map_err(|_| ConfigError::new(format!("{label} exceeds TOML's signed integer range")))
}

fn timestamp() -> Result<String, ConfigError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| ConfigError::new(format!("system clock precedes Unix epoch: {error}")))?;
    Ok(format!(
        "unix-{}-{:09}",
        duration.as_secs(),
        duration.subsec_nanos()
    ))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn io_error(action: &str, path: &Path, error: io::Error) -> ConfigError {
    ConfigError::new(format!("could not {action} {}: {error}", path.display()))
}
