# Saorsa WebRTC Swift Bindings

Swift bindings for the Saorsa WebRTC library, providing a native Swift API for iOS and macOS applications.

## Features

- ✅ Type-safe Swift API
- ✅ Automatic memory management
- ✅ Comprehensive error handling
- ✅ Full test coverage
- ✅ iOS 14+ and macOS 11+ support

## Installation

### Swift Package Manager

Add to your `Package.swift`:

```swift
dependencies: [
    .package(url: "https://github.com/dirvine/saorsa-webrtc", from: "0.2.1")
]
```

Or in Xcode: File → Add Packages → Enter repository URL

## Usage

```swift
import SaorsaWebRTC

// Initialize the service
let service = try SaorsaWebRTC(identity: "alice-bob-charlie-david")

// Initiate a call
let callId = try service.call(peer: "bob-smith-jones-wilson")
print("Call initiated: \(callId)")

// Check call state
let state = try service.getCallState(callId: callId)
print("Call state: \(state)")

// End the call
try service.endCall(callId: callId)
```

## API Reference

### `SaorsaWebRTC`

#### Initialization

```swift
init(identity: String) throws
```

Initialize the WebRTC service with a four-word identity string.

**Throws:** `SaorsaError.invalidParameter` if identity is empty

#### Methods

```swift
func call(peer: String) throws -> String
```

Initiate a call to a peer. Returns a unique call ID.

**Parameters:**
- `peer`: The peer's identity string

**Returns:** Call ID for tracking this call

**Throws:** 
- `SaorsaError.invalidParameter` if peer is empty
- `SaorsaError.connectionFailed` if call initiation fails

```swift
func getCallState(callId: String) throws -> CallState
```

Get the current state of a call.

**Parameters:**
- `callId`: The call ID from `call()`

**Returns:** Current `CallState`

**Throws:** 
- `SaorsaError.invalidParameter` if callId is empty
- `SaorsaError.callNotFound` if call doesn't exist

```swift
func endCall(callId: String) throws
```

End an active call.

**Parameters:**
- `callId`: The call ID to end

**Throws:** 
- `SaorsaError.invalidParameter` if callId is empty
- `SaorsaError.callNotFound` if call doesn't exist

### `CallState`

Enum representing the state of a call:

- `.connecting` - Call is being established
- `.active` - Call is connected
- `.ended` - Call has ended normally
- `.failed` - Call failed

### `SaorsaError`

Error types that can occur:

- `.invalidParameter(String)` - Invalid input parameter
- `.outOfMemory` - Memory allocation failed
- `.notInitialized` - Service not initialized
- `.alreadyInitialized` - Service already initialized
- `.connectionFailed` - Connection could not be established
- `.internalError` - Internal error occurred
- `.invalidHandle` - Invalid service handle
- `.callNotFound` - Specified call not found

## Testing

Run tests:

```bash
swift test
```

## Requirements

- iOS 14.0+ / macOS 11.0+
- Xcode 13.0+
- Swift 5.9+

## License

AGPL-3.0
