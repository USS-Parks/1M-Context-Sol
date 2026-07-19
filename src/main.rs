#![forbid(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use context_continuum::model_catalog::{
    OfficialSolLimits, OverlayPolicy, ParsedCatalog, capture_installed_catalog,
};
use context_continuum::probe::{ProbeOptions, capture};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("cctx: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_help();
        return Ok(());
    };

    match command.as_str() {
        "-h" | "--help" | "help" => {
            print_help();
            Ok(())
        }
        "-V" | "--version" => {
            println!("cctx {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        "probe" => run_probe(args.collect()),
        "catalog" => run_catalog(args.collect()),
        other => Err(format!("unknown command `{other}`; run `cctx --help`")),
    }
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
        "Context Continuum for GPT-5.6 Sol\n\nUSAGE:\n    cctx <COMMAND>\n\nCOMMANDS:\n    probe      Capture a sanitized, read-only Codex capability report\n    catalog    Parse and generate a version-pinned Sol-only catalog\n    help       Print this help\n\nRun `cctx <COMMAND> --help` for command options."
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
