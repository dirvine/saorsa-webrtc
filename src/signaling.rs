//! WebRTC signaling protocol
//!
//! Handles SDP exchange and ICE candidate gathering for WebRTC connections.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::SocketAddr;
use std::str::FromStr;
use thiserror::Error;

/// Signaling errors
#[derive(Error, Debug)]
pub enum SignalingError {
    /// Invalid SDP
    #[error("Invalid SDP: {0}")]
    InvalidSdp(String),
    
    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    /// Transport error
    #[error("Transport error: {0}")]
    TransportError(String),
}

/// Signaling transport trait
///
/// Implement this for your specific transport (DHT, gossip, etc.)
#[async_trait]
pub trait SignalingTransport: Send + Sync {
    /// Peer identifier type
    type PeerId: Clone + Send + Sync + fmt::Debug + fmt::Display + FromStr;
    
    /// Transport error type
    type Error: std::error::Error + Send + Sync + 'static;
    
    /// Send a signaling message
    async fn send_message(
        &self,
        peer: &Self::PeerId,
        message: SignalingMessage,
    ) -> Result<(), Self::Error>;
    
    /// Receive a signaling message
    async fn receive_message(&self) -> Result<(Self::PeerId, SignalingMessage), Self::Error>;
    
    /// Discover peer endpoint
    async fn discover_peer_endpoint(
        &self,
        peer: &Self::PeerId,
    ) -> Result<Option<SocketAddr>, Self::Error>;
}

/// Signaling message types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SignalingMessage {
    /// SDP offer
    Offer {
        /// Session ID
        session_id: String,
        /// SDP content
        sdp: String,
        /// Optional QUIC endpoint
        quic_endpoint: Option<SocketAddr>,
    },
    
    /// SDP answer
    Answer {
        /// Session ID
        session_id: String,
        /// SDP content
        sdp: String,
        /// Optional QUIC endpoint
        quic_endpoint: Option<SocketAddr>,
    },
    
    /// ICE candidate
    IceCandidate {
        /// Session ID
        session_id: String,
        /// Candidate string
        candidate: String,
        /// SDP mid
        sdp_mid: Option<String>,
        /// SDP mline index
        sdp_mline_index: Option<u16>,
    },
    
    /// ICE gathering complete
    IceComplete {
        /// Session ID
        session_id: String,
    },
    
    /// Close session
    Bye {
        /// Session ID
        session_id: String,
        /// Optional reason
        reason: Option<String>,
    },
}

impl SignalingMessage {
    /// Get the session ID
    #[must_use]
    pub fn session_id(&self) -> &str {
        match self {
            Self::Offer { session_id, .. }
            | Self::Answer { session_id, .. }
            | Self::IceCandidate { session_id, .. }
            | Self::IceComplete { session_id }
            | Self::Bye { session_id, .. } => session_id,
        }
    }
}

/// Signaling handler
pub struct SignalingHandler<T: SignalingTransport> {
    transport: std::sync::Arc<T>,
}

impl<T: SignalingTransport> SignalingHandler<T> {
    /// Create new signaling handler
    #[must_use]
    pub fn new(transport: std::sync::Arc<T>) -> Self {
        Self { transport }
    }
}
