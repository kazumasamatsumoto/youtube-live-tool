use eframe::egui;

#[derive(Default)]
pub struct BannerTab {
    pub banners: Vec<Banner>,
    pub default_duration: u32,
}

#[derive(Default)]
pub struct Banner {
    pub text: String,
    pub enabled: bool,
    pub color: [f32; 3],
    pub duration: u32,
    pub position: BannerPosition,
}

#[derive(Default, PartialEq)]
pub enum BannerPosition {
    #[default]
    Top,
    Bottom,
}

impl BannerTab {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("バナー設定");

        ui.horizontal(|ui| {
            ui.label("デフォルト表示時間:");
            ui.add(egui::DragValue::new(&mut self.default_duration)
                .speed(1)
                .suffix("秒")
                .clamp_range(1..=300));
        });

        for banner in &mut self.banners {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut banner.enabled, "");
                    ui.text_edit_multiline(&mut banner.text);
                });

                ui.horizontal(|ui| {
                    ui.label("表示位置:");
                    ui.radio_value(&mut banner.position, BannerPosition::Top, "上");
                    ui.radio_value(&mut banner.position, BannerPosition::Bottom, "下");
                });

                ui.horizontal(|ui| {
                    ui.label("文字色:");
                    ui.color_edit_button_rgb(&mut banner.color);
                    ui.label("表示時間:");
                    ui.add(egui::DragValue::new(&mut banner.duration)
                        .speed(1)
                        .suffix("秒")
                        .clamp_range(1..=300));
                });
            });
        }

        if ui.button("バナーを追加").clicked() {
            self.banners.push(Banner {
                text: "新しいバナー".to_string(),
                enabled: true,
                color: [1.0, 1.0, 1.0],
                duration: self.default_duration,
                position: BannerPosition::Top,
            });
        }
    }
} 