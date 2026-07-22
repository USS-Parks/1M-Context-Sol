#!/usr/bin/env python3
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MAC = ROOT / "ticker" / "macos"

fixture = json.loads((ROOT / "ticker/fixtures/behavior-cases.json").read_text(encoding="utf-8"))
sources = "\n".join(path.read_text(encoding="utf-8") for path in (MAC / "Sources").rglob("*.swift"))
tests = "\n".join(path.read_text(encoding="utf-8") for path in (MAC / "Tests").rglob("*.swift"))
package = (MAC / "Package.swift").read_text(encoding="utf-8")

for required in [
    "last_token_usage",
    "model_context_window",
    "requiredHostWindow: Int64 = 1_008_000",
    "com.openai.codex",
    "CGWindowListCopyWindowInfo",
    ".nonactivatingPanel",
    "ignoresMouseEvents = true",
    "orderFrontRegardless",
    "Context: !",
]:
    assert required in sources, f"missing required source contract: {required}"

for forbidden in [
    "CGWindowListCreateImage",
    "ScreenCaptureKit",
    "SCStream",
    ".toolTip",
    "NSTrackingArea",
    "addGlobalMonitorForEvents",
    "transcript",
]:
    assert forbidden not in sources, f"forbidden source surface: {forbidden}"

assert '.macOS(.v13)' in package
assert 'OneMContextTickerCoreTests' in package
assert 'testSharedTokenCases' in tests
assert 'testWrongWindowFailsClosed' in tests
assert 'testSharedSelectionCases' in tests
assert 'testSharedLayoutCases' in tests
assert 'testPanelIsPassiveAndFitsCompleteFace' in tests

baseline = fixture["baseline_tokens"]
for case in fixture["token_cases"]:
    used = case["active_total_tokens"]
    window = case["context_window"]
    effective = window - baseline
    remaining = max(effective - max(used - baseline, 0), 0)
    percent = int((remaining / effective * 100) + 0.5)
    expected = case["expected"]
    assert window == 1_008_000
    assert used == expected["used_tokens"]
    assert effective == expected["effective_window"]
    assert remaining == expected["remaining_tokens"]
    assert percent == expected["percent_remaining"]

for case in fixture["selection_cases"]:
    explicit = case["explicit_thread_id"]
    eligible = [
        candidate
        for candidate in case["candidates"]
        if candidate["session_id"] == explicit
        if explicit is not None
    ] if explicit is not None else [
        candidate for candidate in case["candidates"] if candidate["thread_source"] != "subagent"
    ]
    eligible.sort(key=lambda candidate: candidate["last_write_offset_seconds"], reverse=True)
    assert eligible[0]["session_id"] == case["expected_session_id"]
    ambiguous = explicit is None and len(eligible) > 1 and abs(
        eligible[0]["last_write_offset_seconds"] - eligible[1]["last_write_offset_seconds"]
    ) <= 15
    assert ambiguous == case["expected_ambiguous"]

for case in fixture["layout_cases"]:
    width = case["window_right"] - case["window_left"]
    navigation = width * 0.15625
    sidebar = width * 0.203125 if case["sidebar_open"] else 0
    center = round(case["window_left"] + navigation + ((width - navigation - sidebar) / 2))
    assert center == case["expected_center"]

print(
    "MAC-01 source contract passed: "
    f"{len(fixture['token_cases'])} token, {len(fixture['selection_cases'])} selection, "
    f"{len(fixture['layout_cases'])} layout cases; passive AppKit surface present"
)
