// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "FluxaPlayerKit",
    platforms: [.iOS(.v17), .tvOS(.v17), .macOS(.v13)],
    products: [
        .library(name: "FluxaPlayerKit", targets: ["FluxaPlayerKit"])
    ],
    targets: [
        .target(name: "FluxaPlayerKit")
    ]
)
