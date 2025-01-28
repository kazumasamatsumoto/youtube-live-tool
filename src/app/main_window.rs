use eframe::egui;
use crate::tabs::{StreamTab, AudioTab, VideoTab, BannerTab, CommentTab, StreamStatus, StatusTab};

pub struct MainWindow {
    selected_tab: Tab,
    stream_tab: StreamTab,
    audio_tab: AudioTab,
    video_tab: VideoTab,
    banner_tab: BannerTab,
    comment_tab: CommentTab,
    status_tab: StatusTab,
    show_exit_confirmation: bool,
    show_stream_settings: bool,
}

impl Default for MainWindow {
    fn default() -> Self {
        Self {
            selected_tab: Tab::default(),
            stream_tab: StreamTab::default(),
            audio_tab: AudioTab::default(),
            video_tab: VideoTab::default(),
            banner_tab: BannerTab::default(),
            comment_tab: CommentTab::default(),
            status_tab: StatusTab::default(),
            show_exit_confirmation: false,
            show_stream_settings: false,
        }
    }
}

#[derive(Default, PartialEq)]
pub enum Tab {
    #[default]
    Status,
    Settings,
    Audio,
    Video,
    Banner,
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);  // 左マージン
                
                ui.add_space(8.0);  // 左マージン
                
                for (tab, label) in [
                    (Tab::Status, "配信状況"),
                    (Tab::Settings, "配信設定"),
                    (Tab::Audio, "音声設定"),
                    (Tab::Video, "映像設定"),
                    (Tab::Banner, "バナー設定"),
                ] {
                    let is_selected = self.selected_tab == tab;
                    let response = ui.add(
                        egui::SelectableLabel::new(is_selected, label)
                    );
                    
                    if response.clicked() {
                        self.selected_tab = tab;
                    }
                }
                
                // 右寄せの終了ボタン
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("終了").clicked() {
                        if self.status_tab.is_streaming {
                            // 配信中の場合は確認ダイアログを表示
                            self.show_exit_confirmation = true;
                        } else {
                            // 配信中でない場合は直接終了
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });
            });

            ui.add_space(8.0);  // タブと内容の間のスペース

            // 終了確認ダイアログ
            if self.show_exit_confirmation {
                egui::Window::new("確認")
                    .collapsible(false)
                    .resizable(false)
                    .show(ctx, |ui| {
                        ui.label("配信中です。本当に終了しますか？");
                        ui.horizontal(|ui| {
                            if ui.button("はい").clicked() {
                                // 配信を停止して終了
                                self.status_tab.is_streaming = false;
                                self.status_tab.status = StreamStatus::Offline;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            if ui.button("いいえ").clicked() {
                                self.show_exit_confirmation = false;
                            }
                        });
                    });
            }

            self.show_tab_content(ui);

            // 配信設定ウィンドウ
            if self.show_stream_settings {
                egui::Window::new("配信設定")
                    .collapsible(false)
                    .resizable(true)
                    .default_size([400.0, 500.0])
                    .show(ctx, |ui| {
                        self.stream_tab.ui(ui);
                        ui.separator();
                        if ui.button("閉じる").clicked() {
                            self.show_stream_settings = false;
                        }
                    });
            }
        });
    }

}

impl MainWindow {
    fn show_tab_content(&mut self, ui: &mut egui::Ui) {
        let transition = ui.ctx().animate_bool_with_time(
            egui::Id::new("tab_transition"),
            true,
            0.2
        );
        
        ui.scope(|ui| {
            ui.set_clip_rect(ui.available_rect_before_wrap());
            ui.set_enabled(transition > 0.0);  // フェードイン効果の代替
            
            match self.selected_tab {
                Tab::Status => self.status_tab.ui(ui),
                Tab::Settings => self.stream_tab.ui(ui),
                Tab::Audio => self.audio_tab.ui(ui),
                Tab::Video => self.video_tab.ui(ui),
                Tab::Banner => self.banner_tab.ui(ui),
            }
        });
    }
}
