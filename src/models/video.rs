#[derive(Default)]
pub struct VideoConfig {
    pub camera_settings: Vec<CameraSettings>,
    pub screen_capture: ScreenCaptureSettings,
}

#[derive(Default)]
pub struct CameraSettings {
    pub device_id: String,
    pub name: String,
    pub enabled: bool,
    pub position: (f32, f32),
    pub size: (f32, f32),
}

#[derive(Default)]
pub struct ScreenCaptureSettings {
    pub enabled: bool,
    pub area_type: CaptureAreaType,
    pub position: (f32, f32),
    pub size: (f32, f32),
}

#[derive(Default, PartialEq)]
pub enum CaptureAreaType {
    #[default]
    FullScreen,
    Window,
    Custom,
} 