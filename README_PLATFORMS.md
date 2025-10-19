# Platform Support Guide

This document provides a quick overview of all platform bindings available for Saorsa WebRTC.

## Supported Platforms

| Platform | Language | Status | Tests | Documentation |
|----------|----------|--------|-------|---------------|
| **Native** | Rust | ✅ Production | 78 | [Core Docs](./saorsa-webrtc-core/) |
| **iOS/macOS** | Swift | ✅ Production | 16 | [Swift README](./saorsa-webrtc-swift/README.md) |
| **Android/JVM** | Kotlin | ✅ Production | 16 | [Kotlin README](./saorsa-webrtc-kotlin/README.md) |
| **Desktop Apps** | Tauri | ✅ Production | 4 | [Tauri Docs](./saorsa-webrtc-tauri/) |
| **C/C++** | FFI | ✅ Production | 12 | [FFI Docs](./saorsa-webrtc-ffi/) |

**Total: 110 tests across all platforms**

## Quick Start by Platform

### Rust (Native)

```toml
[dependencies]
saorsa-webrtc-core = "0.2.1"
```

```rust
use saorsa_webrtc_core::prelude::*;

let service = WebRtcService::builder(signaling).build().await?;
let call_id = service.initiate_call(peer, constraints).await?;
```

### Swift (iOS/macOS)

```swift
// Package.swift
dependencies: [
    .package(url: "https://github.com/dirvine/saorsa-webrtc", from: "0.2.1")
]
```

```swift
import SaorsaWebRTC

let service = try SaorsaWebRTC(identity: "alice")
let callId = try service.call(peer: "bob")
```

[Full Swift Documentation →](./saorsa-webrtc-swift/README.md)

### Kotlin (Android/JVM)

```kotlin
// build.gradle.kts
dependencies {
    implementation("com.saorsalabs:saorsa-webrtc-kotlin:0.2.1")
}
```

```kotlin
import com.saorsalabs.webrtc.SaorsaWebRTC

SaorsaWebRTC("alice").use { service ->
    val callId = service.call("bob")
}
```

[Full Kotlin Documentation →](./saorsa-webrtc-kotlin/README.md)

### JavaScript/TypeScript (via Tauri)

```javascript
import { invoke } from '@tauri-apps/api/tauri';

await invoke('initialize', { identity: 'alice' });
const callId = await invoke('call', { peer: 'bob' });
```

### C/C++

```c
#include "saorsa_webrtc_ffi.h"

void* handle = saorsa_init("alice");
char* call_id = saorsa_call(handle, "bob");
saorsa_free_string(call_id);
saorsa_free(handle);
```

## Feature Matrix

| Feature | Rust | Swift | Kotlin | Tauri | C/FFI |
|---------|------|-------|--------|-------|-------|
| Initialize Service | ✅ | ✅ | ✅ | ✅ | ✅ |
| Initiate Call | ✅ | ✅ | ✅ | ✅ | ✅ |
| Get Call State | ✅ | ✅ | ✅ | ✅ | ✅ |
| End Call | ✅ | ✅ | ✅ | ✅ | ✅ |
| Error Handling | ✅ | ✅ | ✅ | ✅ | ✅ |
| Memory Safety | ✅ | ✅ | ✅ | ✅ | ✅ |
| Type Safety | ✅ | ✅ | ✅ | ✅ | ⚠️ |
| Mock Mode for Testing | N/A | ✅ | ✅ | N/A | N/A |

## Platform-Specific Features

### Swift (iOS/macOS)
- ✅ Swift Package Manager
- ✅ Automatic memory management (RAII)
- ✅ Type-safe error handling with throws
- ✅ Mock mode for unit testing
- ✅ iOS 14+ / macOS 11+ support

### Kotlin (Android/JVM)
- ✅ Gradle & Maven support
- ✅ AutoCloseable for resource management
- ✅ Sealed classes for type-safe errors
- ✅ Mock mode for unit testing
- ✅ Android API 24+ / JVM 17+ support

### Tauri (Desktop)
- ✅ Cross-platform desktop apps
- ✅ JavaScript/TypeScript API
- ✅ Async command handlers
- ✅ State management
- ✅ Windows, macOS, Linux support

### C/FFI (Universal)
- ✅ Standard C ABI
- ✅ Compatible with C++, Go, Python, etc.
- ✅ Explicit resource management
- ✅ Clear result codes
- ✅ Platform-independent

## Testing

Each platform has comprehensive tests:

```bash
# Rust
cargo test --workspace

# Swift
cd saorsa-webrtc-swift && swift test

# Kotlin
cd saorsa-webrtc-kotlin && ./gradlew test

# All Rust tests
cargo test --workspace -- --nocapture
```

## Build Requirements

### Rust
- Rust 1.70+
- cargo

### Swift
- Xcode 13+
- Swift 5.9+
- iOS 14+ / macOS 11+

### Kotlin
- JDK 17+
- Gradle 8.0+
- Android SDK 24+ (for Android)

### Tauri
- Rust 1.70+
- Node.js 16+
- Platform-specific build tools

## Architecture

```
┌─────────────────────────────────────────┐
│         Application Layer               │
├──────────┬──────────┬─────────┬─────────┤
│  Swift   │  Kotlin  │  Tauri  │  C/C++  │
│  (iOS)   │ (Android)│ (Desktop)│ (Native)│
└────┬─────┴────┬─────┴────┬────┴────┬────┘
     │          │          │         │
     └──────────┴────┬─────┴─────────┘
                     │
              ┌──────▼──────┐
              │  FFI Layer  │
              └──────┬──────┘
                     │
              ┌──────▼──────┐
              │  Rust Core  │
              │  (Native)   │
              └─────────────┘
```

## Documentation

- [Implementation Status](./IMPLEMENTATION_STATUS.md)
- [Platform Bindings Summary](./PLATFORM_BINDINGS_SUMMARY.md)
- [TDD Completion Summary](./TDD_COMPLETION_SUMMARY.md)
- [Final Completion Summary](./FINAL_COMPLETION_SUMMARY.md)

## License

AGPL-3.0

## Contributing

All contributions should follow the TDD methodology used throughout this project:
1. Write tests first
2. Implement to pass tests
3. Ensure zero panics/unwraps/expects
4. Document public APIs
5. Update platform documentation
