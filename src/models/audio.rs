#[derive(Default)]
pub struct AudioConfig {
    pub master_volume: f32,
    pub bgm_volume: f32,
    pub effects_volume: f32,
}

#[derive(Default)]
pub struct BGMTrack {
    pub name: String,
    pub file_path: String,
    pub volume: f32,
}

#[derive(Default)]
pub struct SoundEffect {
    pub name: String,
    pub file_path: String,
    pub hotkey: String,
    pub volume: f32,
} 