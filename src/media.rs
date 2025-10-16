//! Media stream management for WebRTC
//!
//! This module handles audio, video, and screen share media streams.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::broadcast;
use webrtc::track::track_local::track_local_static_sample::TrackLocalStaticSample;
use webrtc::rtp_transceiver::rtp_codec::RTCRtpCodecCapability;
use crate::types::MediaType;

/// Media-related errors
#[derive(Error, Debug)]
pub enum MediaError {
    /// Device not found
    #[error("Device not found: {0}")]
    DeviceNotFound(String),

    /// Stream error
    #[error("Stream error: {0}")]
    StreamError(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Media events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaEvent {
    /// Device connected
    DeviceConnected {
        /// Device identifier
        device_id: String,
    },
    /// Device disconnected
    DeviceDisconnected {
        /// Device identifier
        device_id: String,
    },
    /// Stream started
    StreamStarted {
        /// Stream identifier
        stream_id: String,
    },
    /// Stream stopped
    StreamStopped {
        /// Stream identifier
        stream_id: String,
    },
}

/// Audio device
#[derive(Debug, Clone)]
pub struct AudioDevice {
    /// Device identifier
    pub id: String,
    /// Device name
    pub name: String,
}

/// Video device
#[derive(Debug, Clone)]
pub struct VideoDevice {
    /// Device identifier
    pub id: String,
    /// Device name
    pub name: String,
}

/// Audio track
#[derive(Debug, Clone)]
pub struct AudioTrack {
    /// Track identifier
    pub id: String,
}

/// Video track
#[derive(Debug, Clone)]
pub struct VideoTrack {
    /// Track identifier
    pub id: String,
}

/// WebRTC media track wrapper
#[derive(Debug, Clone)]
pub struct WebRtcTrack {
    /// Local WebRTC track
    pub track: Arc<TrackLocalStaticSample>,
    /// Track type
    pub track_type: MediaType,
    /// Track ID
    pub id: String,
}

/// Media stream
#[derive(Debug, Clone)]
pub struct MediaStream {
    /// Stream identifier
    pub id: String,
}

/// Media stream manager
pub struct MediaStreamManager {
    event_sender: broadcast::Sender<MediaEvent>,
    #[allow(dead_code)]
    audio_devices: Vec<AudioDevice>,
    #[allow(dead_code)]
    video_devices: Vec<VideoDevice>,
    webrtc_tracks: Vec<WebRtcTrack>,
}

impl MediaStreamManager {
    /// Create new media stream manager
    #[must_use]
    pub fn new() -> Self {
        let (event_sender, _) = broadcast::channel(100);
        Self {
            event_sender,
            audio_devices: Vec::new(),
            video_devices: Vec::new(),
            webrtc_tracks: Vec::new(),
        }
    }

    /// Initialize media devices
    ///
    /// # Errors
    ///
    /// Returns error if device initialization fails
    pub async fn initialize(&self) -> Result<(), MediaError> {
        // For now, add some fake devices for testing
        // In a real implementation, this would enumerate actual hardware devices
        let audio_device = AudioDevice {
            id: "default-audio".to_string(),
            name: "Default Audio Device".to_string(),
        };

        let video_device = VideoDevice {
            id: "default-video".to_string(),
            name: "Default Video Device".to_string(),
        };

        // Emit device connected events
        let _ = self.event_sender.send(MediaEvent::DeviceConnected {
            device_id: audio_device.id.clone(),
        });

        let _ = self.event_sender.send(MediaEvent::DeviceConnected {
            device_id: video_device.id.clone(),
        });

        Ok(())
    }

    /// Get available audio devices
    #[must_use]
    pub fn get_audio_devices(&self) -> &[AudioDevice] {
        // Return empty for now, as we can't enumerate real devices easily
        // In a real implementation, this would return actual devices
        &[]
    }

    /// Get available video devices
    #[must_use]
    pub fn get_video_devices(&self) -> &[VideoDevice] {
        // Return empty for now
        &[]
    }

    /// Create a new audio track
    ///
    /// # Errors
    ///
    /// Returns error if track creation fails
    pub async fn create_audio_track(&mut self) -> Result<&WebRtcTrack, MediaError> {
        let track_id = format!("audio-{}", self.webrtc_tracks.len());

        let codec = RTCRtpCodecCapability {
            mime_type: "audio/opus".to_string(),
            clock_rate: 48000,
            channels: 2,
            sdp_fmtp_line: "".to_string(),
            rtcp_feedback: vec![],
        };

        let track = Arc::new(TrackLocalStaticSample::new(
            codec,
            track_id.clone(),
            "audio".to_string(),
        ));

        let webrtc_track = WebRtcTrack {
            track,
            track_type: MediaType::Audio,
            id: track_id,
        };

        self.webrtc_tracks.push(webrtc_track);
        self.webrtc_tracks
            .last()
            .ok_or(MediaError::StreamError(
                "Failed to get last track after push".to_string(),
            ))
    }

    /// Create a new video track
    ///
    /// # Errors
    ///
    /// Returns error if track creation fails
    pub async fn create_video_track(&mut self) -> Result<&WebRtcTrack, MediaError> {
        let track_id = format!("video-{}", self.webrtc_tracks.len());

        let codec = RTCRtpCodecCapability {
            mime_type: "video/VP8".to_string(),
            clock_rate: 90000,
            channels: 0,
            sdp_fmtp_line: "".to_string(),
            rtcp_feedback: vec![],
        };

        let track = Arc::new(TrackLocalStaticSample::new(
            codec,
            track_id.clone(),
            "video".to_string(),
        ));

        let webrtc_track = WebRtcTrack {
            track,
            track_type: MediaType::Video,
            id: track_id,
        };

        self.webrtc_tracks.push(webrtc_track);
        self.webrtc_tracks
            .last()
            .ok_or(MediaError::StreamError(
                "Failed to get last track after push".to_string(),
            ))
    }

    /// Get all WebRTC tracks
    #[must_use]
    pub fn get_webrtc_tracks(&self) -> &[WebRtcTrack] {
        &self.webrtc_tracks
    }

    /// Subscribe to media events
    #[must_use]
    pub fn subscribe_events(&self) -> broadcast::Receiver<MediaEvent> {
        self.event_sender.subscribe()
    }

    /// Remove a track by ID
    ///
    /// Returns true if the track was found and removed
    pub fn remove_track(&mut self, track_id: &str) -> bool {
        if let Some(pos) = self.webrtc_tracks.iter().position(|t| t.id == track_id) {
            self.webrtc_tracks.remove(pos);
            tracing::debug!("Removed track: {}", track_id);
            true
        } else {
            tracing::warn!("Track not found for removal: {}", track_id);
            false
        }
    }
}

impl Default for MediaStreamManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_media_stream_manager_initialize() {
        let manager = MediaStreamManager::new();

        let result = manager.initialize().await;
        assert!(result.is_ok());

        // Check that events were sent
        let _events = manager.subscribe_events();
        // Note: In a real test, we'd need to handle the async nature,
        // but for now this is a basic structure test
    }

    #[tokio::test]
    async fn test_media_stream_manager_get_devices() {
        let manager = MediaStreamManager::new();

        let audio_devices = manager.get_audio_devices();
        assert!(audio_devices.is_empty());

        let video_devices = manager.get_video_devices();
        assert!(video_devices.is_empty());
    }

    #[tokio::test]
    async fn test_media_stream_manager_create_audio_track() {
        let mut manager = MediaStreamManager::new();

        let track = manager.create_audio_track().await.unwrap();
        assert_eq!(track.track_type, MediaType::Audio);
        assert!(track.id.starts_with("audio-"));

        let tracks = manager.get_webrtc_tracks();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].track_type, MediaType::Audio);
    }

    #[tokio::test]
    async fn test_media_stream_manager_create_video_track() {
        let mut manager = MediaStreamManager::new();

        let track = manager.create_video_track().await.unwrap();
        assert_eq!(track.track_type, MediaType::Video);
        assert!(track.id.starts_with("video-"));

        let tracks = manager.get_webrtc_tracks();
        assert_eq!(tracks.len(), 1);
        assert_eq!(tracks[0].track_type, MediaType::Video);
    }

    #[tokio::test]
    async fn test_media_stream_manager_multiple_tracks() {
        let mut manager = MediaStreamManager::new();

        manager.create_audio_track().await.unwrap();
        manager.create_video_track().await.unwrap();

        let tracks = manager.get_webrtc_tracks();
        assert_eq!(tracks.len(), 2);

        // Check track IDs are different
        assert_ne!(tracks[0].id, tracks[1].id);

        // Check that we have one audio and one video track
        let audio_count = tracks.iter().filter(|t| t.track_type == MediaType::Audio).count();
        let video_count = tracks.iter().filter(|t| t.track_type == MediaType::Video).count();

        assert_eq!(audio_count, 1);
        assert_eq!(video_count, 1);
    }
}
