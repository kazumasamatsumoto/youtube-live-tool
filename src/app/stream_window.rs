use eframe::egui;
use crate::models::screen_capture::ScreenCapture;
use egui::ColorImage;

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
        egui::CentralPanel::default().show(ctx, |ui| {
            // メインレイアウト（垂直方向に分割）
            ui.vertical(|ui| {
                // 上部エリア（配信画面とコメント - 横方向に4:1で分割）
                ui.horizontal(|ui| {
                    // 配信画面エリア（4/5のスペース）
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width() * 0.8, ui.available_height() * 0.9),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            // フレームの取得状態をログ出力
                            match self.screen_capture.get_frame() {
                                Some(frame_image) => {
                                    let (width, height) = frame_image.dimensions();
                                    let pixels: Vec<u8> = frame_image.into_raw();
                                    let size = [width as usize, height as usize];
                                    let expected_size = size[0] * size[1] * 3; // RGB各1バイト                                    
                                    if pixels.len() != expected_size {
                                        println!("ピクセルデータのサイズが不正: {} != {}", pixels.len(), expected_size);
                                        return;
                                    }
                                    let color_image = ColorImage::from_rgb(size, pixels.as_slice());
                                    let texture = self.texture_handle.get_or_insert_with(|| {
                                        ui.ctx().load_texture(
                                            "screen-capture",
                                            color_image.clone(),
                                            egui::TextureOptions::default()
                                        )
                                    });
                                    texture.set(
                                        color_image,
                                        egui::TextureOptions::default()
                                    );
                                    // アスペクト比を維持しながら表示
                                    let available_size = ui.available_size();
                                    let aspect_ratio = size[0] as f32 / size[1] as f32;
                                    let display_size = if available_size.x / available_size.y > aspect_ratio {
                                        egui::vec2(available_size.y * aspect_ratio, available_size.y)
                                    } else {
                                        egui::vec2(available_size.x, available_size.x / aspect_ratio)
                                    };
                                    ui.image((texture.id(), display_size));
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
    }
}
