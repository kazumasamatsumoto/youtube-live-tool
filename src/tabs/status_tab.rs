use eframe::egui;
use crate::tabs::StreamStatus;

pub struct StatusTab {
    preview_size: egui::Vec2,
    pub is_streaming: bool,
    pub status: StreamStatus,
}

impl Default for StatusTab {
    fn default() -> Self {
        Self {
            preview_size: egui::Vec2::new(480.0, 270.0), // 16:9 アスペクト比
            is_streaming: false,
            status: StreamStatus::Offline,
        }
    }
}

impl StatusTab {
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
                        // 配信開始/停止ボタン
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
                    // 16:9のアスペクト比を維持しながら、利用可能な幅に合わせる
                    let preview_height = preview_width * 9.0 / 16.0;
                    let preview_size = egui::Vec2::new(preview_width, preview_height);
                    let (_, preview_rect) = ui.allocate_space(preview_size);
                    ui.painter().rect_filled(
                        preview_rect,
                        4.0,
                        egui::Color32::from_rgb(40, 40, 40)
                    );
                    // TODO: 実際のプレビュー映像を表示
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
}
