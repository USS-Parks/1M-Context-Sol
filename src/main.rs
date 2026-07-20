#![forbid(unsafe_code)]

use std::env;
use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use context_continuum::config_manager::{
    AutoCompactScope, OwnedConfig, apply as apply_config, plan as plan_config,
    restore as restore_config, uninstall as uninstall_config,
};
use context_continuum::doctor::{
    DoctorOptions, EXIT_RUNTIME_ERROR, EXIT_USAGE, StatusReport, capture_live, render_doctor,
    render_status,
};
use context_continuum::model_catalog::{
    OfficialSolLimits, OverlayPolicy, ParsedCatalog, capture_installed_catalog,
};
use context_continuum::probe::{ProbeOptions, capture};
use context_continuum::startup_policy::{
    enforce_and_audit, generic_fail_closed_response, parse_hook_input, read_bounded_hook_input,
};

fn main() -> ExitCode {
    match run() {
        Ok(exit_code) => exit_code,
        Err((message, exit_code)) => {
            eprintln!("cctx: {message}");
            ExitCode::from(exit_code)
        }
    }
}

fn run() -> Result<ExitCode, (String, u8)> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_help();
        return Ok(ExitCode::SUCCESS);
    };

    match command.as_str() {
        "-h" | "--help" | "help" => {
            print_help();
            Ok(ExitCode::SUCCESS)
        }
        "-V" | "--version" => {
            println!("cctx {}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::SUCCESS)
        }
        "probe" => ordinary_command(run_probe(args.collect())),
        "catalog" => ordinary_command(run_catalog(args.collect())),
        "config" => ordinary_command(run_config(args.collect())),
        "doctor" => run_doctor_or_status(args.collect(), false),
        "status" => run_doctor_or_status(args.collect(), true),
        "hook" => ordinary_command(run_hook(args.collect())),
        other => Err((
            format!("unknown command `{other}`; run `cctx --help`"),
            EXIT_USAGE,
        )),
    }
}

fn run_hook(args: Vec<String>) -> Result<(), String> {
    let Some(subcommand) = args.first() else {
        print_hook_help();
        return Ok(());
    };
    match subcommand.as_str() {
        "startup-policy" => run_startup_policy_hook(args[1..].to_vec()),
        "-h" | "--help" | "help" => {
            print_hook_help();
            Ok(())
        }
        other => Err(format!(
            "unknown hook command `{other}`; run `cctx hook --help`"
        )),
    }
}

fn run_startup_policy_hook(args: Vec<String>) -> Result<(), String> {
    let mut options = DoctorOptions::default();
    let mut audit_dir: Option<PathBuf> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--codex" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return print_json(&generic_fail_closed_response(
                        "startup-policy hook option --codex requires a value",
                    ));
                };
                options.probe.codex_command = value.clone();
            }
            "--config" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return print_json(&generic_fail_closed_response(
                        "startup-policy hook option --config requires a value",
                    ));
                };
                options.probe.config_path = Some(value.into());
            }
            "--audit-dir" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return print_json(&generic_fail_closed_response(
                        "startup-policy hook option --audit-dir requires a value",
                    ));
                };
                audit_dir = Some(value.into());
            }
            "-h" | "--help" => {
                print_startup_policy_hook_help();
                return Ok(());
            }
            other => {
                return print_json(&generic_fail_closed_response(format!(
                    "unknown startup-policy hook option `{other}`"
                )));
            }
        }
        index += 1;
    }
    let Some(audit_dir) = audit_dir else {
        return print_json(&generic_fail_closed_response(
            "startup-policy hook requires --audit-dir <ABSOLUTE_DIR>",
        ));
    };

    let stdin = io::stdin();
    let mut locked = stdin.lock();
    let bytes = match read_bounded_hook_input(&mut locked) {
        Ok(bytes) => bytes,
        Err(error) => return print_json(&generic_fail_closed_response(error.to_string())),
    };
    let input = match parse_hook_input(&bytes) {
        Ok(input) => input,
        Err(error) => return print_json(&generic_fail_closed_response(error.to_string())),
    };
    let doctor = match capture_live(&options) {
        Ok(report) => report,
        Err(error) => return print_json(&generic_fail_closed_response(error.to_string())),
    };
    let outcome = enforce_and_audit(&input, &doctor, &audit_dir);
    print_json(&outcome.response)
}

fn ordinary_command(result: Result<(), String>) -> Result<ExitCode, (String, u8)> {
    result
        .map(|()| ExitCode::SUCCESS)
        .map_err(|message| (message, EXIT_RUNTIME_ERROR))
}

fn run_doctor_or_status(args: Vec<String>, status_mode: bool) -> Result<ExitCode, (String, u8)> {
    let mut options = DoctorOptions::default();
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--codex" => {
                index += 1;
                options.probe.codex_command = doctor_required_value(&args, index, "--codex")?;
            }
            "--config" => {
                index += 1;
                options.probe.config_path =
                    Some(doctor_required_value(&args, index, "--config")?.into());
            }
            "--json" => json = true,
            "-h" | "--help" => {
                print_doctor_help(status_mode);
                return Ok(ExitCode::SUCCESS);
            }
            other => {
                return Err((
                    format!(
                        "unknown {} option `{other}`",
                        doctor_command_name(status_mode)
                    ),
                    EXIT_USAGE,
                ));
            }
        }
        index += 1;
    }

    let report = capture_live(&options).map_err(|error| (error.to_string(), EXIT_RUNTIME_ERROR))?;
    if status_mode {
        let status = StatusReport::from(&report);
        if json {
            print_json(&status).map_err(|message| (message, EXIT_RUNTIME_ERROR))?;
        } else {
            print!("{}", render_status(&status));
        }
    } else if json {
        print_json(&report).map_err(|message| (message, EXIT_RUNTIME_ERROR))?;
    } else {
        print!("{}", render_doctor(&report));
    }
    Ok(ExitCode::from(report.exit_code))
}

fn doctor_required_value(
    args: &[String],
    index: usize,
    option: &str,
) -> Result<String, (String, u8)> {
    args.get(index)
        .cloned()
        .ok_or_else(|| (format!("{option} requires a value"), EXIT_USAGE))
}

fn doctor_command_name(status_mode: bool) -> &'static str {
    if status_mode { "status" } else { "doctor" }
}

fn run_config(args: Vec<String>) -> Result<(), String> {
    let Some(subcommand) = args.first() else {
        print_config_help();
        return Ok(());
    };
    match subcommand.as_str() {
        "plan" => run_config_plan_or_apply(args[1..].to_vec(), false),
        "apply" => run_config_plan_or_apply(args[1..].to_vec(), true),
        "restore" => run_config_restore_or_uninstall(args[1..].to_vec(), false),
        "uninstall" => run_config_restore_or_uninstall(args[1..].to_vec(), true),
        "-h" | "--help" | "help" => {
            print_config_help();
            Ok(())
        }
        other => Err(format!(
            "unknown config command `{other}`; run `cctx config --help`"
        )),
    }
}

fn run_config_plan_or_apply(args: Vec<String>, should_apply: bool) -> Result<(), String> {
    let mut config: Option<PathBuf> = None;
    let mut state_dir: Option<PathBuf> = None;
    let mut catalog: Option<PathBuf> = None;
    let mut cctx: Option<PathBuf> = None;
    let mut auto_compact_limit: Option<u64> = None;
    let mut auto_compact_scope: Option<AutoCompactScope> = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--config" => {
                index += 1;
                config = Some(required_value(&args, index, "--config")?.into());
            }
            "--state-dir" => {
                index += 1;
                state_dir = Some(required_value(&args, index, "--state-dir")?.into());
            }
            "--catalog" => {
                index += 1;
                catalog = Some(required_value(&args, index, "--catalog")?.into());
            }
            "--cctx" => {
                index += 1;
                cctx = Some(required_value(&args, index, "--cctx")?.into());
            }
            "--auto-compact-token-limit" => {
                index += 1;
                auto_compact_limit = Some(required_u64_value(
                    &args,
                    index,
                    "--auto-compact-token-limit",
                )?);
            }
            "--auto-compact-scope" => {
                index += 1;
                auto_compact_scope = Some(
                    match required_value(&args, index, "--auto-compact-scope")? {
                        "total" => AutoCompactScope::Total,
                        "body_after_prefix" => AutoCompactScope::BodyAfterPrefix,
                        other => {
                            return Err(format!(
                                "--auto-compact-scope must be `total` or `body_after_prefix`, not `{other}`"
                            ));
                        }
                    },
                );
            }
            "-h" | "--help" => {
                print_config_plan_help(should_apply);
                return Ok(());
            }
            other => return Err(format!("unknown config option `{other}`")),
        }
        index += 1;
    }

    if auto_compact_limit.is_some() != auto_compact_scope.is_some() {
        return Err(
            "--auto-compact-token-limit and --auto-compact-scope must be supplied together"
                .to_owned(),
        );
    }
    let config = config.ok_or_else(|| "config command requires --config".to_owned())?;
    let state_dir = state_dir.ok_or_else(|| "config command requires --state-dir".to_owned())?;
    let catalog = catalog.ok_or_else(|| "config command requires --catalog".to_owned())?;
    let cctx = cctx.ok_or_else(|| "config command requires --cctx".to_owned())?;
    let mut desired = OwnedConfig::candidate(&catalog, &cctx);
    desired.model_auto_compact_token_limit = auto_compact_limit;
    desired.model_auto_compact_token_limit_scope = auto_compact_scope;
    let plan = plan_config(&config, &state_dir, desired).map_err(|error| error.to_string())?;

    if should_apply {
        let outcome = apply_config(&plan).map_err(|error| error.to_string())?;
        print_json(&outcome)
    } else {
        print_json(plan.diff())
    }
}

fn run_config_restore_or_uninstall(
    args: Vec<String>,
    should_uninstall: bool,
) -> Result<(), String> {
    let mut config: Option<PathBuf> = None;
    let mut state_dir: Option<PathBuf> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--config" => {
                index += 1;
                config = Some(required_value(&args, index, "--config")?.into());
            }
            "--state-dir" => {
                index += 1;
                state_dir = Some(required_value(&args, index, "--state-dir")?.into());
            }
            "-h" | "--help" => {
                print_config_restore_help(should_uninstall);
                return Ok(());
            }
            other => return Err(format!("unknown config option `{other}`")),
        }
        index += 1;
    }
    let config = config.ok_or_else(|| "config command requires --config".to_owned())?;
    let state_dir = state_dir.ok_or_else(|| "config command requires --state-dir".to_owned())?;
    let outcome = if should_uninstall {
        uninstall_config(&config, &state_dir)
    } else {
        restore_config(&config, &state_dir)
    }
    .map_err(|error| error.to_string())?;
    print_json(&outcome)
}

fn print_json(value: &impl serde::Serialize) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|error| format!("could not serialize command result: {error}"))?;
    println!("{json}");
    Ok(())
}

fn run_catalog(args: Vec<String>) -> Result<(), String> {
    let Some(subcommand) = args.first() else {
        print_catalog_help();
        return Ok(());
    };
    match subcommand.as_str() {
        "generate" => run_catalog_generate(args[1..].to_vec()),
        "-h" | "--help" | "help" => {
            print_catalog_help();
            Ok(())
        }
        other => Err(format!(
            "unknown catalog command `{other}`; run `cctx catalog --help`"
        )),
    }
}

fn run_catalog_generate(args: Vec<String>) -> Result<(), String> {
    let mut input: Option<PathBuf> = None;
    let mut codex_command = "codex".to_owned();
    let mut codex_was_selected = false;
    let mut codex_version: Option<String> = None;
    let mut output: Option<PathBuf> = None;
    let mut manifest: Option<PathBuf> = None;
    let mut policy = OverlayPolicy::sol_1m_candidate();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--input" => {
                index += 1;
                input = Some(required_value(&args, index, "--input")?.into());
            }
            "--codex" => {
                index += 1;
                codex_command = required_value(&args, index, "--codex")?.to_owned();
                codex_was_selected = true;
            }
            "--codex-version" => {
                index += 1;
                codex_version = Some(required_value(&args, index, "--codex-version")?.to_owned());
            }
            "--output" => {
                index += 1;
                output = Some(required_value(&args, index, "--output")?.into());
            }
            "--manifest" => {
                index += 1;
                manifest = Some(required_value(&args, index, "--manifest")?.into());
            }
            "--effective-percent" => {
                index += 1;
                policy.effective_context_window_percent =
                    required_u64_value(&args, index, "--effective-percent")?;
            }
            "--auto-compact-token-limit" => {
                index += 1;
                policy.auto_compact_token_limit = Some(required_u64_value(
                    &args,
                    index,
                    "--auto-compact-token-limit",
                )?);
            }
            "-h" | "--help" => {
                print_catalog_generate_help();
                return Ok(());
            }
            other => return Err(format!("unknown catalog generate option `{other}`")),
        }
        index += 1;
    }

    if input.is_some() && codex_was_selected {
        return Err("--input and --codex are mutually exclusive".to_owned());
    }
    let output = output.ok_or_else(|| "catalog generate requires --output".to_owned())?;
    let manifest = manifest.ok_or_else(|| "catalog generate requires --manifest".to_owned())?;
    if output == manifest {
        return Err("--output and --manifest must be different paths".to_owned());
    }

    let (source, version) = if let Some(input) = input {
        if input == output || input == manifest {
            return Err("catalog input cannot also be an output path".to_owned());
        }
        let version = codex_version.ok_or_else(|| {
            "file input requires --codex-version from the source Codex build".to_owned()
        })?;
        let bytes = std::fs::read(&input)
            .map_err(|error| format!("could not read {}: {error}", input.display()))?;
        (bytes, version)
    } else {
        if codex_version.is_some() {
            return Err("--codex-version is only valid with --input".to_owned());
        }
        let installed =
            capture_installed_catalog(&codex_command).map_err(|error| error.to_string())?;
        (installed.json, installed.codex_version)
    };

    let parsed = ParsedCatalog::parse(&source, &version).map_err(|error| error.to_string())?;
    let generated = parsed
        .generate(&OfficialSolLimits::pinned(), &policy)
        .map_err(|error| error.to_string())?;
    let manifest_json = generated
        .manifest_json()
        .map_err(|error| error.to_string())?;

    std::fs::write(&output, &generated.catalog_json)
        .map_err(|error| format!("could not write {}: {error}", output.display()))?;
    std::fs::write(&manifest, manifest_json)
        .map_err(|error| format!("could not write {}: {error}", manifest.display()))?;
    println!(
        "generated one-model gpt-5.6-sol catalog {} and manifest {} (catalog sha256 {})",
        output.display(),
        manifest.display(),
        generated.manifest.output_catalog_sha256
    );
    Ok(())
}

fn run_probe(args: Vec<String>) -> Result<(), String> {
    let mut options = ProbeOptions::default();
    let mut output: Option<PathBuf> = None;
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--codex" => {
                index += 1;
                options.codex_command = required_value(&args, index, "--codex")?.into();
            }
            "--config" => {
                index += 1;
                options.config_path = Some(required_value(&args, index, "--config")?.into());
            }
            "--output" => {
                index += 1;
                output = Some(required_value(&args, index, "--output")?.into());
            }
            "-h" | "--help" => {
                print_probe_help();
                return Ok(());
            }
            other => return Err(format!("unknown probe option `{other}`")),
        }
        index += 1;
    }

    let report = capture(&options).map_err(|error| error.to_string())?;
    let json = serde_json::to_string_pretty(&report)
        .map_err(|error| format!("could not serialize probe report: {error}"))?;

    if let Some(path) = output {
        std::fs::write(&path, format!("{json}\n"))
            .map_err(|error| format!("could not write {}: {error}", path.display()))?;
    } else {
        println!("{json}");
    }

    Ok(())
}

fn required_value<'a>(args: &'a [String], index: usize, option: &str) -> Result<&'a str, String> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("{option} requires a value"))
}

fn required_u64_value(args: &[String], index: usize, option: &str) -> Result<u64, String> {
    required_value(args, index, option)?
        .replace('_', "")
        .parse()
        .map_err(|error| format!("{option} requires an unsigned integer: {error}"))
}

fn print_help() {
    println!(
        "Context Continuum for GPT-5.6 Sol\n\nUSAGE:\n    cctx <COMMAND>\n\nCOMMANDS:\n    probe      Capture a sanitized, read-only Codex capability report\n    doctor     Explain exact-Sol policy readiness and remediation\n    status     Print a compact claim-safe readiness summary\n    catalog    Parse and generate a version-pinned Sol-only catalog\n    config     Plan, apply, restore, or uninstall owned Codex settings\n    hook       Enforce bounded Codex lifecycle policy\n    help       Print this help\n\nRun `cctx <COMMAND> --help` for command options."
    );
}

fn print_hook_help() {
    println!(
        "Run fail-closed Codex lifecycle policy handlers.\n\nUSAGE:\n    cctx hook <COMMAND>\n\nCOMMANDS:\n    startup-policy   Enforce exact-Sol doctor policy at SessionStart/UserPromptSubmit"
    );
}

fn print_startup_policy_hook_help() {
    println!(
        "Read one official Codex SessionStart or UserPromptSubmit JSON envelope from stdin, block unless exact GPT-5.6 Sol and doctor policy are green, and write a prompt-free audit.\n\nUSAGE:\n    cctx hook startup-policy --audit-dir <ABSOLUTE_DIR> [--codex <COMMAND>] [--config <FILE>]\n\nThe hook always emits protocol-valid JSON for runtime policy decisions. A green doctor policy is configuration evidence, not live native-window proof."
    );
}

fn print_probe_help() {
    println!(
        "Capture a sanitized Codex capability report without changing configuration or making a model request.\n\nUSAGE:\n    cctx probe [--codex <COMMAND>] [--config <FILE>] [--output <FILE>]"
    );
}

fn print_catalog_help() {
    println!(
        "Generate and inspect version-pinned GPT-5.6 Sol-only model catalogs.\n\nUSAGE:\n    cctx catalog <COMMAND>\n\nCOMMANDS:\n    generate   Generate a deterministic one-model overlay and hash manifest"
    );
}

fn print_catalog_generate_help() {
    println!(
        "Generate an uninstalled Sol-1M candidate catalog while preserving installed Sol metadata.\n\nUSAGE:\n    cctx catalog generate (--input <FILE> --codex-version <VERSION> | --codex <COMMAND>) --output <FILE> --manifest <FILE> [--effective-percent <1-100>] [--auto-compact-token-limit <TOKENS>]\n\nIf neither --input nor --codex is supplied, the installed `codex` command is inspected read-only."
    );
}

fn print_config_help() {
    println!(
        "Manage only Context Continuum-owned Codex settings with exact-byte rollback.\n\nUSAGE:\n    cctx config <COMMAND>\n\nCOMMANDS:\n    plan       Print an owned-field-only dry-run diff\n    apply      Atomically apply a fresh plan and save ownership state\n    restore    Restore exact pre-install bytes when no later edit exists\n    uninstall  Alias for guarded exact-byte restore\n\nAll paths are required and must be absolute; no real user path is assumed."
    );
}

fn print_config_plan_help(should_apply: bool) {
    let command = if should_apply { "apply" } else { "plan" };
    println!(
        "USAGE:\n    cctx config {command} --config <FILE> --state-dir <DIR> --catalog <FILE> --cctx <EXECUTABLE> [--auto-compact-token-limit <TOKENS> --auto-compact-scope <total|body_after_prefix>]"
    );
}

fn print_config_restore_help(should_uninstall: bool) {
    let command = if should_uninstall {
        "uninstall"
    } else {
        "restore"
    };
    println!(
        "USAGE:\n    cctx config {command} --config <FILE> --state-dir <DIR>\n\nThe command refuses to write if the installed config differs from the owned snapshot."
    );
}

fn print_doctor_help(status_mode: bool) {
    let command = doctor_command_name(status_mode);
    println!(
        "Inspect exact GPT-5.6 Sol identity, authentication, Codex/catalog policy, canonical capacity dimensions, operational threshold, and compaction guard without sending a model request.\n\nUSAGE:\n    cctx {command} [--codex <COMMAND>] [--config <FILE>] [--json]\n\nEXIT CODES:\n    0   Configuration policy ready (not live native-window proof)\n    1   Runtime inspection failure\n    2   Inspected but not ready\n    3   Unsupported or untrustworthy input\n    64  Invalid usage"
    );
}
