#[derive(Default)]
pub struct StreamConfig {
    pub stream_key: String,
    pub quality_settings: QualitySettings,
}

#[derive(Default)]
pub struct QualitySettings {
    pub video_bitrate: u32,
    pub audio_bitrate: u32,
    pub resolution: Resolution,
    pub fps: u32,
}

#[derive(Default)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
} 