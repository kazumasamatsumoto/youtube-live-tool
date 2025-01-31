use std::time::{Duration, Instant};
use std::sync::Arc;
use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use image::RgbImage;
use log::error;
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

                    let mut last_frame_time = Instant::now();
                    let frame_duration = Duration::from_micros(8333); // 120 FPS

                    while running.load(Ordering::Relaxed) {
                        let now = Instant::now();
                        let elapsed = now.duration_since(last_frame_time);
                        
                        if elapsed < frame_duration {
                            thread::sleep(frame_duration - elapsed);
                            continue;
                        }
                        
                        last_frame_time = now;

                        let mut frame_resource = None;
                        let mut frame_info = Default::default();

                        // フレーム取得のタイムアウトを短く設定
                        match duplication.AcquireNextFrame(0, &mut frame_info, &mut frame_resource) {
                            Ok(()) => {
                                if let Some(resource) = frame_resource {
                                    let texture: ID3D11Texture2D = resource.cast().unwrap();
                                    
                                    // staging_textureを静的に保持して再利用
                                    static mut STAGING_TEXTURE: Option<Option<ID3D11Texture2D>> = None;
                                    if STAGING_TEXTURE.is_none() {
                                        STAGING_TEXTURE = Some(None);
                                    }
                                    
                                    let result = ScreenCapture::process_frame_gpu(
                                        &texture,
                                        &device_context,
                                        STAGING_TEXTURE.as_mut().unwrap(),
                                    );

                                    if let Some(frame_image) = result {
                                        let mut frame_lock = frame.write();
                                        *frame_lock = Some(frame_image);
                                    }

                                    duplication.ReleaseFrame().unwrap();
                                }
                            }
                            Err(e) => {
                                if e.code() != DXGI_ERROR_WAIT_TIMEOUT {
                                    error!("Failed to acquire frame: {:?}", e);
                                }
                                continue;
                            }
                        }
                    }
                }
            });

            // スレッド優先度とアフィニティを最適化
            #[cfg(windows)]
            unsafe {
                use windows::Win32::System::Threading::{
                    GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_TIME_CRITICAL,
                    SetThreadAffinityMask, SetProcessPriorityBoost, GetCurrentProcess
                };
                // プロセスの優先度ブーストを無効化して安定性を向上
                SetProcessPriorityBoost(GetCurrentProcess(), false);
                // スレッド優先度を最高に設定
                SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_TIME_CRITICAL);
                // 最初のCPUコアに固定（他のコアとの競合を防ぐ）
                SetThreadAffinityMask(GetCurrentThread(), 0b1);
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

    pub fn get_frame(&self) -> Option<RgbImage> {
        let frame = self.frame.read().clone();
        frame
    }

    // process_frame_gpuをスタティック関数に変更
    unsafe fn process_frame_gpu(
        texture: &ID3D11Texture2D,
        device_context: &ID3D11DeviceContext,
        staging_texture: &mut Option<ID3D11Texture2D>,
    ) -> Option<RgbImage> {
        // 処理開始のログ
        let frame_start = Instant::now();
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        texture.GetDesc(&mut desc);
        // バッファサイズを実際のテクスチャサイズに合わせる
        let stride = ((desc.Width * 3 + 63) & !63) as usize; // 64バイトアラインメント
        let buffer_size = stride * desc.Height as usize;
        // 静的バッファの初期化をより効率的に
        static mut BUFFER: Option<Vec<u8>> = None;
        static mut OUTPUT_BUFFER: Option<Vec<u8>> = None;
        if BUFFER.is_none() {
            // アラインメントされたメモリ確保
            let layout = std::alloc::Layout::from_size_align(buffer_size, 64).unwrap();
            let ptr = std::alloc::alloc_zeroed(layout);
            BUFFER = Some(unsafe { Vec::from_raw_parts(ptr, buffer_size, buffer_size) });
        }
        if OUTPUT_BUFFER.is_none() {
            let layout = std::alloc::Layout::from_size_align(buffer_size, 64).unwrap();
            let ptr = std::alloc::alloc_zeroed(layout);
            OUTPUT_BUFFER = Some(unsafe { Vec::from_raw_parts(ptr, buffer_size, buffer_size) });
        }

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

        device_context.CopyResource(staging, texture);
        
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        // エラーハンドリングを追加
        if let Err(e) = device_context.Map(
            staging,
            0,
            D3D11_MAP_READ,
            0,
            Some(&mut mapped),
        ) {
            println!("テクスチャのマッピングに失敗: {:?}", e);
            return None;
        }

        // スケーリングテーブルを静的に保持
        static mut SCALE_X_TABLE: Option<Vec<usize>> = None;
        static mut SCALE_Y_TABLE: Option<Vec<usize>> = None;

        if SCALE_X_TABLE.is_none() {
            let mut x_table = Vec::with_capacity(1280);
            let scale_x = (desc.Width as f32 / 1280.0) * 65536.0;
            for x in 0..1280 {
                let src_x = ((x as f32 * scale_x) as i32 >> 16) as usize;
                x_table.push(src_x * 4);
            }
            SCALE_X_TABLE = Some(x_table);
        }

        if SCALE_Y_TABLE.is_none() {
            let mut y_table = Vec::with_capacity(720);
            let scale_y = (desc.Height as f32 / 720.0) * 65536.0;
            for y in 0..720 {
                let src_y = ((y as f32 * scale_y) as i32 >> 16) as usize;
                y_table.push(src_y * mapped.RowPitch as usize);
            }
            SCALE_Y_TABLE = Some(y_table);
        }

        let _buffer = BUFFER.as_mut().unwrap();
        let x_table = SCALE_X_TABLE.as_ref().unwrap();
        let _y_table = SCALE_Y_TABLE.as_ref().unwrap();

        // プリフェッチのためのテーブルを事前計算
        let mut prefetch_offsets = Vec::with_capacity(1280);
        for x in (0..1280).step_by(16) {
            let base_offset = x_table[x];
            prefetch_offsets.push([
                base_offset + 128,
                base_offset + 160,
                base_offset + 192,
                base_offset + 224
            ]);
        }
        
        // AVX2を使用した高速ピクセル処理
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::*;
            for y in 0..720 {
                let row_ptr = (mapped.pData as *const u8).add(y_table[y]);
                let dst_row_offset = y * stride;

                let mut x = 0;
                while x < 1280 {
                    let src_ptr = row_ptr.add(x_table[x]);
                    let dst_ptr = _buffer.as_mut_ptr().add(dst_row_offset + x * 3);

                    unsafe {
                        // 16ピクセルずつ処理
                        let pixels1 = _mm256_loadu_si256(src_ptr as *const __m256i);
                        let pixels2 = _mm256_loadu_si256(src_ptr.add(32) as *const __m256i);
                        let pixels3 = _mm256_loadu_si256(src_ptr.add(64) as *const __m256i);
                        let pixels4 = _mm256_loadu_si256(src_ptr.add(96) as *const __m256i);

                        // BGRに並び替え
                        let mask = _mm256_set_epi8(
                            15, 14, 13, 11, 10, 9, 7, 6, 5, 3, 2, 1, -1, -1, -1, -1,
                            15, 14, 13, 11, 10, 9, 7, 6, 5, 3, 2, 1, -1, -1, -1, -1
                        );

                        // プリフェッチを最適化
                        let prefetch = prefetch_offsets[x / 16];
                        _mm_prefetch(row_ptr.add(prefetch[0]) as *const i8, _MM_HINT_T0);
                        _mm_prefetch(row_ptr.add(prefetch[1]) as *const i8, _MM_HINT_T0);
                        _mm_prefetch(row_ptr.add(prefetch[2]) as *const i8, _MM_HINT_T0);
                        _mm_prefetch(row_ptr.add(prefetch[3]) as *const i8, _MM_HINT_T0);

                        // 次の行のデータもプリフェッチ
                        if y < 719 {
                            let next_row = (mapped.pData as *const u8).add(y_table[y + 1]);
                            _mm_prefetch(next_row.add(x_table[x]) as *const i8, _MM_HINT_T0);
                        }

                        let shuffled1 = _mm256_shuffle_epi8(pixels1, mask);
                        let shuffled2 = _mm256_shuffle_epi8(pixels2, mask);
                        let shuffled3 = _mm256_shuffle_epi8(pixels3, mask);
                        let shuffled4 = _mm256_shuffle_epi8(pixels4, mask);

                        // 非時間的なストア命令を使用
                        _mm256_stream_si256(dst_ptr as *mut __m256i, shuffled1);
                        _mm256_stream_si256(dst_ptr.add(24) as *mut __m256i, shuffled2);
                        _mm256_stream_si256(dst_ptr.add(48) as *mut __m256i, shuffled3);
                        _mm256_stream_si256(dst_ptr.add(72) as *mut __m256i, shuffled4);

                        // キャッシュラインのフラッシュを防ぐ
                        _mm_sfence();
                    }
                    x += 16;
                }
            }
        }

        device_context.Unmap(staging, 0);

        let _output_buffer = OUTPUT_BUFFER.as_mut().unwrap();
        // ストリーミングコピーを使用
        #[cfg(target_arch = "x86_64")]
        unsafe {
            use std::arch::x86_64::*;
            let mut src = _buffer.as_ptr();
            let mut dst = output_buffer.as_mut_ptr();
            let mut remaining = buffer_size;
            
            while remaining >= 32 {
                let v = _mm256_load_si256(src as *const __m256i);
                _mm256_stream_si256(dst as *mut __m256i, v);
                src = src.add(32);
                dst = dst.add(32);
                remaining -= 32;
            }
            _mm_sfence();
        }
        
        // 処理時間のログ
        println!("フレーム処理時間: {:?}", frame_start.elapsed());

        // BGRAからRGBに変換
        #[cfg(target_arch = "x86_64")]
        {
            use std::arch::x86_64::*;
            let mut rgb_data = Vec::with_capacity((desc.Width * desc.Height * 3) as usize);
            rgb_data.set_len((desc.Width * desc.Height * 3) as usize);
            
            // BGRAからRGBへの変換マスク
            let shuffle_mask = _mm256_set_epi8(
                -1, -1, -1, -1,  // 未使用
                14, 13, 12,      // 4番目のピクセル
                10, 9, 8,        // 3番目のピクセル
                6, 5, 4,         // 2番目のピクセル
                2, 1, 0,         // 1番目のピクセル
                -1, -1, -1, -1,  // 未使用
                14, 13, 12,      // 4番目のピクセル
                10, 9, 8,        // 3番目のピクセル
                6, 5, 4,         // 2番目のピクセル
                2, 1, 0          // 1番目のピクセル
            );

            for y in 0..desc.Height {
                let src_row = (mapped.pData as *const u8).add((y * mapped.RowPitch as u32) as usize);
                let dst_row = rgb_data.as_mut_ptr().add((y * desc.Width * 3) as usize);
                
                // 16バイトアラインメントのためのプリフェッチ
                _mm_prefetch(src_row as *const i8, _MM_HINT_T0);
                
                let mut x = 0;
                while x + 8 <= desc.Width {
                    // 8ピクセル(32バイト)ずつ処理
                    let bgra = _mm256_loadu_si256(src_row.add(x * 4) as *const __m256i);
                    let rgb = _mm256_shuffle_epi8(bgra, shuffle_mask);
                    
                    // RGBデータを書き込み
                    _mm256_storeu_si256(dst_row.add(x * 3) as *mut __m256i, rgb);
                    
                    x += 8;
                }
                
                // 残りのピクセルを処理
                while x < desc.Width {
                    let pixel_start = src_row.add(x * 4);
                    let dst = dst_row.add(x * 3);
                    *dst = *pixel_start.add(2);      // R
                    *dst.add(1) = *pixel_start.add(1); // G
                    *dst.add(2) = *pixel_start;      // B
                    x += 1;
                }
            }

            device_context.Unmap(staging, 0);
            
            // 処理時間のログ
            println!("フレーム処理時間: {:?}", frame_start.elapsed());
            
            Some(RgbImage::from_raw(desc.Width as u32, desc.Height as u32, rgb_data).unwrap())
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            // 非x86_64アーキテクチャ用の実装
            let mut rgb_data = Vec::with_capacity((desc.Width * desc.Height * 3) as usize);
            let src_ptr = mapped.pData as *const u8;
            
            for y in 0..desc.Height {
                let row_start = src_ptr.add((y * mapped.RowPitch as u32) as usize);
                for x in 0..desc.Width {
                    let pixel_start = row_start.add((x * 4) as usize);
                    rgb_data.push(*pixel_start.add(2));  // R
                    rgb_data.push(*pixel_start.add(1));  // G
                    rgb_data.push(*pixel_start);         // B
                }
            }

            device_context.Unmap(staging, 0);
            
            Some(RgbImage::from_raw(desc.Width as u32, desc.Height as u32, rgb_data).unwrap())
        }
    }
}

// ScreenCaptureの実装...
