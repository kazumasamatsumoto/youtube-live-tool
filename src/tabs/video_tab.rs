use eframe::egui;

#[derive(Default)]
pub struct VideoTab {
    pub cameras: Vec<CameraDevice>,
    pub screen_capture: ScreenCapture,
    pub layout: LayoutSettings,
}

#[derive(Default)]
pub struct CameraDevice {
    pub name: String,
    pub enabled: bool,
    pub position: (f32, f32),
    pub size: (f32, f32),
}

#[derive(Default)]
pub struct ScreenCapture {
    pub enabled: bool,
    pub capture_area: CaptureArea,
    pub position: (f32, f32),
    pub size: (f32, f32),
}

#[derive(Default, PartialEq)]
pub enum CaptureArea {
    #[default]
    FullScreen,
    Window,
    CustomArea,
}

#[derive(Default)]
pub struct LayoutSettings {
    pub background_color: [f32; 3],
    pub show_grid: bool,
    pub grid_size: u32,
}

impl VideoTab {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("映像設定");

        // カメラ設定
        ui.collapsing("カメラ設定", |ui| {
            for (_index, camera) in self.cameras.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut camera.enabled, &camera.name);
                    if camera.enabled {
                        ui.label("位置:");
                        ui.add(egui::DragValue::new(&mut camera.position.0).speed(1.0).suffix("x"));
                        ui.add(egui::DragValue::new(&mut camera.position.1).speed(1.0).suffix("y"));
                        ui.label("サイズ:");
                        ui.add(egui::DragValue::new(&mut camera.size.0).speed(1.0).suffix("w"));
                        ui.add(egui::DragValue::new(&mut camera.size.1).speed(1.0).suffix("h"));
                    }
                });
            }

            if ui.button("カメラを追加").clicked() {
                self.cameras.push(CameraDevice {
                    name: format!("カメラ {}", self.cameras.len() + 1),
                    enabled: true,
                    position: (0.0, 0.0),
                    size: (320.0, 240.0),
                });
            }
        });

        // 画面キャプチャ設定
        ui.collapsing("画面キャプチャ", |ui| {
            ui.checkbox(&mut self.screen_capture.enabled, "画面キャプチャを有効化");
            
            if self.screen_capture.enabled {
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.screen_capture.capture_area, CaptureArea::FullScreen, "全画面");
                    ui.radio_value(&mut self.screen_capture.capture_area, CaptureArea::Window, "ウィンドウ");
                    ui.radio_value(&mut self.screen_capture.capture_area, CaptureArea::CustomArea, "カスタム");
                });

                ui.horizontal(|ui| {
                    ui.label("位置:");
                    ui.add(egui::DragValue::new(&mut self.screen_capture.position.0).speed(1.0).suffix("x"));
                    ui.add(egui::DragValue::new(&mut self.screen_capture.position.1).speed(1.0).suffix("y"));
                });

                ui.horizontal(|ui| {
                    ui.label("サイズ:");
                    ui.add(egui::DragValue::new(&mut self.screen_capture.size.0).speed(1.0).suffix("w"));
                    ui.add(egui::DragValue::new(&mut self.screen_capture.size.1).speed(1.0).suffix("h"));
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
} 