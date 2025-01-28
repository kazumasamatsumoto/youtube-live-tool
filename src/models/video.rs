use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::{Camera, NokhwaError};
use image::RgbImage;
use std::time::{Duration, Instant};
use log::{info, error};

#[derive(Default)]
pub struct VideoConfig {
    pub camera_settings: Vec<CameraSettings>,
    pub screen_capture: ScreenCaptureSettings,
}

pub struct CameraSettings {
    pub device_id: String,
    pub name: String,
    pub enabled: bool,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub camera: Option<Camera>,
    pub frame: Option<RgbImage>,
    pub frame_buffer: Option<Vec<u8>>,
    pub last_frame_time: Instant,
    pub frame_interval: Duration,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            device_id: String::new(),
            name: String::new(),
            enabled: false,
            position: (0.0, 0.0),
            size: (640.0, 480.0),
            camera: None,
            frame: None,
            frame_buffer: None,
            last_frame_time: Instant::now(),
            frame_interval: Duration::from_micros(16667), // 60 FPS
        }
    }
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

impl CameraSettings {
    pub fn new(index: usize) -> Result<Self, NokhwaError> {
        info!("[Camera] カメラの初期化開始 - インデックス: {}", index);
        let camera_index = CameraIndex::Index(index.try_into().unwrap());
        
        // カメラの初期化を試みる
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);
        info!("[Camera] カメラインスタンスの作成を試行");
        let mut camera = Camera::new(camera_index, requested)?;
        info!("[Camera] カメラインスタンスの作成成功");
        
        // フレームレートの設定を試みる（60FPSを目標に）
        let target_fps = 60.0;
        let frame_interval = if let Ok(supported_formats) = camera.compatible_camera_formats() {
            let max_fps = supported_formats
                .iter()
                .map(|f| f.frame_rate() as u32)
                .max()
                .unwrap_or(60);
            
            let fps = f32::min(target_fps, max_fps as f32) as u32;
            info!("[Camera] フレームレート設定 - 目標: {}, 最大: {}, 設定値: {}", target_fps, max_fps, fps);
            if let Err(e) = camera.set_frame_rate(fps) {
                error!("[Camera] フレームレート設定エラー: {}、デフォルト値を使用します", e);
            }
            Duration::from_secs_f32(1.0 / fps as f32)
        } else {
            info!("[Camera] フレームレート情報の取得失敗 - デフォルト値(60FPS)を使用");
            Duration::from_micros(16667) // デフォルトで60FPS
        };

        // ストリームを開く
        info!("[Camera] ストリームのオープンを試行");
        camera.open_stream()?;
        info!("[Camera] ストリームのオープン成功");

        Ok(CameraSettings {
            device_id: index.to_string(),
            name: camera.info().human_name(),
            enabled: true,
            position: (0.0, 0.0),
            size: (640.0, 480.0),
            camera: Some(camera),
            frame: None,
            frame_buffer: None,
            last_frame_time: Instant::now(),
            frame_interval,
        })
    }

    pub fn capture_frame(&mut self) -> Result<(), NokhwaError> {
        info!("[Camera] フレーム取得開始 - カメラ: {}", self.name);
        let camera = self.camera.as_mut().ok_or_else(|| {
            NokhwaError::GeneralError("カメラが初期化されていません".to_string())
        })?;

        // フレームレート制御
        let elapsed = self.last_frame_time.elapsed();
        if elapsed < self.frame_interval {
            info!("[Camera] フレームレート制御 - 次のフレームまで待機 (経過: {:?}, 間隔: {:?})", elapsed, self.frame_interval);
            return Ok(());
        }

        info!("[Camera] フレーム取得処理開始 - 最大3回まで再試行");
        // フレームの取得（最大3回まで再試行）
        let mut retry_count = 0;
        let max_retries = 3;
        
        while retry_count < max_retries {
            match camera.frame() {
                Ok(frame) => {
                    info!("[Camera] フレーム取得成功 - 試行回数: {}", retry_count + 1);
                    match frame.decode_image::<RgbFormat>() {
                        Ok(decoded) => {
                            info!("[Camera] フレームデコード成功 - サイズ: {}x{}", decoded.width(), decoded.height());
                            let width = decoded.width();
                            let height = decoded.height();
                            let raw_data = decoded.into_raw();
                            
                            // フレームバッファの再利用
                            if let Some(buffer) = &mut self.frame_buffer {
                                if buffer.len() == raw_data.len() {
                                    info!("[Camera] 既存バッファ再利用 - サイズ: {}", buffer.len());
                                    buffer.copy_from_slice(&raw_data);
                                    if let Some(image) = RgbImage::from_raw(width, height, buffer.clone()) {
                                        self.frame = Some(image);
                                        self.last_frame_time = Instant::now();
                                        info!("[Camera] フレーム更新完了 - バッファ再利用");
                                        return Ok(());
                                    }
                                }
                            }
                            
                            // 新しいバッファの作成
                            info!("[Camera] 新規バッファ作成 - サイズ: {}", raw_data.len());
                            self.frame_buffer = Some(raw_data.clone());
                            if let Some(image) = RgbImage::from_raw(width, height, raw_data) {
                                self.frame = Some(image);
                                self.last_frame_time = Instant::now();
                                info!("[Camera] フレーム更新完了 - 新規バッファ");
                                return Ok(());
                            }
                            
                            error!("[Camera] フレームデータの変換に失敗");
                            return Err(NokhwaError::GeneralError("フレームデータの変換に失敗しました".to_string()));
                        }
                        Err(e) => {
                            error!("[Camera] フレームのデコードエラー - 試行 {}/{}: {}", retry_count + 1, max_retries, e);
                            retry_count += 1;
                            if retry_count == max_retries {
                                error!("[Camera] デコードエラー - 最大試行回数到達");
                                return Err(e);
                            }
                            info!("[Camera] 5ms待機後に再試行");
                            std::thread::sleep(Duration::from_millis(5));
                            continue;
                        }
                    }
                }
                Err(e) => {
                    error!("[Camera] フレームの取得エラー - 試行 {}/{}: {}", retry_count + 1, max_retries, e);
                    retry_count += 1;
                    if retry_count == max_retries {
                        error!("[Camera] フレーム取得エラー - 最大試行回数到達");
                        return Err(e);
                    }
                    info!("[Camera] 5ms待機後に再試行");
                    std::thread::sleep(Duration::from_millis(5));
                    continue;
                }
            }
        }
        
        error!("[Camera] フレーム取得失敗 - 最大試行回数を超過");
        Err(NokhwaError::GeneralError("フレーム取得の再試行回数を超過しました".to_string()))
    }
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
