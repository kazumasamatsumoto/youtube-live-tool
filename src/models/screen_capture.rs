use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use image::RgbImage;
use log::error;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC;
use windows::core::ComInterface;

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

pub struct ScreenCapture {
    frame: Arc<Mutex<Option<RgbImage>>>,
    running: Arc<AtomicBool>,
    capture_thread: Option<thread::JoinHandle<()>>,
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self {
            frame: Arc::new(Mutex::new(None)),
            running: Arc::new(AtomicBool::new(false)),
            capture_thread: None,
        }
    }

    pub fn start(&mut self) {
        if self.capture_thread.is_none() {
            let frame = Arc::clone(&self.frame);
            let running = Arc::clone(&self.running);

            running.store(true, Ordering::SeqCst);

            let handle = thread::spawn(move || {
                unsafe {
                    // DXGIファクトリを作成
                    let factory: IDXGIFactory1 = CreateDXGIFactory1().unwrap();

                    // デフォルトアダプターを取得
                    let adapter = factory.EnumAdapters(0).unwrap();

                    // 基本的なデバイス作成フラグ
                    let flags = D3D11_CREATE_DEVICE_FLAG(
                        D3D11_CREATE_DEVICE_BGRA_SUPPORT.0
                    );

                    // デバイスとコンテキストの作成
                    let mut device = None;
                    let mut device_context = None;
                    let mut feature_level = D3D_FEATURE_LEVEL_11_0;

                    D3D11CreateDevice(
                        Some(&adapter),
                        D3D_DRIVER_TYPE_UNKNOWN,
                        None,
                        flags,
                        Some(&[
                            D3D_FEATURE_LEVEL_11_0,
                            D3D_FEATURE_LEVEL_10_1,
                            D3D_FEATURE_LEVEL_10_0,
                        ]),
                        D3D11_SDK_VERSION,
                        Some(&mut device),
                        Some(&mut feature_level),
                        Some(&mut device_context),
                    ).expect("Failed to create D3D11 device");

                    let device = device.unwrap();
                    let device_context = device_context.unwrap();

                    // プライマリディスプレイの取得
                    let output = adapter.EnumOutputs(0).unwrap();
                    let output1: IDXGIOutput1 = output.cast().unwrap();

                    println!("Device and output created successfully");

                    // 出力の複製を作成
                    let duplication = match output1.DuplicateOutput(&device) {
                        Ok(dup) => {
                            println!("Output duplication created successfully");
                            dup
                        },
                        Err(e) => {
                            error!("Failed to duplicate output: {:?}", e);
                            return;
                        }
                    };

                    let mut last_fps_check = Instant::now();
                    let mut frame_count = 0;
                    let mut buffer = Vec::with_capacity(2304 * 1536 * 4);
                    let mut staging = None;  // ステージングテクスチャを再利用

                    while running.load(Ordering::Relaxed) {
                        let mut frame_resource = None;
                        let mut frame_info = Default::default();

                        match duplication.AcquireNextFrame(
                            33,  // 33msは約30FPSを想定
                            &mut frame_info,
                            &mut frame_resource,
                        ) {
                            Ok(()) => {
                                if let Some(resource) = frame_resource {
                                    let texture: ID3D11Texture2D = resource.cast().unwrap();
                                    let mut desc = D3D11_TEXTURE2D_DESC::default();
                                    texture.GetDesc(&mut desc);

                                    // ステージングテクスチャの作成または再利用
                                    if staging.is_none() {
                                        let staging_desc = D3D11_TEXTURE2D_DESC {
                                            Width: desc.Width,
                                            Height: desc.Height,
                                            MipLevels: 1,
                                            ArraySize: 1,
                                            Format: desc.Format,
                                            SampleDesc: DXGI_SAMPLE_DESC {
                                                Count: 1,
                                                Quality: 0,
                                            },
                                            Usage: D3D11_USAGE_STAGING,
                                            BindFlags: D3D11_BIND_FLAG(0),
                                            CPUAccessFlags: D3D11_CPU_ACCESS_READ,
                                            MiscFlags: D3D11_RESOURCE_MISC_FLAG(0),
                                        };

                                        match device.CreateTexture2D(
                                            &staging_desc,
                                            None,
                                            Some(&mut staging),
                                        ) {
                                            Ok(()) => (),
                                            Err(e) => {
                                                error!("ステージングテクスチャの作成に失敗: {:?}", e);
                                                continue;
                                            }
                                        };
                                    }

                                    // フレームのコピー
                                    device_context.CopyResource(
                                        staging.as_ref().unwrap(),
                                        &texture,
                                    );

                                    // データの読み取り
                                    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
                                    device_context.Map(
                                        staging.as_ref().unwrap(),
                                        0,
                                        D3D11_MAP_READ,
                                        0,
                                        Some(&mut mapped),
                                    ).unwrap();

                                    let row_pitch = mapped.RowPitch as usize;
                                    let width = desc.Width as usize;
                                    let height = desc.Height as usize;

                                    buffer.clear();
                                    buffer.reserve(width * height * 3);
                                    
                                    // SIMD的なアプローチで一度に複数ピクセルを処理
                                    for y in 0..height {
                                        let row = std::slice::from_raw_parts(
                                            (mapped.pData as *const u8).add(y * row_pitch),
                                            width * 4,
                                        );
                                        
                                        let chunks = row.chunks_exact(8);
                                        let remainder = chunks.remainder();
                                        
                                        for chunk in chunks {
                                            buffer.extend_from_slice(&[
                                                chunk[2], chunk[1], chunk[0],  // 1つ目のピクセル
                                                chunk[6], chunk[5], chunk[4],  // 2つ目のピクセル
                                            ]);
                                        }
                                        
                                        if !remainder.is_empty() {
                                            buffer.extend_from_slice(&[
                                                remainder[2], remainder[1], remainder[0]
                                            ]);
                                        }
                                    }

                                    device_context.Unmap(staging.as_ref().unwrap(), 0);

                                    // フレームをImageBufferとして保存
                                    if let Some(rgb_image) = RgbImage::from_raw(
                                        width as u32,
                                        height as u32,
                                        buffer.clone(),
                                    ) {
                                        if let Ok(mut frame_guard) = frame.lock() {
                                            *frame_guard = Some(rgb_image);
                                        }
                                    }

                                    duplication.ReleaseFrame().unwrap();
                                }

                                // フレームレート制御
                                frame_count += 1;
                                if last_fps_check.elapsed() >= Duration::from_secs(1) {
                                    println!("FPS: {}", frame_count);
                                    frame_count = 0;
                                    last_fps_check = Instant::now();
                                } else {
                                    // フレーム間の適切な待機時間を設定
                                    let target_frame_time = Duration::from_millis(33);
                                    if let Some(sleep_time) = target_frame_time.checked_sub(last_fps_check.elapsed()) {
                                        thread::sleep(sleep_time);
                                    }
                                }
                            }
                            Err(e) => {
                                if e.code() != DXGI_ERROR_WAIT_TIMEOUT {
                                    error!("Failed to acquire frame: {:?}", e);
                                }
                                thread::sleep(Duration::from_micros(100));
                                continue;
                            }
                        }
                    }
                }
            });

            self.capture_thread = Some(handle);
        }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.capture_thread.take() {
            let _ = thread.join();
        }
    }

    pub fn get_frame(&self) -> Option<RgbImage> {
        self.frame.lock().ok()?.clone()
    }
}

// ScreenCaptureの実装... 