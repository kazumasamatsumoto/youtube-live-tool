use eframe::egui;
use crate::tabs::StreamStatus;
use crate::models::{
    camera::CameraSettings,
    video_frame::VideoFrame,
    screen_capture::ScreenCapture
};

pub struct StatusTab {
    #[allow(dead_code)]
    preview_size: egui::Vec2,
    pub is_streaming: bool,
    pub status: StreamStatus,
    #[allow(dead_code)]
    current_frame: Option<VideoFrame>,
    camera: Option<CameraSettings>,
    camera_texture: Option<egui::TextureHandle>,
    screen_capture: Option<ScreenCapture>,
    screen_texture: Option<egui::TextureHandle>,
    is_screen_sharing: bool,
}

impl Default for StatusTab {
    fn default() -> Self {
        Self {
            preview_size: egui::Vec2::new(480.0, 270.0), // 16:9 アスペクト比
            is_streaming: false,
            status: StreamStatus::Offline,
            current_frame: None,
            camera: None,
            camera_texture: None,
            screen_capture: None,
            screen_texture: None,
            is_screen_sharing: false,
        }
    }
}

impl StatusTab {
    fn initialize_camera(&mut self) {
        if self.camera.is_none() {
            match CameraSettings::new(0) {
                Ok(camera) => {
                    self.camera = Some(camera);
                }
                Err(e) => {
                    eprintln!("カメラの初期化に失敗: {}", e);
                }
            }
        }
    }

    fn initialize_screen_capture(&mut self) {
        if self.screen_capture.is_none() {
            let mut screen_capture = ScreenCapture::new();
            screen_capture.start();
            self.screen_capture = Some(screen_capture);
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // 利用可能な幅を取得
        let available_width = ui.available_width();
        
        // プレビューエリアとコメントエリアの幅を5:2の比率で計算
        let preview_width = available_width * 0.714; // 71.4% (5/7)
        let comment_width = available_width * 0.286; // 28.6% (2/7)
        
        ui.horizontal(|ui| {
            // 左側: プレビューと配信情報 (80%)
            ui.vertical(|ui| {
                ui.set_width(preview_width);
                
                // プレビューエリアのヘッダー
                ui.horizontal(|ui| {
                    ui.heading("配信プレビュー");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // 画面共有ボタンを追加
                        if ui.button(if self.is_screen_sharing { "画面共有停止" } else { "画面共有開始" }).clicked() {
                            self.is_screen_sharing = !self.is_screen_sharing;
                            if self.is_screen_sharing {
                                self.initialize_screen_capture();
                            } else if let Some(screen_capture) = &mut self.screen_capture {
                                screen_capture.stop();
                                self.screen_capture = None;
                            }
                        }

                        // カメラ追加ボタンを追加
                        if ui.button("カメラを追加").clicked() {
                            self.initialize_camera();
                        }
                        
                        // 既存の配信開始/停止ボタン
                        if self.is_streaming {
                            if ui.button("配信停止").clicked() {
                                self.is_streaming = false;
                                self.status = StreamStatus::Offline;
                            }
                        } else {
                            if ui.button("配信開始").clicked() {
                                self.is_streaming = true;
                                self.status = StreamStatus::Live;
                            }
                        }
                    });
                });

                // プレビュー表示エリア
                ui.group(|ui| {
                    let preview_height = preview_width * 9.0 / 16.0;
                    let preview_size = egui::Vec2::new(preview_width, preview_height);
                    let (_, preview_rect) = ui.allocate_space(preview_size);
                    
                    if self.is_screen_sharing {
                        // 画面共有の表示
                        if let Some(screen_capture) = &mut self.screen_capture {
                            if let Some(frame) = screen_capture.get_frame() {
                                let texture = self.screen_texture.get_or_insert_with(|| {
                                    ui.ctx().load_texture(
                                        "screen-preview",
                                        egui::ColorImage::from_rgb(
                                            [frame.width() as usize, frame.height() as usize],
                                            frame.as_raw(),
                                        ),
                                        egui::TextureOptions {
                                            magnification: egui::TextureFilter::Linear,
                                            minification: egui::TextureFilter::Linear,
                                            ..Default::default()
                                        },
                                    )
                                });

                                // フレームバッファの更新を最適化
                                if frame.width() > 0 && frame.height() > 0 {
                                    texture.set(
                                        egui::ColorImage::from_rgb(
                                            [frame.width() as usize, frame.height() as usize],
                                            frame.as_raw(),
                                        ),
                                        egui::TextureOptions {
                                            magnification: egui::TextureFilter::Linear,
                                            minification: egui::TextureFilter::Linear,
                                            ..Default::default()
                                        },
                                    );
                                }

                                ui.painter().image(
                                    texture.id(),
                                    preview_rect,
                                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                    egui::Color32::WHITE,
                                );
                            }
                        }
                    } else {
                        // カメラ映像の更新と表示
                        if let Some(camera) = &mut self.camera {
                            if let Some(frame) = camera.get_frame() {
                                let texture = self.camera_texture.get_or_insert_with(|| {
                                    ui.ctx().load_texture(
                                        "camera-preview",
                                        egui::ColorImage::from_rgb(
                                            [frame.width() as usize, frame.height() as usize],
                                            frame.as_raw(),
                                        ),
                                        Default::default(),
                                    )
                                });

                                // テクスチャの更新
                                texture.set(
                                    egui::ColorImage::from_rgb(
                                        [frame.width() as usize, frame.height() as usize],
                                        frame.as_raw(),
                                    ),
                                    Default::default(),
                                );

                                // 映像の描画
                                ui.painter().image(
                                    texture.id(),
                                    preview_rect,
                                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                                    egui::Color32::WHITE,
                                );
                            }
                        } else {
                            // カメラがない場合は黒背景とNo Signalを表示
                            ui.painter().rect_filled(
                                preview_rect,
                                4.0,
                                egui::Color32::from_rgb(40, 40, 40)
                            );
                            let text = "No Signal";
                            let text_color = egui::Color32::from_rgb(200, 200, 200);
                            ui.painter().text(
                                preview_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                text,
                                egui::FontId::proportional(20.0),
                                text_color,
                            );
                        }
                    }
                });

                // 配信情報
                ui.group(|ui| {
                    ui.heading("配信情報");
                    ui.horizontal(|ui| {
                        ui.label("視聴者数: 0");
                        ui.add_space(20.0);
                        ui.label("配信時間: 00:00:00");
                        ui.add_space(20.0);
                        ui.label("ビットレート: 0 Mbps");
                    });
                });
            });

            // 右側: コメント表示エリア (20%)
            ui.vertical(|ui| {
                ui.set_width(comment_width);
                ui.group(|ui| {
                    ui.heading("コメント");
                    let comment_area = egui::ScrollArea::vertical()
                        .max_height(ui.available_height() - 40.0) // ヘッダー分を引く
                        .auto_shrink([false; 2]);
                    
                    comment_area.show(ui, |ui| {
                        // TODO: 実際のコメントを表示
                        ui.label("まだコメントはありません");
                    });
                });
            });
        });
    }

    #[allow(dead_code)]
    pub fn update_frame(&mut self, frame: VideoFrame) {
        self.current_frame = Some(frame);
    }
}
