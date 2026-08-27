// swift-tools-version: 5.9
import PackageDescription

let targets: [Target] = [
    .target(
        name: "FluxaPlayerKit",
        dependencies: []
    )
]

let allTargets = targets + [
    .testTarget(
        name: "FluxaPlayerKitTests",
        dependencies: ["FluxaPlayerKit"]
    )
]

let package = Package(
    name: "FluxaPlayerKit",
    platforms: [.iOS(.v17), .tvOS(.v17), .macOS(.v13)],
    products: [
        .library(name: "FluxaPlayerKit", targets: ["FluxaPlayerKit"])
    ],
    targets: allTargets
)
