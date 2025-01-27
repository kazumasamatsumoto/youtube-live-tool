#[derive(Default)]
pub struct BannerConfig {
    pub banners: Vec<BannerSettings>,
    pub default_duration: u32,
}

#[derive(Default)]
pub struct BannerSettings {
    pub text: String,
    pub enabled: bool,
    pub color: [f32; 3],
    pub duration: u32,
    pub position: BannerPosition,
}

#[derive(Default, PartialEq)]
pub enum BannerPosition {
    #[default]
    Top,
    Bottom,
} 