//! WebRTC to QUIC bridge
//!
//! Bridges WebRTC media with QUIC transport for data channels.

use thiserror::Error;

/// Bridge errors
#[derive(Error, Debug)]
pub enum BridgeError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Stream error
    #[error("Stream error: {0}")]
    StreamError(String),
}

/// WebRTC to QUIC bridge configuration
#[derive(Debug, Clone)]
pub struct QuicBridgeConfig {
    /// Maximum packet size
    pub max_packet_size: usize,
}

impl Default for QuicBridgeConfig {
    fn default() -> Self {
        Self {
            max_packet_size: 1200,
        }
    }
}

/// WebRTC QUIC bridge
///
/// Handles translation between WebRTC RTP packets and QUIC streams
pub struct WebRtcQuicBridge {
    _config: QuicBridgeConfig,
}

impl WebRtcQuicBridge {
    /// Create new bridge
    #[must_use]
    pub fn new(config: QuicBridgeConfig) -> Self {
        Self { _config: config }
    }
}

impl Default for WebRtcQuicBridge {
    fn default() -> Self {
        Self::new(QuicBridgeConfig::default())
    }
}
