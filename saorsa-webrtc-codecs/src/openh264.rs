//! OpenH264 codec implementation

use crate::{VideoDecoder, VideoEncoder, VideoFrame};
use anyhow::{anyhow, Result};
use bytes::Bytes;

/// OpenH264 video encoder (stub implementation for now)
/// TODO: Replace with full OpenH264 integration when API is available
pub struct OpenH264Encoder {
    width: u32,
    height: u32,
}

impl OpenH264Encoder {
    /// Create a new H.264 encoder
    pub fn new() -> Result<Self> {
        // Default to 640x480 for now
        Ok(Self {
            width: 640,
            height: 480,
        })
    }

    /// Create a new H.264 encoder with specified dimensions
    pub fn with_dimensions(width: u32, height: u32) -> Self {
        Self { width, height }
    }
}

impl VideoEncoder for OpenH264Encoder {
    fn encode(&mut self, frame: &VideoFrame) -> Result<Bytes> {
        // Validate frame dimensions
        if frame.width != self.width || frame.height != self.height {
            return Err(anyhow!(
                "Frame dimensions ({},{}) don't match encoder config ({},{})",
                frame.width, frame.height, self.width, self.height
            ));
        }

        // TODO: Implement actual H.264 encoding with OpenH264
        // For now, simulate compression by returning a smaller buffer
        // In a real implementation, this would:
        // 1. Convert RGB to YUV420
        // 2. Encode with OpenH264
        // 3. Return H.264 bitstream

        // Simulate some compression (real H.264 would compress much more)
        let original_size = frame.data.len();
        let compressed_size = original_size / 4; // Rough simulation

        // Create a simple compressed representation
        let mut compressed = Vec::with_capacity(compressed_size + 8);
        compressed.extend_from_slice(&(frame.width as u32).to_le_bytes());
        compressed.extend_from_slice(&(frame.height as u32).to_le_bytes());
        compressed.extend_from_slice(&(frame.timestamp as u32).to_le_bytes());

        // Simple RLE compression simulation
        let mut i = 0;
        while i < frame.data.len() && compressed.len() < compressed_size {
            let mut count = 1;
            while i + count < frame.data.len() && frame.data[i] == frame.data[i + count] && count < 255 {
                count += 1;
            }
            compressed.push(count as u8);
            compressed.push(frame.data[i]);
            i += count;
        }

        Ok(Bytes::from(compressed))
    }

    fn request_keyframe(&mut self) {
        // TODO: Implement keyframe request in OpenH264
        // For now, this is a no-op
    }
}

/// OpenH264 video decoder (stub implementation for now)
pub struct OpenH264Decoder;

impl OpenH264Decoder {
    /// Create a new H.264 decoder
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl VideoDecoder for OpenH264Decoder {
    fn decode(&mut self, data: &[u8]) -> Result<VideoFrame> {
        // TODO: Implement actual H.264 decoding with OpenH264
        // For now, simulate decompression

        if data.len() < 12 {
            return Err(anyhow!("Compressed data too small"));
        }

        // Read header (simulated)
        let width = u32::from_le_bytes(data[0..4].try_into().unwrap());
        let height = u32::from_le_bytes(data[4..8].try_into().unwrap());
        let timestamp = u32::from_le_bytes(data[8..12].try_into().unwrap()) as u64;

        // Simulate decompression
        let expected_rgb_size = (width * height * 3) as usize;
        let mut rgb_data = Vec::with_capacity(expected_rgb_size);

        let mut i = 12; // Skip header
        while i < data.len() && rgb_data.len() < expected_rgb_size {
            if i + 1 >= data.len() {
                break;
            }
            let count = data[i] as usize;
            let value = data[i + 1];
            for _ in 0..count {
                if rgb_data.len() < expected_rgb_size {
                    rgb_data.push(value);
                }
            }
            i += 2;
        }

        // Fill remaining with zeros if needed
        while rgb_data.len() < expected_rgb_size {
            rgb_data.push(0);
        }

        Ok(VideoFrame {
            data: rgb_data,
            width,
            height,
            timestamp,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openh264_encoder_creation() {
        let result = OpenH264Encoder::new();
        assert!(result.is_ok());
        let encoder = result.unwrap();
        assert_eq!(encoder.width, 640);
        assert_eq!(encoder.height, 480);
    }

    #[test]
    fn test_openh264_decoder_creation() {
        let result = OpenH264Decoder::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_encoder_basic_functionality() {
        let mut encoder = OpenH264Encoder::new().unwrap();

        // Create a test frame
        let frame = VideoFrame {
            data: vec![128; 640 * 480 * 3], // Gray frame
            width: 640,
            height: 480,
            timestamp: 12345,
        };

        let result = encoder.encode(&frame);
        assert!(result.is_ok());

        let compressed = result.unwrap();
        assert!(compressed.len() > 0);
        assert!(compressed.len() < frame.data.len()); // Should be compressed
    }

    #[test]
    fn test_decoder_basic_functionality() {
        let mut encoder = OpenH264Encoder::new().unwrap();
        let mut decoder = OpenH264Decoder::new().unwrap();

        // Create and encode a frame
        let original_frame = VideoFrame {
            data: vec![200; 640 * 480 * 3], // Light gray frame
            width: 640,
            height: 480,
            timestamp: 67890,
        };

        let compressed = encoder.encode(&original_frame).unwrap();

        // Decode the frame
        let decoded_frame = decoder.decode(&compressed).unwrap();

        // Check that dimensions are preserved
        assert_eq!(decoded_frame.width, original_frame.width);
        assert_eq!(decoded_frame.height, original_frame.height);
        assert_eq!(decoded_frame.timestamp, original_frame.timestamp);

        // Check that data is reconstructed (will be approximate due to compression)
        assert_eq!(decoded_frame.data.len(), original_frame.data.len());
    }

    #[test]
    fn test_encoder_invalid_frame_size() {
        let mut encoder = OpenH264Encoder::new().unwrap();

        // Create frame with wrong dimensions
        let frame = VideoFrame {
            data: vec![0; 320 * 240 * 3], // 320x240 instead of 640x480
            width: 320,
            height: 240,
            timestamp: 0,
        };

        let result = encoder.encode(&frame);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("dimensions"));
    }
}
