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
    config: TransportConfig,
    // TODO: Add actual ant-quic connection state
}

impl AntQuicTransport {
    /// Create new ant-quic transport
    #[must_use]
    pub fn new(config: TransportConfig) -> Self {
        Self { config }
    }

    /// Get transport configuration
    #[must_use]
    pub fn config(&self) -> &TransportConfig {
        &self.config
    }
}

#[async_trait]
impl SignalingTransport for AntQuicTransport {
    type PeerId = String;
    type Error = TransportError;

    async fn send_message(
        &self,
        peer: &String,
        _message: SignalingMessage,
    ) -> Result<(), TransportError> {
        // TODO: Implement actual ant-quic message sending
        // For now, validate inputs and return success
        if peer.is_empty() {
            return Err(TransportError::SendError("Peer ID cannot be empty".to_string()));
        }

        // In a real implementation, this would:
        // 1. Serialize the message
        // 2. Establish or use existing QUIC connection to peer
        // 3. Send the message over the connection

        tracing::debug!("Sending signaling message to peer: {}", peer);
        Ok(())
    }

    async fn receive_message(&self) -> Result<(String, SignalingMessage), TransportError> {
        // TODO: Implement actual ant-quic message receiving
        // For now, this is a placeholder - in a real implementation,
        // this would listen for incoming messages on established connections

        Err(TransportError::ReceiveError(
            "Receive not implemented - requires active connection management".to_string(),
        ))
    }

    async fn discover_peer_endpoint(
        &self,
        peer: &String,
    ) -> Result<Option<SocketAddr>, TransportError> {
        // TODO: Implement actual peer discovery via DHT or gossip
        // For now, return None to indicate discovery not available

        tracing::debug!("Attempting to discover endpoint for peer: {}", peer);
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ant_quic_transport_send_message_valid() {
        let config = TransportConfig::default();
        let transport = AntQuicTransport::new(config);

        let message = SignalingMessage::Offer {
            session_id: "test-session".to_string(),
            sdp: "test-sdp".to_string(),
            quic_endpoint: None,
        };

        let result = transport.send_message(&"peer1".to_string(), message).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_ant_quic_transport_send_message_empty_peer() {
        let config = TransportConfig::default();
        let transport = AntQuicTransport::new(config);

        let message = SignalingMessage::Offer {
            session_id: "test-session".to_string(),
            sdp: "test-sdp".to_string(),
            quic_endpoint: None,
        };

        let result = transport.send_message(&"".to_string(), message).await;
        assert!(matches!(result, Err(TransportError::SendError(_))));
    }

    #[tokio::test]
    async fn test_ant_quic_transport_receive_message() {
        let config = TransportConfig::default();
        let transport = AntQuicTransport::new(config);

        let result = transport.receive_message().await;
        assert!(matches!(result, Err(TransportError::ReceiveError(_))));
    }

    #[tokio::test]
    async fn test_ant_quic_transport_discover_peer_endpoint() {
        let config = TransportConfig::default();
        let transport = AntQuicTransport::new(config);

        let result = transport.discover_peer_endpoint(&"peer1".to_string()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_ant_quic_transport_config() {
        let config = TransportConfig {
            local_addr: Some("127.0.0.1:8080".parse().unwrap()),
        };
        let transport = AntQuicTransport::new(config.clone());

        assert_eq!(transport.config().local_addr, config.local_addr);
    }

    #[test]
    fn test_transport_config_default() {
        let config = TransportConfig::default();
        assert!(config.local_addr.is_none());
    }
}
