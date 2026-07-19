use std::collections::BTreeSet;

use context_continuum::REQUIRED_MODEL;
use context_continuum::model_catalog::{
    CATALOG_SCHEMA_ID, MINIMUM_EFFECTIVE_BUDGET, OfficialSolLimits, OverlayPolicy, ParsedCatalog,
    SUPPORTED_CODEX_VERSION,
};
use serde_json::{Map, Value};

const SOURCE: &[u8] = include_bytes!("fixtures/catalog-v1-current.json");
const DRIFT: &[u8] = include_bytes!("fixtures/catalog-v1-drift-unknown-field.json");
const MALFORMED: &[u8] = include_bytes!("fixtures/catalog-v1-malformed.json");
const POLICY_FIELDS: [&str; 4] = [
    "auto_compact_token_limit",
    "context_window",
    "effective_context_window_percent",
    "max_context_window",
];

fn generate() -> context_continuum::model_catalog::CatalogGeneration {
    ParsedCatalog::parse(SOURCE, SUPPORTED_CODEX_VERSION)
        .expect("current schema fixture should parse")
        .generate(
            &OfficialSolLimits::pinned(),
            &OverlayPolicy::sol_1m_candidate(),
        )
        .expect("candidate overlay should generate")
}

fn sol(value: &Value) -> &Map<String, Value> {
    value["models"][0]
        .as_object()
        .expect("first model must be an object")
}

fn without_policy_fields(model: &Map<String, Value>) -> Map<String, Value> {
    model
        .iter()
        .filter(|(field, _)| !POLICY_FIELDS.contains(&field.as_str()))
        .map(|(field, value)| (field.clone(), value.clone()))
        .collect()
}

#[test]
fn emits_exactly_one_sol_and_changes_only_policy_fields() {
    let source: Value = serde_json::from_slice(SOURCE).expect("source fixture must be JSON");
    let generated = generate();
    let output: Value =
        serde_json::from_slice(&generated.catalog_json).expect("overlay must be JSON");
    let models = output["models"]
        .as_array()
        .expect("models must be an array");
    assert_eq!(models.len(), 1);
    assert_eq!(models[0]["slug"], REQUIRED_MODEL);

    let source_sol = sol(&source);
    let output_sol = sol(&output);
    assert_eq!(
        without_policy_fields(source_sol),
        without_policy_fields(output_sol)
    );
    assert_eq!(
        source_sol["base_instructions"].as_str().unwrap().as_bytes(),
        output_sol["base_instructions"].as_str().unwrap().as_bytes()
    );
    assert_eq!(source_sol["model_messages"], output_sol["model_messages"]);
    for flag in [
        "supports_parallel_tool_calls",
        "supports_image_detail_original",
        "supports_search_tool",
        "use_responses_lite",
        "tool_mode",
        "multi_agent_version",
    ] {
        assert_eq!(source_sol[flag], output_sol[flag], "flag drift: {flag}");
    }

    assert_eq!(output_sol["context_window"], 1_050_000);
    assert_eq!(output_sol["max_context_window"], 1_050_000);
    assert_eq!(output_sol["effective_context_window_percent"], 96);
    assert!(!output_sol.contains_key("auto_compact_token_limit"));
    assert_eq!(
        generated.manifest.changed_policy_fields,
        [
            "context_window",
            "effective_context_window_percent",
            "max_context_window"
        ]
    );
    assert_eq!(
        generated
            .manifest
            .overlay_policy
            .effective_budget()
            .unwrap(),
        1_008_000
    );
    assert!(
        generated
            .manifest
            .overlay_policy
            .effective_budget()
            .unwrap()
            >= MINIMUM_EFFECTIVE_BUDGET
    );
}

#[test]
fn serialization_and_hash_manifest_are_deterministic() {
    let first = generate();
    let second = generate();
    assert_eq!(first.catalog_json, second.catalog_json);
    assert_eq!(first.manifest, second.manifest);
    assert_eq!(
        first.manifest_json().unwrap(),
        second.manifest_json().unwrap()
    );
    assert_eq!(first.manifest.schema_version, 1);
    assert_eq!(first.manifest.catalog_schema_id, CATALOG_SCHEMA_ID);
    assert_eq!(
        first.manifest.supported_codex_version,
        SUPPORTED_CODEX_VERSION
    );
    assert_eq!(first.manifest.output_model_count, 1);
    for hash in [
        &first.manifest.source_catalog_sha256,
        &first.manifest.source_normalized_sha256,
        &first.manifest.source_sol_sha256,
        &first.manifest.preserved_sol_sha256,
        &first.manifest.output_catalog_sha256,
        &first.manifest.output_sol_sha256,
    ] {
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|character| character.is_ascii_hexdigit()));
    }
}

#[test]
fn output_and_manifest_conform_to_draft_2020_12_schemas() {
    let generated = generate();
    let output: Value = serde_json::from_slice(&generated.catalog_json).unwrap();
    let manifest = serde_json::to_value(&generated.manifest).unwrap();
    for (document, schema_text) in [
        (
            output,
            include_str!("../schemas/sol-catalog-overlay.schema.json"),
        ),
        (
            manifest,
            include_str!("../schemas/catalog-overlay-manifest.schema.json"),
        ),
    ] {
        let schema: Value = serde_json::from_str(schema_text).unwrap();
        let validator = jsonschema::options()
            .with_draft(jsonschema::Draft::Draft202012)
            .build(&schema)
            .expect("catalog schema must compile");
        let errors = validator
            .iter_errors(&document)
            .map(|error| error.to_string())
            .collect::<Vec<_>>();
        assert!(errors.is_empty(), "schema errors: {}", errors.join("\n"));
    }
}

#[test]
fn rejects_unknown_version_root_and_sol_schema_drift() {
    let error = ParsedCatalog::parse(SOURCE, "0.145.0").unwrap_err();
    assert!(error.to_string().contains("unsupported Codex version"));

    let mut root: Value = serde_json::from_slice(SOURCE).unwrap();
    root.as_object_mut()
        .unwrap()
        .insert("schema_version".to_owned(), Value::from(2));
    let root_bytes = serde_json::to_vec(&root).unwrap();
    let error = ParsedCatalog::parse(&root_bytes, SUPPORTED_CODEX_VERSION).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("unknown model catalog root schema")
    );

    let error = ParsedCatalog::parse(DRIFT, SUPPORTED_CODEX_VERSION).unwrap_err();
    assert!(error.to_string().contains("future_unreviewed_flag"));
}

#[test]
fn rejects_malformed_missing_and_duplicate_sol_data() {
    let error = ParsedCatalog::parse(MALFORMED, SUPPORTED_CODEX_VERSION).unwrap_err();
    assert!(error.to_string().contains("invalid JSON"));

    let missing = br#"{"models":[{"slug":"fixture-non-sol"}]}"#;
    let error = ParsedCatalog::parse(missing, SUPPORTED_CODEX_VERSION).unwrap_err();
    assert!(error.to_string().contains(REQUIRED_MODEL));

    let mut duplicate: Value = serde_json::from_slice(SOURCE).unwrap();
    let source_sol = duplicate["models"][0].clone();
    duplicate["models"].as_array_mut().unwrap().push(source_sol);
    let duplicate_bytes = serde_json::to_vec(&duplicate).unwrap();
    let error = ParsedCatalog::parse(&duplicate_bytes, SUPPORTED_CODEX_VERSION).unwrap_err();
    assert!(error.to_string().contains("duplicate slug"));
}

#[test]
fn rejects_official_limit_regression_and_untruthful_candidate_policy() {
    let parsed = ParsedCatalog::parse(SOURCE, SUPPORTED_CODEX_VERSION).unwrap();
    let regressed = OfficialSolLimits {
        slug: REQUIRED_MODEL.to_owned(),
        total_context_window: 1_049_999,
        max_input: 921_999,
        max_output: 128_000,
    };
    let error = parsed
        .generate(&regressed, &OverlayPolicy::sol_1m_candidate())
        .unwrap_err();
    assert!(error.to_string().contains("regressed"));

    let low_effective = OverlayPolicy {
        effective_context_window_percent: 95,
        ..OverlayPolicy::sol_1m_candidate()
    };
    let error = parsed
        .generate(&OfficialSolLimits::pinned(), &low_effective)
        .unwrap_err();
    assert!(error.to_string().contains("at least 1000000"));

    let above_codex_clamp = OverlayPolicy {
        auto_compact_token_limit: Some(945_001),
        ..OverlayPolicy::sol_1m_candidate()
    };
    let error = parsed
        .generate(&OfficialSolLimits::pinned(), &above_codex_clamp)
        .unwrap_err();
    assert!(error.to_string().contains("Codex clamps at 90%"));
}

#[test]
fn changed_field_manifest_never_names_unapproved_metadata() {
    let generated = generate();
    let approved = BTreeSet::from(POLICY_FIELDS);
    assert!(
        generated
            .manifest
            .changed_policy_fields
            .iter()
            .all(|field| approved.contains(field.as_str()))
    );
}
