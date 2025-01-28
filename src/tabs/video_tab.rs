use eframe::egui;
use crate::models::video::{CameraSettings, VideoConfig, ScreenCaptureSettings, CaptureAreaType};
use nokhwa::NokhwaError;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Instant;

pub struct VideoTab {
    pub video_config: VideoConfig,
    pub layout: LayoutSettings,
    texture_cache: HashMap<String, (egui::TextureHandle, Instant)>,
    last_frame_time: Instant,
    cached_camera_list: Option<Vec<String>>,
    error_message: Option<String>,
    error_time: Option<Instant>,
}

impl Default for VideoTab {
    fn default() -> Self {
        Self {
            video_config: VideoConfig::default(),
            layout: LayoutSettings::default(),
            texture_cache: HashMap::new(),
            last_frame_time: Instant::now(),
            cached_camera_list: None,
            error_message: None,
            error_time: None,
        }
    }
}

#[derive(Default)]
pub struct LayoutSettings {
    pub background_color: [f32; 3],
    pub show_grid: bool,
    pub grid_size: u32,
}

impl VideoTab {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // 古いテクスチャを定期的にクリーンアップ
        if self.last_frame_time.elapsed().as_secs() >= 5 {
            self.cleanup_texture_cache();
        }

        ui.heading("映像設定");

        // カメラ設定
        ui.collapsing("カメラ設定", |ui| {
            // カメラ追加ボタンがクリックされたときのみカメラリストを取得
            // エラーメッセージの表示（5秒間表示）
            if let Some(error_time) = self.error_time {
                if error_time.elapsed().as_secs() < 5 {
                    if let Some(error) = &self.error_message {
                        ui.colored_label(egui::Color32::RED, error);
                    }
                } else {
                    self.error_message = None;
                    self.error_time = None;
                }
            }

            ui.horizontal(|ui| {
                if ui.button("カメラを追加").clicked() {
                    // キャッシュされたカメラリストがない場合、または更新が必要な場合のみ取得
                    if self.cached_camera_list.is_none() {
                        self.cached_camera_list = Some(VideoConfig::list_cameras());
                    }
                    
                    if let Some(camera_list) = &self.cached_camera_list {
                        if camera_list.is_empty() {
                            self.error_message = Some("利用可能なカメラが見つかりません".to_string());
                            self.error_time = Some(Instant::now());
                        } else {
                            let new_index = self.video_config.camera_settings.len();
                            match CameraSettings::new(new_index) {
                                Ok(camera_settings) => {
                                    self.video_config.camera_settings.push(camera_settings);
                                }
                                Err(e) => {
                                    self.error_message = Some(format!("カメラの初期化エラー: {}", e));
                                    self.error_time = Some(Instant::now());
                                }
                            }
                        }
                    }
                }
            });

            // 各カメラの設定と映像表示
            for camera in self.video_config.camera_settings.iter_mut() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut camera.enabled, &camera.name);
                    if camera.enabled {
                        ui.label("位置:");
                        ui.add(egui::DragValue::new(&mut camera.position.0).speed(1.0).suffix("x"));
                        ui.add(egui::DragValue::new(&mut camera.position.1).speed(1.0).suffix("y"));
                        ui.label("サイズ:");
                        ui.add(egui::DragValue::new(&mut camera.size.0).speed(1.0).suffix("w"));
                        ui.add(egui::DragValue::new(&mut camera.size.1).speed(1.0).suffix("h"));

                        // カメラフレームの取得と表示
                        if let Err(e) = camera.capture_frame() {
                            self.error_message = Some(format!("カメラエラー: {}", e));
                            self.error_time = Some(Instant::now());
                        }
                        
                        // フレームの表示（テクスチャキャッシュを使用）
                        if let Some(frame) = &camera.frame {
                            let size = egui::vec2(camera.size.0, camera.size.1);
                            let texture_id = format!("camera_{}", camera.device_id);
                            
                            // フレームレート制御 (16.6ms = 約60FPS)
                            let now = Instant::now();
                            if now.duration_since(self.last_frame_time).as_millis() >= 16 {
                                let image = Arc::new(egui::ColorImage::from_rgb(
                                    [frame.width() as usize, frame.height() as usize],
                                    frame.as_raw()
                                ));
                                
                                // テクスチャの更新または作成
                                if let Some((texture, _)) = self.texture_cache.get_mut(&texture_id) {
                                    texture.set(image, egui::TextureOptions::default());
                                } else {
                                    let texture = ui.ctx().load_texture(
                                        &texture_id,
                                        image,
                                        egui::TextureOptions::default(),
                                    );
                                    self.texture_cache.insert(texture_id.clone(), (texture, now));
                                }
                                self.last_frame_time = now;
                            }
                            
                            // キャッシュされたテクスチャを使用
                            if let Some((texture, _)) = self.texture_cache.get(&texture_id) {
                                ui.image((texture.id(), size));
                            }
                        }
                    }
                });
            }
        });

        // 画面キャプチャ設定
        ui.collapsing("画面キャプチャ", |ui| {
            ui.checkbox(&mut self.video_config.screen_capture.enabled, "画面キャプチャを有効化");
            
            if self.video_config.screen_capture.enabled {
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.video_config.screen_capture.area_type, CaptureAreaType::FullScreen, "全画面");
                    ui.radio_value(&mut self.video_config.screen_capture.area_type, CaptureAreaType::Window, "ウィンドウ");
                    ui.radio_value(&mut self.video_config.screen_capture.area_type, CaptureAreaType::Custom, "カスタム");
                });

                ui.horizontal(|ui| {
                    ui.label("位置:");
                    ui.add(egui::DragValue::new(&mut self.video_config.screen_capture.position.0).speed(1.0).suffix("x"));
                    ui.add(egui::DragValue::new(&mut self.video_config.screen_capture.position.1).speed(1.0).suffix("y"));
                });

                ui.horizontal(|ui| {
                    ui.label("サイズ:");
                    ui.add(egui::DragValue::new(&mut self.video_config.screen_capture.size.0).speed(1.0).suffix("w"));
                    ui.add(egui::DragValue::new(&mut self.video_config.screen_capture.size.1).speed(1.0).suffix("h"));
                });
            }
        });

        // レイアウト設定
        ui.collapsing("レイアウト設定", |ui| {
            ui.horizontal(|ui| {
                ui.label("背景色:");
                ui.color_edit_button_rgb(&mut self.layout.background_color);
            });

            ui.checkbox(&mut self.layout.show_grid, "グリッドを表示");
            
            if self.layout.show_grid {
                ui.horizontal(|ui| {
                    ui.label("グリッドサイズ:");
                    ui.add(egui::DragValue::new(&mut self.layout.grid_size)
                        .speed(1)
                        .clamp_range(8..=64));
                });
            }
        });
    }

    fn cleanup_texture_cache(&mut self) {
        let now = Instant::now();
        self.texture_cache.retain(|_, (_, last_used)| {
            // 5秒以上使用されていないテクスチャを削除
            now.duration_since(*last_used).as_secs() < 5
        });
    }
}
