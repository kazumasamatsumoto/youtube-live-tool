mod app;
mod tabs;
mod models;

use app::main_window::MainWindow;
use eframe::egui;
use eframe::egui::ViewportBuilder;
use std::default::Default;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("YouTube Live 配信ツール"),
        ..Default::default()
    };

    eframe::run_native(
        "YouTube Live 配信ツール",
        options,
        Box::new(|cc| {
            setup_fonts_and_style(cc);
            Box::new(MainWindow::default())
        }),
    )
}

fn setup_fonts_and_style(cc: &eframe::CreationContext) {
    // フォントの設定
    let mut fonts = egui::FontDefinitions::default();
    
    fonts.font_data.insert(
        "notosans_jp".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/fonts/static/NotoSansJP-Regular.ttf")),
    );

    fonts.families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "notosans_jp".to_owned());

    cc.egui_ctx.set_fonts(fonts);

    // ダークテーマの設定
    let mut style = (*cc.egui_ctx.style()).clone();
    let mut visuals = style.visuals.clone();
    
    visuals.dark_mode = true;
    visuals.panel_fill = egui::Color32::from_rgb(32, 33, 36);  // Googleダークテーマ風
    visuals.window_fill = egui::Color32::from_rgb(40, 41, 45);
    
    // アクセントカラー
    visuals.selection.bg_fill = egui::Color32::from_rgb(76, 175, 80);  // Material Design Green
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(76, 175, 80);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(67, 160, 71);
    
    // 角丸設定
    visuals.window_rounding = 8.0.into();
    visuals.widgets.noninteractive.rounding = 4.0.into();
    visuals.widgets.inactive.rounding = 4.0.into();
    visuals.widgets.hovered.rounding = 4.0.into();
    visuals.widgets.active.rounding = 4.0.into();
    
    style.visuals = visuals;
    cc.egui_ctx.set_style(style);
}
