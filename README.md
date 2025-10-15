# Saorsa WebRTC

A WebRTC implementation over ant-quic transport with pluggable signaling mechanisms.

## Overview

`saorsa-webrtc` provides a WebRTC implementation that uses **ant-quic as the transport layer** instead of traditional ICE/STUN/TURN protocols. This approach leverages QUIC's built-in NAT traversal, post-quantum cryptography, and multiplexing capabilities while maintaining WebRTC's media streaming features.

## Key Features

- **Native QUIC Transport**: Uses ant-quic for reliable, encrypted connections with automatic NAT traversal
- **Transport-Agnostic Signaling**: Pluggable signaling layer supporting multiple backends:
  - DHT-based signaling (for saorsa-core integration)
  - Gossip-based rendezvous (for communitas integration)
  - Custom transport implementations via `SignalingTransport` trait
- **Post-Quantum Cryptography**: Built-in PQC support via ant-quic
- **Generic Peer Identity**: Abstracted peer identification via `PeerIdentity` trait
- **High Performance**: Low-latency media streaming with configurable QoS parameters
- **Type-Safe**: Generic over identity and transport types with full type safety

## Architecture

### Core Components

1. **Signaling Layer** (`signaling.rs`)
   - `SignalingTransport` trait for pluggable transport mechanisms
   - SDP offer/answer exchange
   - ICE candidate negotiation
   - Connection state management

2. **Media Management** (`media.rs`)
   - Audio/video device enumeration
   - Media stream management
   - Track handling
   - Device event notifications

3. **Call Management** (`call.rs`)
   - Call state machine (Idle → Calling → Connecting → Connected → Ending)
   - Call initiation and acceptance
   - Call lifecycle management
   - Event broadcasting

4. **Transport Integration** (`transport.rs`)
   - ant-quic transport adapter
   - Endpoint discovery
   - Message routing

5. **QUIC Bridge** (`quic_bridge.rs`, `quic_streams.rs`)
   - RTP packet to QUIC stream translation
   - QoS parameter management (audio: 50ms, video: 150ms, screen share: 200ms)
   - Media prioritization

### Generic Architecture

The library is generic over two key traits:

```rust
pub trait PeerIdentity:
    Clone + Debug + Display + Serialize + Deserialize + Send + Sync + 'static
{
    fn to_string_repr(&self) -> String;
    fn from_string_repr(s: &str) -> anyhow::Result<Self>;
    fn unique_id(&self) -> String;
}

pub trait SignalingTransport: Send + Sync {
    type PeerId: Clone + Send + Sync + Debug + Display + FromStr;
    type Error: std::error::Error + Send + Sync + 'static;

    async fn send_message(&self, peer: &Self::PeerId, message: SignalingMessage)
        -> Result<(), Self::Error>;
    async fn receive_message(&self)
        -> Result<(Self::PeerId, SignalingMessage), Self::Error>;
    async fn discover_peer_endpoint(&self, peer: &Self::PeerId)
        -> Result<Option<SocketAddr>, Self::Error>;
}
```

This allows the library to work with different peer identity schemes (e.g., FourWordAddress in saorsa-core, gossip IDs in communitas) and different signaling mechanisms (DHT, gossip, centralized servers).

## Usage

### Basic Example

```rust
use saorsa_webrtc::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create WebRTC service with string-based peer identity
    let service = WebRtcService::<PeerIdentityString, AntQuicTransport>::builder()
        .with_identity("alice-bob-charlie-david")
        .build()
        .await?;

    // Start the service
    service.start().await?;

    // Subscribe to WebRTC events
    let mut events = service.subscribe_events();

    // Initiate a video call
    let call_id = service.initiate_call(
        "eve-frank-grace-henry",
        MediaConstraints::video_call()
    ).await?;

    // Handle events
    while let Ok(event) = events.recv().await {
        match event {
            WebRtcEvent::IncomingCall { from, call_id, constraints } => {
                println!("Incoming call from {}: {:?}", from, constraints);
                service.accept_call(&call_id).await?;
            }
            WebRtcEvent::CallConnected { call_id } => {
                println!("Call {} connected", call_id);
            }
            WebRtcEvent::CallEnded { call_id, reason } => {
                println!("Call {} ended: {:?}", call_id, reason);
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
```

### Integration with saorsa-core (DHT Signaling)

```rust
use saorsa_webrtc::prelude::*;
use saorsa_core::FourWordAddress;

// Use DHT-based signaling transport
let service = WebRtcService::<FourWordAddress, DhtSignalingTransport>::builder()
    .with_identity(my_four_word_address)
    .with_transport(dht_transport)
    .build()
    .await?;
```

### Integration with communitas (Gossip Signaling)

```rust
use saorsa_webrtc::prelude::*;
use communitas::GossipIdentity;

// Use gossip-based rendezvous signaling
let service = WebRtcService::<GossipIdentity, GossipSignalingTransport>::builder()
    .with_identity(my_gossip_identity)
    .with_transport(gossip_transport)
    .build()
    .await?;
```

### Custom Signaling Transport

Implement the `SignalingTransport` trait for your own signaling mechanism:

```rust
use saorsa_webrtc::{SignalingTransport, SignalingMessage};
use async_trait::async_trait;

pub struct MyCustomTransport {
    // Your transport state
}

#[async_trait]
impl SignalingTransport for MyCustomTransport {
    type PeerId = String;
    type Error = MyError;

    async fn send_message(&self, peer: &String, message: SignalingMessage)
        -> Result<(), MyError>
    {
        // Your implementation
        Ok(())
    }

    async fn receive_message(&self)
        -> Result<(String, SignalingMessage), MyError>
    {
        // Your implementation
        todo!()
    }

    async fn discover_peer_endpoint(&self, peer: &String)
        -> Result<Option<SocketAddr>, MyError>
    {
        // Your implementation
        Ok(None)
    }
}
```

## Dependencies

This library depends on:

- **ant-quic**: QUIC transport with NAT traversal and PQC support (path: `../ant-quic`)
- **tokio**: Async runtime
- **webrtc**: WebRTC protocol implementation
- **serde**: Serialization/deserialization
- **async-trait**: Async trait support

## Differences from Traditional WebRTC

Traditional WebRTC uses:
- ICE for connectivity establishment
- STUN/TURN for NAT traversal
- DTLS for encryption
- Centralized or P2P signaling servers

Saorsa WebRTC uses:
- **ant-quic for connectivity** (built-in NAT traversal via hole punching)
- **No STUN/TURN required** (QUIC handles NAT traversal)
- **QUIC encryption** (with optional post-quantum cryptography)
- **Pluggable signaling** (DHT, gossip, or custom)

This approach provides:
- Simpler deployment (no STUN/TURN infrastructure)
- Better security (PQC support, modern crypto)
- More flexibility (pluggable signaling and identity)
- Improved performance (QUIC's congestion control and multiplexing)

## Project Structure

```
src/
├── lib.rs              # Public API and module exports
├── identity.rs         # PeerIdentity trait and implementations
├── types.rs            # Core data structures (CallId, MediaConstraints, etc.)
├── signaling.rs        # Signaling protocol and transport abstraction
├── media.rs            # Media stream management
├── call.rs             # Call state management
├── service.rs          # WebRtcService and builder
├── transport.rs        # ant-quic transport adapter
├── quic_bridge.rs      # WebRTC to QUIC bridge
└── quic_streams.rs     # QUIC media stream management with QoS
```

## Status

**Current Status**: Core structure implemented, stub implementations in place.

**Completed**:
- Generic architecture with `PeerIdentity` and `SignalingTransport` traits
- Signaling protocol definition
- Type-safe data structures
- Module organization
- Compilation verified (zero errors, minimal warnings)

**In Progress**:
- Full WebRTC implementation
- QUIC stream management
- Media codec integration
- Comprehensive testing

**Planned**:
- Performance benchmarks
- Usage examples
- Integration tests with saorsa-core and communitas
- Documentation improvements

## Contributing

This is part of the Saorsa project ecosystem. For contribution guidelines, see the main Saorsa project.

## License

[License information to be added]

## Related Projects

- **saorsa-core**: Core P2P networking with DHT-based peer discovery
- **ant-quic**: QUIC implementation with NAT traversal and PQC support
- **communitas**: Application using gossip-based signaling

## Contact

Part of the Saorsa Labs ecosystem.
