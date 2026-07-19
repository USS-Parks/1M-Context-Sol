use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
struct EvidenceMap {
    schema_version: u32,
    documents: Vec<EvidenceMapping>,
}

#[derive(Debug, Deserialize)]
struct EvidenceMapping {
    path: String,
    schema: String,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn canonical_pspr_is_structurally_valid() {
    let path = env::var_os("CCTX_PSPR_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join("PLANNING/CODEX-CONTEXT-CONTINUUM-PSPR.md"));
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()));
    validate_pspr(&text).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
}

#[test]
fn mapped_evidence_conforms_to_schema() {
    let evidence_map = load_evidence_map();
    assert_eq!(evidence_map.schema_version, 1);
    assert!(!evidence_map.documents.is_empty());

    for (index, mapping) in evidence_map.documents.iter().enumerate() {
        let document_path = if index == 0 {
            env::var_os("CCTX_EVIDENCE_PATH")
                .map(PathBuf::from)
                .unwrap_or_else(|| root().join(&mapping.path))
        } else {
            root().join(&mapping.path)
        };
        let schema_path = if index == 0 {
            env::var_os("CCTX_EVIDENCE_SCHEMA")
                .map(PathBuf::from)
                .unwrap_or_else(|| root().join(&mapping.schema))
        } else {
            root().join(&mapping.schema)
        };
        validate_json_file(&document_path, &schema_path);
    }
}

#[test]
fn every_json_evidence_file_has_exactly_one_schema_mapping() {
    let evidence_map = load_evidence_map();
    let mapped: BTreeSet<_> = evidence_map
        .documents
        .iter()
        .map(|mapping| mapping.path.replace('\\', "/"))
        .collect();
    assert_eq!(
        mapped.len(),
        evidence_map.documents.len(),
        "evidence map contains duplicate paths"
    );

    let mut discovered = Vec::new();
    collect_json_files(&root().join("docs/evidence"), &mut discovered);
    let discovered: BTreeSet<_> = discovered
        .iter()
        .map(|path| {
            path.strip_prefix(root())
                .expect("evidence must be under repository root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .collect();
    assert_eq!(
        mapped, discovered,
        "evidence/schema map is incomplete or stale"
    );
}

#[test]
fn machine_contracts_conform_to_their_json_schemas() {
    validate_json_file(
        &root().join("contracts/capability-vocabulary.json"),
        &root().join("schemas/capability-vocabulary.schema.json"),
    );
}

#[test]
fn all_mapped_schemas_compile_as_draft_2020_12() {
    let evidence_map = load_evidence_map();
    let mut schemas: BTreeSet<_> = evidence_map
        .documents
        .iter()
        .map(|mapping| mapping.schema.as_str())
        .collect();
    schemas.insert("schemas/capability-vocabulary.schema.json");

    for relative in schemas {
        let path = root().join(relative);
        let schema = read_json(&path);
        jsonschema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .build(&schema)
            .unwrap_or_else(|error| panic!("schema {} does not compile: {error}", path.display()));
    }
}

fn load_evidence_map() -> EvidenceMap {
    let path = root().join("contracts/evidence-schema-map.json");
    serde_json::from_value(read_json(&path))
        .unwrap_or_else(|error| panic!("invalid evidence map {}: {error}", path.display()))
}

fn validate_json_file(document_path: &Path, schema_path: &Path) {
    let document = read_json(document_path);
    let schema = read_json(schema_path);
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .build(&schema)
        .unwrap_or_else(|error| {
            panic!("schema {} does not compile: {error}", schema_path.display())
        });
    let errors: Vec<_> = validator
        .iter_errors(&document)
        .map(|error| error.to_string())
        .collect();
    assert!(
        errors.is_empty(),
        "{} does not conform to {}:\n{}",
        document_path.display(),
        schema_path.display(),
        errors.join("\n")
    );
}

fn read_json(path: &Path) -> Value {
    let text = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("invalid JSON in {}: {error}", path.display()))
}

fn collect_json_files(directory: &Path, files: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("could not read {}: {error}", directory.display()));
    for entry in entries {
        let entry = entry.expect("evidence directory entry must be readable");
        let path = entry.path();
        if path.is_dir() {
            collect_json_files(&path, files);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            files.push(path);
        }
    }
}

fn validate_pspr(text: &str) -> Result<(), String> {
    for required in [
        "> **Status:** APPROVED — FULL STS EXECUTION ACTIVE — GPT-5.6 SOL ONLY",
        "## 1. Authorization and execution state",
        "## 2. Mission",
        "## 5. Governance",
        "## 8. Verification gates",
        "## 9. Milestones",
        "## 10. Sequential prompt roster",
        "## 12. Definition of done",
    ] {
        if !text.contains(required) {
            return Err(format!("missing required PSPR marker: {required}"));
        }
    }

    let lines: Vec<_> = text.lines().collect();
    let prompt_starts: Vec<_> = lines
        .iter()
        .enumerate()
        .filter_map(|(index, line)| {
            line.strip_prefix("#### **CAC-")
                .map(|_| index)
                .filter(|_| line.ends_with("**") && line.contains(" — "))
        })
        .collect();
    if prompt_starts.len() != 47 {
        return Err(format!(
            "expected 47 prompt headers, found {}",
            prompt_starts.len()
        ));
    }

    let mut ids = BTreeSet::new();
    for (position, start) in prompt_starts.iter().copied().enumerate() {
        let header = lines[start]
            .strip_prefix("#### **")
            .and_then(|line| line.strip_suffix("**"))
            .ok_or_else(|| format!("malformed prompt header at line {}", start + 1))?;
        let id = header
            .split_once(" — ")
            .map(|(id, _)| id)
            .ok_or_else(|| format!("missing prompt delimiter at line {}", start + 1))?;
        if !ids.insert(id) {
            return Err(format!("duplicate prompt ID: {id}"));
        }

        let end = prompt_starts
            .get(position + 1)
            .copied()
            .unwrap_or(lines.len());
        let block = lines[start + 1..end].join("\n");
        if !block.contains("**Objective:**") {
            return Err(format!("{id} is missing an objective"));
        }
        if !block.contains("**Acceptance gate:**") {
            return Err(format!("{id} is missing an acceptance gate"));
        }
    }

    for required_id in [
        "CAC-00", "CAC-16", "CAC-27", "CAC-36", "CAC-45", "CAC-56", "CAC-63",
    ] {
        if !ids.contains(required_id) {
            return Err(format!("missing roster boundary prompt {required_id}"));
        }
    }
    Ok(())
}
