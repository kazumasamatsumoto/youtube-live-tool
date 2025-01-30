use std::time::{Duration, Instant};
use std::sync::Arc;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use image::RgbImage;
use log::error;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC;
use windows::core::ComInterface;
use parking_lot::RwLock;

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
    frame: Arc<RwLock<Option<RgbImage>>>,
    running: Arc<AtomicBool>,
    capture_thread: Option<thread::JoinHandle<()>>,
}

impl ScreenCapture {
    pub fn new() -> Self {
        Self {
            frame: Arc::new(RwLock::new(None)),
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
                    let mut buffer = Vec::with_capacity(1280 * 720 * 4); // 720pに変更
                    let mut staging = None;

                    while running.load(Ordering::Relaxed) {
                        let mut frame_resource = None;
                        let mut frame_info = Default::default();

                        match duplication.AcquireNextFrame(
                            16,  // 16msは約60FPSを想定
                            &mut frame_info,
                            &mut frame_resource,
                        ) {
                            Ok(()) => {
                                if let Some(resource) = frame_resource {
                                    let texture: ID3D11Texture2D = resource.cast().unwrap();
                                    let mut desc = D3D11_TEXTURE2D_DESC::default();
                                    texture.GetDesc(&mut desc);

                                    // 中間バッファを作成して解像度変換を行う
                                    if staging.is_none() {
                                        // 元のサイズ用のステージングテクスチャ
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
                                            Usage: D3D11_USAGE_STAGING,  // STAGINGに変更
                                            BindFlags: D3D11_BIND_FLAG(0),
                                            CPUAccessFlags: D3D11_CPU_ACCESS_READ,
                                            MiscFlags: D3D11_RESOURCE_MISC_FLAG(0),
                                        };

                                        device.CreateTexture2D(&staging_desc, None, Some(&mut staging))
                                            .expect("Failed to create staging texture");
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

                                    // バッファサイズを720pに合わせる
                                    buffer.clear();
                                    buffer.reserve(1280 * 720 * 3);

                                    // スケーリングを行いながらデータを読み取り
                                    let src_width = desc.Width as usize;
                                    let src_height = desc.Height as usize;
                                    let scale_x = src_width as f32 / 1280.0;
                                    let scale_y = src_height as f32 / 720.0;

                                    for y in 0..720 {
                                        for x in 0..1280 {
                                            let src_x = (x as f32 * scale_x) as usize;
                                            let src_y = (y as f32 * scale_y) as usize;
                                            let src_offset = src_y * mapped.RowPitch as usize + src_x * 4;
                                            
                                            let src_pixel = std::slice::from_raw_parts(
                                                (mapped.pData as *const u8).add(src_offset),
                                                4,
                                            );
                                            
                                            buffer.extend_from_slice(&[src_pixel[2], src_pixel[1], src_pixel[0]]);
                                        }
                                    }

                                    device_context.Unmap(staging.as_ref().unwrap(), 0);

                                    // フレームをImageBufferとして保存
                                    if let Some(rgb_image) = RgbImage::from_raw(
                                        1280,  // 720p固定
                                        720,
                                        buffer.clone(),
                                    ) {
                                        let mut frame_lock = frame.write();
                                        *frame_lock = Some(rgb_image);
                                    }

                                    duplication.ReleaseFrame().unwrap();
                                    
                                    // FPS計測
                                    frame_count += 1;
                                    if last_fps_check.elapsed() >= Duration::from_secs(1) {
                                        println!("FPS: {}", frame_count);
                                        frame_count = 0;
                                        last_fps_check = Instant::now();
                                    }
                                }
                            }
                            Err(e) => {
                                if e.code() != DXGI_ERROR_WAIT_TIMEOUT {
                                    error!("Failed to acquire frame: {:?}", e);
                                }
                                thread::sleep(Duration::from_micros(100));
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
        self.frame.read().clone()
    }
}

// ScreenCaptureの実装... 