//! Enhanced mock signaling transport for testing

use crate::fixtures::test_network::NetworkConditions;
use saorsa_webrtc_core::signaling::{SignalingMessage, SignalingTransport};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;

/// Configuration for mock transport behavior
#[derive(Debug, Clone)]
pub struct MockTransportConfig {
    /// Simulated network latency
    pub latency: Duration,
    /// Packet loss rate (0.0 to 1.0)
    pub packet_loss: f32,
    /// Maximum message queue size per peer
    pub max_queue_size: usize,
    /// Whether to simulate connection failures
    pub simulate_failures: bool,
    /// Failure rate (0.0 to 1.0)
    pub failure_rate: f32,
}

impl Default for MockTransportConfig {
    fn default() -> Self {
        Self {
            latency: Duration::from_millis(10),
            packet_loss: 0.0,
            max_queue_size: 1000,
            simulate_failures: false,
            failure_rate: 0.0,
        }
    }
}

impl MockTransportConfig {
    /// Create config for perfect network conditions
    pub fn perfect() -> Self {
        Self {
            latency: Duration::from_millis(1),
            packet_loss: 0.0,
            max_queue_size: 1000,
            simulate_failures: false,
            failure_rate: 0.0,
        }
    }

    /// Create config for poor network conditions
    pub fn poor() -> Self {
        Self {
            latency: Duration::from_millis(200),
            packet_loss: 0.1, // 10% packet loss
            max_queue_size: 100,
            simulate_failures: true,
            failure_rate: 0.05, // 5% failure rate
        }
    }

    /// Create config for mobile network conditions
    pub fn mobile() -> Self {
        Self {
            latency: Duration::from_millis(100),
            packet_loss: 0.02, // 2% packet loss
            max_queue_size: 500,
            simulate_failures: false,
            failure_rate: 0.01, // 1% failure rate
        }
    }
}

/// Enhanced mock transport for testing signaling
#[derive(Debug)]
pub struct MockSignalingTransport {
    config: MockTransportConfig,
    peer_id: String,
    message_queues: Arc<RwLock<HashMap<String, VecDeque<SignalingMessage>>>>,
    connected_peers: Arc<RwLock<HashMap<String, bool>>>,
    message_counter: Arc<Mutex<u64>>,
    network_conditions: Arc<RwLock<NetworkConditions>>,
}

impl MockSignalingTransport {
    /// Create a new mock transport with default config
    pub fn new(peer_id: impl Into<String>) -> Self {
        Self::with_config(peer_id, MockTransportConfig::default())
    }

    /// Create a new mock transport with custom config
    pub fn with_config(peer_id: impl Into<String>, config: MockTransportConfig) -> Self {
        let peer_id = peer_id.into();
        Self {
            config,
            peer_id: peer_id.clone(),
            message_queues: Arc::new(RwLock::new(HashMap::new())),
            connected_peers: Arc::new(RwLock::new(HashMap::new())),
            message_counter: Arc::new(Mutex::new(0)),
            network_conditions: Arc::new(RwLock::new(NetworkConditions::default())),
        }
    }

    /// Connect to another mock transport (bidirectional)
    pub async fn connect_to(&self, other: &MockSignalingTransport) {
        let mut peers = self.connected_peers.write().await;
        let mut other_peers = other.connected_peers.write().await;
        
        peers.insert(other.peer_id.clone(), true);
        other_peers.insert(self.peer_id.clone(), true);
    }

    /// Disconnect from a peer
    pub async fn disconnect_from(&self, peer_id: &str) {
        let mut peers = self.connected_peers.write().await;
        peers.remove(peer_id);
    }

    /// Check if connected to a specific peer
    pub async fn is_connected_to(&self, peer_id: &str) -> bool {
        let peers = self.connected_peers.read().await;
        peers.get(peer_id).copied().unwrap_or(false)
    }

    /// Get list of connected peers
    pub async fn connected_peers(&self) -> Vec<String> {
        let peers = self.connected_peers.read().await;
        peers.keys().cloned().collect()
    }

    /// Simulate network condition changes
    pub async fn set_network_conditions(&self, conditions: NetworkConditions) {
        let mut network = self.network_conditions.write().await;
        *network = conditions;
    }

    /// Get current network conditions
    pub async fn network_conditions(&self) -> NetworkConditions {
        let network = self.network_conditions.read().await;
        network.clone()
    }

    /// Get message statistics
    pub async fn message_count(&self) -> u64 {
        let counter = self.message_counter.lock().await;
        *counter
    }

    /// Clear all message queues
    pub async fn clear_queues(&self) {
        let mut queues = self.message_queues.write().await;
        queues.clear();
    }

    /// Get number of queued messages for a peer
    pub async fn queued_message_count(&self, peer_id: &str) -> usize {
        let queues = self.message_queues.read().await;
        queues.get(peer_id).map_or(0, |queue| queue.len())
    }

    /// Internal method to simulate sending with network conditions
    async fn simulate_send(&self, peer_id: &str, message: SignalingMessage) -> Result<(), MockTransportError> {
        // Simulate packet loss
        if self.config.packet_loss > 0.0 && rand::random::<f32>() < self.config.packet_loss {
            return Err(MockTransportError::PacketLoss);
        }

        // Simulate failures
        if self.config.simulate_failures && rand::random::<f32>() < self.config.failure_rate {
            return Err(MockTransportError::ConnectionFailed);
        }

        // Check queue size
        {
            let mut queues = self.message_queues.write().await;
            let queue = queues.entry(peer_id.to_string()).or_insert_with(VecDeque::new);
            
            if queue.len() >= self.config.max_queue_size {
                return Err(MockTransportError::QueueFull);
            }
            
            queue.push_back(message);
        }

        // Increment counter
        {
            let mut counter = self.message_counter.lock().await;
            *counter += 1;
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl SignalingTransport for MockSignalingTransport {
    type PeerId = String;
    type Error = MockTransportError;

    async fn send_message(
        &self,
        peer: &Self::PeerId,
        message: SignalingMessage,
    ) -> Result<(), Self::Error> {
        // Check if connected
        if !self.is_connected_to(peer).await {
            return Err(MockTransportError::NotConnected(peer.clone()));
        }

        // Simulate network latency
        if self.config.latency > Duration::ZERO {
            sleep(self.config.latency).await;
        }

        self.simulate_send(peer, message).await
    }

    async fn receive_message(&self) -> Result<(Self::PeerId, SignalingMessage), Self::Error> {
        let queues = self.message_queues.read().await;
        
        // Find first non-empty queue (round-robin style)
        for (peer_id, queue) in queues.iter() {
            if let Some(message) = queue.front() {
                // We need to drop the read lock before modifying
                drop(queues);
                
                // Remove the message from queue
                let mut queues = self.message_queues.write().await;
                if let Some(queue) = queues.get_mut(peer_id) {
                    if let Some(message) = queue.pop_front() {
                        return Ok((peer_id.clone(), message));
                    }
                }
            }
        }

        // No messages available
        Err(MockTransportError::NoMessages)
    }

    async fn discover_peer_endpoint(&self, peer: &Self::PeerId) -> Result<Option<std::net::SocketAddr>, Self::Error> {
        // Simulate endpoint discovery
        if self.is_connected_to(peer).await {
            Ok(Some(format!("127.0.0.1:{}", 8000 + peer.len() as u16).parse().unwrap()))
        } else {
            Ok(None)
        }
    }
}

/// Errors that can occur in mock transport
#[derive(Debug, thiserror::Error)]
pub enum MockTransportError {
    #[error("Not connected to peer: {0}")]
    NotConnected(String),
    
    #[error("Packet loss simulated")]
    PacketLoss,
    
    #[error("Connection failed")]
    ConnectionFailed,
    
    #[error("Message queue full")]
    QueueFull,
    
    #[error("No messages available")]
    NoMessages,
    
    #[error("Network error: {0}")]
    Network(String),
}

impl std::fmt::Display for MockTransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

/// Utility functions for creating connected mock transport pairs
pub struct MockTransportPair;

impl MockTransportPair {
    /// Create a bidirectional pair of connected mock transports
    pub async fn connected() -> (MockSignalingTransport, MockSignalingTransport) {
        Self::connected_with_config(MockTransportConfig::default()).await
    }

    /// Create a bidirectional pair with custom config
    pub async fn connected_with_config(config: MockTransportConfig) -> (MockSignalingTransport, MockSignalingTransport) {
        let transport1 = MockSignalingTransport::with_config("peer1", config.clone());
        let transport2 = MockSignalingTransport::with_config("peer2", config);
        
        transport1.connect_to(&transport2).await;
        
        (transport1, transport2)
    }

    /// Create a pair with poor network conditions
    pub async fn connected_poor_network() -> (MockSignalingTransport, MockSignalingTransport) {
        Self::connected_with_config(MockTransportConfig::poor()).await
    }

    /// Create a pair with mobile network conditions
    pub async fn connected_mobile_network() -> (MockSignalingTransport, MockSignalingTransport) {
        Self::connected_with_config(MockTransportConfig::mobile()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use saorsa_webrtc_core::signaling::SignalingMessage;

    #[tokio::test]
    async fn test_mock_transport_basic_functionality() {
        let (t1, t2) = MockTransportPair::connected().await;
        
        assert!(t1.is_connected_to("peer2").await);
        assert!(t2.is_connected_to("peer1").await);
        
        let message = SignalingMessage::Offer {
            session_id: "test".to_string(),
            sdp: "test-sdp".to_string(),
            quic_endpoint: None,
        };
        
        // Send message
        t1.send_message(&"peer2".to_string(), message.clone()).await.unwrap();
        
        // Receive message
        let (peer, received) = t2.receive_message().await.unwrap();
        assert_eq!(peer, "peer1");
        assert_eq!(received.session_id(), message.session_id());
    }

    #[tokio::test]
    async fn test_mock_transport_packet_loss() {
        let config = MockTransportConfig {
            packet_loss: 1.0, // 100% packet loss
            ..Default::default()
        };
        
        let (t1, _t2) = MockTransportPair::connected_with_config(config).await;
        
        let message = SignalingMessage::Offer {
            session_id: "test".to_string(),
            sdp: "test-sdp".to_string(),
            quic_endpoint: None,
        };
        
        // Send should fail due to packet loss
        let result = t1.send_message(&"peer2".to_string(), message).await;
        assert!(matches!(result, Err(MockTransportError::PacketLoss)));
    }

    #[tokio::test]
    async fn test_mock_transport_not_connected() {
        let t1 = MockSignalingTransport::new("peer1");
        let _t2 = MockSignalingTransport::new("peer2");
        
        let message = SignalingMessage::Offer {
            session_id: "test".to_string(),
            sdp: "test-sdp".to_string(),
            quic_endpoint: None,
        };
        
        // Send should fail since not connected
        let result = t1.send_message(&"peer2".to_string(), message).await;
        assert!(matches!(result, Err(MockTransportError::NotConnected(_))));
    }
}