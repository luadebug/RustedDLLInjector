use egui::Response;
use egui_twemoji::EmojiLabel;

pub struct EmojiLabelWidget {
    label: EmojiLabel,
}

impl EmojiLabelWidget {
    pub fn new(label: &str) -> Self {
        Self { label: EmojiLabel::new(label) }
    }
}

impl egui::Widget for EmojiLabelWidget {
    fn ui(self, ui_: &mut egui::Ui) -> Response {
        let resp = self.label.show(ui_);
        // Ensure the cursor remains as an arrow when hovering over the label
        if resp.hovered() {
            ui_.output_mut(|o| o.cursor_icon = egui::CursorIcon::Default);
        }
        resp
    }
}
