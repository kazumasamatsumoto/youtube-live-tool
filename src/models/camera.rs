use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use nokhwa::{Camera, NokhwaError};
use image::RgbImage;

struct ThreadSafeCamera(Camera);
unsafe impl Send for ThreadSafeCamera {}

pub struct CameraSettings {
    pub device_id: String,
    pub name: String,
    pub enabled: bool,
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub(crate) frame: Arc<Mutex<Option<RgbImage>>>,
    camera: Arc<Mutex<Option<ThreadSafeCamera>>>,
    frame_front: Arc<Mutex<Option<RgbImage>>>,
    frame_back: Arc<Mutex<Option<RgbImage>>>,
    buffer_swap_needed: Arc<AtomicBool>,
    #[allow(dead_code)]
    last_frame_time: Instant,
    frame_interval: Duration,
    running: Arc<Mutex<bool>>,
    capture_thread: Option<thread::JoinHandle<()>>,
}

impl CameraSettings {
    pub fn new(index: usize) -> Result<Self, NokhwaError> {
        let camera_index = CameraIndex::Index(index.try_into().unwrap());
        let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);
        let mut camera = Camera::new(camera_index, requested)?;

        // カメラの設定を最適化
        if let Ok(formats) = camera.compatible_camera_formats() {
            let optimal_format = formats.into_iter()
                .filter(|f| f.width() == 640 && f.height() == 480)
                .max_by_key(|f| f.frame_rate());

            if let Some(format) = optimal_format {
                let requested = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Exact(format));
                camera.set_camera_requset(requested)?;
            }
        }

        let frame_interval = Duration::from_secs_f32(1.0 / 120.0);
        camera.open_stream()?;

        let mut settings = CameraSettings {
            device_id: index.to_string(),
            name: camera.info().human_name(),
            enabled: true,
            position: (0.0, 0.0),
            size: (640.0, 480.0),
            camera: Arc::new(Mutex::new(Some(ThreadSafeCamera(camera)))),
            frame: Arc::new(Mutex::new(None)),
            frame_front: Arc::new(Mutex::new(None)),
            frame_back: Arc::new(Mutex::new(None)),
            buffer_swap_needed: Arc::new(AtomicBool::new(false)),
            last_frame_time: Instant::now(),
            frame_interval,
            running: Arc::new(Mutex::new(true)),
            capture_thread: None,
        };

        settings.start_capture_thread();
        Ok(settings)
    }

    #[allow(dead_code)]
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if enabled && self.capture_thread.is_none() {
            self.start_capture_thread();
        } else if !enabled {
            if let Ok(mut running) = self.running.lock() {
                *running = false;
            }
            if let Some(thread) = self.capture_thread.take() {
                let _ = thread.join();
            }
        }
    }

    fn start_capture_thread(&mut self) {
        let frame = Arc::clone(&self.frame);
        let frame_back = Arc::clone(&self.frame_back);
        let buffer_swap_needed = Arc::clone(&self.buffer_swap_needed);
        let running = Arc::clone(&self.running);
        let camera = Arc::clone(&self.camera);
        let frame_interval = self.frame_interval;

        *running.lock().unwrap() = true;

        let handle = thread::spawn(move || {
            let mut last_frame_time = Instant::now();
            let mut buffer = vec![0u8; 640 * 480 * 3];

            while *running.lock().unwrap() {
                if let Ok(mut camera_guard) = camera.lock() {
                    if let Some(ThreadSafeCamera(ref mut cam)) = camera_guard.as_mut() {
                        match cam.frame() {
                            Ok(frame_data) => {
                                if let Ok(decoded) = frame_data.decode_image::<RgbFormat>() {
                                    let raw_data = decoded.into_raw();
                                    if raw_data.len() == buffer.len() {
                                        buffer.copy_from_slice(&raw_data);
                                        
                                        // 新しいイメージを作成
                                        if let Some(image) = RgbImage::from_raw(640, 480, buffer.clone()) {
                                            // 従来のframeも更新
                                            if let Ok(mut frame_guard) = frame.lock() {
                                                *frame_guard = Some(image.clone());
                                            }
                                            
                                            // バックバッファに書き込み
                                            if let Ok(mut back_guard) = frame_back.lock() {
                                                *back_guard = Some(image);
                                                buffer_swap_needed.store(true, Ordering::Release);
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("フレーム取得エラー: {}", e);
                                thread::sleep(Duration::from_micros(1));
                                continue;
                            }
                        }
                    }
                }

                let elapsed = last_frame_time.elapsed();
                if elapsed < frame_interval {
                    while last_frame_time.elapsed() < frame_interval {
                        std::hint::spin_loop();
                    }
                }
                last_frame_time = Instant::now();
            }
        });

        self.capture_thread = Some(handle);
    }

    pub fn get_frame(&self) -> Option<RgbImage> {
        // バッファのスワップが必要な場合のみスワップを実行
        if self.buffer_swap_needed.load(Ordering::Acquire) {
            if let (Ok(mut front), Ok(back)) = (self.frame_front.lock(), self.frame_back.lock()) {
                if let Some(back_frame) = back.as_ref() {
                    *front = Some(back_frame.clone());
                    self.buffer_swap_needed.store(false, Ordering::Release);
                }
            }
        }

        // フロントバッファから最新フレームを取得
        if let Ok(front) = self.frame_front.lock() {
            front.clone()
        } else {
            None
        }
    }
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            device_id: String::new(),
            name: String::new(),
            enabled: false,
            position: (0.0, 0.0),
            size: (640.0, 480.0),
            camera: Arc::new(Mutex::new(None)),
            frame: Arc::new(Mutex::new(None)),
            frame_front: Arc::new(Mutex::new(None)),
            frame_back: Arc::new(Mutex::new(None)),
            buffer_swap_needed: Arc::new(AtomicBool::new(false)),
            last_frame_time: Instant::now(),
            frame_interval: Duration::from_micros(16667), // 60 FPS
            running: Arc::new(Mutex::new(false)),
            capture_thread: None,
        }
    }
} 