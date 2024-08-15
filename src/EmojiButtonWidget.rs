use egui::{Button, Color32, Frame, Pos2, Rect, Response, Rounding, Sense, Stroke, Style, Vec2, Widget};
use egui_twemoji::EmojiLabel;
use crate::EmojiLabelWidget::EmojiLabelWidget;

pub struct EmojiButtonWidget {
    label: String,
    button: Button<'static>,
}

impl EmojiButtonWidget {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            button: Button::new(""), // Initialize with an empty label since we'll draw the emoji label manually.
        }
    }
    pub fn min_size(mut self, size: Vec2) -> Self {
        self.button = self.button.min_size(size);
        self
    }
}

impl Widget for EmojiButtonWidget {

    fn ui(self, ui: &mut egui::Ui) -> Response {
        let EmojiButtonWidget { label, button } = self;

        // Check if the button or label is hovered
        let is_hovered = ui.rect_contains_pointer(ui.available_rect_before_wrap());

        // Define the animation effect (e.g., change the background color)
        let frame = if is_hovered {
            Frame::canvas(&Style::default())
                .fill(ui.visuals().widgets.hovered.bg_fill)
                .rounding(ui.visuals().widgets.hovered.rounding)
                .stroke(ui.visuals().widgets.hovered.bg_stroke)
        } else {
            Frame::default()
                .fill(ui.visuals().widgets.inactive.bg_fill)
                .rounding(ui.visuals().widgets.inactive.rounding)
                .stroke(ui.visuals().widgets.inactive.bg_stroke)
        };

        // Apply the frame to the button
        let response = frame.show(ui, |ui| ui.add(button)).inner;

        // Manually draw the emoji label on top of the button
        let emoji_label = EmojiLabelWidget::new(&label);
        let label_response = ui.put(response.rect, emoji_label);

        // Combine the responses
        let combined_response = response.union(label_response);

        // Check for clicks
        if combined_response.clicked() {
            println!("Button with custom label was clicked!");
        }

        combined_response
    }

}