use eframe::egui;

#[derive(Default)]
pub struct CommentTab {
    pub filter_settings: CommentFilter,
    pub voice_settings: VoiceSettings,
    pub display_settings: CommentDisplaySettings,
}

#[derive(Default)]
pub struct CommentFilter {
    pub block_words: Vec<String>,
    pub min_account_age_days: u32,
    pub block_non_members: bool,
    pub block_first_time: bool,
}

#[derive(Default)]
pub struct VoiceSettings {
    pub enabled: bool,
    pub voice_type: String,
    pub speed: f32,
    pub pitch: f32,
    pub volume: f32,
}

#[derive(Default)]
pub struct CommentDisplaySettings {
    pub font_size: u32,
    pub display_time: u32,
    pub show_username: bool,
    pub show_member_icon: bool,
    pub color_member_names: bool,
}

impl CommentTab {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("コメント設定");

        // フィルター設定
        ui.collapsing("フィルター設定", |ui| {
            ui.horizontal(|ui| {
                ui.label("アカウント最小作成日数:");
                ui.add(egui::DragValue::new(&mut self.filter_settings.min_account_age_days)
                    .speed(1)
                    .suffix("日")
                    .clamp_range(0..=365));
            });

            ui.checkbox(&mut self.filter_settings.block_non_members, "メンバーのみ許可");
            ui.checkbox(&mut self.filter_settings.block_first_time, "初めてのコメントをブロック");

            ui.label("ブロックワード:");
            for word in &mut self.filter_settings.block_words {
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(word);
                    if ui.button("削除").clicked() {
                        // ブロックワードの削除処理は後で実装
                    }
                });
            }

            if ui.button("ブロックワードを追加").clicked() {
                self.filter_settings.block_words.push(String::new());
            }
        });

        // 読み上げ設定
        ui.collapsing("読み上げ設定", |ui| {
            ui.checkbox(&mut self.voice_settings.enabled, "コメント読み上げを有効化");

            if self.voice_settings.enabled {
                ui.horizontal(|ui| {
                    ui.label("声の種類:");
                    ui.text_edit_singleline(&mut self.voice_settings.voice_type);
                });

                ui.horizontal(|ui| {
                    ui.label("速度:");
                    ui.add(egui::Slider::new(&mut self.voice_settings.speed, 0.5..=2.0));
                });

                ui.horizontal(|ui| {
                    ui.label("ピッチ:");
                    ui.add(egui::Slider::new(&mut self.voice_settings.pitch, 0.5..=2.0));
                });

                ui.horizontal(|ui| {
                    ui.label("音量:");
                    ui.add(egui::Slider::new(&mut self.voice_settings.volume, 0.0..=1.0));
                });
            }
        });

        // 表示設定
        ui.collapsing("表示設定", |ui| {
            ui.horizontal(|ui| {
                ui.label("フォントサイズ:");
                ui.add(egui::DragValue::new(&mut self.display_settings.font_size)
                    .speed(1)
                    .suffix("px")
                    .clamp_range(8..=72));
            });

            ui.horizontal(|ui| {
                ui.label("表示時間:");
                ui.add(egui::DragValue::new(&mut self.display_settings.display_time)
                    .speed(1)
                    .suffix("秒")
                    .clamp_range(1..=60));
            });

            ui.checkbox(&mut self.display_settings.show_username, "ユーザー名を表示");
            ui.checkbox(&mut self.display_settings.show_member_icon, "メンバーアイコンを表示");
            ui.checkbox(&mut self.display_settings.color_member_names, "メンバー名を色付け");
        });
    }
} 