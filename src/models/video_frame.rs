use eframe::egui;

#[allow(dead_code)]
pub struct VideoFrame {
    texture: Option<egui::TextureId>,
    width: u32,
    height: u32,
}

impl VideoFrame {
    #[allow(dead_code)]
    pub fn new(texture: egui::TextureId, width: u32, height: u32) -> Self {
        Self {
            texture: Some(texture),
            width,
            height,
        }
    }

    #[allow(dead_code)]
    pub fn texture_id(&self) -> Option<egui::TextureId> {
        self.texture
    }

    #[allow(dead_code)]
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
} 