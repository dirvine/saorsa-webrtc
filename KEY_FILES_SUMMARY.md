# Key Files & Code Snippets Summary

## Directory Structure

```
/Users/davidirvine/Desktop/Devel/projects/saorsa-core/
├── Cargo.toml (WebRTC deps: lines 118-128)
├── src/
│   ├── lib.rs
│   ├── messaging/
│   │   ├── mod.rs (WebRTC imports: lines 22-23, 49-50)
│   │   ├── service.rs (WebRTC integration)
│   │   ├── webrtc/  ← MAIN IMPLEMENTATION (3,566 lines)
│   │   │   ├── mod.rs (360 lines)
│   │   │   ├── types.rs (352 lines)
│   │   │   ├── call_manager.rs (699 lines)
│   │   │   ├── signaling.rs (799 lines)
│   │   │   ├── media.rs (890 lines)
│   │   │   ├── integration_test.rs (466 lines)
│   │   │   └── tests/
│   │   │       ├── mod.rs
│   │   │       ├── call_flow_tests.rs
│   │   │       ├── multi_party_tests.rs
│   │   │       ├── media_streaming_tests.rs
│   │   │       └── quic_connectivity_tests.rs
│   │   ├── webrtc_quic_bridge.rs (470+ lines)
│   │   └── webrtc_tests.rs (50+ lines)
│   └── ...
└── tests/
    └── webrtc_quic_bridge_test.rs (80 lines)
```

## File-by-File Summary

### 1. Cargo.toml (lines 118-128)
**WebRTC Dependencies to Remove**:
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

**Replace with**:
```toml
saorsa-webrtc = { path = "../saorsa-webrtc", version = "0.1.0" }
```

---

### 2. src/messaging/mod.rs (lines 22-23, 49-50)

**Current - Module Declaration**:
```rust
pub mod composer;
pub mod database;
pub mod encryption;
pub mod key_exchange;
pub mod media;
#[cfg(any(test, feature = "mocks"))]
pub mod mocks;
pub mod network_config;
pub mod quic_media_streams;
pub mod reactions;
pub mod search;
pub mod service;
pub mod sync;
pub mod threads;
pub mod transport;
pub mod types;
pub mod user_handle;
pub mod user_resolver;
pub mod webrtc;                    // ← REMOVE
pub mod webrtc_quic_bridge;        // ← REMOVE
```

**Current - Type Re-exports (line 49-50)**:
```rust
pub use webrtc::{CallEvent, CallManager, WebRtcEvent, WebRtcService};
pub use webrtc_quic_bridge::{RtpPacket, StreamConfig, StreamType, WebRtcQuicBridge};
```

**After Migration**:
```rust
// Remove webrtc and webrtc_quic_bridge from pub mod list
// Add at re-exports section:
pub use saorsa_webrtc::{
    CallEvent, CallManager, WebRtcEvent, WebRtcService,
    SignalingEvent, SignalingHandler, SignalingState,
    MediaEvent, MediaStream, MediaStreamManager,
    RtpPacket, StreamConfig, StreamType, WebRtcQuicBridge,
    // All types from webrtc::types
    CallId, CallState, MediaConstraints, CallOffer, CallAnswer,
    IceCandidate, CallQualityMetrics, MultiPartyCall, CallArchitecture,
    RecordingConsent, ConsentStatus, NativeQuicConfiguration,
    SignalingMessage, CallSession, VideoResolution, AdaptationSettings,
};
```

---

### 3. src/messaging/webrtc/mod.rs (360 lines)

**Main Components**:
```rust
pub mod call_manager;
pub mod media;
pub mod signaling;
pub mod types;

pub struct WebRtcService {
    local_identity: FourWordAddress,
    signaling: Arc<SignalingHandler>,
    media: Arc<MediaStreamManager>,
    call_manager: Arc<CallManager>,
    event_sender: broadcast::Sender<WebRtcEvent>,
}

pub struct WebRtcConfig {
    pub quic_config: NativeQuicConfiguration,
    pub default_constraints: MediaConstraints,
    pub echo_cancellation: bool,
    pub noise_suppression: bool,
    pub auto_gain_control: bool,
    pub max_call_duration_hours: u32,
}
```

**Key Methods**:
- `new(local_identity, dht_client) -> Result<Self>`
- `start() -> Result<()>`
- `initiate_call(callee, constraints) -> Result<CallId>`
- `accept_call(call_id, constraints) -> Result<()>`
- `reject_call(call_id) -> Result<()>`
- `end_call(call_id) -> Result<()>`
- `get_call_state(call_id) -> Option<CallState>`
- `subscribe_events() -> broadcast::Receiver<WebRtcEvent>`

---

### 4. src/messaging/webrtc/types.rs (352 lines)

**Core Types** (all should be exported from saorsa-webrtc):
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CallId(pub Uuid);

#[derive(Debug, Clone)]
pub struct MediaConstraints {
    pub audio: bool,
    pub video: bool,
    pub screen_share: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaType { Audio, Video, ScreenShare, DataChannel }

#[derive(Debug, Clone)]
pub struct CallOffer {
    pub call_id: CallId,
    pub caller: FourWordAddress,
    pub callee: FourWordAddress,
    pub sdp: String,
    pub media_types: Vec<MediaType>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CallQualityMetrics {
    pub rtt_ms: u32,
    pub packet_loss_percent: f32,
    pub jitter_ms: u32,
    pub bandwidth_kbps: u32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallState { Idle, Calling, Connecting, Connected, Ending, Failed }

#[derive(Debug, Clone)]
pub struct CallSession {
    pub call_id: CallId,
    pub participants: Vec<UserHandle>,
    pub state: CallState,
    pub media_constraints: MediaConstraints,
    pub quality_metrics: Vec<CallQualityMetrics>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallArchitecture { Mesh, SFU }

#[derive(Debug, Clone)]
pub struct NativeQuicConfiguration {
    pub dht_discovery: bool,
    pub hole_punching: bool,
}

#[derive(Debug, Clone)]
pub enum WebRtcEvent {
    Signaling(SignalingEvent),
    Media(MediaEvent),
    Call(CallEvent),
}

#[derive(Debug, Clone)]
pub enum CallEvent {
    IncomingCall { offer: CallOffer },
    CallInitiated { call_id: CallId, callee: UserHandle, constraints: MediaConstraints },
    CallAccepted { call_id: CallId, answer: CallAnswer },
    CallRejected { call_id: CallId },
    CallEnded { call_id: CallId },
    ConnectionEstablished { call_id: CallId },
    ConnectionFailed { call_id: CallId, error: String },
    QualityChanged { call_id: CallId, metrics: CallQualityMetrics },
}
```

---

### 5. src/messaging/webrtc/signaling.rs (799 lines)

**Key Structure**:
```rust
pub struct SignalingHandler {
    local_identity: FourWordAddress,
    _dht_client: Arc<RwLock<DhtCoreEngine>>,  // DHT integration point
    sessions: Arc<RwLock<HashMap<CallId, SignalingSession>>>,
    event_sender: broadcast::Sender<SignalingEvent>,
    _message_receiver: Arc<RwLock<Option<mpsc::Receiver<SignalingMessage>>>>,
    quic_config: NativeQuicConfiguration,
}

pub enum SignalingEvent {
    OfferReceived { offer: CallOffer },
    AnswerReceived { answer: CallAnswer },
    IceCandidateReceived { candidate: IceCandidate },
    CallEnded { call_id: CallId },
}
```

**Key Methods**:
- `new(local_identity, dht_client) -> Self`
- `start() -> Result<()>`
- `validate_sdp(sdp: &str) -> Result<()>`
- `setup_dht_listener() -> Result<()>`

---

### 6. src/messaging/webrtc/media.rs (890 lines)

**Key Structure**:
```rust
pub struct MediaStreamManager {
    local_identity: FourWordAddress,
    streams: Arc<RwLock<HashMap<CallId, MediaStream>>>,
    audio_processor: Arc<AudioProcessor>,
    video_processor: Arc<VideoProcessor>,
    event_sender: broadcast::Sender<MediaEvent>,
    constraints: Arc<RwLock<MediaConstraints>>,
}

pub enum MediaEvent {
    DevicesInitialized,
    StreamCreated { call_id: CallId, stream: MediaStream },
    TrackAdded { call_id: CallId, track: String },
    StreamClosed { call_id: CallId },
}

pub struct MediaStream {
    pub call_id: CallId,
    pub audio_track: Option<AudioTrack>,
    pub video_track: Option<VideoTrack>,
    pub screen_track: Option<VideoTrack>,
}
```

**WebRTC Integration**:
```rust
use webrtc::api::media_engine::MediaEngine;
use webrtc::track::track_local::track_local_static_rtp::TrackLocalStaticRTP;
```

---

### 7. src/messaging/webrtc/call_manager.rs (699 lines)

**Key Structure**:
```rust
pub struct CallManager {
    local_identity: FourWordAddress,
    signaling: Arc<SignalingHandler>,
    media: Arc<MediaStreamManager>,
    calls: Arc<RwLock<HashMap<CallId, CallSession>>>,
    network_adapter: Arc<NetworkAdapter>,
    event_sender: broadcast::Sender<CallEvent>,
    _cleanup_handle: tokio::task::JoinHandle<()>,
}

pub struct NetworkAdapter;
```

**Key Methods**:
- `new(local_identity, signaling, media) -> Result<Self>`
- `start() -> Result<()>`
- `initiate_call(callee, constraints) -> Result<CallId>`
- `accept_call(call_id, constraints) -> Result<()>`
- `reject_call(call_id) -> Result<()>`
- `end_call(call_id) -> Result<()>`
- `get_call_state(call_id) -> Option<CallState>`
- `get_quality_metrics(call_id) -> Option<CallQualityMetrics>`

---

### 8. src/messaging/webrtc_quic_bridge.rs (470+ lines)

**Key Structures**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtpPacket {
    pub version: u8,
    pub padding: bool,
    pub extension: bool,
    pub csrc_count: u8,
    pub marker: bool,
    pub payload_type: u8,
    pub sequence_number: u16,
    pub timestamp: u32,
    pub ssrc: u32,
    pub payload: Vec<u8>,
    pub stream_type: StreamType,
}

pub enum StreamType { Audio, Video, ScreenShare, DataChannel }

pub struct WebRtcQuicBridge {
    network_node: Arc<P2PNetworkNode>,
    // peer connection management
}
```

**Integration Point**:
```rust
use crate::transport::ant_quic_adapter::P2PNetworkNode;
use ant_quic::nat_traversal_api::PeerId;
```

---

### 9. tests/webrtc_quic_bridge_test.rs (80 lines)

**Tests Coverage**:
```rust
#[tokio::test]
async fn test_bridge_creation() -> Result<()>

#[tokio::test]
async fn test_peer_connection_simulation() -> Result<()>
// More tests...
```

**Setup**:
```rust
async fn create_test_network_node() -> Result<Arc<P2PNetworkNode>> {
    let config = QuicNodeConfig {
        role: EndpointRole::Bootstrap,
        bootstrap_nodes: vec![],
        enable_coordinator: false,
        // ... more config
    };
    let node = P2PNetworkNode::new_with_config(addr, config).await?;
    Ok(Arc::new(node))
}
```

---

## Integration Points Summary

### DHT Integration
- **File**: `src/messaging/webrtc/signaling.rs`
- **Type**: `Arc<RwLock<DhtCoreEngine>>`
- **Usage**: Peer discovery and signaling message routing

### QUIC Integration  
- **File**: `src/messaging/webrtc_quic_bridge.rs`
- **Type**: `Arc<P2PNetworkNode>` from `ant_quic_adapter`
- **Usage**: Media transport via QUIC streams

### Event System
- **Primary Channel**: `broadcast::Sender<WebRtcEvent>` in WebRtcService
- **Sub-channels**: SignalingHandler, MediaStreamManager, CallManager
- **Event Types**: WebRtcEvent → Signaling/Media/Call variant enums

---

## Files to Delete When Migrating

1. ✂️ `src/messaging/webrtc/` (entire directory - 3,566 lines)
   - mod.rs (360)
   - types.rs (352)
   - call_manager.rs (699)
   - signaling.rs (799)
   - media.rs (890)
   - integration_test.rs (466)
   - tests/ (subdirectory with 4 test files)

2. ✂️ `src/messaging/webrtc_quic_bridge.rs` (470+ lines)

3. ✂️ `src/messaging/webrtc_tests.rs` (50+ lines)

4. ✂️ `tests/webrtc_quic_bridge_test.rs` (80 lines)

---

## Files to Modify

1. ✏️ `Cargo.toml` - Remove 10 WebRTC deps, add saorsa-webrtc
2. ✏️ `src/messaging/mod.rs` - Update imports (2 lines modify, 2 lines remove)
3. ✏️ `src/messaging/service.rs` - Update type imports (if WebRTC is used)

