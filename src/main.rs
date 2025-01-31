mod app;
mod tabs;
mod models;

use app::main_window::MainWindow;
use log::{info, LevelFilter};
use log4rs::{
    append::file::FileAppender,
    config::{Appender, Config, Root},
    encode::pattern::PatternEncoder,
};
use std::fs;
use eframe::egui;
use eframe::egui::ViewportBuilder;
use std::default::Default;
use log4rs;

fn init_logger() -> Result<(), Box<dyn std::error::Error>> {
    // ログディレクトリの作成
    fs::create_dir_all("log")?;

    // ファイルアペンダーの設定
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d(%Y-%m-%d %H:%M:%S)} [{l}] {m}{n}")))
        .build("log/camera.log")?;

    // ログ設定
    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder()
            .appender("logfile")
            .build(LevelFilter::Info))?;

    // ログ設定の適用
    log4rs::init_config(config)?;
    
    info!("ログシステムを初期化しました");
    Ok(())
}

fn main() -> Result<(), eframe::Error> {
    // 一つのログ設定だけを使用
    if let Err(e) = log4rs::init_file("config/log4rs.yaml", Default::default()) {
        eprintln!("ログ設定の初期化に失敗しました: {}", e);
    }

    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("YouTube Live 配信ツール")
            .with_active(true)  // アクティブ状態を維持
            .with_always_on_top(),
        renderer: eframe::Renderer::default(),
        follow_system_theme: false,
        centered: true,
        persist_window: true,  // ウィンドウの状態を維持
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
