use super::camera::CameraSettings;
use super::screen_capture::ScreenCaptureSettings;
use nokhwa;
use log::error;

#[derive(Default)]
pub struct VideoConfig {
    pub camera_settings: Vec<CameraSettings>,
    pub screen_capture: ScreenCaptureSettings,
}

impl VideoConfig {
    pub fn list_cameras() -> Vec<String> {
        match nokhwa::query(nokhwa::utils::ApiBackend::Auto) {
            Ok(cameras) => cameras.into_iter().map(|info| info.human_name()).collect(),
            Err(e) => {
                error!("カメラの検出エラー: {}", e);
                Vec::new()
            }
        }
    }
} 