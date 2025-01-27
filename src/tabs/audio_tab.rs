use eframe::egui;

#[derive(Default)]
pub struct AudioTab {
    pub bgm_tracks: Vec<BGMTrack>,
    pub effects: Vec<SoundEffect>,
    pub master_volume: f32,
    pub bgm_volume: f32,
    pub effects_volume: f32,
}

#[derive(Default)]
pub struct BGMTrack {
    pub name: String,
    pub volume: f32,
    pub is_playing: bool,
}

#[derive(Default)]
pub struct SoundEffect {
    pub name: String,
    pub volume: f32,
    pub hotkey: String,
}

impl AudioTab {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("音声設定");

        // マスターボリューム
        ui.horizontal(|ui| {
            ui.label("マスター音量:");
            ui.add(egui::Slider::new(&mut self.master_volume, 0.0..=1.0));
        });

        // BGM設定
        ui.collapsing("BGM設定", |ui| {
            ui.horizontal(|ui| {
                ui.label("BGM音量:");
                ui.add(egui::Slider::new(&mut self.bgm_volume, 0.0..=1.0));
            });

            for track in &mut self.bgm_tracks {
                ui.horizontal(|ui| {
                    if track.is_playing {
                        if ui.button("⏸").clicked() {
                            track.is_playing = false;
                        }
                    } else {
                        if ui.button("▶").clicked() {
                            track.is_playing = true;
                        }
                    }
                    ui.label(&track.name);
                    ui.add(egui::Slider::new(&mut track.volume, 0.0..=1.0).text("音量"));
                });
            }

            if ui.button("BGMを追加").clicked() {
                self.bgm_tracks.push(BGMTrack {
                    name: "新しいBGM".to_string(),
                    volume: 0.5,
                    is_playing: false,
                });
            }
        });

        // 効果音設定
        ui.collapsing("効果音設定", |ui| {
            ui.horizontal(|ui| {
                ui.label("効果音音量:");
                ui.add(egui::Slider::new(&mut self.effects_volume, 0.0..=1.0));
            });

            for effect in &mut self.effects {
                ui.horizontal(|ui| {
                    if ui.button("▶").clicked() {
                        // 効果音再生処理
                    }
                    ui.label(&effect.name);
                    ui.add(egui::Slider::new(&mut effect.volume, 0.0..=1.0).text("音量"));
                    ui.label("ホットキー:");
                    ui.text_edit_singleline(&mut effect.hotkey);
                });
            }

            if ui.button("効果音を追加").clicked() {
                self.effects.push(SoundEffect {
                    name: "新しい効果音".to_string(),
                    volume: 0.5,
                    hotkey: String::new(),
                });
            }
        });
    }
} 