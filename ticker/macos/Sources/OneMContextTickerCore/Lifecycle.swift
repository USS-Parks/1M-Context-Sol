import AppKit
import CryptoKit
import Foundation
import ServiceManagement

public enum LoginItemState: String, Codable {
    case notRegistered
    case enabled
    case requiresApproval
    case notFound
    case unknown
}

public protocol LoginItemServicing {
    var state: LoginItemState { get }
    func register() throws
    func unregister() throws
}

public final class SystemLoginItemService: LoginItemServicing {
    public init() {}

    public var state: LoginItemState {
        switch SMAppService.mainApp.status {
        case .notRegistered: return .notRegistered
        case .enabled: return .enabled
        case .requiresApproval: return .requiresApproval
        case .notFound: return .notFound
        @unknown default: return .unknown
        }
    }

    public func register() throws {
        try SMAppService.mainApp.register()
    }

    public func unregister() throws {
        try SMAppService.mainApp.unregister()
    }
}

public protocol TickerProcessControlling {
    func stopOtherInstances() -> Int
}

public struct SystemTickerProcessController: TickerProcessControlling {
    private let bundleIdentifier: String

    public init(bundleIdentifier: String = "com.ussparks.1m-context-ticker") {
        self.bundleIdentifier = bundleIdentifier
    }

    public func stopOtherInstances() -> Int {
        let current = ProcessInfo.processInfo.processIdentifier
        var stopped = 0
        for application in NSRunningApplication.runningApplications(withBundleIdentifier: bundleIdentifier)
            where application.processIdentifier != current
        {
            if application.terminate() { stopped += 1 }
        }
        return stopped
    }
}

public struct LifecyclePaths {
    public let config: URL
    public let applicationSupport: URL
    public let catalog: URL
    public let stateDirectory: URL
    public let manifest: URL
    public let backup: URL
    public let runtimeStatus: URL

    public init(config: URL, applicationSupport: URL) {
        self.config = config
        self.applicationSupport = applicationSupport
        catalog = applicationSupport.appendingPathComponent("sol-1m-models.json")
        stateDirectory = applicationSupport.appendingPathComponent("state", isDirectory: true)
        manifest = stateDirectory.appendingPathComponent("install-manifest.json")
        backup = stateDirectory.appendingPathComponent("config.before.toml")
        runtimeStatus = stateDirectory.appendingPathComponent("runtime-status.json")
    }

    public static func userDefaults(home: URL = FileManager.default.homeDirectoryForCurrentUser) -> LifecyclePaths {
        LifecyclePaths(
            config: home.appendingPathComponent(".codex/config.toml"),
            applicationSupport: home.appendingPathComponent(
                "Library/Application Support/1M Context Ticker",
                isDirectory: true
            )
        )
    }
}

public struct InstallManifest: Codable, Equatable {
    public let schemaVersion: Int
    public let configPath: String
    public let originalConfigSHA256: String
    public let installedConfigSHA256: String
    public let catalogPath: String
    public var catalogSHA256: String
    public let ownedValues: [String: String]
    public let installedAtUTC: String

    enum CodingKeys: String, CodingKey {
        case schemaVersion = "schema_version"
        case configPath = "config_path"
        case originalConfigSHA256 = "original_config_sha256"
        case installedConfigSHA256 = "installed_config_sha256"
        case catalogPath = "catalog_path"
        case catalogSHA256 = "catalog_sha256"
        case ownedValues = "owned_values"
        case installedAtUTC = "installed_at_utc"
    }
}

public struct LifecycleStatus: Codable, Equatable {
    public let installed: Bool
    public let configurationMatches: Bool
    public let catalogMatches: Bool
    public let backupMatches: Bool
    public let loginItem: LoginItemState
    public let configPath: String
    public let manifestPath: String
    public let error: String?

    enum CodingKeys: String, CodingKey {
        case installed
        case configurationMatches = "configuration_matches"
        case catalogMatches = "catalog_matches"
        case backupMatches = "backup_matches"
        case loginItem = "login_item"
        case configPath = "config_path"
        case manifestPath = "manifest_path"
        case error
    }
}

public final class LifecycleManager {
    public static let ownedKeys = [
        "model_context_window",
        "model_auto_compact_token_limit",
        "model_auto_compact_token_limit_scope",
        "model_catalog_json"
    ]

    private let paths: LifecyclePaths
    private let sourceCatalog: URL
    private let loginItem: LoginItemServicing
    private let fileManager: FileManager

    public init(
        paths: LifecyclePaths,
        sourceCatalog: URL,
        loginItem: LoginItemServicing,
        fileManager: FileManager = .default
    ) {
        self.paths = paths
        self.sourceCatalog = sourceCatalog
        self.loginItem = loginItem
        self.fileManager = fileManager
    }

    public func ensureInstalled() throws -> LifecycleStatus {
        if fileManager.fileExists(atPath: paths.manifest.path) {
            let current = status()
            guard current.configurationMatches, current.catalogMatches, current.backupMatches else {
                throw TickerFailure.invalid(current.error ?? "Installed ticker state is not valid.")
            }
            if loginItem.state == .notRegistered { try loginItem.register() }
            return status()
        }
        return try install()
    }

    public func install() throws -> LifecycleStatus {
        guard !fileManager.fileExists(atPath: paths.manifest.path),
              !fileManager.fileExists(atPath: paths.backup.path),
              !fileManager.fileExists(atPath: paths.catalog.path)
        else {
            throw TickerFailure.invalid("App-owned install state already exists without a complete manifest.")
        }
        let original = try Data(contentsOf: paths.config)
        let catalog = try validatedCatalogData()
        let owned = ownedValues(catalogPath: paths.catalog.path)
        let candidate = try ConfigDocument(data: original).installing(ownedValues: owned)

        try fileManager.createDirectory(at: paths.stateDirectory, withIntermediateDirectories: true)
        var configWasChanged = false
        do {
            try writeAtomically(original, to: paths.backup)
            try writeAtomically(catalog, to: paths.catalog)
            try writeAtomically(candidate, to: paths.config)
            configWasChanged = true

            let manifest = InstallManifest(
                schemaVersion: 1,
                configPath: paths.config.path,
                originalConfigSHA256: sha256(original),
                installedConfigSHA256: sha256(candidate),
                catalogPath: paths.catalog.path,
                catalogSHA256: sha256(catalog),
                ownedValues: owned,
                installedAtUTC: ISO8601DateFormatter().string(from: Date())
            )
            try writeManifest(manifest)
            try loginItem.register()
            return status()
        } catch {
            if configWasChanged { try? writeAtomically(original, to: paths.config) }
            try? loginItem.unregister()
            removeIfPresent(paths.manifest)
            removeIfPresent(paths.catalog)
            removeIfPresent(paths.backup)
            throw error
        }
    }

    public func upgrade() throws -> LifecycleStatus {
        var manifest = try readManifest()
        try validateManifestPaths(manifest)
        let current = try Data(contentsOf: paths.config)
        guard try ConfigDocument(data: current).containsAll(ownedValues: manifest.ownedValues) else {
            throw TickerFailure.invalid("Owned configuration values changed; upgrade refused.")
        }
        let backup = try Data(contentsOf: paths.backup)
        guard sha256(backup) == manifest.originalConfigSHA256 else {
            throw TickerFailure.invalid("Byte-exact configuration backup does not match the manifest.")
        }
        let catalog = try validatedCatalogData()
        try writeAtomically(catalog, to: paths.catalog)
        manifest.catalogSHA256 = sha256(catalog)
        try writeManifest(manifest)
        if loginItem.state == .notRegistered { try loginItem.register() }
        return status()
    }

    public func startAtLogin() throws -> LifecycleStatus {
        _ = try readManifest()
        try loginItem.register()
        return status()
    }

    public func uninstall() throws -> LifecycleStatus {
        let manifest = try readManifest()
        try validateManifestPaths(manifest)
        let backup = try Data(contentsOf: paths.backup)
        guard sha256(backup) == manifest.originalConfigSHA256 else {
            throw TickerFailure.invalid("Byte-exact configuration backup does not match the manifest.")
        }
        let current = try Data(contentsOf: paths.config)

        try loginItem.unregister()
        if sha256(current) == manifest.installedConfigSHA256 {
            try writeAtomically(backup, to: paths.config)
        } else {
            let restored = try ConfigDocument(data: current).removingStillOwned(ownedValues: manifest.ownedValues)
            try writeAtomically(restored, to: paths.config)
        }

        removeIfPresent(paths.runtimeStatus)
        removeIfPresent(paths.manifest)
        removeIfPresent(paths.catalog)
        removeIfPresent(paths.backup)
        return status()
    }

    public func status() -> LifecycleStatus {
        guard fileManager.fileExists(atPath: paths.manifest.path) else {
            return LifecycleStatus(
                installed: false,
                configurationMatches: false,
                catalogMatches: false,
                backupMatches: false,
                loginItem: loginItem.state,
                configPath: paths.config.path,
                manifestPath: paths.manifest.path,
                error: nil
            )
        }
        do {
            let manifest = try readManifest()
            try validateManifestPaths(manifest)
            let config = try Data(contentsOf: paths.config)
            let catalog = try Data(contentsOf: paths.catalog)
            let backup = try Data(contentsOf: paths.backup)
            let configurationMatches = try ConfigDocument(data: config)
                .containsAll(ownedValues: manifest.ownedValues)
            return LifecycleStatus(
                installed: true,
                configurationMatches: configurationMatches,
                catalogMatches: sha256(catalog) == manifest.catalogSHA256,
                backupMatches: sha256(backup) == manifest.originalConfigSHA256,
                loginItem: loginItem.state,
                configPath: paths.config.path,
                manifestPath: paths.manifest.path,
                error: configurationMatches ? nil : "Owned configuration values do not match."
            )
        } catch {
            return LifecycleStatus(
                installed: true,
                configurationMatches: false,
                catalogMatches: false,
                backupMatches: false,
                loginItem: loginItem.state,
                configPath: paths.config.path,
                manifestPath: paths.manifest.path,
                error: error.localizedDescription
            )
        }
    }

    private func ownedValues(catalogPath: String) -> [String: String] {
        [
            "model_context_window": "1050000",
            "model_auto_compact_token_limit": "900000",
            "model_auto_compact_token_limit_scope": "\"total\"",
            "model_catalog_json": "\"\(tomlEscaped(catalogPath))\""
        ]
    }

    private func validatedCatalogData() throws -> Data {
        let data = try Data(contentsOf: sourceCatalog)
        guard let root = try JSONSerialization.jsonObject(with: data) as? [String: Any],
              let models = root["models"] as? [[String: Any]],
              models.count == 1,
              let model = models.first,
              model["slug"] as? String == "gpt-5.6-sol",
              (model["context_window"] as? NSNumber)?.int64Value == 1_050_000,
              (model["max_context_window"] as? NSNumber)?.int64Value == 1_050_000,
              (model["auto_compact_token_limit"] as? NSNumber)?.int64Value == 900_000
        else {
            throw TickerFailure.invalid("The bundled Sol catalog does not match the frozen contract.")
        }
        return data
    }

    private func readManifest() throws -> InstallManifest {
        let data = try Data(contentsOf: paths.manifest)
        let manifest = try JSONDecoder().decode(InstallManifest.self, from: data)
        guard manifest.schemaVersion == 1 else {
            throw TickerFailure.invalid("Unsupported install manifest schema.")
        }
        return manifest
    }

    private func writeManifest(_ manifest: InstallManifest) throws {
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.prettyPrinted, .sortedKeys, .withoutEscapingSlashes]
        try writeAtomically(encoder.encode(manifest), to: paths.manifest)
    }

    private func validateManifestPaths(_ manifest: InstallManifest) throws {
        guard manifest.configPath == paths.config.path,
              manifest.catalogPath == paths.catalog.path,
              Set(manifest.ownedValues.keys) == Set(Self.ownedKeys)
        else {
            throw TickerFailure.invalid("Install manifest paths or owned keys do not match.")
        }
    }

    private func writeAtomically(_ data: Data, to url: URL) throws {
        try fileManager.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
        try data.write(to: url, options: .atomic)
    }

    private func removeIfPresent(_ url: URL) {
        if fileManager.fileExists(atPath: url.path) { try? fileManager.removeItem(at: url) }
    }

    private func sha256(_ data: Data) -> String {
        SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined()
    }

    private func tomlEscaped(_ value: String) -> String {
        value.replacingOccurrences(of: "\\", with: "\\\\")
            .replacingOccurrences(of: "\"", with: "\\\"")
    }
}

private struct ConfigDocument {
    private let lines: [String]
    private let newline: String
    private let endsWithNewline: Bool
    private let hasByteOrderMark: Bool

    init(data: Data) throws {
        hasByteOrderMark = data.starts(with: [0xEF, 0xBB, 0xBF])
        let body = hasByteOrderMark ? data.dropFirst(3) : data[...]
        guard let text = String(data: Data(body), encoding: .utf8) else {
            throw TickerFailure.invalid("Codex configuration is not valid UTF-8.")
        }
        newline = text.contains("\r\n") ? "\r\n" : "\n"
        endsWithNewline = text.hasSuffix(newline)
        var parsed = text.components(separatedBy: newline)
        if endsWithNewline, parsed.last == "" { parsed.removeLast() }
        lines = parsed
    }

    func installing(ownedValues: [String: String]) throws -> Data {
        let assignments = rootAssignments()
        guard assignments["model"] == "\"gpt-5.6-sol\"" else {
            throw TickerFailure.invalid("The user-owned model must already be exact gpt-5.6-sol.")
        }
        for key in LifecycleManager.ownedKeys where assignments[key] != nil {
            throw TickerFailure.invalid("Pre-existing owned configuration key refused: \(key)")
        }

        var result = lines
        let tableIndex = result.firstIndex { line in
            let value = line.trimmingCharacters(in: .whitespaces)
            return value.hasPrefix("[")
        } ?? result.count
        let block = LifecycleManager.ownedKeys.map { key in
            "\(key) = \(ownedValues[key]!)"
        } + [""]
        result.insert(contentsOf: block, at: tableIndex)
        return encoded(result)
    }

    func containsAll(ownedValues: [String: String]) throws -> Bool {
        let assignments = rootAssignments()
        return LifecycleManager.ownedKeys.allSatisfy { key in assignments[key] == ownedValues[key] }
    }

    func removingStillOwned(ownedValues: [String: String]) throws -> Data {
        var inTable = false
        var result: [String] = []
        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            if trimmed.hasPrefix("[") { inTable = true }
            if !inTable,
               let assignment = Self.assignment(line),
               LifecycleManager.ownedKeys.contains(assignment.key),
               ownedValues[assignment.key] == assignment.value
            {
                continue
            }
            result.append(line)
        }
        return encoded(result)
    }

    private func rootAssignments() -> [String: String] {
        var result: [String: String] = [:]
        for line in lines {
            let trimmed = line.trimmingCharacters(in: .whitespaces)
            if trimmed.hasPrefix("[") { break }
            if let assignment = Self.assignment(line) { result[assignment.key] = assignment.value }
        }
        return result
    }

    private func encoded(_ value: [String]) -> Data {
        var text = value.joined(separator: newline)
        if endsWithNewline { text += newline }
        var data = Data()
        if hasByteOrderMark { data.append(contentsOf: [0xEF, 0xBB, 0xBF]) }
        data.append(Data(text.utf8))
        return data
    }

    private static func assignment(_ line: String) -> (key: String, value: String)? {
        let pattern = #"^\s*([A-Za-z0-9_-]+)\s*=\s*(.*?)\s*$"#
        guard let expression = try? NSRegularExpression(pattern: pattern),
              let match = expression.firstMatch(
                  in: line,
                  range: NSRange(line.startIndex..., in: line)
              ),
              match.numberOfRanges == 3,
              let keyRange = Range(match.range(at: 1), in: line),
              let valueRange = Range(match.range(at: 2), in: line)
        else { return nil }
        return (String(line[keyRange]), String(line[valueRange]))
    }
}
