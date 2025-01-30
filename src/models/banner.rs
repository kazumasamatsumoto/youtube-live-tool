#[derive(Default)]
#[allow(dead_code)]
pub struct BannerConfig {
    pub banners: Vec<BannerSettings>,
    pub default_duration: u32,
}

#[derive(Default)]
#[allow(dead_code)]
pub struct BannerSettings {
    pub text: String,
    pub enabled: bool,
    pub color: [f32; 3],
    pub duration: u32,
    pub position: BannerPosition,
}

#[derive(Default)]
#[allow(dead_code)]
pub enum BannerPosition {
    #[default]
    Top,
    Bottom,
} 