// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "SaorsaWebRTC",
    platforms: [
        .iOS(.v14),
        .macOS(.v11)
    ],
    products: [
        .library(
            name: "SaorsaWebRTC",
            targets: ["SaorsaWebRTC"]
        ),
    ],
    targets: [
        .target(
            name: "SaorsaWebRTC",
            dependencies: ["SaorsaWebRTCFFI"]
        ),
        .systemLibrary(
            name: "SaorsaWebRTCFFI",
            path: "Sources/SaorsaWebRTCFFI",
            pkgConfig: "saorsa-webrtc-ffi"
        ),
        .testTarget(
            name: "SaorsaWebRTCTests",
            dependencies: ["SaorsaWebRTC"]
        ),
    ]
)
