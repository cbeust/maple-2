use eframe::egui;
use eframe::egui::{Align2, Context, InnerResponse, Rect, Ui, Vec2, vec2};
use eframe::egui_glow::check_for_gl_error;

#[derive(Default)]
pub struct Container {
    x: f32,
    y: f32,
}

impl Container {
    pub fn add(&mut self, ctx: &Context, title: &str, title_bar: bool, add_content: impl FnOnce(&mut Ui))
            -> InnerResponse<Option<()>> {
        let mut w = egui::Window::new(title)
            .anchor(Align2::RIGHT_TOP, Vec2::new(self.x, self.y))
            .title_bar(title_bar)
            .resizable(false)
            .show(ctx, |ui| {
                // ui.set_min_width(self.min_width);
                add_content(ui)
            }).unwrap();
        self.y += w.response.rect.height() + 5.0;
        w
    }

    pub fn add_window(&mut self, ctx: &Context, title: &str, add_content: impl FnOnce(&mut Ui))
            -> InnerResponse<Option<()>> {
        self.add(ctx, title, false, add_content)
    }
}
