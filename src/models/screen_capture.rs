use std::time::{Duration, Instant};
use std::sync::Arc;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use log::{info, error};
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::core::ComInterface;
use parking_lot::RwLock;
use windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC;

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
    frame: Arc<RwLock<Option<(Vec<u8>, u32, u32)>>>,  // サイズ情報も保持
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

                    let frame_duration = Duration::from_micros(16667); // 60 FPS
                    let mut last_frame_time = Instant::now();
                    let mut frame_count = 0;
                    let mut fps_timer = Instant::now();

                    while running.load(Ordering::Relaxed) {
                        let now = Instant::now();
                        let elapsed = now.duration_since(last_frame_time);

                        // より精密なフレーム同期
                        if elapsed < frame_duration {
                            let sleep_time = frame_duration - elapsed;
                            if sleep_time > Duration::from_micros(100) {
                                thread::sleep(sleep_time - Duration::from_micros(100));
                                thread::yield_now();
                            }
                            continue;
                        }

                        last_frame_time = now;

                        let mut frame_resource = None;
                        let mut frame_info = Default::default();

                        // フレーム取得とFPS計測
                        match duplication.AcquireNextFrame(0, &mut frame_info, &mut frame_resource) {
                            Ok(()) => {
                                info!("フレーム取得: OK");
                                if let Some(resource) = frame_resource {
                                    let texture: ID3D11Texture2D = resource.cast().unwrap();
                                    
                                    // staging_textureを静的に保持して再利用
                                    static mut STAGING_TEXTURE: Option<Option<ID3D11Texture2D>> = None;
                                    if STAGING_TEXTURE.is_none() {
                                        info!("新しいステージングテクスチャを作成");
                                        STAGING_TEXTURE = Some(None);
                                    }
                                    
                                    let frame_start = Instant::now();
                                    let result = ScreenCapture::process_frame_gpu(
                                        &texture,
                                        &device_context,
                                        STAGING_TEXTURE.as_mut().unwrap(),
                                    );
                                    info!("フレーム処理時間: {:?}", frame_start.elapsed());

                                    if let Some((frame_image, width, height)) = result {
                                        info!("フレーム変換成功: {}x{}", width, height);
                                        let mut frame_lock = frame.write();
                                        *frame_lock = Some((frame_image, width, height));
                                    } else {
                                        info!("フレーム変換失敗");
                                    }

                                    duplication.ReleaseFrame().unwrap();
                                }
                                frame_count += 1;
                                if fps_timer.elapsed() >= Duration::from_secs(1) {
                                    info!("FPS: {}", frame_count);
                                    frame_count = 0;
                                    fps_timer = Instant::now();
                                }
                            }
                            Err(e) => {
                                if e.code() != DXGI_ERROR_WAIT_TIMEOUT {
                                    error!("フレーム取得エラー: {:?}", e);
                                }
                                continue;
                            }
                        }
                    }
                }
            });

            // スレッド優先度をさらに最適化
            #[cfg(windows)]
            unsafe {
                use windows::Win32::System::Threading::*;
                SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);
                SetThreadAffinityMask(GetCurrentThread(), 0b1);  // 最初のコアに固定
            }

            self.capture_thread = Some(handle);
        }
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        if let Some(thread) = self.capture_thread.take() {
            let _ = thread.join();
        }
    }

    pub fn get_frame(&self) -> Option<(Vec<u8>, u32, u32)> {
        self.frame.read().clone()
    }

    // process_frame_gpuをスタティック関数に変更
    unsafe fn process_frame_gpu(
        texture: &ID3D11Texture2D,
        device_context: &ID3D11DeviceContext,
        staging_texture: &mut Option<ID3D11Texture2D>,
    ) -> Option<(Vec<u8>, u32, u32)> {
        let process_start = Instant::now();
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        texture.GetDesc(&mut desc);
        info!("テクスチャサイズ: {}x{}", desc.Width, desc.Height);

        // ステージングテクスチャを再利用
        if staging_texture.is_none() {
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

            let d3d_device = device_context.GetDevice().unwrap();
            let mut new_texture = None;
            d3d_device.CreateTexture2D(&staging_desc, None, Some(&mut new_texture)).unwrap();
            *staging_texture = new_texture;
        }

        let staging = match staging_texture.as_ref() {
            Some(staging) => staging,
            None => return None,
        };

        // テクスチャコピー時間を計測
        let copy_start = Instant::now();
        device_context.CopyResource(staging, texture);
        info!("テクスチャコピー時間: {:?}", copy_start.elapsed());
        
        let map_start = Instant::now();
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        if let Err(e) = device_context.Map(
            staging,
            0,
            D3D11_MAP_READ,
            0,
            Some(&mut mapped),
        ) {
            error!("テクスチャマッピング失敗: {:?}", e);
            return None;
        }
        info!("テクスチャマッピング時間: {:?}", map_start.elapsed());

        // BGRAデータをそのままコピー
        let data_size = (desc.Width * desc.Height * 4) as usize;
        let mut bgra_data = Vec::with_capacity(data_size);
        bgra_data.set_len(data_size);

        std::ptr::copy_nonoverlapping(
            mapped.pData as *const u8,
            bgra_data.as_mut_ptr(),
            data_size
        );

        device_context.Unmap(staging, 0);
        info!("全体の処理時間: {:?}", process_start.elapsed());
        
        Some((bgra_data, desc.Width, desc.Height))
    }
}

// ScreenCaptureの実装...
