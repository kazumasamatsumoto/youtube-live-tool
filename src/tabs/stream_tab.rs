use eframe::egui;

#[derive(Default)]
pub struct StreamTab {
    pub is_streaming: bool,
    pub stream_key: String,
    pub quality_settings: QualitySettings,
    pub status: StreamStatus,
}

#[derive(Default)]
pub struct QualitySettings {
    pub video_bitrate: u32,
    pub audio_bitrate: u32,
    pub resolution: Resolution,
    pub fps: u32,
}

#[derive(Default)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Default, PartialEq)]
#[allow(dead_code)]
pub enum StreamStatus {
    #[default]
    Offline,
    Starting,
    Live,
    Ending,
    Error(String),
}

impl StreamTab {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("配信設定");
        
        // ステータス表示
        ui.horizontal(|ui| {
            ui.label("ステータス:");
            match &self.status {
                StreamStatus::Offline => { ui.label("オフライン"); }
                StreamStatus::Starting => { ui.label("配信開始中..."); }
                StreamStatus::Live => { ui.colored_label(egui::Color32::GREEN, "配信中"); }
                StreamStatus::Ending => { ui.label("配信終了中..."); }
                StreamStatus::Error(msg) => { ui.colored_label(egui::Color32::RED, msg); }
            }
        });

        // 配信コントロール
        ui.horizontal(|ui| {
            if self.is_streaming {
                if ui.button("配信停止").clicked() {
                    self.is_streaming = false;
                    self.status = StreamStatus::Ending;
                }
            } else {
                if ui.button("配信開始").clicked() {
                    self.is_streaming = true;
                    self.status = StreamStatus::Starting;
                    self.status = StreamStatus::Live;
                }
            }
        });

        // エラーハンドリングの例を追加
        if ui.button("接続テスト").clicked() {
            self.status = StreamStatus::Error("接続に失敗しました".to_string());
        }

        ui.add_space(8.0);
        
        // 配信キー設定
        ui.horizontal(|ui| {
            ui.label("配信キー:");
            ui.add(egui::TextEdit::singleline(&mut self.stream_key).password(true));
        });

        // 品質設定
        ui.collapsing("品質設定", |ui| {
            ui.horizontal(|ui| {
                ui.label("映像ビットレート:");
                ui.add(egui::DragValue::new(&mut self.quality_settings.video_bitrate)
                    .speed(100)
                    .suffix(" kbps")
                    .clamp_range(1000..=10000));
            });

            ui.horizontal(|ui| {
                ui.label("音声ビットレート:");
                ui.add(egui::DragValue::new(&mut self.quality_settings.audio_bitrate)
                    .speed(10)
                    .suffix(" kbps")
                    .clamp_range(64..=320));
            });

            ui.horizontal(|ui| {
                ui.label("解像度:");
                ui.add(egui::DragValue::new(&mut self.quality_settings.resolution.width)
                    .speed(160)
                    .suffix("x"));
                ui.add(egui::DragValue::new(&mut self.quality_settings.resolution.height)
                    .speed(160));
            });

            ui.horizontal(|ui| {
                ui.label("フレームレート:");
                ui.add(egui::DragValue::new(&mut self.quality_settings.fps)
                    .speed(1)
                    .suffix(" fps")
                    .clamp_range(1..=60));
            });
        });
    }
} 