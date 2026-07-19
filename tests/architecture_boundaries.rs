use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;

const COMPONENT_IDS: [&str; 11] = [
    "codex_host",
    "codex_auth",
    "launcher_installer",
    "native_window_controller",
    "lifecycle_capture",
    "reservoir",
    "recall_controller",
    "compaction_guard",
    "successor_rollover",
    "mcp_server",
    "doctor_status",
];

fn architecture() -> Value {
    serde_json::from_str(include_str!("../contracts/architecture-boundaries.json"))
        .expect("architecture boundary contract must be valid JSON")
}

#[test]
fn every_component_has_one_owner_and_a_complete_failure_contract() {
    let architecture = architecture();
    let components = architecture["components"]
        .as_array()
        .expect("components must be an array");
    assert_eq!(components.len(), COMPONENT_IDS.len());

    let mut ids = BTreeSet::new();
    for component in components {
        let id = text(component, "id");
        assert!(ids.insert(id), "duplicate component ID: {id}");
        assert!(!text(component, "owner").is_empty());
        assert!(!text(component, "role").is_empty());
        assert!(!text(component, "failure_mode").is_empty());
        assert!(!array(component, "inputs").is_empty());
        assert!(!array(component, "outputs").is_empty());
        assert!(component["external"].is_boolean());
    }
    assert_eq!(
        ids,
        COMPONENT_IDS.into_iter().collect(),
        "component roster drifted"
    );
}

#[test]
fn every_authority_has_exactly_one_existing_owner() {
    let architecture = architecture();
    let components: BTreeSet<_> = architecture["components"]
        .as_array()
        .expect("components must be an array")
        .iter()
        .map(|component| text(component, "id"))
        .collect();
    let authorities = architecture["authority_invariants"]
        .as_object()
        .expect("authority invariants must be an object");
    assert_eq!(authorities.len(), 6);

    for (authority, owner) in authorities {
        let owner = owner.as_str().expect("authority owner must be a string");
        assert!(
            components.contains(owner),
            "authority {authority} has unknown owner {owner}"
        );
    }
    assert_eq!(authorities["agent_loop"], "codex_host");
    assert_eq!(authorities["credentials"], "codex_auth");
    assert_eq!(authorities["durable_context"], "reservoir");
}

#[test]
fn all_trust_transitions_have_owned_endpoints_and_fail_closed_controls() {
    let architecture = architecture();
    let mut endpoints: BTreeSet<_> = architecture["components"]
        .as_array()
        .expect("components must be an array")
        .iter()
        .map(|component| text(component, "id"))
        .collect();
    endpoints.insert("operator");

    let transitions = architecture["trust_transitions"]
        .as_array()
        .expect("trust transitions must be an array");
    assert_eq!(transitions.len(), 20);
    let mut ids = BTreeSet::new();
    let mut pairs = BTreeSet::new();

    for transition in transitions {
        let id = text(transition, "id");
        let source = text(transition, "source");
        let destination = text(transition, "destination");
        assert!(ids.insert(id), "duplicate transition ID: {id}");
        assert!(endpoints.contains(source), "unknown source: {source}");
        assert!(
            endpoints.contains(destination),
            "unknown destination: {destination}"
        );
        assert!(!text(transition, "data").is_empty());
        assert!(!text(transition, "control").is_empty());
        assert!(!text(transition, "failure").is_empty());
        pairs.insert((source, destination));
    }

    for required in [
        ("operator", "codex_auth"),
        ("codex_host", "lifecycle_capture"),
        ("lifecycle_capture", "reservoir"),
        ("codex_host", "compaction_guard"),
        ("compaction_guard", "reservoir"),
        ("codex_host", "mcp_server"),
        ("recall_controller", "reservoir"),
        ("reservoir", "recall_controller"),
        ("reservoir", "successor_rollover"),
        ("successor_rollover", "codex_host"),
    ] {
        assert!(
            pairs.contains(&required),
            "missing high-risk boundary {required:?}"
        );
    }
}

#[test]
fn forbidden_parallel_subsystems_are_explicit_and_absent() {
    let architecture = architecture();
    let forbidden: BTreeSet<_> = architecture["forbidden_subsystems"]
        .as_array()
        .expect("forbidden subsystems must be an array")
        .iter()
        .map(|value| value.as_str().expect("forbidden ID must be text"))
        .collect();
    assert_eq!(
        forbidden,
        [
            "alternate_model_router",
            "cloud_transcript_backend",
            "parallel_agent_orchestrator",
            "second_durable_store",
        ]
        .into_iter()
        .collect()
    );

    let component_ids: BTreeSet<_> = architecture["components"]
        .as_array()
        .expect("components must be an array")
        .iter()
        .map(|component| text(component, "id"))
        .collect();
    assert!(forbidden.is_disjoint(&component_ids));
}

#[test]
fn all_architecture_decisions_record_owner_io_and_failure_mode() {
    let decisions = BTreeMap::from([
        (
            "ADR-0001",
            include_str!("../docs/architecture/decisions/ADR-0001-single-native-binary.md"),
        ),
        (
            "ADR-0002",
            include_str!("../docs/architecture/decisions/ADR-0002-sol-only-catalog-replacement.md"),
        ),
        (
            "ADR-0003",
            include_str!("../docs/architecture/decisions/ADR-0003-authentication-lanes.md"),
        ),
        (
            "ADR-0004",
            include_str!("../docs/architecture/decisions/ADR-0004-sqlite-reservoir.md"),
        ),
        (
            "ADR-0005",
            include_str!("../docs/architecture/decisions/ADR-0005-hooks-and-mcp-separation.md"),
        ),
        (
            "ADR-0006",
            include_str!("../docs/architecture/decisions/ADR-0006-strict-compaction-blocking.md"),
        ),
        (
            "ADR-0007",
            include_str!("../docs/architecture/decisions/ADR-0007-successor-rollover.md"),
        ),
        (
            "ADR-0008",
            include_str!("../docs/architecture/decisions/ADR-0008-local-only-privacy.md"),
        ),
        (
            "ADR-0009",
            include_str!("../docs/architecture/decisions/ADR-0009-fork-and-upstream-escalation.md"),
        ),
    ]);
    assert_eq!(decisions.len(), 9);
    for (id, decision) in decisions {
        assert!(decision.contains("- **Status:** Accepted"), "{id}");
        assert!(decision.contains("## Contract"), "{id}");
        assert!(decision.contains("- **Owner:"), "{id}");
        assert!(decision.contains("- **Inputs:"), "{id}");
        assert!(decision.contains("- **Outputs:"), "{id}");
        assert!(decision.contains("- **Failure mode:"), "{id}");
    }
}

#[test]
fn threat_model_is_repository_scoped_and_cache_keyed() {
    let threat_model = include_str!("../docs/security/THREAT-MODEL.md");
    for section in [
        "## Overview",
        "## Threat Model, Trust Boundaries, and Assumptions",
        "## Attack Surface, Mitigations, and Attacker Stories",
        "## Severity Calibration (Critical, High, Medium, Low)",
    ] {
        assert!(threat_model.contains(section), "missing section: {section}");
    }
    assert!(threat_model.ends_with(
        "Repository: target_sha256_fd57868a15df2bb7a521c39ba3326e0121373a048d37a819508d58c426f474b7\nVersion: f21a702ea33b0b7c6adbbd05b830ec54b72a3699\n"
    ));
}

fn text<'a>(value: &'a Value, field: &str) -> &'a str {
    value[field].as_str().expect("field must be text")
}

fn array<'a>(value: &'a Value, field: &str) -> &'a [Value] {
    value[field].as_array().expect("field must be an array")
}
