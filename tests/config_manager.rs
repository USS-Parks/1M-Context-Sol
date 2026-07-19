use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use context_continuum::config_manager::{
    AutoCompactScope, ManifestState, OwnedConfig, OwnershipManifest, apply, plan, restore,
    uninstall,
};
use serde_json::Value;

static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new(label: &str) -> Self {
        let sequence = NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "context-continuum-cac11-{label}-{}-{sequence}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn paths(root: &TestDir) -> (PathBuf, PathBuf, PathBuf, PathBuf) {
    let config = root.path().join("codex").join("config.toml");
    let state = root.path().join("continuum-state");
    let catalog = root.path().join("sol-catalog.json");
    let cctx = root
        .path()
        .join(if cfg!(windows) { "cctx.exe" } else { "cctx" });
    fs::write(&catalog, b"{}\n").unwrap();
    fs::write(&cctx, b"test executable placeholder\n").unwrap();
    (config, state, catalog, cctx)
}

fn desired(catalog: &Path, cctx: &Path) -> OwnedConfig {
    OwnedConfig::candidate(catalog, cctx)
}

fn read_manifest(state: &Path) -> OwnershipManifest {
    serde_json::from_slice(&fs::read(state.join("ownership.json")).unwrap()).unwrap()
}

#[test]
fn missing_config_is_created_then_removed_on_restore() {
    let root = TestDir::new("missing");
    let (config, state, catalog, cctx) = paths(&root);
    let config_plan = plan(&config, &state, desired(&catalog, &cctx)).unwrap();
    assert!(!config.exists());
    assert!(!state.exists(), "planning must not create state");
    assert_eq!(config_plan.diff().observed_sha256, None);
    assert!(
        config_plan
            .diff()
            .changes
            .iter()
            .all(|change| change.before.is_none())
    );

    let applied = apply(&config_plan).unwrap();
    assert!(applied.changed);
    let installed = fs::read_to_string(&config).unwrap();
    assert!(installed.contains("model = \"gpt-5.6-sol\""));
    assert!(installed.contains("model_context_window = 1050000"));
    assert!(installed.contains("[mcp_servers.context_continuum]"));
    assert_eq!(read_manifest(&state).state, ManifestState::Installed);

    let outcome = restore(&config, &state).unwrap();
    assert!(!outcome.restored_original);
    assert!(outcome.removed_installed_file);
    assert!(!config.exists());
    assert_eq!(read_manifest(&state).state, ManifestState::Uninstalled);
}

#[test]
fn partial_commented_and_crlf_configs_round_trip_exact_bytes() {
    let shapes: &[&[u8]] = &[
        b"approval_policy = \"on-request\"\n",
        b"# model = \"commented-model\"\n# keep this comment\n",
        b"approval_policy = \"never\"\r\n\r\n[features]\r\n# hooks = false\r\nweb_search_request = true\r\n",
    ];
    for (index, original) in shapes.iter().enumerate() {
        let root = TestDir::new(&format!("shape-{index}"));
        let (config, state, catalog, cctx) = paths(&root);
        fs::create_dir_all(config.parent().unwrap()).unwrap();
        fs::write(&config, original).unwrap();

        let config_plan = plan(&config, &state, desired(&catalog, &cctx)).unwrap();
        let safe_diff = serde_json::to_string(config_plan.diff()).unwrap();
        assert!(!safe_diff.contains("approval_policy"));
        assert!(!safe_diff.contains("keep this comment"));
        apply(&config_plan).unwrap();
        let installed = fs::read_to_string(&config).unwrap();
        for original_line in String::from_utf8_lossy(original)
            .lines()
            .filter(|line| !line.is_empty())
        {
            assert!(
                installed.contains(original_line),
                "installed config lost original line: {original_line}"
            );
        }
        restore(&config, &state).unwrap();
        assert_eq!(fs::read(&config).unwrap(), *original);
    }
}

#[test]
fn apply_refuses_a_config_changed_after_the_plan() {
    let root = TestDir::new("concurrent");
    let (config, state, catalog, cctx) = paths(&root);
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    fs::write(&config, b"approval_policy = \"never\"\n").unwrap();
    let config_plan = plan(&config, &state, desired(&catalog, &cctx)).unwrap();
    let concurrent = b"approval_policy = \"on-request\"\n";
    fs::write(&config, concurrent).unwrap();

    let error = apply(&config_plan).unwrap_err();
    assert!(error.to_string().contains("concurrent config edit"));
    assert_eq!(fs::read(&config).unwrap(), concurrent);
    assert!(!state.join("ownership.json").exists());
}

#[test]
fn restore_refuses_to_overwrite_a_later_user_edit() {
    let root = TestDir::new("user-edit");
    let (config, state, catalog, cctx) = paths(&root);
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    let original = b"approval_policy = \"never\"\n";
    fs::write(&config, original).unwrap();
    let config_plan = plan(&config, &state, desired(&catalog, &cctx)).unwrap();
    apply(&config_plan).unwrap();
    let mut edited = fs::read(&config).unwrap();
    edited.extend_from_slice(b"\n# user edit after install\n");
    fs::write(&config, &edited).unwrap();

    let error = restore(&config, &state).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("refusing to overwrite user changes")
    );
    assert_eq!(fs::read(&config).unwrap(), edited);
    assert_eq!(read_manifest(&state).state, ManifestState::Installed);
}

#[test]
fn unowned_mcp_name_collision_fails_closed() {
    let root = TestDir::new("collision");
    let (config, state, catalog, cctx) = paths(&root);
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    let original = b"[mcp_servers.context_continuum]\ncommand = \"unrelated\"\n";
    fs::write(&config, original).unwrap();

    let error = plan(&config, &state, desired(&catalog, &cctx)).unwrap_err();
    assert!(error.to_string().contains("ownership conflict"));
    assert_eq!(fs::read(&config).unwrap(), original);
    assert!(!state.exists());
}

#[test]
fn managed_update_preserves_first_backup_and_restores_first_bytes() {
    let root = TestDir::new("update");
    let (config, state, catalog, cctx) = paths(&root);
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    let original = b"# original\napproval_policy = \"on-request\"\n";
    fs::write(&config, original).unwrap();
    let first_plan = plan(&config, &state, desired(&catalog, &cctx)).unwrap();
    apply(&first_plan).unwrap();
    let first_manifest = read_manifest(&state);

    let mut updated = desired(&catalog, &cctx);
    updated.model_auto_compact_token_limit = Some(900_000);
    updated.model_auto_compact_token_limit_scope = Some(AutoCompactScope::Total);
    updated.mcp.startup_timeout_sec = Some(15);
    let second_plan = plan(&config, &state, updated).unwrap();
    assert!(second_plan.diff().managed_update);
    apply(&second_plan).unwrap();
    let second_manifest = read_manifest(&state);
    assert_eq!(second_manifest.backup_file, first_manifest.backup_file);
    assert_eq!(
        second_manifest.pre_install_sha256,
        first_manifest.pre_install_sha256
    );
    assert_eq!(second_manifest.created_at, first_manifest.created_at);
    let installed = fs::read_to_string(&config).unwrap();
    assert!(installed.contains("model_auto_compact_token_limit = 900000"));
    assert!(installed.contains("model_auto_compact_token_limit_scope = \"total\""));

    uninstall(&config, &state).unwrap();
    assert_eq!(fs::read(&config).unwrap(), original);
}

#[test]
fn invalid_toml_and_non_table_parent_fail_without_writes() {
    let root = TestDir::new("invalid");
    let (config, state, catalog, cctx) = paths(&root);
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    let invalid = b"this = [is not valid TOML\n";
    fs::write(&config, invalid).unwrap();
    let error = plan(&config, &state, desired(&catalog, &cctx)).unwrap_err();
    assert!(error.to_string().contains("invalid TOML"));
    assert_eq!(fs::read(&config).unwrap(), invalid);

    let scalar_parent = b"features = \"not-a-table\"\n";
    fs::write(&config, scalar_parent).unwrap();
    let error = plan(&config, &state, desired(&catalog, &cctx)).unwrap_err();
    assert!(error.to_string().contains("not a table"));
    assert_eq!(fs::read(&config).unwrap(), scalar_parent);
    assert!(!state.exists());
}

#[test]
fn exact_sol_policy_and_compaction_pair_are_enforced() {
    let root = TestDir::new("sol-policy");
    let (config, state, catalog, cctx) = paths(&root);

    let mut wrong_model = desired(&catalog, &cctx);
    wrong_model.model = "gpt-5.6".to_owned();
    let error = plan(&config, &state, wrong_model).unwrap_err();
    assert!(error.to_string().contains("exact `gpt-5.6-sol`"));

    let mut unpaired = desired(&catalog, &cctx);
    unpaired.model_auto_compact_token_limit = Some(900_000);
    let error = plan(&config, &state, unpaired).unwrap_err();
    assert!(error.to_string().contains("requires an explicit scope"));

    let mut above_clamp = desired(&catalog, &cctx);
    above_clamp.model_auto_compact_token_limit = Some(945_001);
    above_clamp.model_auto_compact_token_limit_scope = Some(AutoCompactScope::Total);
    let error = plan(&config, &state, above_clamp).unwrap_err();
    assert!(error.to_string().contains("1..=945000"));
    assert!(!config.exists());
    assert!(!state.exists());
}

#[test]
fn a_concurrent_manager_lock_fails_without_config_writes() {
    let root = TestDir::new("manager-lock");
    let (config, state, catalog, cctx) = paths(&root);
    let config_plan = plan(&config, &state, desired(&catalog, &cctx)).unwrap();
    fs::create_dir_all(&state).unwrap();
    let lock_path = state.join("config-manager.lock");
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .unwrap();
    lock.try_lock().unwrap();

    let error = apply(&config_plan).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("another config-manager operation")
    );
    assert!(!config.exists());
    assert!(!state.join("ownership.json").exists());
}

#[test]
fn managed_noop_does_not_rewrite_config_or_manifest() {
    let root = TestDir::new("noop");
    let (config, state, catalog, cctx) = paths(&root);
    let first_plan = plan(&config, &state, desired(&catalog, &cctx)).unwrap();
    apply(&first_plan).unwrap();
    let config_before = fs::read(&config).unwrap();
    let manifest_before = fs::read(state.join("ownership.json")).unwrap();

    let noop = plan(&config, &state, desired(&catalog, &cctx)).unwrap();
    assert!(noop.diff().changes.is_empty());
    let outcome = apply(&noop).unwrap();
    assert!(!outcome.changed);
    assert_eq!(fs::read(&config).unwrap(), config_before);
    assert_eq!(
        fs::read(state.join("ownership.json")).unwrap(),
        manifest_before
    );
}

#[test]
fn emitted_manifest_conforms_to_draft_2020_12_schema() {
    let root = TestDir::new("schema");
    let (config, state, catalog, cctx) = paths(&root);
    let config_plan = plan(&config, &state, desired(&catalog, &cctx)).unwrap();
    apply(&config_plan).unwrap();
    let document: Value =
        serde_json::from_slice(&fs::read(state.join("ownership.json")).unwrap()).unwrap();
    let schema: Value = serde_json::from_str(include_str!(
        "../schemas/config-ownership-manifest.schema.json"
    ))
    .unwrap();
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema)
        .unwrap();
    let errors = validator
        .iter_errors(&document)
        .map(|error| error.to_string())
        .collect::<Vec<_>>();
    assert!(errors.is_empty(), "schema errors: {}", errors.join("\n"));
}

#[test]
fn cli_plan_is_read_only_and_cli_apply_uninstall_round_trips() {
    let root = TestDir::new("cli");
    let (config, state, catalog, _) = paths(&root);
    let binary = PathBuf::from(env!("CARGO_BIN_EXE_cctx"));
    let base_args = [
        "--config",
        config.to_str().unwrap(),
        "--state-dir",
        state.to_str().unwrap(),
        "--catalog",
        catalog.to_str().unwrap(),
        "--cctx",
        binary.to_str().unwrap(),
    ];

    let output = Command::new(&binary)
        .args(["config", "plan"])
        .args(base_args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let diff: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(diff["config_existed"], false);
    assert!(!config.exists());
    assert!(!state.exists());

    let output = Command::new(&binary)
        .args(["config", "apply"])
        .args(base_args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(config.exists());

    let output = Command::new(&binary)
        .args([
            "config",
            "uninstall",
            "--config",
            config.to_str().unwrap(),
            "--state-dir",
            state.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!config.exists());
}
