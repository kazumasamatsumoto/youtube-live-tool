use eframe::egui;
use crate::models::{banner::BannerConfig, stream::StreamConfig};
use log::info;

#[allow(dead_code)]
pub struct StreamWindow {
    stream: StreamConfig,
    comments: Vec<String>, // 一時的にStringに変更
    banner: BannerConfig,
    banner_text: String,
}

impl Default for StreamWindow {
    fn default() -> Self {
        Self {
            stream: StreamConfig::default(),
            comments: Vec::new(),
            banner: BannerConfig::default(),
            banner_text: "Welcome to the stream!".to_string(),
        }
    }
}

impl eframe::App for StreamWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // マウスの状態を確認
        ctx.input(|input| {
            // マウスの位置
            if let Some(pos) = input.pointer.hover_pos() {
                info!("[Mouse] 位置: ({:.1}, {:.1})", pos.x, pos.y);
            }

            // マウスのフォーカス状態
            if input.pointer.has_pointer() {
                info!("[Mouse] ウィンドウ内");
            } else {
                info!("[Mouse] ウィンドウ外");
            }

            // クリックイベント
            if input.pointer.primary_clicked() {
                info!("[Mouse] 左クリック");
            }
            if input.pointer.secondary_clicked() {
                info!("[Mouse] 右クリック");
            }
        });

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
                            let frame = egui::Frame::none()
                                .fill(egui::Color32::from_rgb(40, 40, 40));
                            frame.show(ui, |ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label("配信画面");
                                });
                            });
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
