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
use wgpu; // GPUアクセラレーションのために追加
use windows::Win32::Graphics::Dxgi::Common::DXGI_SAMPLE_DESC;  // 追加

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
    // GPU処理用の新しいフィールド
    wgpu_device: Option<wgpu::Device>,
    wgpu_queue: Option<wgpu::Queue>,
    compute_pipeline: Option<wgpu::ComputePipeline>,
}

// GPUバッファとバインドグループを保持する構造体を追加
struct GpuResources {
    input_buffer: wgpu::Buffer,
    output_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl ScreenCapture {
    pub fn new() -> Self {
        // GPUデバイスの初期化
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
            flags: wgpu::InstanceFlags::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::default(),
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        })).unwrap();

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
            },
            None,
        )).unwrap();

        Self {
            frame: Arc::new(RwLock::new(None)),
            running: Arc::new(AtomicBool::new(false)),
            capture_thread: None,
            wgpu_device: Some(device),
            wgpu_queue: Some(queue),
            compute_pipeline: None,
        }
    }

    fn create_compute_pipeline(&mut self) {
        let device = self.wgpu_device.as_ref().unwrap();
        
        // コンピュートシェーダーの作成
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(include_str!("../shaders/downscale.wgsl"))),
        });

        // パイプラインの作成
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &shader,
            entry_point: "main",
        });

        self.compute_pipeline = Some(pipeline);
    }

    pub fn start(&mut self) {
        if self.capture_thread.is_none() {
            self.create_compute_pipeline();
            
            let frame = Arc::clone(&self.frame);
            let running = Arc::clone(&self.running);
            // wgpuの型はCloneを実装していないので、参照を使用
            let device = self.wgpu_device.as_ref().unwrap();
            let queue = self.wgpu_queue.as_ref().unwrap();
            let pipeline = self.compute_pipeline.as_ref().unwrap();

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

                    while running.load(Ordering::Relaxed) {
                        let mut frame_resource = None;
                        let mut frame_info = Default::default();

                        match duplication.AcquireNextFrame(4, &mut frame_info, &mut frame_resource) {
                            Ok(()) => {
                                if let Some(resource) = frame_resource {
                                    let texture: ID3D11Texture2D = resource.cast().unwrap();
                                    
                                    // フレームを処理
                                    let mut staging_texture = None;
                                    let result = ScreenCapture::process_frame_gpu(
                                        &texture,
                                        &device_context,
                                        &mut staging_texture
                                    );

                                    if let Some(frame_image) = result {
                                        let mut frame_lock = frame.write();
                                        *frame_lock = Some(frame_image);
                                    }

                                    duplication.ReleaseFrame().unwrap();
                                }

                                frame_count += 1;
                                if last_fps_check.elapsed() >= Duration::from_secs(1) {
                                    println!("FPS: {}", frame_count);
                                    frame_count = 0;
                                    last_fps_check = Instant::now();
                                }
                            }
                            Err(e) => {
                                if e.code() != DXGI_ERROR_WAIT_TIMEOUT {
                                    error!("Failed to acquire frame: {:?}", e);
                                }
                                thread::sleep(Duration::from_micros(50));
                            }
                        }
                    }
                }
            });

            // スレッド優先度を上げる
            #[cfg(windows)]
            unsafe {
                use windows::Win32::System::Threading::{GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_HIGHEST};
                SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_HIGHEST);
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
        self.frame.read().clone()
    }

    fn create_gpu_resources(&self, device: &wgpu::Device, input_size: (u32, u32)) -> GpuResources {
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniform Buffer"),
            size: std::mem::size_of::<[u32; 6]>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let input_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Input Buffer"),
            size: (input_size.0 * input_size.1 * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Output Buffer"),
            size: (1280 * 720 * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: input_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: output_buffer.as_entire_binding(),
                },
            ],
        });

        GpuResources {
            input_buffer,
            output_buffer,
            uniform_buffer,
            bind_group,
        }
    }

    // process_frame_gpuをスタティック関数に変更
    unsafe fn process_frame_gpu(
        texture: &ID3D11Texture2D,
        device_context: &ID3D11DeviceContext,
        staging_texture: &mut Option<ID3D11Texture2D>,
    ) -> Option<RgbImage> {
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        texture.GetDesc(&mut desc);

        // バッファを事前に確保して再利用
        static mut BUFFER: Option<Vec<u8>> = None;
        if BUFFER.is_none() {
            let mut vec = Vec::with_capacity(1280 * 720 * 3);
            vec.resize(1280 * 720 * 3, 0);  // ここで実際にメモリを確保
            BUFFER = Some(vec);
        }

        // ステージングテクスチャの作成
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

        // テクスチャをコピー
        if let Some(staging) = staging_texture.as_ref() {
            device_context.CopyResource(staging, texture);

            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            device_context.Map(
                staging,
                0,
                D3D11_MAP_READ,
                0,
                Some(&mut mapped),
            ).unwrap();

            let buffer = BUFFER.as_mut().unwrap();

            // SIMD的なアプローチでピクセル処理を最適化
            let src_width = desc.Width as usize;
            let src_height = desc.Height as usize;
            let scale_x = src_width as f32 / 1280.0;
            let scale_y = src_height as f32 / 720.0;

            // 8ピクセルずつ処理
            for y in 0..720 {
                let src_y = (y as f32 * scale_y) as usize;
                let row_offset = src_y * mapped.RowPitch as usize;
                let row_ptr = (mapped.pData as *const u8).add(row_offset);

                let mut x = 0;
                while x < 1280 {
                    let chunk_size = std::cmp::min(8, 1280 - x);
                    for i in 0..chunk_size {
                        let src_x = ((x + i) as f32 * scale_x) as usize;
                        let src_pixel = std::slice::from_raw_parts(
                            row_ptr.add(src_x * 4),
                            4,
                        );

                        let idx = (y * 1280 + x + i) * 3;
                        buffer[idx] = src_pixel[2];     // R
                        buffer[idx + 1] = src_pixel[1]; // G
                        buffer[idx + 2] = src_pixel[0]; // B
                    }
                    x += chunk_size;
                }
            }

            device_context.Unmap(staging, 0);

            // バッファをクローンして新しいRgbImageを作成
            Some(RgbImage::from_raw(1280, 720, buffer.clone()).unwrap())
        } else {
            None
        }
    }
}

// ScreenCaptureの実装... 