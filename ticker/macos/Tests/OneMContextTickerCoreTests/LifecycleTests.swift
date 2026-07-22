import Foundation
import XCTest
@testable import OneMContextTickerCore

final class LifecycleTests: XCTestCase {
    func testInstallStatusAndExactUninstall() throws {
        try withFixture { fixture in
            let original = try Data(contentsOf: fixture.paths.config)
            let installed = try fixture.manager.install()

            XCTAssertTrue(installed.installed)
            XCTAssertTrue(installed.configurationMatches)
            XCTAssertTrue(installed.catalogMatches)
            XCTAssertTrue(installed.backupMatches)
            XCTAssertEqual(installed.loginItem, .enabled)
            XCTAssertEqual(fixture.login.registerCount, 1)
            XCTAssertEqual(try Data(contentsOf: fixture.paths.backup), original)

            let configured = try String(contentsOf: fixture.paths.config, encoding: .utf8)
            for key in LifecycleManager.ownedKeys {
                XCTAssertTrue(configured.contains("\(key) ="), key)
            }
            XCTAssertTrue(configured.contains("model = \"gpt-5.6-sol\""))

            let removed = try fixture.manager.uninstall()
            XCTAssertFalse(removed.installed)
            XCTAssertEqual(removed.loginItem, .notRegistered)
            XCTAssertEqual(try Data(contentsOf: fixture.paths.config), original)
            XCTAssertFalse(FileManager.default.fileExists(atPath: fixture.paths.manifest.path))
            XCTAssertFalse(FileManager.default.fileExists(atPath: fixture.paths.catalog.path))
            XCTAssertFalse(FileManager.default.fileExists(atPath: fixture.paths.backup.path))
            XCTAssertEqual(fixture.login.unregisterCount, 1)
        }
    }

    func testLaterUserEditIsPreservedOnUninstall() throws {
        try withFixture { fixture in
            _ = try fixture.manager.install()
            let installed = try String(contentsOf: fixture.paths.config, encoding: .utf8)
            let later = installed.replacingOccurrences(
                of: "approval_policy = \"on-request\"",
                with: "approval_policy = \"never\""
            )
            try Data(later.utf8).write(to: fixture.paths.config, options: .atomic)

            _ = try fixture.manager.uninstall()
            let restored = try String(contentsOf: fixture.paths.config, encoding: .utf8)
            XCTAssertTrue(restored.contains("approval_policy = \"never\""))
            for key in LifecycleManager.ownedKeys {
                XCTAssertFalse(restored.contains("\(key) ="), key)
            }
            XCTAssertTrue(restored.contains("model = \"gpt-5.6-sol\""))
        }
    }

    func testPreexistingOwnedKeyIsRefusedWithoutChange() throws {
        try withFixture(config: """
        model = "gpt-5.6-sol"
        model_context_window = 272000

        [features]
        unified_exec = true
        """) { fixture in
            let original = try Data(contentsOf: fixture.paths.config)
            XCTAssertThrowsError(try fixture.manager.install()) { error in
                XCTAssertTrue(error.localizedDescription.contains("Pre-existing owned"))
            }
            XCTAssertEqual(try Data(contentsOf: fixture.paths.config), original)
            XCTAssertFalse(FileManager.default.fileExists(atPath: fixture.paths.manifest.path))
            XCTAssertEqual(fixture.login.registerCount, 0)
        }
    }

    func testWrongUserOwnedModelIsRefused() throws {
        try withFixture(config: "model = \"gpt-5.6\"\n") { fixture in
            let original = try Data(contentsOf: fixture.paths.config)
            XCTAssertThrowsError(try fixture.manager.install())
            XCTAssertEqual(try Data(contentsOf: fixture.paths.config), original)
            XCTAssertEqual(fixture.login.registerCount, 0)
        }
    }

    func testRegistrationFailureRollsBackEveryOwnedFile() throws {
        try withFixture { fixture in
            let original = try Data(contentsOf: fixture.paths.config)
            fixture.login.failRegistration = true
            XCTAssertThrowsError(try fixture.manager.install())
            XCTAssertEqual(try Data(contentsOf: fixture.paths.config), original)
            XCTAssertFalse(FileManager.default.fileExists(atPath: fixture.paths.manifest.path))
            XCTAssertFalse(FileManager.default.fileExists(atPath: fixture.paths.catalog.path))
            XCTAssertFalse(FileManager.default.fileExists(atPath: fixture.paths.backup.path))
        }
    }

    func testFirstLaunchIsIdempotentAndUpgradeRetainsBackup() throws {
        try withFixture { fixture in
            _ = try fixture.manager.ensureInstalled()
            let backup = try Data(contentsOf: fixture.paths.backup)
            let firstManifest = try Data(contentsOf: fixture.paths.manifest)

            let second = try fixture.manager.ensureInstalled()
            XCTAssertTrue(second.installed)
            XCTAssertEqual(fixture.login.registerCount, 1)
            XCTAssertEqual(try Data(contentsOf: fixture.paths.backup), backup)

            fixture.login.state = .notRegistered
            let upgraded = try fixture.manager.upgrade()
            XCTAssertTrue(upgraded.installed)
            XCTAssertEqual(fixture.login.registerCount, 2)
            XCTAssertEqual(try Data(contentsOf: fixture.paths.backup), backup)
            XCTAssertNotEqual(try Data(contentsOf: fixture.paths.manifest), Data())
            XCTAssertFalse(firstManifest.isEmpty)
        }
    }

    func testStartAtLoginReportsApprovalState() throws {
        try withFixture { fixture in
            _ = try fixture.manager.install()
            fixture.login.stateAfterRegistration = .requiresApproval
            let status = try fixture.manager.startAtLogin()
            XCTAssertEqual(status.loginItem, .requiresApproval)
            XCTAssertEqual(fixture.login.registerCount, 2)
        }
    }

    func testStopUsesOnlyTheInjectedTickerProcessController() throws {
        try withFixture { fixture in
            let processes = FakeTickerProcessController(stopped: 2)
            XCTAssertEqual(fixture.manager.stop(processes: processes), 2)
            XCTAssertEqual(processes.callCount, 1)
        }
    }

    func testBundleAndLoginItemStructure() throws {
        let macOS = macOSRoot()
        let plistData = try Data(contentsOf: macOS.appendingPathComponent("Info.plist"))
        let plist = try XCTUnwrap(
            try PropertyListSerialization.propertyList(from: plistData, format: nil) as? [String: Any]
        )
        XCTAssertEqual(plist["CFBundleIdentifier"] as? String, "com.ussparks.1m-context-ticker")
        XCTAssertEqual(plist["LSMinimumSystemVersion"] as? String, "14.0")
        XCTAssertEqual(plist["LSUIElement"] as? Bool, true)

        let lifecycle = try String(
            contentsOf: macOS.appendingPathComponent("Sources/OneMContextTickerCore/Lifecycle.swift"),
            encoding: .utf8
        )
        XCTAssertTrue(lifecycle.contains("SMAppService.mainApp.register()"))
        XCTAssertTrue(lifecycle.contains("SMAppService.mainApp.unregister()"))
    }

    private func withFixture(
        config: String = """
        model = "gpt-5.6-sol"
        approval_policy = "on-request"

        [features]
        unified_exec = true
        """,
        body: (Fixture) throws -> Void
    ) throws {
        let root = FileManager.default.temporaryDirectory
            .appendingPathComponent("1mct-macos-lifecycle-\(UUID().uuidString)", isDirectory: true)
        let home = root.appendingPathComponent("home", isDirectory: true)
        let paths = LifecyclePaths.userDefaults(home: home)
        try FileManager.default.createDirectory(
            at: paths.config.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try Data(config.utf8).write(to: paths.config)
        let login = FakeLoginItemService()
        let manager = LifecycleManager(
            paths: paths,
            sourceCatalog: sharedCatalogURL(),
            loginItem: login
        )
        defer { try? FileManager.default.removeItem(at: root) }
        try body(Fixture(paths: paths, login: login, manager: manager))
    }

    private func sharedCatalogURL() -> URL {
        macOSRoot().deletingLastPathComponent().deletingLastPathComponent()
            .appendingPathComponent("overlay/sol-1m-models.json")
    }

    private func macOSRoot() -> URL {
        var url = URL(fileURLWithPath: #filePath)
        for _ in 0..<3 { url.deleteLastPathComponent() }
        return url
    }
}

private struct Fixture {
    let paths: LifecyclePaths
    let login: FakeLoginItemService
    let manager: LifecycleManager
}

private final class FakeLoginItemService: LoginItemServicing {
    var state: LoginItemState = .notRegistered
    var stateAfterRegistration: LoginItemState = .enabled
    var failRegistration = false
    var registerCount = 0
    var unregisterCount = 0

    func register() throws {
        registerCount += 1
        if failRegistration { throw TickerFailure.invalid("injected login-item failure") }
        state = stateAfterRegistration
    }

    func unregister() throws {
        unregisterCount += 1
        state = .notRegistered
    }
}

private final class FakeTickerProcessController: TickerProcessControlling {
    let stopped: Int
    var callCount = 0

    init(stopped: Int) {
        self.stopped = stopped
    }

    func stopOtherInstances() -> Int {
        callCount += 1
        return stopped
    }
}
