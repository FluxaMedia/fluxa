// swift-tools-version: 5.9
import Foundation
import PackageDescription

let packageRoot = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let ffmpegFramework = packageRoot.appendingPathComponent("Vendor/CFFmpeg.xcframework")
let hasFFmpeg = FileManager.default.fileExists(atPath: ffmpegFramework.path)

var targets: [Target] = [
    .target(
        name: "FluxaPlayerKit",
        dependencies: hasFFmpeg ? ["CFFmpeg"] : [],
        swiftSettings: hasFFmpeg ? [.define("FLUXA_FFMPEG")] : [],
        linkerSettings: hasFFmpeg ? [
            .linkedLibrary("z"),
            .linkedLibrary("bz2"),
            .linkedLibrary("iconv"),
            .linkedFramework("VideoToolbox"),
            .linkedFramework("AudioToolbox"),
            .linkedFramework("CoreMedia"),
            .linkedFramework("CoreVideo")
        ] : []
    )
]

if hasFFmpeg {
    targets.append(.binaryTarget(name: "CFFmpeg", path: "Vendor/CFFmpeg.xcframework"))
}

targets.append(
    .testTarget(
        name: "FluxaPlayerKitTests",
        dependencies: ["FluxaPlayerKit"]
    )
)

let package = Package(
    name: "FluxaPlayerKit",
    platforms: [.iOS(.v17), .tvOS(.v17), .macOS(.v13)],
    products: [
        .library(name: "FluxaPlayerKit", targets: ["FluxaPlayerKit"])
    ],
    targets: targets
)
