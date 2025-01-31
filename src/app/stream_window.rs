use eframe::egui;
use crate::models::screen_capture::ScreenCapture;
use egui::{ColorImage, TextureOptions};
use std::time::Instant;
use log::info;

pub struct StreamWindow {
    banner_text: String,
    screen_capture: ScreenCapture,
    texture_handle: Option<egui::TextureHandle>,
}

impl Default for StreamWindow {
    fn default() -> Self {
        let mut screen_capture = ScreenCapture::new();
        screen_capture.start();
        
        Self {
            banner_text: "Welcome to the stream!".to_string(),
            screen_capture,
            texture_handle: None,
        }
    }
}

impl eframe::App for StreamWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let update_start = Instant::now();
        
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width() * 0.8, ui.available_height() * 0.9),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            // フレーム取得時間を計測
                            let frame_fetch_start = Instant::now();
                            match self.screen_capture.get_frame() {
                                Some((frame_data, width, height)) => {
                                    info!("フレーム取得時間: {:?}", frame_fetch_start.elapsed());
                                    
                                    // BGRAデータから直接ColorImageを作成
                                    let color_image = ColorImage::from_rgba_unmultiplied(
                                        [width as usize, height as usize],
                                        &frame_data  // BGRAデータをそのまま使用
                                    );

                                    let texture = self.texture_handle.get_or_insert_with(|| {
                                        ctx.load_texture(
                                            "screen-capture",
                                            color_image.clone(),
                                            TextureOptions::default()
                                        )
                                    });

                                    texture.set(color_image, TextureOptions::default());
                                    
                                    // 描画時間を計測
                                    let draw_start = Instant::now();
                                    let available_size = ui.available_size();
                                    let aspect_ratio = width as f32 / height as f32;
                                    let display_size = if available_size.x / available_size.y > aspect_ratio {
                                        egui::vec2(available_size.y * aspect_ratio, available_size.y)
                                    } else {
                                        egui::vec2(available_size.x, available_size.x / aspect_ratio)
                                    };
                                    ui.image((texture.id(), display_size));
                                    info!("描画時間: {:?}", draw_start.elapsed());
                                }
                                None => {
                                    let frame = egui::Frame::none()
                                        .fill(egui::Color32::from_rgb(40, 40, 40));
                                    frame.show(ui, |ui| {
                                        ui.centered_and_justified(|ui| {
                                            ui.label("画面キャプチャを待機中...");
                                        });
                                    });
                                }
                            }
                        },
                    );

                    // コメントエリア（1/5のスペース）
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), ui.available_height() * 0.9),
                        egui::Layout::top_down(egui::Align::Center),
                        |ui| {
                            let frame = egui::Frame::none()
                                .fill(egui::Color32::from_rgb(30, 30, 30));
                            frame.show(ui, |ui| {
                                egui::ScrollArea::vertical()
                                    .auto_shrink([false; 2])
                                    .show(ui, |ui| {
                                        for i in 0..10 {
                                            ui.label(format!("コメント {}", i));
                                        }
                                    });
                            });
                        },
                    );
                });

                // 下部エリア（バナー表示）
                ui.allocate_ui_with_layout(
                    egui::vec2(ui.available_width(), 30.0),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        // バナーテキストのアニメーション
                        let time = ui.ctx().input(|i| i.time) as f32;
                        let offset = ((time * 50.0) % (ui.available_width() + 200.0)) - 200.0;
                        
                        let text_pos = egui::pos2(ui.available_width() - offset, ui.min_rect().center().y);
                        ui.painter().text(
                            text_pos,
                            egui::Align2::CENTER_CENTER,
                            &self.banner_text,
                            egui::FontId::proportional(20.0),
                            egui::Color32::WHITE,
                        );
                    },
                );

                // 配信終了ボタン
                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    if ui.button("配信終了").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
            });
        });
        info!("全体の更新時間: {:?}", update_start.elapsed());
    }
}
