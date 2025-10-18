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

/// QUIC stream type for media
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaStreamType {
    /// Audio stream
    Audio,
    /// Video stream
    Video,
    /// Screen share stream
    ScreenShare,
    /// Data channel
    DataChannel,
}

/// Active QUIC media stream
pub struct QuicMediaStream {
    /// Stream type
    pub stream_type: MediaStreamType,
    /// QoS parameters
    pub qos_params: QoSParams,
    /// Stream ID (placeholder for actual QUIC stream)
    pub stream_id: u64,
}

/// QUIC media stream manager
pub struct QuicMediaStreamManager {
    streams: std::collections::HashMap<u64, QuicMediaStream>,
    next_stream_id: u64,
}

impl QuicMediaStreamManager {
    /// Create new stream manager with QoS parameters
    #[must_use]
    pub fn new(_qos: QoSParams) -> Self {
        Self {
            streams: std::collections::HashMap::new(),
            next_stream_id: 0,
        }
    }

    /// Create a new media stream
    ///
    /// # Errors
    ///
    /// Returns error if stream creation fails
    pub fn create_stream(&mut self, stream_type: MediaStreamType) -> Result<u64, StreamError> {
        let stream_id = self.next_stream_id;
        self.next_stream_id += 1;

        let qos_params = match stream_type {
            MediaStreamType::Audio => QoSParams::audio(),
            MediaStreamType::Video => QoSParams::video(),
            MediaStreamType::ScreenShare => QoSParams::screen_share(),
            MediaStreamType::DataChannel => QoSParams::audio(), // Default QoS for data
        };

        let stream = QuicMediaStream {
            stream_type,
            qos_params,
            stream_id,
        };

        self.streams.insert(stream_id, stream);
        Ok(stream_id)
    }

    /// Get stream by ID
    #[must_use]
    pub fn get_stream(&self, stream_id: u64) -> Option<&QuicMediaStream> {
        self.streams.get(&stream_id)
    }

    /// Close a stream
    ///
    /// # Errors
    ///
    /// Returns error if stream not found
    pub fn close_stream(&mut self, stream_id: u64) -> Result<(), StreamError> {
        if self.streams.remove(&stream_id).is_some() {
            Ok(())
        } else {
            Err(StreamError::OperationError("Stream not found".to_string()))
        }
    }

    /// Send data on a stream
    ///
    /// # Errors
    ///
    /// Returns error if sending fails
    pub async fn send_data(&self, stream_id: u64, _data: &[u8]) -> Result<(), StreamError> {
        if self.streams.contains_key(&stream_id) {
            // TODO: Implement actual QUIC stream sending
            Ok(())
        } else {
            Err(StreamError::OperationError("Stream not found".to_string()))
        }
    }

    /// Receive data from a stream
    ///
    /// # Errors
    ///
    /// Returns error if receiving fails
    pub async fn receive_data(&self, stream_id: u64) -> Result<Vec<u8>, StreamError> {
        if self.streams.contains_key(&stream_id) {
            // TODO: Implement actual QUIC stream receiving
            Err(StreamError::OperationError("Not implemented".to_string()))
        } else {
            Err(StreamError::OperationError("Stream not found".to_string()))
        }
    }

    /// Get all active streams
    #[must_use]
    pub fn active_streams(&self) -> Vec<&QuicMediaStream> {
        self.streams.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quic_media_stream_manager_create_stream() {
        let mut manager = QuicMediaStreamManager::new(QoSParams::audio());

        let stream_id = manager.create_stream(MediaStreamType::Audio).unwrap();
        assert_eq!(stream_id, 0);

        let stream = manager.get_stream(stream_id).unwrap();
        assert_eq!(stream.stream_type, MediaStreamType::Audio);
        assert_eq!(stream.qos_params.priority, QoSParams::audio().priority);
    }

    #[test]
    fn test_quic_media_stream_manager_multiple_streams() {
        let mut manager = QuicMediaStreamManager::new(QoSParams::audio());

        let audio_id = manager.create_stream(MediaStreamType::Audio).unwrap();
        let video_id = manager.create_stream(MediaStreamType::Video).unwrap();
        let screen_id = manager.create_stream(MediaStreamType::ScreenShare).unwrap();

        assert_eq!(audio_id, 0);
        assert_eq!(video_id, 1);
        assert_eq!(screen_id, 2);

        let active = manager.active_streams();
        assert_eq!(active.len(), 3);
    }

    #[test]
    fn test_quic_media_stream_manager_close_stream() {
        let mut manager = QuicMediaStreamManager::new(QoSParams::audio());

        let stream_id = manager.create_stream(MediaStreamType::Audio).unwrap();
        assert!(manager.get_stream(stream_id).is_some());

        manager.close_stream(stream_id).unwrap();
        assert!(manager.get_stream(stream_id).is_none());
    }

    #[test]
    fn test_quic_media_stream_manager_close_nonexistent_stream() {
        let mut manager = QuicMediaStreamManager::new(QoSParams::audio());

        let result = manager.close_stream(999);
        assert!(matches!(result, Err(StreamError::OperationError(_))));
    }

    #[tokio::test]
    async fn test_quic_media_stream_manager_send_data() {
        let mut manager = QuicMediaStreamManager::new(QoSParams::audio());

        let stream_id = manager.create_stream(MediaStreamType::Audio).unwrap();

        let data = vec![1, 2, 3, 4];
        let result = manager.send_data(stream_id, &data).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_quic_media_stream_manager_send_data_nonexistent_stream() {
        let manager = QuicMediaStreamManager::new(QoSParams::audio());

        let data = vec![1, 2, 3, 4];
        let result = manager.send_data(999, &data).await;
        assert!(matches!(result, Err(StreamError::OperationError(_))));
    }

    #[tokio::test]
    async fn test_quic_media_stream_manager_receive_data() {
        let mut manager = QuicMediaStreamManager::new(QoSParams::audio());

        let stream_id = manager.create_stream(MediaStreamType::Audio).unwrap();

        let result = manager.receive_data(stream_id).await;
        assert!(matches!(result, Err(StreamError::OperationError(_))));
    }

    #[test]
    fn test_quic_media_stream_manager_get_nonexistent_stream() {
        let manager = QuicMediaStreamManager::new(QoSParams::audio());

        assert!(manager.get_stream(999).is_none());
    }

    #[test]
    fn test_qos_params_audio() {
        let audio = QoSParams::audio();
        assert_eq!(audio.target_latency_ms, 50);
        assert_eq!(audio.priority, 10);
    }

    #[test]
    fn test_qos_params_video() {
        let video = QoSParams::video();
        assert_eq!(video.target_latency_ms, 150);
        assert_eq!(video.priority, 5);
    }

    #[test]
    fn test_qos_params_screen_share() {
        let screen = QoSParams::screen_share();
        assert_eq!(screen.target_latency_ms, 200);
        assert_eq!(screen.priority, 3);
    }
}
