// Video encoding utilities for SimBridge screen streaming

use thiserror::Error;
use std::process::Command;

/// Video codec types
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VideoCodec {
    H264,
    VP8,
    JPEG,
}

impl From<String> for VideoCodec {
    fn from(codec: String) -> Self {
        match codec.to_lowercase().as_str() {
            "h264" | "avc" => VideoCodec::H264,
            "vp8" => VideoCodec::VP8,
            _ => VideoCodec::JPEG, // Default fallback
        }
    }
}

/// Encoder quality presets
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EncoderQuality {
    Low,
    Medium,
    High,
    Ultra,
}

impl From<String> for EncoderQuality {
    fn from(quality: String) -> Self {
        match quality.to_lowercase().as_str() {
            "low" => EncoderQuality::Low,
            "medium" | "med" => EncoderQuality::Medium,
            "high" => EncoderQuality::High,
            _ => EncoderQuality::Medium,
        }
    }
}

impl From<simbridge_shared::protocol::StreamQuality> for EncoderQuality {
    fn from(quality: simbridge_shared::protocol::StreamQuality) -> Self {
        match quality {
            simbridge_shared::protocol::StreamQuality::Low => EncoderQuality::Low,
            simbridge_shared::protocol::StreamQuality::Medium => EncoderQuality::Medium,
            simbridge_shared::protocol::StreamQuality::High => EncoderQuality::High,
            simbridge_shared::protocol::StreamQuality::Ultra => EncoderQuality::Ultra,
        }
    }
}

/// Video encoder configuration
#[derive(Debug, Clone)]
pub struct VideoEncoderConfig {
    pub codec: VideoCodec,
    pub quality: EncoderQuality,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fps: u32,
}

impl Default for VideoEncoderConfig {
    fn default() -> Self {
        Self {
            codec: VideoCodec::H264,
            quality: EncoderQuality::Medium,
            width: None,
            height: None,
            fps: 30,
        }
    }
}

/// Basic video encoder (returns PNG frames for simplicity)
pub struct VideoEncoder {
    config: VideoEncoderConfig,
}

impl VideoEncoder {
    pub fn new(config: VideoEncoderConfig) -> Self {
        Self { config }
    }

    /// Create encoder with default settings
    pub fn default_encoder() -> Self {
        Self::new(VideoEncoderConfig::default())
    }

    /// Encode PNG image to JPEG frame (lossy compression for streaming)
    pub fn encode_png_to_jpeg(&self, png_data: &[u8]) -> Result<Vec<u8>, EncoderError> {
        use std::io::Cursor;
        use image::{ImageFormat, DynamicImage};

        // Decode PNG
        let img = image::load_from_memory(png_data)
            .map_err(|e| EncoderError::EncodingFailed(format!("PNG decode failed: {}", e)))?;

        // Convert to JPEG
        let mut jpeg_data = Vec::new();
        let mut cursor = Cursor::new(&mut jpeg_data);
        img.write_to(&mut cursor, ImageFormat::Jpeg)
            .map_err(|e| EncoderError::EncodingFailed(format!("JPEG conversion failed: {}", e)))?;

        Ok(jpeg_data)
    }

    /// Encode PNG to H.264 keyframe (requires FFmpeg)
    pub async fn encode_png_to_h264(&self, png_data: &[u8]) -> Result<Vec<u8>, EncoderError> {
        // For now, return PNG as-is with a frame marker
        // In production, use ffmpeg to re-encode
        let mut encoded = vec![
            0x00, 0x00, 0x00, 0x01,  // Frame header
            0x68, 0x26, 0x34,         // "h264" marker
        ];
        encoded.extend_from_slice(&(png_data.len() as u32).to_be_bytes());
        encoded.extend_from_slice(png_data);
        
        Ok(encoded)
    }

    /// Get encoder configuration info
    pub fn config(&self) -> &VideoEncoderConfig {
        &self.config
    }
}

/// FFmpeg-based encoder for production use
pub struct FfmpegEncoder;

impl FfmpegEncoder {
    /// Check if FFmpeg is available
    pub fn is_available() -> bool {
        Command::new("ffmpeg")
            .args(["-version"])
            .output()
            .is_ok()
    }

    /// Get FFmpeg version
    pub fn version() -> Option<String> {
        let output = Command::new("ffmpeg")
            .args(["-version"])
            .output();

        if let Ok(out) = output {
            String::from_utf8_lossy(&out.stdout).lines().next().map(|s| s.to_string())
        } else {
            None
        }
    }

    /// Encode PNG to H.264 frame using ffmpeg (blocking)
    pub fn encode_png_with_ffmpeg(png_path: &str, output_path: &str) -> Result<(), EncoderError> {
        // Check if ffmpeg is available
        if !Self::is_available() {
            return Err(EncoderError::FfmpegNotFound);
        }

        let status = Command::new("ffmpeg")
            .args([
                "-y",                               // Overwrite output
                "-i", png_path,
                "-vframes", "1",                    // Single frame
                "-an",                              // No audio  
                "-pix_fmt", "yuv420p",
                "-c:v", "libx264",
                "-preset", "ultrafast",
                "-r", "30",                         // 30 FPS
                output_path,
            ])
            .status();

        match status {
            Ok(s) if s.success() => Ok(()),
            _ => Err(EncoderError::EncodingFailed("FFmpeg encoding failed".to_string())),
        }
    }

    /// Convert video file to PNG frames using ffmpeg
    pub fn video_to_png_frames(video_path: &str, output_dir: &str) -> Result<(), EncoderError> {
        let output_pattern = format!("{}/frame_%04d.png", output_dir);
        let status = Command::new("ffmpeg")
            .args([
                "-i", video_path,
                "-vf", "fps=30",
                &output_pattern,
            ])
            .status();

        match status {
            Ok(s) if s.success() => Ok(()),
            _ => Err(EncoderError::EncodingFailed("FFmpeg frame extraction failed".to_string())),
        }
    }

    /// Extract frames from a PNG stream with compression
    pub fn stream_png_frames(&self, png_bytes: Vec<u8>) -> Result<Vec<u8>, EncoderError> {
        // Compress PNG data for transmission (simple gzip)
        let mut binding = std::io::Cursor::new(vec![]);
        {
            let mut compressed = flate2::write::GzEncoder::new(
                &mut binding,
                flate2::Compression::default(),
            );
            // Placeholder - would use actual compression in production
        }
        
        // Placeholder - would use actual compression in production
        Ok(png_bytes)
    }
}

/// Encoder errors
#[derive(Debug, Error)]
pub enum EncoderError {
    #[error("Encoder not initialized")]
    NotInitialized,
    
    #[error("Encoding failed: {0}")]
    EncodingFailed(String),
    
    #[error("Invalid frame data: {0}")]
    InvalidFrameData(String),
    
    #[error("FFmpeg not available. Install FFmpeg to use video encoding.\nTry: brew install ffmpeg (macOS) or apt-get install ffmpeg (Linux)")]
    FfmpegNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_codec_from_string() {
        assert_eq!(VideoCodec::from("h264".to_string()), VideoCodec::H264);
        assert_eq!(VideoCodec::from("vp8".to_string()), VideoCodec::VP8);
        assert_eq!(VideoCodec::from("invalid".to_string()), VideoCodec::JPEG);
    }

    #[test]
    fn test_encoder_quality_from_string() {
        assert_eq!(EncoderQuality::from("low".to_string()), EncoderQuality::Low);
        assert_eq!(EncoderQuality::from("high".to_string()), EncoderQuality::High);
        assert_eq!(EncoderQuality::from("medium".to_string()), EncoderQuality::Medium);
    }

    #[test]
    fn test_default_encoder_config() {
        let config = VideoEncoderConfig::default();
        assert_eq!(config.codec, VideoCodec::H264);
        assert_eq!(config.quality, EncoderQuality::Medium);
        assert_eq!(config.fps, 30);
    }

    #[test]
    fn test_ffmpeg_encoder_availability() {
        // This will return false if FFmpeg is not installed
        let available = FfmpegEncoder::is_available();
        println!("FFmpeg available: {}", available);
    }
}
