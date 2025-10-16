//! Saorsa WebRTC - WebRTC implementation over ant-quic transport
//!
//! This library provides a WebRTC implementation that uses ant-quic as the underlying
//! transport layer instead of traditional ICE/STUN/TURN. It features:
//!
//! - **Native QUIC Transport**: Uses ant-quic for reliable, encrypted connections
//! - **DHT-based Signaling**: Distributed signaling without centralized servers
//! - **Post-Quantum Cryptography**: Built-in PQC support via ant-quic
//! - **NAT Traversal**: Automatic hole punching and relay fallback
//! - **High Performance**: Low-latency media streaming with QoS
//!
//! # Examples
//!
//! ```rust,no_run
//! use saorsa_webrtc::{WebRtcService, MediaConstraints, SignalingHandler, AntQuicTransport, TransportConfig};
//! use std::sync::Arc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create signaling transport
//! let transport = Arc::new(AntQuicTransport::new(TransportConfig::default()));
//! let signaling = Arc::new(SignalingHandler::new(transport));
//!
//! // Create WebRTC service
//! let service = WebRtcService::<saorsa_webrtc::PeerIdentityString, AntQuicTransport>::new(
//!     signaling,
//!     Default::default()
//! ).await?;
//!
//! // Start the service
//! service.start().await?;
//!
//! // Initiate a video call
//! let call_id = service.initiate_call(
//!     saorsa_webrtc::PeerIdentityString::new("eve-frank-grace-henry"),
//!     MediaConstraints::video_call()
//! ).await?;
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]
#![deny(unsafe_code)]
#![warn(clippy::all)]
// Allow pedantic warnings for stub implementations
#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(clippy::unused_async)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::derivable_impls)]

/// Core WebRTC types and data structures
pub mod types;

/// WebRTC service and configuration
pub mod service;

/// Media stream management
pub mod media;

/// Call management and state
pub mod call;

/// Signaling protocol and handlers
pub mod signaling;

/// ant-quic transport integration
pub mod transport;

/// QUIC media stream management with QoS
pub mod quic_streams;

/// Bridge between WebRTC and QUIC
pub mod quic_bridge;

/// Peer identity abstraction
pub mod identity;

// Re-export main types at crate root
pub use call::{CallManager, CallManagerConfig};
pub use identity::{PeerIdentity, PeerIdentityString};
pub use media::{
    AudioDevice, AudioTrack, MediaEvent, MediaStream, MediaStreamManager, VideoDevice, VideoTrack,
};
pub use quic_bridge::{RtpPacket, StreamConfig, StreamType, WebRtcQuicBridge};
pub use service::{WebRtcConfig, WebRtcEvent, WebRtcService, WebRtcServiceBuilder};
pub use signaling::{
    SignalingHandler, SignalingMessage as SignalingMessageType, SignalingTransport,
};
pub use transport::{AntQuicTransport, TransportConfig};
pub use types::*;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::call::{CallManager, CallManagerConfig};
    pub use crate::identity::{PeerIdentity, PeerIdentityString};
    pub use crate::media::{MediaEvent, MediaStreamManager};
    pub use crate::service::{WebRtcConfig, WebRtcEvent, WebRtcService, WebRtcServiceBuilder};
    pub use crate::signaling::{SignalingHandler, SignalingTransport};
    pub use crate::transport::{AntQuicTransport, TransportConfig};
    pub use crate::types::{
        CallEvent, CallId, CallState, MediaConstraints, MediaType, NativeQuicConfiguration,
    };
}
