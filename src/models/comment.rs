#[allow(dead_code)]
pub struct CommentConfig {
    pub filter: FilterSettings,
    pub voice: VoiceSettings,
    pub display: DisplaySettings,
}

#[allow(dead_code)]
pub struct FilterSettings {
    pub block_words: Vec<String>,
    pub min_account_age_days: u32,
    pub block_non_members: bool,
    pub block_first_time: bool,
}

#[allow(dead_code)]
pub struct VoiceSettings {
    pub enabled: bool,
    pub voice_type: String,
    pub speed: f32,
    pub pitch: f32,
    pub volume: f32,
}

#[allow(dead_code)]
pub struct DisplaySettings {
    pub font_size: u32,
    pub display_time: u32,
    pub show_username: bool,
    pub show_member_icon: bool,
    pub color_member_names: bool,
} 