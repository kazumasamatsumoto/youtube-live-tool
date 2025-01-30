#[allow(dead_code)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub bgm_volume: f32,
    pub effects_volume: f32,
}

#[allow(dead_code)]
pub struct BGMTrack {
    pub name: String,
    pub file_path: String,
    pub volume: f32,
}

#[allow(dead_code)]
pub struct SoundEffect {
    pub name: String,
    pub file_path: String,
    pub hotkey: String,
    pub volume: f32,
} 