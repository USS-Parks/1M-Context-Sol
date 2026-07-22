import AppKit
import Darwin
import Foundation
import OneMContextTickerCore

private struct Options {
    let sessionsRoot: URL
    let threadID: String?
    let statusURL: URL
    let staleAfterSeconds: Int

    static func parse(_ arguments: [String]) throws -> Options {
        let home = FileManager.default.homeDirectoryForCurrentUser
        var sessionsRoot = home.appendingPathComponent(".codex/sessions", isDirectory: true)
        var threadID: String?
        var statusURL = home.appendingPathComponent(
            "Library/Application Support/1M Context Ticker/state/runtime-status.json"
        )
        var staleAfterSeconds = 300
        var index = 0

        func value(after option: String) throws -> String {
            let valueIndex = index + 1
            guard valueIndex < arguments.count, !arguments[valueIndex].isEmpty else {
                throw TickerFailure.invalid("\(option) requires a value.")
            }
            index = valueIndex
            return arguments[valueIndex]
        }

        while index < arguments.count {
            let option = arguments[index]
            switch option {
            case "--sessions-root":
                sessionsRoot = URL(fileURLWithPath: try value(after: option), isDirectory: true)
            case "--thread-id":
                threadID = try value(after: option)
            case "--status-path":
                statusURL = URL(fileURLWithPath: try value(after: option))
            case "--stale-after-seconds":
                guard let parsed = Int(try value(after: option)), parsed > 0 else {
                    throw TickerFailure.invalid("--stale-after-seconds must be positive.")
                }
                staleAfterSeconds = parsed
            default:
                throw TickerFailure.invalid("Unknown option: \(option)")
            }
            index += 1
        }
        return Options(
            sessionsRoot: sessionsRoot,
            threadID: threadID,
            statusURL: statusURL,
            staleAfterSeconds: staleAfterSeconds
        )
    }
}

private final class AppController: NSObject, NSApplicationDelegate {
    private let options: Options
    private let panel = TickerPanel()
    private let numberFormatter: NumberFormatter
    private var timer: Timer?
    private var selectedURL: URL?
    private var selectedSessionID: String?
    private var previousUsedTokens: Int64?

    init(options: Options) {
        self.options = options
        numberFormatter = NumberFormatter()
        numberFormatter.numberStyle = .decimal
        super.init()
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        tick()
        timer = Timer.scheduledTimer(withTimeInterval: 1, repeats: true) { [weak self] _ in
            self?.tick()
        }
    }

    func applicationWillTerminate(_ notification: Notification) {
        timer?.invalidate()
        panel.orderOut(nil)
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        false
    }

    private func tick() {
        var host: HostWindow?
        do {
            host = try HostWindowProvider.frontmostWindow()
            guard let host else {
                panel.orderOut(nil)
                writeStatus(visible: false, state: nil, host: nil, ambiguous: false, error: nil)
                return
            }

            let selection = try RolloutSelector.select(
                sessionsRoot: options.sessionsRoot,
                explicitThreadID: options.threadID
            )
            adopt(selection.candidate)
            guard let selectedURL else {
                throw TickerFailure.invalid("No active root task was selected.")
            }
            let state = try RolloutSelector.readState(
                at: selectedURL,
                now: Date(),
                staleAfterSeconds: options.staleAfterSeconds,
                previousUsedTokens: previousUsedTokens
            )
            previousUsedTokens = state.usedTokens

            let text = numberFormatter.string(from: NSNumber(value: state.usedTokens))
                ?? String(state.usedTokens)
            let faceState: FaceState = state.isStale ? .stale : (selection.ambiguous ? .ambiguous : .normal)
            panel.setFace("Context: \(text) / 1M", state: faceState)
            try placePanel(over: host)
            panel.orderFrontRegardless()
            writeStatus(
                visible: true,
                state: state,
                host: host,
                ambiguous: selection.ambiguous,
                error: nil
            )
        } catch {
            guard let host else {
                panel.orderOut(nil)
                writeStatus(visible: false, state: nil, host: nil, ambiguous: false, error: error.localizedDescription)
                return
            }
            panel.setFace("Context: !", state: .invalid)
            do {
                try placePanel(over: host)
                panel.orderFrontRegardless()
                writeStatus(
                    visible: true,
                    state: nil,
                    host: host,
                    ambiguous: false,
                    error: error.localizedDescription
                )
            } catch {
                panel.orderOut(nil)
                writeStatus(visible: false, state: nil, host: host, ambiguous: false, error: error.localizedDescription)
            }
        }
    }

    private func adopt(_ candidate: RolloutCandidate) {
        guard let selectedURL else {
            self.selectedURL = candidate.url
            selectedSessionID = candidate.sessionID
            return
        }
        guard candidate.url != selectedURL else { return }

        let currentModified = (try? selectedURL.resourceValues(forKeys: [.contentModificationDateKey]))?
            .contentModificationDate ?? .distantPast
        if candidate.modifiedAt > currentModified.addingTimeInterval(3) {
            self.selectedURL = candidate.url
            selectedSessionID = candidate.sessionID
            previousUsedTokens = nil
        }
    }

    private func placePanel(over host: HostWindow) throws {
        let mainScreenTop = NSScreen.screens.map { $0.frame.maxY }.max() ?? 0
        let origin = try LayoutEngine.panelOrigin(
            quartzWindow: host.quartzBounds,
            panelSize: panel.frame.size,
            mainScreenTop: mainScreenTop,
            sidebarOpen: false
        )
        panel.setFrameOrigin(origin)
    }

    private func writeStatus(
        visible: Bool,
        state: TokenState?,
        host: HostWindow?,
        ambiguous: Bool,
        error: String?
    ) {
        let value: [String: Any] = [
            "mode": "native-macos",
            "process_id": ProcessInfo.processInfo.processIdentifier,
            "visible": visible,
            "session_id": jsonValue(selectedSessionID),
            "used_tokens": jsonValue(state?.usedTokens),
            "context_window": jsonValue(state?.contextWindow),
            "remaining_tokens": jsonValue(state?.remainingTokens),
            "percent_remaining": jsonValue(state?.percentRemaining),
            "is_stale": jsonValue(state?.isStale),
            "selection_ambiguous": ambiguous,
            "required_host_window": TokenEngine.requiredHostWindow,
            "one_m_context_verified": state?.contextWindow == TokenEngine.requiredHostWindow,
            "display_state": state == nil ? "invalid" : "normal",
            "host_bundle_identifier": HostWindowProvider.supportedBundleIdentifier,
            "host_process_id": jsonValue(host?.processIdentifier),
            "error": jsonValue(error),
            "updated_utc": ISO8601DateFormatter().string(from: Date())
        ]
        do {
            try FileManager.default.createDirectory(
                at: options.statusURL.deletingLastPathComponent(),
                withIntermediateDirectories: true
            )
            let data = try JSONSerialization.data(withJSONObject: value, options: [.sortedKeys])
            try data.write(to: options.statusURL, options: .atomic)
        } catch {
            // Status is diagnostic only; the panel remains fail-closed independently.
        }
    }

    private func jsonValue<T>(_ value: T?) -> Any {
        value.map { $0 as Any } ?? NSNull()
    }
}

do {
    let options = try Options.parse(Array(CommandLine.arguments.dropFirst()))
    let application = NSApplication.shared
    application.setActivationPolicy(.accessory)
    let controller = AppController(options: options)
    application.delegate = controller
    application.run()
} catch {
    let message = "1M Context Ticker: \(error.localizedDescription)\n"
    FileHandle.standardError.write(Data(message.utf8))
    Darwin.exit(1)
}
