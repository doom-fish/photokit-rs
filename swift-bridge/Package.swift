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
            name: "PhotoKitObjCBridge",
            path: "Sources/PhotoKitObjCBridge",
            publicHeadersPath: "include"),
        .target(
            name: "PhotoKitBridge",
            dependencies: ["PhotoKitObjCBridge"],
            path: "Sources/PhotoKitBridge")
    ]
)
