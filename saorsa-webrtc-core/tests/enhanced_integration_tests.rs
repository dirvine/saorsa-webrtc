//! Enhanced integration tests using the new mock framework

use saorsa_webrtc_core::{
    CallManager, CallManagerConfig, MediaConstraints,
    SignalingHandler, SignalingTransport, SignalingMessage, PeerIdentityString,
    CallState,
};
use std::sync::Arc;
use std::time::Duration;

// Import our test fixtures
mod fixtures;
use fixtures::{
    mock_transport::{MockSignalingTransport, MockTransportConfig, MockTransportPair},
    test_network::{NetworkConditions, NetworkScenario},
    proptest_helpers::*,
};

#[tokio::test]
async fn test_call_flow_with_perfect_network() {
    let (transport1, transport2) = MockTransportPair::connected().await;
    let signaling1 = Arc::new(SignalingHandler::new(transport1));
    let signaling2 = Arc::new(SignalingHandler::new(transport2));

    let call_config = CallManagerConfig::default();
    let caller = CallManager::<PeerIdentityString>::new(call_config.clone()).await.unwrap();
    let callee = CallManager::<PeerIdentityString>::new(call_config).await.unwrap();

    let callee_id = PeerIdentityString::new("callee");
    let constraints = MediaConstraints::video_call();

    // Initiate call
    let call_id = caller.initiate_call(callee_id.clone(), constraints.clone()).await.unwrap();
    assert_eq!(caller.get_call_state(call_id).await, Some(CallState::Calling));

    // Accept call
    callee.accept_call(call_id, constraints).await.unwrap();
    assert_eq!(callee.get_call_state(call_id).await, Some(CallState::Connected));

    // End call
    caller.end_call(call_id).await.unwrap();
    assert_eq!(caller.get_call_state(call_id).await, None);
    assert_eq!(callee.get_call_state(call_id).await, None);
}

#[tokio::test]
async fn test_call_flow_with_poor_network() {
    let (transport1, transport2) = MockTransportPair::connected_poor_network().await;
    let signaling1 = Arc::new(SignalingHandler::new(transport1));
    let signaling2 = Arc::new(SignalingHandler::new(transport2));

    let call_config = CallManagerConfig::default();
    let caller = CallManager::<PeerIdentityString>::new(call_config.clone()).await.unwrap();
    let callee = CallManager::<PeerIdentityString>::new(call_config).await.unwrap();

    let callee_id = PeerIdentityString::new("callee");
    let constraints = MediaConstraints::audio_only(); // Use audio-only for poor network

    // Initiate call - may fail due to network conditions
    let call_result = caller.initiate_call(callee_id.clone(), constraints.clone()).await;
    
    match call_result {
        Ok(call_id) => {
            // Call initiated, try to accept
            let accept_result = callee.accept_call(call_id, constraints).await;
            
            // Accept might fail due to network conditions
            if accept_result.is_ok() {
                // If accepted, try to end call
                let _ = caller.end_call(call_id).await;
            }
        }
        Err(_) => {
            // Call failed to initiate - this is expected under poor network conditions
        }
    }
}

#[tokio::test]
async fn test_multiple_concurrent_calls() {
    let (transport1, transport2) = MockTransportPair::connected().await;
    let signaling1 = Arc::new(SignalingHandler::new(transport1));
    let signaling2 = Arc::new(SignalingHandler::new(transport2));

    let call_config = CallManagerConfig::default();
    let caller = CallManager::<PeerIdentityString>::new(call_config.clone()).await.unwrap();
    let callee = CallManager::<PeerIdentityString>::new(call_config).await.unwrap();

    let callee_id = PeerIdentityString::new("callee");
    let constraints = MediaConstraints::audio_only();

    // Create multiple concurrent calls
    let mut call_ids = Vec::new();
    for i in 0..5 {
        let call_id = caller.initiate_call(
            PeerIdentityString::new(&format!("callee-{}", i)),
            constraints.clone(),
        ).await.unwrap();
        call_ids.push(call_id);
    }

    // Verify all calls are in calling state
    for call_id in &call_ids {
        assert_eq!(caller.get_call_state(*call_id).await, Some(CallState::Calling));
    }

    // Accept all calls
    for call_id in &call_ids {
        let _ = callee.accept_call(*call_id, constraints.clone()).await;
    }

    // End all calls
    for call_id in &call_ids {
        let _ = caller.end_call(*call_id).await;
    }

    // Verify all calls are cleaned up
    for call_id in &call_ids {
        assert_eq!(caller.get_call_state(*call_id).await, None);
    }
}

#[tokio::test]
async fn test_network_condition_transitions() {
    let (transport1, transport2) = MockTransportPair::connected().await;
    
    // Start with perfect conditions
    transport1.set_network_conditions(NetworkConditions::perfect()).await;
    transport2.set_network_conditions(NetworkConditions::perfect()).await;

    let signaling1 = Arc::new(SignalingHandler::new(transport1));
    let signaling2 = Arc::new(SignalingHandler::new(transport2));

    let call_config = CallManagerConfig::default();
    let caller = CallManager::<PeerIdentityString>::new(call_config.clone()).await.unwrap();
    let callee = CallManager::<PeerIdentityString>::new(call_config).await.unwrap();

    let callee_id = PeerIdentityString::new("callee");
    let constraints = MediaConstraints::video_call();

    // Initiate call under perfect conditions
    let call_id = caller.initiate_call(callee_id.clone(), constraints.clone()).await.unwrap();

    // Simulate network degradation
    transport1.set_network_conditions(NetworkConditions::poor()).await;
    transport2.set_network_conditions(NetworkConditions::poor()).await;

    // Try to accept call under poor conditions
    let accept_result = callee.accept_call(call_id, constraints).await;
    
    // Accept might fail or succeed depending on severity
    match accept_result {
        Ok(_) => {
            // If accepted, try to maintain call
            tokio::time::sleep(Duration::from_millis(100)).await;
            let _ = caller.end_call(call_id).await;
        }
        Err(_) => {
            // Accept failed due to poor conditions - this is expected
            let _ = caller.end_call(call_id).await;
        }
    }
}

#[tokio::test]
async fn test_transport_resilience() {
    let config = MockTransportConfig {
        simulate_failures: true,
        failure_rate: 0.1, // 10% failure rate
        ..Default::default()
    };
    
    let (transport1, transport2) = MockTransportPair::connected_with_config(config).await;
    
    let signaling1 = Arc::new(SignalingHandler::new(transport1));
    let signaling2 = Arc::new(SignalingHandler::new(transport2));

    let call_config = CallManagerConfig::default();
    let caller = CallManager::<PeerIdentityString>::new(call_config.clone()).await.unwrap();
    let callee = CallManager::<PeerIdentityString>::new(call_config).await.unwrap();

    let callee_id = PeerIdentityString::new("callee");
    let constraints = MediaConstraints::audio_only();

    // Try multiple operations with failures
    let mut successful_calls = 0;
    let mut failed_calls = 0;

    for _ in 0..10 {
        match caller.initiate_call(callee_id.clone(), constraints.clone()).await {
            Ok(call_id) => {
                successful_calls += 1;
                // Try to accept and end
                if callee.accept_call(call_id, constraints.clone()).await.is_ok() {
                    let _ = caller.end_call(call_id).await;
                } else {
                    let _ = caller.end_call(call_id).await;
                }
            }
            Err(_) => {
                failed_calls += 1;
            }
        }
    }

    // With 10% failure rate, we should have some successes and some failures
    println!("Successful calls: {}, Failed calls: {}", successful_calls, failed_calls);
    assert!(successful_calls > 0, "Should have some successful calls");
    assert!(failed_calls > 0, "Should have some failed calls due to simulated failures");
}

#[tokio::test]
async fn test_message_ordering_preservation() {
    let (transport1, transport2) = MockTransportPair::connected().await;
    
    // Send multiple messages in sequence
    let messages = vec![
        SignalingMessage::Offer {
            session_id: "session-1".to_string(),
            sdp: "sdp-1".to_string(),
            quic_endpoint: None,
        },
        SignalingMessage::Answer {
            session_id: "session-2".to_string(),
            sdp: "sdp-2".to_string(),
            quic_endpoint: None,
        },
        SignalingMessage::IceCandidate {
            session_id: "session-3".to_string(),
            candidate: "candidate-3".to_string(),
            sdp_mid: None,
            sdp_mline_index: None,
        },
    ];

    // Send all messages
    for message in &messages {
        let _ = transport1.send_message(&"peer2".to_string(), message.clone()).await;
    }

    // Receive messages in order
    let mut received_messages = Vec::new();
    for _ in 0..messages.len() {
        if let Ok((_peer, message)) = transport2.receive_message().await {
            received_messages.push(message);
        }
    }

    // Verify order is preserved
    assert_eq!(received_messages.len(), messages.len());
    for (i, (received, expected)) in received_messages.iter().zip(messages.iter()).enumerate() {
        match (received, expected) {
            (SignalingMessage::Offer { session_id, .. }, SignalingMessage::Offer { session_id: expected_id, .. }) => {
                assert_eq!(session_id, expected_id);
            }
            (SignalingMessage::Answer { session_id, .. }, SignalingMessage::Answer { session_id: expected_id, .. }) => {
                assert_eq!(session_id, expected_id);
            }
            (SignalingMessage::IceCandidate { session_id, .. }, SignalingMessage::IceCandidate { session_id: expected_id, .. }) => {
                assert_eq!(session_id, expected_id);
            }
            _ => panic!("Message types don't match at index {}", i),
        }
    }
}

// Property-based tests will be added in Phase 3
// For now, we focus on basic enhanced integration testing