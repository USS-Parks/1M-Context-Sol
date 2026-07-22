import AppKit
import Foundation
import XCTest
@testable import OneMContextTickerCore

final class TickerCoreTests: XCTestCase {
    func testSharedTokenCases() throws {
        let fixture = try loadFixture()
        let cases = try XCTUnwrap(fixture["token_cases"] as? [[String: Any]])

        for item in cases {
            let identifier = try string(item, "id")
            let event: [String: Any] = [
                "timestamp": try string(item, "timestamp"),
                "type": "event_msg",
                "payload": [
                    "type": "token_count",
                    "info": [
                        "total_token_usage": ["total_tokens": try integer(item, "cumulative_total_tokens")],
                        "last_token_usage": ["total_tokens": try integer(item, "active_total_tokens")],
                        "model_context_window": try integer(item, "context_window")
                    ]
                ]
            ]
            let data = try JSONSerialization.data(withJSONObject: event, options: [.sortedKeys])
            let line = try XCTUnwrap(String(data: data, encoding: .utf8))
            let previous = (item["previous_used_tokens"] as? NSNumber)?.int64Value
            let state = try TokenEngine.state(
                from: [line, "malformed-json"],
                now: try date(try string(item, "now")),
                staleAfterSeconds: Int(try integer(item, "stale_after_seconds")),
                previousUsedTokens: previous
            )
            let expected = try XCTUnwrap(item["expected"] as? [String: Any])

            XCTAssertEqual(state.usedTokens, try integer(expected, "used_tokens"), identifier)
            XCTAssertEqual(state.effectiveWindow, try integer(expected, "effective_window"), identifier)
            XCTAssertEqual(state.remainingTokens, try integer(expected, "remaining_tokens"), identifier)
            XCTAssertEqual(state.percentRemaining, Int(try integer(expected, "percent_remaining")), identifier)
            XCTAssertEqual(state.isStale, try boolean(expected, "is_stale"), identifier)
            XCTAssertEqual(state.wasCompacted, try boolean(expected, "was_compacted"), identifier)
        }
    }

    func testWrongWindowFailsClosed() throws {
        let event: [String: Any] = [
            "timestamp": "2026-07-20T12:00:00Z",
            "type": "event_msg",
            "payload": [
                "type": "token_count",
                "info": [
                    "last_token_usage": ["total_tokens": 112_000],
                    "model_context_window": 272_000
                ]
            ]
        ]
        let data = try JSONSerialization.data(withJSONObject: event)
        let line = try XCTUnwrap(String(data: data, encoding: .utf8))
        XCTAssertThrowsError(
            try TokenEngine.state(
                from: [line],
                now: try date("2026-07-20T12:00:01Z"),
                staleAfterSeconds: 60,
                previousUsedTokens: nil
            )
        ) { error in
            XCTAssertTrue(error.localizedDescription.contains("required 1M budget"))
        }
    }

    func testSharedSelectionCases() throws {
        let fixture = try loadFixture()
        let cases = try XCTUnwrap(fixture["selection_cases"] as? [[String: Any]])
        let testRoot = FileManager.default.temporaryDirectory
            .appendingPathComponent("1mct-macos-selection-\(UUID().uuidString)", isDirectory: true)
        try FileManager.default.createDirectory(at: testRoot, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: testRoot) }
        let base = try date("2026-07-20T12:00:00Z")

        for item in cases {
            let identifier = try string(item, "id")
            let caseRoot = testRoot.appendingPathComponent(identifier, isDirectory: true)
            try FileManager.default.createDirectory(at: caseRoot, withIntermediateDirectories: true)
            let candidates = try XCTUnwrap(item["candidates"] as? [[String: Any]])
            for candidate in candidates {
                let sessionID = try string(candidate, "session_id")
                let metadata: [String: Any] = [
                    "type": "session_meta",
                    "payload": [
                        "id": sessionID,
                        "originator": "Codex Desktop",
                        "thread_source": try string(candidate, "thread_source")
                    ]
                ]
                let url = caseRoot.appendingPathComponent("rollout-\(sessionID).jsonl")
                try JSONSerialization.data(withJSONObject: metadata).write(to: url)
                let modified = base.addingTimeInterval(TimeInterval(try integer(candidate, "last_write_offset_seconds")))
                try FileManager.default.setAttributes([.modificationDate: modified], ofItemAtPath: url.path)
            }
            let explicit = item["explicit_thread_id"] as? String
            let selection = try RolloutSelector.select(sessionsRoot: caseRoot, explicitThreadID: explicit)
            XCTAssertEqual(selection.candidate.sessionID, try string(item, "expected_session_id"), identifier)
            XCTAssertEqual(selection.ambiguous, try boolean(item, "expected_ambiguous"), identifier)
        }
    }

    func testSharedLayoutCases() throws {
        let fixture = try loadFixture()
        let cases = try XCTUnwrap(fixture["layout_cases"] as? [[String: Any]])
        for item in cases {
            let identifier = try string(item, "id")
            let center = try LayoutEngine.composerCenter(
                windowLeft: CGFloat(try integer(item, "window_left")),
                windowRight: CGFloat(try integer(item, "window_right")),
                sidebarOpen: try boolean(item, "sidebar_open")
            )
            XCTAssertEqual(center, CGFloat(try integer(item, "expected_center")), identifier)
        }
    }

    func testPanelIsPassiveAndFitsCompleteFace() {
        _ = NSApplication.shared
        let panel = TickerPanel()
        XCTAssertTrue(panel.styleMask.contains(.borderless))
        XCTAssertTrue(panel.styleMask.contains(.nonactivatingPanel))
        XCTAssertEqual(panel.level, .floating)
        XCTAssertTrue(panel.ignoresMouseEvents)
        XCTAssertFalse(panel.canBecomeKey)
        XCTAssertFalse(panel.canBecomeMain)

        panel.setFace("Context: 1,008,000 / 1M", state: .normal)
        XCTAssertEqual(panel.faceLabel.stringValue, "Context: 1,008,000 / 1M")
        XCTAssertGreaterThanOrEqual(panel.frame.width, ceil(panel.faceLabel.intrinsicContentSize.width + 22))
        XCTAssertEqual(panel.contentView?.subviews.count, 1)
    }

    func testSourcesContainNoInteractiveOrCaptureSurface() throws {
        let sources = macOSRoot().appendingPathComponent("Sources", isDirectory: true)
        let forbidden = [
            "CGWindowListCreateImage",
            "ScreenCaptureKit",
            "SCStream",
            ".toolTip",
            "NSTrackingArea",
            "addGlobalMonitorForEvents",
            "transcript"
        ]
        let enumerator = try XCTUnwrap(FileManager.default.enumerator(at: sources, includingPropertiesForKeys: nil))
        var text = ""
        for case let url as URL in enumerator where url.pathExtension == "swift" {
            text += try String(contentsOf: url, encoding: .utf8)
        }
        for value in forbidden {
            XCTAssertFalse(text.contains(value), "forbidden macOS surface: \(value)")
        }
    }

    private func loadFixture() throws -> [String: Any] {
        let url = macOSRoot().deletingLastPathComponent()
            .appendingPathComponent("fixtures/behavior-cases.json")
        let value = try JSONSerialization.jsonObject(with: Data(contentsOf: url))
        return try XCTUnwrap(value as? [String: Any])
    }

    private func macOSRoot() -> URL {
        var url = URL(fileURLWithPath: #filePath)
        for _ in 0..<3 { url.deleteLastPathComponent() }
        return url
    }

    private func string(_ value: [String: Any], _ key: String) throws -> String {
        try XCTUnwrap(value[key] as? String, "Missing string: \(key)")
    }

    private func integer(_ value: [String: Any], _ key: String) throws -> Int64 {
        try XCTUnwrap(value[key] as? NSNumber, "Missing integer: \(key)").int64Value
    }

    private func boolean(_ value: [String: Any], _ key: String) throws -> Bool {
        try XCTUnwrap(value[key] as? NSNumber, "Missing Boolean: \(key)").boolValue
    }

    private func date(_ value: String) throws -> Date {
        try XCTUnwrap(ISO8601DateFormatter().date(from: value), "Invalid timestamp: \(value)")
    }
}
