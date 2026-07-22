#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


def load(relative: str):
    with (ROOT / relative).open(encoding="utf-8") as handle:
        return json.load(handle)


contract = load("ticker/macos/contract.json")
fixtures = load("ticker/fixtures/behavior-cases.json")
catalog = load("overlay/sol-1m-models.json")

assert contract["schema_version"] == 1
assert contract["product"]["deployment_target"] == "13.0"
assert contract["product"]["architectures"] == ["arm64", "x86_64"]
assert contract["host"]["bundle_identifiers"] == ["com.openai.codex"]
assert contract["host"]["unknown_bundle_action"] == "hide"
assert contract["host"]["ambiguous_window_action"] == "hide"
assert contract["host"]["screen_capture"] is False

rollout = contract["rollout"]
assert rollout["required_originator"] == "Codex Desktop"
assert rollout["excluded_automatic_thread_source"] == "subagent"
assert rollout["active_usage_path"] == "payload.info.last_token_usage.total_tokens"
assert rollout["required_host_window"] == 1008000
assert rollout["wrong_window_face"] == "Context: !"

required_token_cases = {
    "active_not_cumulative",
    "baseline_floor",
    "fully_consumed",
    "stale",
    "compacted",
}
token_cases = {case["id"]: case for case in fixtures["token_cases"]}
assert required_token_cases <= token_cases.keys()
assert all(case["context_window"] == 1008000 for case in token_cases.values())
assert token_cases["active_not_cumulative"]["expected"]["used_tokens"] == 112000
assert token_cases["compacted"]["expected"]["was_compacted"] is True

selection_ids = {case["id"] for case in fixtures["selection_cases"]}
assert "automatic_excludes_subagent" in selection_ids
assert "explicit_pin_is_authoritative" in selection_ids
assert len(fixtures["layout_cases"]) >= 4

models = catalog["models"]
assert len(models) == 1
model = models[0]
assert model["slug"] == "gpt-5.6-sol"
assert model["context_window"] == 1050000
assert model["max_context_window"] == 1050000
assert model["auto_compact_token_limit"] == 900000

owned = contract["configuration"]["owned_values"]
assert list(owned) == [
    "model_context_window",
    "model_auto_compact_token_limit",
    "model_auto_compact_token_limit_scope",
    "model_catalog_json",
]
assert contract["configuration"]["existing_owned_key_action"] == "refuse_without_change"

panel = contract["panel"]
assert panel["class"] == "NSPanel"
assert panel["style_masks"] == ["borderless", "nonactivatingPanel"]
assert panel["ignores_mouse_events"] is True
for forbidden in ["activates_app", "tooltip", "hover_handler", "input_capture", "transcript_display"]:
    assert panel[forbidden] is False

verification = contract["verification"]
for unsupported_claim in [
    "physical_mac_acceptance",
    "live_codex_placement",
    "real_login_launch",
    "gatekeeper_interaction",
]:
    assert verification[unsupported_claim] is False

addendum = (ROOT / "PLANNING/1M-CONTEXT-TICKER-MACOS-DMG-ADDENDUM.md").read_text(encoding="utf-8")
agents = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
architecture = (ROOT / "docs/architecture/MACOS-TICKER-CONTRACT.md").read_text(encoding="utf-8")
assert "APPROVED v3 - STS ACTIVE THROUGH MAC-03" in addendum
assert "execute the MAC roster in dependency order" in agents
assert "does not prove placement over a real Codex composer" in architecture

print(
    "MAC-00 contract passed: "
    f"{len(token_cases)} token, {len(fixtures['selection_cases'])} selection, "
    f"{len(fixtures['layout_cases'])} layout cases; exact host window 1008000; "
    "unsupported live claims disabled"
)
