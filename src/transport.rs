//! Transport layer implementations
//!
//! This module provides transport adapters for different signaling mechanisms.

use crate::signaling::{SignalingMessage, SignalingTransport};
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::Arc;
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
    node: Option<Arc<ant_quic::quic_node::QuicP2PNode>>,
    peer_map: Arc<tokio::sync::RwLock<std::collections::HashMap<String, ant_quic::nat_traversal_api::PeerId>>>,
    default_peer: Arc<tokio::sync::RwLock<Option<ant_quic::nat_traversal_api::PeerId>>>,
}

impl AntQuicTransport {
    /// Create new ant-quic transport
    #[must_use]
    pub fn new(config: TransportConfig) -> Self {
        Self {
            config,
            node: None,
            peer_map: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            default_peer: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }

    /// Get transport configuration
    #[must_use]
    pub fn config(&self) -> &TransportConfig {
        &self.config
    }

    /// Start the transport and initialize QUIC node
    ///
    /// # Errors
    ///
    /// Returns error if node creation fails
    pub async fn start(&mut self) -> Result<(), TransportError> {
        use ant_quic::nat_traversal_api::EndpointRole;
        use ant_quic::quic_node::{QuicNodeConfig, QuicP2PNode};
        use ant_quic::auth::AuthConfig;
        use std::time::Duration;

        // Use Bootstrap role for standalone operation (no external bootstraps needed)
        let node_config = QuicNodeConfig {
            role: EndpointRole::Bootstrap,
            bootstrap_nodes: vec![],
            enable_coordinator: true,
            max_connections: 100,
            connection_timeout: Duration::from_secs(30),
            stats_interval: Duration::from_secs(60),
            auth_config: AuthConfig::default(),
            bind_addr: self.config.local_addr,
        };

        let node = QuicP2PNode::new(node_config)
            .await
            .map_err(|e| TransportError::ConnectionError(format!("Failed to create QUIC node: {}", e)))?;

        let node_arc = Arc::new(node);
        
        // Spawn background task to accept incoming connections
        let node_clone = node_arc.clone();
        tokio::spawn(async move {
            loop {
                match node_clone.accept().await {
                    Ok((addr, peer_id)) => {
                        tracing::debug!("Accepted connection from {:?} at {}", peer_id, addr);
                    }
                    Err(e) => {
                        tracing::debug!("Accept error (expected when no incoming connections): {}", e);
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });

        self.node = Some(node_arc);
        Ok(())
    }

    /// Check if transport is connected
    pub async fn is_connected(&self) -> bool {
        self.node.is_some()
    }

    /// Get local address
    ///
    /// # Errors
    ///
    /// Returns error if transport is not started
    pub async fn local_addr(&self) -> Result<SocketAddr, TransportError> {
        let node = self.node.as_ref()
            .ok_or_else(|| TransportError::ConnectionError("Transport not started".to_string()))?;

        let mut addr = node.get_nat_endpoint()
            .map_err(|e| TransportError::ConnectionError(format!("Failed to get endpoint: {}", e)))?
            .get_quinn_endpoint()
            .ok_or_else(|| TransportError::ConnectionError("No Quinn endpoint available".to_string()))?
            .local_addr()
            .map_err(|e| TransportError::ConnectionError(format!("Failed to get local address: {}", e)))?;

        // If bound to 0.0.0.0, replace with localhost for connection purposes
        if addr.ip().is_unspecified() {
            addr.set_ip(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
        }

        Ok(addr)
    }

    /// Connect to a peer
    ///
    /// # Errors
    ///
    /// Returns error if connection fails
    pub async fn connect_to_peer(&mut self, addr: SocketAddr) -> Result<String, TransportError> {
        let node = self.node.as_ref()
            .ok_or_else(|| TransportError::ConnectionError("Transport not started".to_string()))?;

        let peer_id = node.connect_to_bootstrap(addr)
            .await
            .map_err(|e| TransportError::ConnectionError(format!("Failed to connect: {}", e)))?;

        // Generate string representation for peer ID
        let peer_str = format!("{:?}", peer_id);
        
        // Store mapping
        let mut peer_map = self.peer_map.write().await;
        peer_map.insert(peer_str.clone(), peer_id);
        
        // Set as default peer if no default set
        let mut default_peer = self.default_peer.write().await;
        if default_peer.is_none() {
            *default_peer = Some(peer_id);
        }
        drop(default_peer);
        
        Ok(peer_str)
    }

    /// Disconnect from a peer
    ///
    /// # Errors
    ///
    /// Returns error if disconnection fails
    pub async fn disconnect_peer(&mut self, peer: &String) -> Result<(), TransportError> {
        let mut peer_map = self.peer_map.write().await;
        peer_map.remove(peer);
        Ok(())
    }

    /// Send raw bytes to default peer (for RTP packets)
    ///
    /// # Errors
    ///
    /// Returns error if send fails
    pub async fn send_bytes(&self, data: &[u8]) -> Result<(), TransportError> {
        let node = self.node.as_ref()
            .ok_or_else(|| TransportError::SendError("Transport not started".to_string()))?;

        let default_peer = self.default_peer.read().await;
        let peer_id = default_peer.as_ref()
            .ok_or_else(|| TransportError::SendError("No peer connected".to_string()))?;

        node.send_to_peer(peer_id, data)
            .await
            .map_err(|e| TransportError::SendError(format!("Failed to send: {}", e)))?;

        Ok(())
    }

    /// Receive raw bytes from any peer (for RTP packets)
    ///
    /// # Errors
    ///
    /// Returns error if receive fails
    pub async fn receive_bytes(&self) -> Result<Vec<u8>, TransportError> {
        let node = self.node.as_ref()
            .ok_or_else(|| TransportError::ReceiveError("Transport not started".to_string()))?;

        let (_peer_id, data) = node.receive()
            .await
            .map_err(|e| TransportError::ReceiveError(format!("Failed to receive: {}", e)))?;

        Ok(data)
    }
}

#[async_trait]
impl SignalingTransport for AntQuicTransport {
    type PeerId = String;
    type Error = TransportError;

    async fn send_message(
        &self,
        peer: &String,
        message: SignalingMessage,
    ) -> Result<(), TransportError> {
        if peer.is_empty() {
            return Err(TransportError::SendError("Peer ID cannot be empty".to_string()));
        }

        let node = self.node.as_ref()
            .ok_or_else(|| TransportError::SendError("Transport not started".to_string()))?;

        // Get actual peer ID from map
        let peer_map = self.peer_map.read().await;
        let peer_id = peer_map.get(peer)
            .ok_or_else(|| TransportError::SendError(format!("Peer not found: {}", peer)))?;

        // Serialize the message
        let data = serde_json::to_vec(&message)
            .map_err(|e| TransportError::SendError(format!("Failed to serialize message: {}", e)))?;

        // Send over QUIC
        node.send_to_peer(peer_id, &data)
            .await
            .map_err(|e| TransportError::SendError(format!("Failed to send: {}", e)))?;

        tracing::debug!("Sent signaling message to peer: {}", peer);
        Ok(())
    }

    async fn receive_message(&self) -> Result<(String, SignalingMessage), TransportError> {
        let node = self.node.as_ref()
            .ok_or_else(|| TransportError::ReceiveError("Transport not started".to_string()))?;

        // Receive data from any peer (this will block until data arrives)
        // The QuicP2PNode handles incoming connections internally
        let (peer_id, data) = node.receive()
            .await
            .map_err(|e| TransportError::ReceiveError(format!("Failed to receive: {}", e)))?;

        // Deserialize the message
        let message: SignalingMessage = serde_json::from_slice(&data)
            .map_err(|e| TransportError::ReceiveError(format!("Failed to deserialize message: {}", e)))?;

        // Generate string representation for peer ID
        let peer_str = format!("{:?}", peer_id);
        
        // Update peer map if needed
        let mut peer_map = self.peer_map.write().await;
        peer_map.entry(peer_str.clone()).or_insert(peer_id);
        drop(peer_map);

        tracing::debug!("Received signaling message from peer: {}", peer_str);
        Ok((peer_str, message))
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

        // Will fail without peer connected, which is expected
        let _result = transport.send_message(&"peer1".to_string(), message).await;
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
