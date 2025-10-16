# Saorsa-Core WebRTC Implementation Analysis

## Executive Summary

The saorsa-core codebase contains a comprehensive WebRTC implementation (~3,566 lines) designed for P2P voice/video calling integrated with:
- DHT-based peer discovery (via DhtCoreEngine)
- Native QUIC transport (via ant-quic)
- Codec negotiation via SDP
- Media streaming and quality metrics

**Key Insight**: The current implementation uses standard webrtc crate dependencies but bypasses traditional ICE/STUN/TURN by leveraging QUIC for transport, creating a custom bridge between WebRTC codecs and ant-quic networking.

---

## 1. Current WebRTC Implementation Structure

### Main Directory
**Location**: `/Users/davidirvine/Desktop/Devel/projects/saorsa-core/src/messaging/webrtc/`

### Core Files (3,566 total lines)

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 360 | Main WebRTC service orchestrator |
| `types.rs` | 352 | Data structures and enums |
| `call_manager.rs` | 699 | High-level call state management |
| `signaling.rs` | 799 | Offer/answer exchange via DHT |
| `media.rs` | 890 | Audio/video stream capture/processing |
| `integration_test.rs` | 466 | Test runner and integration tests |
| **Subtotal** | **3,566** | **Core implementation** |

### Additional Related Files
- `webrtc_quic_bridge.rs` (470+ lines): Bridges WebRTC RTP packets to QUIC streams
- `webrtc_tests.rs` (50+ lines): Basic unit tests
- `webrtc/tests/` directory: Test suites (multi_party, call_flow, media_streaming, quic_connectivity)

---

## 2. Current WebRTC Dependencies

### Cargo.toml WebRTC-Related Entries (Lines 118-128)
```toml
# WebRTC dependencies for voice/video calling - updated for security
webrtc = "0.13"
webrtc-ice = "0.13"
webrtc-media = "0.10"
webrtc-sctp = "0.12"
webrtc-srtp = "0.15"
webrtc-dtls = "0.12"
webrtc-data = "0.11"
interceptor = "0.14"
rtcp = "0.13"
rtp = "0.13"
```

**Total**: 10 WebRTC-related crate dependencies

### Other Supporting Dependencies Used
- `tokio` - Async runtime
- `ant-quic` - QUIC transport (0.10.0 with pqc feature)
- `saorsa-pqc` - Post-quantum cryptography
- `chrono` - Time handling
- `serde` / `serde_json` - Serialization
- `tracing` - Logging

---

## 3. Module Organization & Integration Points

### Module Import Structure
**File**: `src/messaging/mod.rs` (Lines 22-23, 49-50)

```rust
pub mod webrtc;
pub mod webrtc_quic_bridge;

pub use webrtc::{CallEvent, CallManager, WebRtcEvent, WebRtcService};
pub use webrtc_quic_bridge::{RtpPacket, StreamConfig, StreamType, WebRtcQuicBridge};
```

### Messaging Service Integration
**File**: `src/messaging/service.rs` (Line ~52+)

The `MessagingService` struct optionally includes WebRTC support:
```rust
pub struct MessagingService {
    identity: FourWordAddress,
    store: MessageStore,
    transport: Arc<MessageTransport>,
    // ... other fields
}
```

---

## 4. Key Classes & Their Responsibilities

### A. WebRtcService (mod.rs - Main Orchestrator)
**Responsibilities**:
- Coordinates all WebRTC components
- Manages signaling handler, media manager, and call manager
- Broadcasts WebRTC events
- Provides high-level API for call initiation/acceptance/rejection

**Key Methods**:
```rust
pub async fn new(local_identity, dht_client) -> Result<Self>
pub async fn start() -> Result<()>
pub async fn initiate_call(callee, constraints) -> Result<CallId>
pub async fn accept_call(call_id, constraints) -> Result<()>
pub async fn reject_call(call_id) -> Result<()>
pub async fn end_call(call_id) -> Result<()>
pub fn subscribe_events() -> broadcast::Receiver<WebRtcEvent>
```

**Configuration**: `WebRtcConfig` and `WebRtcServiceBuilder` for setup

### B. SignalingHandler (signaling.rs)
**Responsibilities**:
- Manages WebRTC signaling protocol (offer/answer/ICE)
- Validates SDP content
- Routes signaling messages via DHT
- Maintains active signaling sessions

**Key Components**:
- SDP validation and sanitization
- Native QUIC configuration (DHT discovery + hole punching)
- Session lifetime management with cleanup tasks
- DHT listener for incoming messages

**Integration**: Uses `DhtCoreEngine` for message exchange

### C. CallManager (call_manager.rs)
**Responsibilities**:
- High-level call state management
- Coordinates signaling and media streams
- Monitors call quality metrics
- Handles network adaptation
- Manages active call sessions

**Key State Machine**:
- `Idle` → `Calling` → `Connecting` → `Connected` → `Ending` → `Failed`

**NetworkAdapter**: Provides quality monitoring and adaptation strategies

### D. MediaStreamManager (media.rs)
**Responsibilities**:
- Audio/video device initialization
- Media stream creation and lifecycle
- Track management (audio, video, screen share)
- Audio/video processing pipelines
- Media event broadcasting

**Device Support**:
- Audio: mic input, audio processing
- Video: camera input, screen capture
- Processing: echo cancellation, noise suppression, gain control

### E. WebRtcQuicBridge (webrtc_quic_bridge.rs)
**Responsibilities**:
- Serializes RTP packets for QUIC transmission
- Maps WebRTC media to QUIC streams
- Manages peer connections via ant-quic
- Tracks connection statistics

**Key Classes**:
- `RtpPacket`: RTP header + payload with stream type classification
- `WebRtcQuicBridge`: Main bridge implementation
- `PeerStats`: Connection quality metrics

---

## 5. Data Structures & Types

### Core Types (types.rs)
```rust
// Call identification
struct CallId(Uuid)

// Constraints for calls
struct MediaConstraints {
    audio: bool,
    video: bool,
    screen_share: bool,
}

// Call signaling
struct CallOffer { call_id, caller, callee, sdp, ... }
struct CallAnswer { call_id, sdp, accepted, ... }
struct IceCandidate { call_id, candidate, sdp_mid, ... }

// Call state
enum CallState { Idle, Calling, Connecting, Connected, Ending, Failed }

// Quality metrics
struct CallQualityMetrics { rtt_ms, packet_loss_percent, jitter_ms, bandwidth_kbps }

// Call session
struct CallSession {
    call_id,
    participants,
    state,
    media_constraints,
    quality_metrics: Vec<CallQualityMetrics>,
}

// Multi-party support
enum CallArchitecture { Mesh, SFU }
struct MultiPartyCall { call_id, participants, architecture, ... }

// Recording consent
struct RecordingConsent { call_id, requester, participants }
enum ConsentStatus { Pending, Granted, Denied, Revoked }

// Native QUIC config
struct NativeQuicConfiguration {
    dht_discovery: bool,
    hole_punching: bool,
}

// Events
enum WebRtcEvent { Signaling(SignalingEvent), Media(MediaEvent), Call(CallEvent) }
```

---

## 6. DHT Integration Points

### Where DHT is Used
1. **Peer Discovery** - Finding remote peer addresses
2. **Signaling Message Routing** - Sending offer/answer/ICE via DHT
3. **Session Establishment** - DHT listener in SignalingHandler

### DHT Integration Code
**File**: `src/messaging/webrtc/signaling.rs` (Lines 1-45)

```rust
pub struct SignalingHandler {
    local_identity: FourWordAddress,
    _dht_client: Arc<RwLock<DhtCoreEngine>>,  // <-- DHT integration
    sessions: Arc<RwLock<HashMap<CallId, SignalingSession>>>,
    quic_config: NativeQuicConfiguration,
}
```

**Method**: `setup_dht_listener()` - Sets up DHT listener for incoming signaling messages

---

## 7. QUIC Integration & WebRTC-QUIC Bridge

### Architecture
The implementation uses **native QUIC connectivity** instead of traditional ICE/STUN/TURN:

1. **Codec Negotiation**: Standard WebRTC SDP for codec agreement
2. **Connection Establishment**: QUIC (via ant-quic) + coordinator hole punching
3. **Media Transport**: RTP over QUIC streams

### Bridge Components
**File**: `src/messaging/webrtc_quic_bridge.rs`

```rust
// Maps WebRTC to QUIC
struct RtpPacket {
    version, padding, extension, csrc_count, marker,
    payload_type, sequence_number, timestamp, ssrc,
    payload,
    stream_type: StreamType,  // <-- Classification for QUIC
}

enum StreamType { Audio, Video, ScreenShare, DataChannel }

struct WebRtcQuicBridge {
    network_node: Arc<P2PNetworkNode>,  // ant-quic integration
    connections: HashMap<PeerId, ConnectionState>,
}
```

### Integration with ant-quic
- Uses `P2PNetworkNode` from `ant_quic_adapter`
- Manages `PeerId` connections
- Handles endpoint role (Bootstrap, Peer, Coordinator)

---

## 8. Testing Infrastructure

### Test Files Location
```
tests/
├── webrtc_quic_bridge_test.rs          (80 lines, integration tests)
src/messaging/
├── webrtc_tests.rs                     (50 lines, unit tests)
└── webrtc/
    ├── integration_test.rs             (466 lines, test runner)
    └── tests/
        ├── mod.rs
        ├── call_flow_tests.rs
        ├── multi_party_tests.rs
        ├── media_streaming_tests.rs
        └── quic_connectivity_tests.rs
```

### Test Coverage Areas
1. **QUIC Configuration** - DHT discovery, hole punching validation
2. **SDP Validation** - Security and structure checks
3. **Signaling** - Offer/answer message serialization
4. **Media Constraints** - Audio/video/screen share configuration
5. **Quality Metrics** - RTT, packet loss, jitter, bandwidth
6. **Call Management** - State transitions, multi-party
7. **Recording Consent** - Privacy consent flows
8. **Connectivity** - QUIC-based peer connections

### Test Fixtures & Utilities
- `create_test_rtp_packet()` - RTP packet factory
- `create_test_network_node()` - QUIC node setup
- `WebRtcQuicTestRunner` - Integration test orchestrator

---

## 9. Event System & Broadcasting

### Event Types
```rust
enum WebRtcEvent {
    Signaling(SignalingEvent),   // Offer/Answer/ICE events
    Media(MediaEvent),           // Device/stream events
    Call(CallEvent),             // Call state events
}

enum SignalingEvent {
    OfferReceived { offer: CallOffer },
    AnswerReceived { answer: CallAnswer },
    IceCandidateReceived { candidate: IceCandidate },
    CallEnded { call_id: CallId },
}

enum MediaEvent {
    DevicesInitialized,
    StreamCreated { call_id, stream },
    TrackAdded { call_id, track },
    StreamClosed { call_id },
}

enum CallEvent {
    IncomingCall { offer },
    CallInitiated { call_id, callee, constraints },
    CallAccepted { call_id, answer },
    CallRejected { call_id },
    CallEnded { call_id },
    ConnectionEstablished { call_id },
    ConnectionFailed { call_id, error },
    QualityChanged { call_id, metrics },
}
```

### Broadcasting Architecture
- `WebRtcService` maintains main `broadcast::Sender<WebRtcEvent>`
- Sub-components (Signaling, Media, CallManager) have their own channels
- Event forwarding tasks merge sub-component events into main channel

---

## 10. What Needs to Be Updated to Use saorsa-webrtc Crate

### 1. Cargo.toml Changes
**Remove**:
```toml
webrtc = "0.13"
webrtc-ice = "0.13"
webrtc-media = "0.10"
webrtc-sctp = "0.12"
webrtc-srtp = "0.15"
webrtc-dtls = "0.12"
webrtc-data = "0.11"
interceptor = "0.14"
rtcp = "0.13"
rtp = "0.13"
```

**Add**:
```toml
saorsa-webrtc = { path = "../saorsa-webrtc", version = "0.1.0" }
```

### 2. Import Updates
**File**: `src/messaging/mod.rs` (Lines 22-23)

**Current**:
```rust
pub mod webrtc;
pub mod webrtc_quic_bridge;
pub use webrtc::{CallEvent, CallManager, WebRtcEvent, WebRtcService};
pub use webrtc_quic_bridge::{RtpPacket, StreamConfig, StreamType, WebRtcQuicBridge};
```

**New**:
```rust
// Remove local modules - use from crate instead
pub use saorsa_webrtc::{
    CallEvent, CallManager, WebRtcEvent, WebRtcService,
    RtpPacket, StreamConfig, StreamType, WebRtcQuicBridge,
    // ... other types
};
```

### 3. Module Removals
**Delete**:
- `src/messaging/webrtc/` (entire directory)
- `src/messaging/webrtc_quic_bridge.rs`
- `src/messaging/webrtc_tests.rs`
- `tests/webrtc_quic_bridge_test.rs`

### 4. Service Integration Updates
**File**: `src/messaging/service.rs`

**Current** (if WebRTC is optionally included):
```rust
use crate::messaging::webrtc::WebRtcService;
```

**New**:
```rust
use saorsa_webrtc::WebRtcService;
```

### 5. Type Re-exports
**File**: `src/messaging/types.rs` or relevant location

Ensure these types are properly imported from saorsa-webrtc:
- `CallId`, `CallState`, `MediaConstraints`
- `CallOffer`, `CallAnswer`, `IceCandidate`
- `CallQualityMetrics`, `CallSession`
- `WebRtcEvent`, `CallEvent`, `SignalingEvent`, `MediaEvent`
- All other public types

### 6. DHT Integration Point
**Key File**: Current implementation passes `Arc<RwLock<DhtCoreEngine>>` to WebRTC components

**Ensure saorsa-webrtc**:
- Accepts DhtCoreEngine or trait-based abstraction
- Maintains compatibility with current DHT integration
- Does NOT hard-code specific DHT implementation

---

## 11. Files Requiring Updates (Minimal Set)

### Critical Files to Update
1. **`src/messaging/mod.rs`** - Import reorg (lines 22-23, 49-50)
2. **`src/messaging/service.rs`** - Type imports (if WebRTC used)
3. **`Cargo.toml`** - Dependency replacement (lines 118-128)

### Files to Remove
1. `src/messaging/webrtc/` (entire directory, 3,566 lines)
2. `src/messaging/webrtc_quic_bridge.rs`
3. `src/messaging/webrtc_tests.rs`
4. `tests/webrtc_quic_bridge_test.rs`

### Minimal Scope
- **3 files to modify** (Cargo.toml, mod.rs, service.rs)
- **4 files/directories to delete** (~3,700+ lines removed)
- **Zero test modifications** needed (tests will use new crate's types)

---

## 12. Dependency Analysis

### saorsa-core WebRTC Dependencies (10 crates)
All 10 will be replaced by saorsa-webrtc:
- webrtc (core)
- webrtc-ice, webrtc-media, webrtc-sctp, webrtc-srtp, webrtc-dtls, webrtc-data (sub-crates)
- interceptor, rtcp, rtp (utilities)

### Maintained Dependencies (NOT removed)
These are used by both saorsa-core and saorsa-webrtc:
- tokio, async-trait, futures
- serde, serde_json, serde_cbor
- chrono, uuid
- anyhow, thiserror, tracing
- ant-quic, four-word-networking
- saorsa-pqc

---

## 13. Architecture Summary

### Current Implementation Stack
```
┌─────────────────────────────────┐
│   saorsa-core (main crate)      │
├─────────────────────────────────┤
│  messaging::WebRtcService       │
│  ├─ SignalingHandler (DHT)      │
│  ├─ MediaStreamManager          │
│  ├─ CallManager                 │
│  └─ WebRtcQuicBridge           │
├─────────────────────────────────┤
│  Dependencies:                  │
│  ├─ webrtc (0.13) & 9 others   │
│  ├─ ant-quic (QUIC transport)   │
│  ├─ DhtCoreEngine (discovery)   │
│  └─ four-word-networking        │
└─────────────────────────────────┘
```

### Post-saorsa-webrtc Integration
```
┌─────────────────────────────────┐
│   saorsa-core (main crate)      │
├─────────────────────────────────┤
│  messaging::* (reexports)       │
│  └─ use saorsa_webrtc::*        │
├─────────────────────────────────┤
│  saorsa-webrtc (new dep)        │
│  ├─ WebRtcService              │
│  ├─ SignalingHandler           │
│  ├─ MediaStreamManager         │
│  ├─ CallManager                │
│  └─ WebRtcQuicBridge           │
├─────────────────────────────────┤
│  Shared Deps:                   │
│  ├─ ant-quic                    │
│  ├─ DhtCoreEngine               │
│  └─ webrtc crates (internal)    │
└─────────────────────────────────┘
```

---

## 14. Key Findings & Recommendations

### Strengths of Current Implementation
1. Clean separation of concerns (Signaling, Media, Call Management)
2. Comprehensive event system with broadcasting
3. QUIC-native transport (bypasses ICE/STUN complexity)
4. DHT-based peer discovery (no centralized signaling)
5. Multi-party call support (Mesh + SFU architectures)
6. Quality metrics and network adaptation
7. Recording consent management

### Extraction Best Practices
1. **Preserve the abstraction**: saorsa-webrtc should accept DHT and QUIC as pluggable dependencies
2. **Maintain event contracts**: Keep the same WebRtcEvent/CallEvent/SignalingEvent enums
3. **Keep type definitions**: Export all types from types.rs
4. **Minimize changes**: saorsa-core should only update imports, not logic

### Testing Strategy Post-Migration
1. Run existing `webrtc_quic_bridge_test.rs` with new crate
2. Verify WebRtcService creation and lifecycle
3. Test call flows (initiate, accept, end)
4. Validate media stream creation
5. Check event broadcasting

---

## 15. Integration Checklist

Before committing saorsa-webrtc as dependency:

- [ ] saorsa-webrtc crate builds with zero warnings
- [ ] All WebRTC tests pass
- [ ] DHT integration point is abstracted (not hardcoded)
- [ ] QUIC integration works with ant-quic
- [ ] Type exports match saorsa-core expectations
- [ ] saorsa-core can successfully import all types
- [ ] saorsa-core tests pass with new dependency
- [ ] No circular dependencies introduced
- [ ] Version aligned (both use same tokio, serde, etc.)

---

**Analysis Complete** - Ready for crate extraction and integration.
