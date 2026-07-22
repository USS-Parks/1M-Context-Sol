// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "OneMContextTicker",
    platforms: [.macOS(.v14)],
    products: [
        .executable(name: "OneMContextTicker", targets: ["OneMContextTicker"])
    ],
    targets: [
        .target(
            name: "OneMContextTickerCore",
            path: "Sources/OneMContextTickerCore"
        ),
        .executableTarget(
            name: "OneMContextTicker",
            dependencies: ["OneMContextTickerCore"],
            path: "Sources/OneMContextTicker"
        ),
        .testTarget(
            name: "OneMContextTickerCoreTests",
            dependencies: ["OneMContextTickerCore"],
            path: "Tests/OneMContextTickerCoreTests"
        )
    ]
)
