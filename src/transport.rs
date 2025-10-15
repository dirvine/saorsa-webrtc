//! Transport layer implementations
//!
//! This module provides transport adapters for different signaling mechanisms.

use crate::signaling::{SignalingMessage, SignalingTransport};
use async_trait::async_trait;
use std::net::SocketAddr;
use thiserror::Error;

/// Transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Local endpoint address
    pub local_addr: Option<SocketAddr>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self { local_addr: None }
    }
}

/// Transport errors
#[derive(Error, Debug)]
pub enum TransportError {
    /// Connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Send error
    #[error("Send error: {0}")]
    SendError(String),

    /// Receive error
    #[error("Receive error: {0}")]
    ReceiveError(String),
}

/// ant-quic transport adapter
///
/// This transport uses ant-quic for NAT traversal and encrypted connections.
/// It can be used with DHT-based peer discovery (saorsa-core) or
/// gossip-based rendezvous (communitas).
pub struct AntQuicTransport {
    _config: TransportConfig,
}

impl AntQuicTransport {
    /// Create new ant-quic transport
    #[must_use]
    pub fn new(config: TransportConfig) -> Self {
        Self { _config: config }
    }
}

#[async_trait]
impl SignalingTransport for AntQuicTransport {
    type PeerId = String;
    type Error = TransportError;

    async fn send_message(
        &self,
        _peer: &String,
        _message: SignalingMessage,
    ) -> Result<(), TransportError> {
        // Implementation will depend on the actual transport mechanism
        // (DHT for saorsa-core, gossip/rendezvous for communitas)
        Ok(())
    }

    async fn receive_message(&self) -> Result<(String, SignalingMessage), TransportError> {
        // Implementation will depend on the actual transport mechanism
        Err(TransportError::ReceiveError(
            "Not implemented - use transport-specific implementation".to_string(),
        ))
    }

    async fn discover_peer_endpoint(
        &self,
        _peer: &String,
    ) -> Result<Option<SocketAddr>, TransportError> {
        Ok(None)
    }
}
