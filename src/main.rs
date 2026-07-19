#![forbid(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

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
        other => Err(format!("unknown command `{other}`; run `cctx --help`")),
    }
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

fn print_help() {
    println!(
        "Context Continuum for GPT-5.6 Sol\n\nUSAGE:\n    cctx <COMMAND>\n\nCOMMANDS:\n    probe      Capture a sanitized, read-only Codex capability report\n    help       Print this help\n\nRun `cctx probe --help` for probe options."
    );
}

fn print_probe_help() {
    println!(
        "Capture a sanitized Codex capability report without changing configuration or making a model request.\n\nUSAGE:\n    cctx probe [--codex <COMMAND>] [--config <FILE>] [--output <FILE>]"
    );
}
