// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "PhotoKitBridge",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "PhotoKitBridge",
            type: .static,
            targets: ["PhotoKitBridge"])
    ],
    targets: [
        .target(
            name: "PhotoKitBridge",
            path: "Sources/PhotoKitBridge",
            publicHeadersPath: "include")
    ]
)
