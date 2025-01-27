use eframe::egui;
use crate::tabs::{StreamTab, AudioTab, VideoTab, BannerTab, CommentTab, StreamStatus};

#[derive(Default)]
pub struct MainWindow {
    selected_tab: Tab,
    stream_tab: StreamTab,
    audio_tab: AudioTab,
    video_tab: VideoTab,
    banner_tab: BannerTab,
    comment_tab: CommentTab,
    show_exit_confirmation: bool,
}

#[derive(Default, PartialEq)]
pub enum Tab {
    #[default]
    Stream,
    Audio,
    Video,
    Banner,
    Comment,
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(8.0);  // 左マージン
                
                for (tab, label) in [
                    (Tab::Stream, "配信"),
                    (Tab::Audio, "音声"),
                    (Tab::Video, "映像"),
                    (Tab::Banner, "バナー"),
                    (Tab::Comment, "コメント"),
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
                        if self.stream_tab.is_streaming {
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
                                self.stream_tab.is_streaming = false;
                                self.stream_tab.status = StreamStatus::Offline;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                            if ui.button("いいえ").clicked() {
                                self.show_exit_confirmation = false;
                            }
                        });
                    });
            }

            self.show_tab_content(ui);
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
                Tab::Stream => self.stream_tab.ui(ui),
                Tab::Audio => self.audio_tab.ui(ui),
                Tab::Video => self.video_tab.ui(ui),
                Tab::Banner => self.banner_tab.ui(ui),
                Tab::Comment => self.comment_tab.ui(ui),
            }
        });
    }
}
