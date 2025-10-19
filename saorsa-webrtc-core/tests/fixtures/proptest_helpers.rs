//! Property-based testing helpers for saorsa-webrtc

use proptest::prelude::*;
use saorsa_webrtc_core::{
    quic_bridge::{RtpPacket, StreamType},
    signaling::SignalingMessage,
    types::{CallId, MediaConstraints, MediaType},
};
use std::net::SocketAddr;

/// Strategy for generating valid RTP packets
pub fn rtp_packet_strategy() -> impl Strategy<Value = RtpPacket> {
    (
        // Payload type (0-127, but we'll use common ranges)
        0u8..127u8,
        // Sequence number (0-65535)
        any::<u16>(),
        // Timestamp (0-2^32-1)
        any::<u32>(),
        // SSRC (0-2^32-1)
        any::<u32>(),
        // Payload (up to 1000 bytes to stay within limits)
        prop::collection::vec(any::<u8>(), 0..1000),
        // Stream type
        prop_oneof![
            Just(StreamType::Audio),
            Just(StreamType::Video),
            Just(StreamType::ScreenShare),
            Just(StreamType::Data),
        ],
    )
        .prop_map(|(payload_type, sequence_number, timestamp, ssrc, payload, stream_type)| {
            // Create packet - this may fail if payload is too large
            RtpPacket::new(payload_type, sequence_number, timestamp, ssrc, payload, stream_type)
                .unwrap_or_else(|_| {
                    // Fallback to smaller payload if too large
                    let small_payload = payload.into_iter().take(500).collect::<Vec<_>>();
                    RtpPacket::new(payload_type, sequence_number, timestamp, ssrc, small_payload, stream_type)
                        .expect("Should create valid packet with smaller payload")
                })
        })
}

/// Strategy for generating valid signaling messages
pub fn signaling_message_strategy() -> impl Strategy<Value = SignalingMessage> {
    prop_oneof![
        // Offer messages
        (any::<String>(), any::<String>(), prop::option::of(any::<SocketAddr>()))
            .prop_map(|(session_id, sdp, quic_endpoint)| SignalingMessage::Offer {
                session_id,
                sdp,
                quic_endpoint,
            }),
        
        // Answer messages
        (any::<String>(), any::<String>(), prop::option::of(any::<SocketAddr>()))
            .prop_map(|(session_id, sdp, quic_endpoint)| SignalingMessage::Answer {
                session_id,
                sdp,
                quic_endpoint,
            }),
        
        // ICE candidates
        (
            any::<String>(),
            any::<String>(),
            prop::option::of(any::<String>()),
            prop::option::of(any::<u16>()),
        )
            .prop_map(|(session_id, candidate, sdp_mid, sdp_mline_index)| {
                SignalingMessage::IceCandidate {
                    session_id,
                    candidate,
                    sdp_mid,
                    sdp_mline_index,
                }
            }),
        
        // ICE complete
        any::<String>().prop_map(|session_id| SignalingMessage::IceComplete { session_id }),
        
        // Bye messages
        any::<String>().prop_map(|session_id| SignalingMessage::Bye { session_id }),
    ]
}

/// Strategy for generating media constraints
pub fn media_constraints_strategy() -> impl Strategy<Value = MediaConstraints> {
    prop_oneof![
        Just(MediaConstraints::audio_only()),
        Just(MediaConstraints::video_call()),
        Just(MediaConstraints::screen_share()),
        (any::<bool>(), any::<bool>(), any::<bool>())
            .prop_map(|(audio, video, screen_share)| MediaConstraints {
                audio,
                video,
                screen_share,
            }),
    ]
}

/// Strategy for generating call IDs
pub fn call_id_strategy() -> impl Strategy<Value = CallId> {
    any::<[u8; 16]>().prop_map(|bytes| CallId(uuid::Uuid::from_bytes(bytes)))
}

/// Strategy for generating media types
pub fn media_type_strategy() -> impl Strategy<Value = MediaType> {
    prop_oneof![
        Just(MediaType::Audio),
        Just(MediaType::Video),
        Just(MediaType::ScreenShare),
        Just(MediaType::DataChannel),
    ]
}

/// Strategy for generating socket addresses
pub fn socket_addr_strategy() -> impl Strategy<Value = SocketAddr> {
    (any::<std::net::Ipv4Addr>(), any::<u16>())
        .prop_map(|(ip, port)| SocketAddr::V4(std::net::SocketAddrV4::new(ip, port)))
}

/// Strategy for generating session IDs
pub fn session_id_strategy() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9_-]{8,32}").unwrap()
}

/// Strategy for generating SDP content
pub fn sdp_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        // Minimal SDP
        Just("v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\n".to_string()),
        
        // SDP with audio
        Just("v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 0\r\n".to_string()),
        
        // SDP with video
        Just("v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\nm=video 9 UDP/TLS/RTP/SAVPF 96\r\n".to_string()),
        
        // SDP with both audio and video
        Just("v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\nm=audio 9 UDP/TLS/RTP/SAVPF 0\r\nm=video 9 UDP/TLS/RTP/SAVPF 96\r\n".to_string()),
        
        // Random SDP-like content
        (any::<String>(), any::<String>())
            .prop_map(|(session_id, content)| {
                format!("v=0\r\no=- {} 0 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\n{}\r\n", 
                        session_id, content)
            }),
    ]
}

/// Strategy for generating ICE candidates
pub fn ice_candidate_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("candidate:1 1 UDP 2130706431 192.168.1.100 54400 typ host".to_string()),
        Just("candidate:2 1 UDP 1694498815 203.0.113.100 54400 typ srflx raddr 192.168.1.100 rport 54400".to_string()),
        Just("candidate:3 1 TCP 2105524479 192.168.1.100 9 typ host tcptype active".to_string()),
        (any::<u32>(), any::<std::net::IpAddr>(), any::<u16>(), any::<String>())
            .prop_map(|(foundation, address, port, candidate_type)| {
                format!("candidate:{} 1 UDP {} {} {} typ {}", 
                        foundation, 2130706431, address, port, candidate_type)
            }),
    ]
}

/// Property-based test utilities
pub struct ProptestTestConfig;

impl ProptestTestConfig {
    /// Default configuration for property tests
    pub fn default() -> proptest::test_runner::Config {
        proptest::test_runner::Config {
            cases: 1000, // Number of test cases
            max_shrink_iters: 1000,
            ..Default::default()
        }
    }

    /// Quick configuration for development
    pub fn quick() -> proptest::test_runner::Config {
        proptest::test_runner::Config {
            cases: 100,
            max_shrink_iters: 100,
            ..Default::default()
        }
    }

    /// Thorough configuration for CI
    pub fn thorough() -> proptest::test_runner::Config {
        proptest::test_runner::Config {
            cases: 10000,
            max_shrink_iters: 10000,
            ..Default::default()
        }
    }
}

/// Macro for running property tests with default configuration
#[macro_export]
macro_rules! proptest_default {
    (fn $test_name:ident($($param:ident: $param_type:ty),*) $body:block) => {
        proptest! {
            #![proptest_config(ProptestTestConfig::default())]
            #[test]
            fn $test_name($($param: $param_type),*) $body
        }
    };
}

/// Macro for running quick property tests during development
#[macro_export]
macro_rules! proptest_quick {
    (fn $test_name:ident($($param:ident: $param_type:ty),*) $body:block) => {
        proptest! {
            #![proptest_config(ProptestTestConfig::quick())]
            #[test]
            fn $test_name($($param: $param_type),*) $body
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestTestConfig::quick())]

        #[test]
        fn test_rtp_packet_properties(packet in rtp_packet_strategy()) {
            // RTP packets should always be valid
            assert!(packet.payload.len() <= 1188); // Max payload size
            assert!(packet.sequence_number <= 65535);
            assert!(packet.payload_type <= 127);
        }

        #[test]
        fn test_signaling_message_properties(message in signaling_message_strategy()) {
            // All signaling messages should have valid session IDs
            match &message {
                SignalingMessage::Offer { session_id, .. } |
                SignalingMessage::Answer { session_id, .. } |
                SignalingMessage::IceCandidate { session_id, .. } |
                SignalingMessage::IceComplete { session_id } |
                SignalingMessage::Bye { session_id } => {
                    assert!(!session_id.is_empty());
                    assert!(session_id.len() <= 100);
                }
            }
        }

        #[test]
        fn test_media_constraints_properties(constraints in media_constraints_strategy()) {
            // Media constraints should be logical
            if constraints.video && constraints.screen_share {
                // This is allowed but unusual
            }
            
            // At least one media type should be enabled for a valid call
            assert!(constraints.audio || constraints.video || constraints.screen_share);
        }

        #[test]
        fn test_call_id_properties(call_id in call_id_strategy()) {
            // Call IDs should be unique and valid
            let call_id_str = call_id.to_string();
            assert!(!call_id_str.is_empty());
            assert_eq!(call_id_str.len(), 36); // UUID format
        }

        #[test]
        fn test_socket_addr_properties(addr in socket_addr_strategy()) {
            // Socket addresses should be valid
            assert!(addr.port() > 0);
            assert!(addr.port() <= 65535);
        }

        #[test]
        fn test_session_id_properties(session_id in session_id_strategy()) {
            // Session IDs should be valid
            assert!(!session_id.is_empty());
            assert!(session_id.len() >= 8);
            assert!(session_id.len() <= 32);
            assert!(session_id.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-'));
        }
    }
}