//! QUIC media stream management with QoS
//!
//! Manages QUIC streams for audio, video, and screen sharing with
//! appropriate quality-of-service parameters.

use thiserror::Error;

/// Stream errors
#[derive(Error, Debug)]
pub enum StreamError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Stream error
    #[error("Stream operation error: {0}")]
    OperationError(String),
}

/// QoS parameters for media streams
#[derive(Debug, Clone)]
pub struct QoSParams {
    /// Target latency in milliseconds
    pub target_latency_ms: u32,
    /// Priority (higher = more important)
    pub priority: u8,
}

impl QoSParams {
    /// Audio QoS parameters (low latency, high priority)
    #[must_use]
    pub fn audio() -> Self {
        Self {
            target_latency_ms: 50,
            priority: 10,
        }
    }

    /// Video QoS parameters (moderate latency, medium priority)
    #[must_use]
    pub fn video() -> Self {
        Self {
            target_latency_ms: 150,
            priority: 5,
        }
    }

    /// Screen share QoS parameters (higher latency acceptable, lower priority)
    #[must_use]
    pub fn screen_share() -> Self {
        Self {
            target_latency_ms: 200,
            priority: 3,
        }
    }
}

/// QUIC media stream manager
pub struct QuicMediaStreamManager {
    _config: QoSParams,
}

impl QuicMediaStreamManager {
    /// Create new stream manager with QoS parameters
    #[must_use]
    pub fn new(qos: QoSParams) -> Self {
        Self { _config: qos }
    }
}
