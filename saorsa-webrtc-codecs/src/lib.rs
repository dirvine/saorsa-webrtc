//! Video and audio codec implementations

pub mod openh264;

use bytes::Bytes;
use anyhow::Result;

/// Video codec selection
#[derive(Debug, Clone, Copy)]
pub enum VideoCodec {
H264,
    // VP8,  // Future
    // VP9,  // Future
}

/// Audio codec selection
#[derive(Debug, Clone, Copy)]
pub enum AudioCodec {
    Opus,
    // G711,  // Future
}

/// Video frame
#[derive(Debug, Clone)]
pub struct VideoFrame {
pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub timestamp: u64,
}

/// Video encoder trait
pub trait VideoEncoder: Send + Sync {
    fn encode(&mut self, frame: &VideoFrame) -> Result<Bytes>;
    fn request_keyframe(&mut self);
}

/// Video decoder trait
pub trait VideoDecoder: Send + Sync {
    fn decode(&mut self, data: &[u8]) -> Result<VideoFrame>;
}

// Re-export OpenH264 implementations
pub use openh264::{OpenH264Encoder, OpenH264Decoder};
