use context_continuum::{
    OFFICIAL_MAX_INPUT, OFFICIAL_MAX_OUTPUT, OFFICIAL_TOTAL_CONTEXT, REQUIRED_MODEL,
    probe::CapabilityReport,
};

#[test]
fn checked_in_capability_evidence_matches_the_contract() {
    let report: CapabilityReport = serde_json::from_str(include_str!(
        "../docs/evidence/CAC-01/capability-baseline.json"
    ))
    .expect("checked-in capability evidence must deserialize");

    assert_eq!(report.schema_version, 1);
    assert_eq!(report.official_sol_limits.slug, REQUIRED_MODEL);
    assert_eq!(
        report.official_sol_limits.total_context_window,
        OFFICIAL_TOTAL_CONTEXT
    );
    assert_eq!(report.official_sol_limits.max_input, OFFICIAL_MAX_INPUT);
    assert_eq!(report.official_sol_limits.max_output, OFFICIAL_MAX_OUTPUT);
    assert_eq!(report.catalogs.resolved.context_window, 272_000);
    assert_eq!(report.catalogs.resolved.effective_budget, 258_400);
    assert_eq!(report.catalogs.bundled.context_window, 372_000);
    assert_eq!(report.catalogs.bundled.effective_budget, 353_400);
    assert!(!report.compliance.native_one_million_gate_ready);
    assert!(!report.sanitization.model_request_sent);
    assert!(report.sanitization.credentials_omitted);
    assert!(report.sanitization.model_instructions_omitted);
}

#[test]
fn capability_schema_is_valid_json_and_sol_specific() {
    let schema: serde_json::Value =
        serde_json::from_str(include_str!("../schemas/capability-probe.schema.json"))
            .expect("capability schema must be valid JSON");

    assert_eq!(
        schema
            .pointer("/properties/official_sol_limits/properties/slug/const")
            .and_then(serde_json::Value::as_str),
        Some(REQUIRED_MODEL)
    );
    assert_eq!(
        schema
            .pointer("/properties/official_sol_limits/properties/total_context_window/const")
            .and_then(serde_json::Value::as_u64),
        Some(OFFICIAL_TOTAL_CONTEXT)
    );
}
