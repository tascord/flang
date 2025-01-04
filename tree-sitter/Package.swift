// swift-tools-version:5.3
import PackageDescription

let package = Package(
    name: "TreeSitterFlang",
    products: [
        .library(name: "TreeSitterFlang", targets: ["TreeSitterFlang"]),
    ],
    dependencies: [
        .package(url: "https://github.com/ChimeHQ/SwiftTreeSitter", from: "0.8.0"),
    ],
    targets: [
        .target(
            name: "TreeSitterFlang",
            dependencies: [],
            path: ".",
            sources: [
                "src/parser.c",
                // NOTE: if your language has an external scanner, add it here.
            ],
            resources: [
                .copy("queries")
            ],
            publicHeadersPath: "bindings/swift",
            cSettings: [.headerSearchPath("src")]
        ),
        .testTarget(
            name: "TreeSitterFlangTests",
            dependencies: [
                "SwiftTreeSitter",
                "TreeSitterFlang",
            ],
            path: "bindings/swift/TreeSitterFlangTests"
        )
    ],
    cLanguageStandard: .c11
)
