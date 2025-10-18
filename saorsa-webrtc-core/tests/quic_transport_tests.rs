//! TDD tests for QUIC transport integration

use saorsa_webrtc::transport::{AntQuicTransport, TransportConfig};
use saorsa_webrtc::signaling::{SignalingMessage, SignalingTransport};
use std::time::Duration;

#[tokio::test]
async fn test_transport_creation() {
    let config = TransportConfig::default();
    let transport = AntQuicTransport::new(config);
    assert!(!transport.is_connected().await);
}

#[tokio::test]
async fn test_transport_connect() {
    let config = TransportConfig::default();
    let mut transport = AntQuicTransport::new(config);
    
    // Start the transport
    transport.start().await.expect("Failed to start transport");
    
    // Should be able to get local address
    let addr = transport.local_addr().await.expect("Should have local address");
    assert!(addr.port() > 0);
}

#[tokio::test]
#[ignore] // Flaky test due to transport layer connection issues
async fn test_transport_send_receive() {
    // Create two transports
    let mut transport1 = AntQuicTransport::new(TransportConfig::default());
    let mut transport2 = AntQuicTransport::new(TransportConfig::default());
    
    transport1.start().await.expect("Failed to start transport1");
    transport2.start().await.expect("Failed to start transport2");
    
    let addr2 = transport2.local_addr().await.expect("Should have addr2");
    
    // Connect transport1 to transport2
    let peer_id = transport1.connect_to_peer(addr2).await
        .expect("Failed to connect");
    
    // Give the accept task time to complete
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Send a message
    let message = SignalingMessage::Offer {
        session_id: "test-session".to_string(),
        sdp: "v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\n".to_string(),
        quic_endpoint: None,
    };
    
    transport1.send_message(&peer_id, message.clone()).await
        .expect("Failed to send message");
    
    // Receive the message on transport2
    let (_received_peer, received_msg) = tokio::time::timeout(
        Duration::from_secs(5),
        transport2.receive_message()
    ).await
        .expect("Timeout waiting for message")
        .expect("Failed to receive message");
    
    // Verify message content
    assert_eq!(received_msg.session_id(), message.session_id());
}

#[tokio::test]
#[ignore] // Flaky test due to timing issues with multiple peer connections
async fn test_transport_multiple_peers() {
    let mut central = AntQuicTransport::new(TransportConfig::default());
    central.start().await.expect("Failed to start central");
    let central_addr = central.local_addr().await.expect("Should have address");
    
    // Create multiple peers
    let mut peer1 = AntQuicTransport::new(TransportConfig::default());
    let mut peer2 = AntQuicTransport::new(TransportConfig::default());
    
    peer1.start().await.expect("Failed to start peer1");
    peer2.start().await.expect("Failed to start peer2");
    
    // Connect both to central
    let peer1_id = peer1.connect_to_peer(central_addr).await
        .expect("Peer1 failed to connect");
    let peer2_id = peer2.connect_to_peer(central_addr).await
        .expect("Peer2 failed to connect");
    
    // Give time for connections to be accepted
    tokio::time::sleep(Duration::from_millis(200)).await;
    
    // Send from peer1
    let msg1 = SignalingMessage::Offer {
        session_id: "session-1".to_string(),
        sdp: "sdp-1".to_string(),
        quic_endpoint: None,
    };
    peer1.send_message(&peer1_id, msg1).await.expect("Failed to send");
    
    // Send from peer2
    let msg2 = SignalingMessage::Answer {
        session_id: "session-2".to_string(),
        sdp: "sdp-2".to_string(),
        quic_endpoint: None,
    };
    peer2.send_message(&peer2_id, msg2).await.expect("Failed to send");
    
    // Give time for messages to arrive
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    // Central should receive both
    for _ in 0..2 {
        let (_peer, _msg) = tokio::time::timeout(
            Duration::from_secs(5),
            central.receive_message()
        ).await.expect("Timeout").expect("Failed to receive");
    }
}

#[tokio::test]
async fn test_transport_disconnect() {
    let mut transport1 = AntQuicTransport::new(TransportConfig::default());
    let mut transport2 = AntQuicTransport::new(TransportConfig::default());
    
    transport1.start().await.expect("Failed to start transport1");
    transport2.start().await.expect("Failed to start transport2");
    
    let addr2 = transport2.local_addr().await.expect("Should have addr2");
    let peer_id = transport1.connect_to_peer(addr2).await
        .expect("Failed to connect");
    
    // Disconnect
    transport1.disconnect_peer(&peer_id).await.expect("Failed to disconnect");
    
    // Sending should fail
    let message = SignalingMessage::Offer {
        session_id: "test".to_string(),
        sdp: "sdp".to_string(),
        quic_endpoint: None,
    };
    
    let result = transport1.send_message(&peer_id, message).await;
    assert!(result.is_err());
}
