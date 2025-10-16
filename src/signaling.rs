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

    /// Send a signaling message to a peer
    ///
    /// # Errors
    ///
    /// Returns error if sending fails
    pub async fn send_message(
        &self,
        peer: &T::PeerId,
        message: SignalingMessage,
    ) -> Result<(), T::Error> {
        self.transport.send_message(peer, message).await
    }

    /// Receive a signaling message
    ///
    /// # Errors
    ///
    /// Returns error if receiving fails
    pub async fn receive_message(&self) -> Result<(T::PeerId, SignalingMessage), T::Error> {
        self.transport.receive_message().await
    }

    /// Discover endpoint for a peer
    ///
    /// # Errors
    ///
    /// Returns error if discovery fails
    pub async fn discover_peer_endpoint(
        &self,
        peer: &T::PeerId,
    ) -> Result<Option<std::net::SocketAddr>, T::Error> {
        self.transport.discover_peer_endpoint(peer).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    // Mock transport for testing
    struct MockTransport {
        messages: Mutex<VecDeque<(String, SignalingMessage)>>,
    }

    #[derive(Debug)]
    struct MockError;

    impl std::fmt::Display for MockError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Mock error")
        }
    }

    impl std::error::Error for MockError {}

    impl MockTransport {
        fn new() -> Self {
            Self {
                messages: Mutex::new(VecDeque::new()),
            }
        }

        fn add_message(&self, peer: String, message: SignalingMessage) {
            self.messages.lock().unwrap().push_back((peer, message));
        }
    }

    #[async_trait]
    impl SignalingTransport for MockTransport {
        type PeerId = String;
        type Error = MockError;

        async fn send_message(
            &self,
            peer: &String,
            message: SignalingMessage,
        ) -> Result<(), MockError> {
            self.messages.lock().unwrap().push_back((peer.clone(), message));
            Ok(())
        }

        async fn receive_message(&self) -> Result<(String, SignalingMessage), MockError> {
            if let Some((peer, message)) = self.messages.lock().unwrap().pop_front() {
                Ok((peer, message))
            } else {
                Err(MockError)
            }
        }

        async fn discover_peer_endpoint(
            &self,
            _peer: &String,
        ) -> Result<Option<std::net::SocketAddr>, MockError> {
            Ok(Some("127.0.0.1:8080".parse().unwrap()))
        }
    }

    #[tokio::test]
    async fn test_signaling_handler_send_message() {
        let transport = Arc::new(MockTransport::new());
        let handler = SignalingHandler::new(transport.clone());

        let message = SignalingMessage::Offer {
            session_id: "test-session".to_string(),
            sdp: "test-sdp".to_string(),
            quic_endpoint: None,
        };

        let result = handler.send_message(&"peer1".to_string(), message.clone()).await;
        assert!(result.is_ok());

        // Check that message was queued
        let received = transport.messages.lock().unwrap().pop_front();
        assert_eq!(received, Some(("peer1".to_string(), message)));
    }

    #[tokio::test]
    async fn test_signaling_handler_receive_message() {
        let transport = Arc::new(MockTransport::new());
        let handler = SignalingHandler::new(transport.clone());

        let message = SignalingMessage::Answer {
            session_id: "test-session".to_string(),
            sdp: "test-sdp".to_string(),
            quic_endpoint: None,
        };

        transport.add_message("peer1".to_string(), message.clone());

        let result = handler.receive_message().await;
        assert!(result.is_ok());
        let (peer, received_message) = result.unwrap();
        assert_eq!(peer, "peer1");
        assert_eq!(received_message, message);
    }

    #[tokio::test]
    async fn test_signaling_handler_discover_endpoint() {
        let transport = Arc::new(MockTransport::new());
        let handler = SignalingHandler::new(transport);

        let result = handler.discover_peer_endpoint(&"peer1".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("127.0.0.1:8080".parse().unwrap()));
    }
}
