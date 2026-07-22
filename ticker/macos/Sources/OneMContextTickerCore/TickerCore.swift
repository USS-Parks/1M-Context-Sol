import AppKit
import CoreGraphics
import Foundation

public enum TickerFailure: LocalizedError {
    case invalid(String)

    public var errorDescription: String? {
        switch self {
        case let .invalid(message): return message
        }
    }
}

public struct TokenState: Equatable {
    public let usedTokens: Int64
    public let contextWindow: Int64
    public let effectiveWindow: Int64
    public let remainingTokens: Int64
    public let percentRemaining: Int
    public let eventTimestamp: Date
    public let ageSeconds: Int
    public let isStale: Bool
    public let wasCompacted: Bool
}

public enum TokenEngine {
    public static let baselineTokens: Int64 = 12_000
    public static let requiredHostWindow: Int64 = 1_008_000

    public static func state(
        from lines: [String],
        now: Date,
        staleAfterSeconds: Int,
        previousUsedTokens: Int64?
    ) throws -> TokenState {
        guard staleAfterSeconds > 0 else {
            throw TickerFailure.invalid("staleAfterSeconds must be positive")
        }

        var selected: [String: Any]?
        for line in lines.reversed() where !line.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
            guard let data = line.data(using: .utf8),
                  let record = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                  record["type"] as? String == "event_msg",
                  let payload = record["payload"] as? [String: Any],
                  payload["type"] as? String == "token_count"
            else { continue }
            selected = record
            break
        }

        guard let record = selected,
              let payload = record["payload"] as? [String: Any],
              let info = payload["info"] as? [String: Any],
              let last = info["last_token_usage"] as? [String: Any]
        else {
            throw TickerFailure.invalid("No valid token_count event was found.")
        }

        let used = try integer(last["total_tokens"], label: "last_token_usage.total_tokens")
        let contextWindow = try integer(info["model_context_window"], label: "model_context_window")
        guard contextWindow == requiredHostWindow else {
            throw TickerFailure.invalid(
                "Host context window \(contextWindow) does not match required 1M budget \(requiredHostWindow)."
            )
        }
        guard used >= 0 else {
            throw TickerFailure.invalid("Invalid active token count.")
        }
        guard let timestampText = record["timestamp"] as? String,
              let timestamp = parseTimestamp(timestampText)
        else {
            throw TickerFailure.invalid("Invalid token event timestamp.")
        }

        let effective = contextWindow - baselineTokens
        let adjustedUsed = max(used - baselineTokens, 0)
        let remaining = max(effective - adjustedUsed, 0)
        let percent = Int(floor((Double(remaining) / Double(effective) * 100.0) + 0.5))
        let age = max(0, Int(now.timeIntervalSince(timestamp)))

        return TokenState(
            usedTokens: used,
            contextWindow: contextWindow,
            effectiveWindow: effective,
            remainingTokens: remaining,
            percentRemaining: percent,
            eventTimestamp: timestamp,
            ageSeconds: age,
            isStale: age > staleAfterSeconds,
            wasCompacted: previousUsedTokens.map { used < $0 } ?? false
        )
    }

    private static func integer(_ value: Any?, label: String) throws -> Int64 {
        guard let number = value as? NSNumber else {
            throw TickerFailure.invalid("Missing or invalid JSON field: \(label)")
        }
        return number.int64Value
    }

    private static func parseTimestamp(_ value: String) -> Date? {
        let fractional = ISO8601DateFormatter()
        fractional.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
        return fractional.date(from: value) ?? ISO8601DateFormatter().date(from: value)
    }
}

public struct RolloutCandidate: Equatable {
    public let url: URL
    public let sessionID: String
    public let threadSource: String
    public let modifiedAt: Date

    public init(url: URL, sessionID: String, threadSource: String, modifiedAt: Date) {
        self.url = url
        self.sessionID = sessionID
        self.threadSource = threadSource
        self.modifiedAt = modifiedAt
    }
}

public struct RolloutSelection: Equatable {
    public let candidate: RolloutCandidate
    public let ambiguous: Bool
}

public enum RolloutSelector {
    public static func select(
        candidates: [RolloutCandidate],
        explicitThreadID: String?,
        ambiguousWithinSeconds: TimeInterval = 15
    ) throws -> RolloutSelection {
        let eligible = candidates
            .filter { candidate in
                if let explicitThreadID { return candidate.sessionID == explicitThreadID }
                return candidate.threadSource != "subagent"
            }
            .sorted { $0.modifiedAt > $1.modifiedAt }

        guard let selected = eligible.first else {
            throw TickerFailure.invalid("No matching root Codex Desktop rollout was found.")
        }
        let ambiguous = explicitThreadID == nil
            && eligible.dropFirst().first.map {
                abs(selected.modifiedAt.timeIntervalSince($0.modifiedAt)) <= ambiguousWithinSeconds
            } ?? false
        return RolloutSelection(candidate: selected, ambiguous: ambiguous)
    }

    public static func select(
        sessionsRoot: URL,
        explicitThreadID: String?,
        ambiguousWithinSeconds: TimeInterval = 15
    ) throws -> RolloutSelection {
        guard let enumerator = FileManager.default.enumerator(
            at: sessionsRoot,
            includingPropertiesForKeys: [.contentModificationDateKey, .isRegularFileKey],
            options: [.skipsHiddenFiles]
        ) else {
            throw TickerFailure.invalid("Codex sessions root does not exist: \(sessionsRoot.path)")
        }

        var candidates: [RolloutCandidate] = []
        for case let url as URL in enumerator {
            guard url.lastPathComponent.hasPrefix("rollout-"), url.pathExtension == "jsonl" else { continue }
            if let explicitThreadID, !url.lastPathComponent.localizedCaseInsensitiveContains(explicitThreadID) {
                continue
            }
            guard let metadata = readMetadata(url) else { continue }
            candidates.append(metadata)
        }
        return try select(
            candidates: candidates,
            explicitThreadID: explicitThreadID,
            ambiguousWithinSeconds: ambiguousWithinSeconds
        )
    }

    public static func readState(
        at url: URL,
        now: Date,
        staleAfterSeconds: Int,
        previousUsedTokens: Int64?
    ) throws -> TokenState {
        var lastError: Error?
        for maximumBytes in [262_144, 1_048_576, 4_194_304, 16_777_216] {
            do {
                let lines = try tailLines(at: url, maximumBytes: maximumBytes)
                return try TokenEngine.state(
                    from: lines,
                    now: now,
                    staleAfterSeconds: staleAfterSeconds,
                    previousUsedTokens: previousUsedTokens
                )
            } catch {
                lastError = error
            }
        }
        throw lastError ?? TickerFailure.invalid("No token state was available.")
    }

    private static func readMetadata(_ url: URL) -> RolloutCandidate? {
        guard let handle = try? FileHandle(forReadingFrom: url) else { return nil }
        defer { try? handle.close() }
        let data = (try? handle.read(upToCount: 65_536)) ?? Data()
        guard let firstLine = String(data: data, encoding: .utf8)?.split(separator: "\n", maxSplits: 1).first,
              let lineData = String(firstLine).data(using: .utf8),
              let record = try? JSONSerialization.jsonObject(with: lineData) as? [String: Any],
              record["type"] as? String == "session_meta",
              let payload = record["payload"] as? [String: Any],
              payload["originator"] as? String == "Codex Desktop",
              let sessionID = payload["id"] as? String,
              let values = try? url.resourceValues(forKeys: [.contentModificationDateKey]),
              let modifiedAt = values.contentModificationDate
        else { return nil }
        return RolloutCandidate(
            url: url,
            sessionID: sessionID,
            threadSource: payload["thread_source"] as? String ?? "",
            modifiedAt: modifiedAt
        )
    }

    private static func tailLines(at url: URL, maximumBytes: Int) throws -> [String] {
        let handle = try FileHandle(forReadingFrom: url)
        defer { try? handle.close() }
        let length = try handle.seekToEnd()
        let count = min(UInt64(maximumBytes), length)
        let start = length - count
        try handle.seek(toOffset: start)
        let data = try handle.readToEnd() ?? Data()
        guard let text = String(data: data, encoding: .utf8) else {
            throw TickerFailure.invalid("Rollout tail is not UTF-8.")
        }
        var lines = text.split(whereSeparator: { $0.isNewline }).map(String.init)
        if start > 0, !lines.isEmpty { lines.removeFirst() }
        return lines
    }
}

public enum LayoutEngine {
    public static func composerCenter(windowLeft: CGFloat, windowRight: CGFloat, sidebarOpen: Bool) throws -> CGFloat {
        let width = windowRight - windowLeft
        guard width > 0 else {
            throw TickerFailure.invalid("windowRight must be greater than windowLeft.")
        }
        let navigationWidth = width * 0.15625
        let sidebarWidth = sidebarOpen ? width * 0.203125 : 0
        return (windowLeft + navigationWidth + ((width - navigationWidth - sidebarWidth) / 2)).rounded()
    }

    public static func panelOrigin(
        quartzWindow: CGRect,
        panelSize: CGSize,
        mainScreenTop: CGFloat,
        sidebarOpen: Bool
    ) throws -> CGPoint {
        let center = try composerCenter(
            windowLeft: quartzWindow.minX,
            windowRight: quartzWindow.maxX,
            sidebarOpen: sidebarOpen
        )
        let appKitWindowBottom = mainScreenTop - quartzWindow.maxY
        return CGPoint(x: center - (panelSize.width / 2), y: appKitWindowBottom + 38)
    }
}

public struct HostWindow: Equatable {
    public let processIdentifier: pid_t
    public let quartzBounds: CGRect
}

public enum HostWindowProvider {
    public static let supportedBundleIdentifier = "com.openai.codex"

    public static func frontmostWindow() throws -> HostWindow? {
        guard let application = NSWorkspace.shared.frontmostApplication,
              application.bundleIdentifier == supportedBundleIdentifier
        else { return nil }

        guard let raw = CGWindowListCopyWindowInfo(
            [.optionOnScreenOnly, .excludeDesktopElements],
            kCGNullWindowID
        ) as? [[String: Any]] else {
            throw TickerFailure.invalid("Window server did not return a window list.")
        }

        let windows = raw.compactMap { entry -> CGRect? in
            guard (entry[kCGWindowOwnerPID as String] as? NSNumber)?.int32Value == application.processIdentifier,
                  (entry[kCGWindowLayer as String] as? NSNumber)?.intValue == 0,
                  ((entry[kCGWindowAlpha as String] as? NSNumber)?.doubleValue ?? 0) > 0,
                  let boundsDictionary = entry[kCGWindowBounds as String] as? NSDictionary,
                  let bounds = CGRect(dictionaryRepresentation: boundsDictionary as CFDictionary),
                  bounds.width >= 300,
                  bounds.height >= 200
            else { return nil }
            return bounds
        }
        guard windows.count == 1, let bounds = windows.first else { return nil }
        return HostWindow(processIdentifier: application.processIdentifier, quartzBounds: bounds)
    }
}

public enum FaceState {
    case normal
    case stale
    case ambiguous
    case invalid
}

public final class TickerPanel: NSPanel {
    public let faceLabel = NSTextField(labelWithString: "Context: -- / 1M")
    private let capsule = NSVisualEffectView()

    public override var canBecomeKey: Bool { false }
    public override var canBecomeMain: Bool { false }

    public init() {
        super.init(
            contentRect: NSRect(x: 0, y: 0, width: 190, height: 26),
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )
        level = .floating
        isOpaque = false
        backgroundColor = .clear
        hasShadow = false
        ignoresMouseEvents = true
        hidesOnDeactivate = false
        becomesKeyOnlyIfNeeded = true
        collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .ignoresCycle]

        capsule.material = .hudWindow
        capsule.blendingMode = .withinWindow
        capsule.state = .active
        capsule.wantsLayer = true
        capsule.layer?.cornerRadius = 13

        faceLabel.font = .monospacedSystemFont(ofSize: 12, weight: .regular)
        faceLabel.textColor = .secondaryLabelColor
        faceLabel.lineBreakMode = .byClipping
        faceLabel.maximumNumberOfLines = 1
        faceLabel.translatesAutoresizingMaskIntoConstraints = false
        capsule.addSubview(faceLabel)
        NSLayoutConstraint.activate([
            faceLabel.leadingAnchor.constraint(equalTo: capsule.leadingAnchor, constant: 5),
            faceLabel.trailingAnchor.constraint(equalTo: capsule.trailingAnchor, constant: -5),
            faceLabel.centerYAnchor.constraint(equalTo: capsule.centerYAnchor)
        ])
        contentView = capsule
        setFace("Context: -- / 1M", state: .normal)
    }

    public func setFace(_ value: String, state: FaceState) {
        faceLabel.stringValue = value
        switch state {
        case .normal: faceLabel.textColor = .secondaryLabelColor
        case .stale: faceLabel.textColor = .tertiaryLabelColor
        case .ambiguous: faceLabel.textColor = .systemYellow
        case .invalid: faceLabel.textColor = .systemOrange
        }
        let width = ceil(faceLabel.intrinsicContentSize.width + 22)
        setContentSize(NSSize(width: width, height: 26))
    }
}
