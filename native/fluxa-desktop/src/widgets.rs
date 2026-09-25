use egui::{Color32, Response, RichText, Ui};
use fluxa_renderer::theme::{shared_ui_tokens, theme};

#[derive(Clone, Copy)]
pub struct NativeTheme {
    pub accent: Color32,
    pub muted: Color32,
    pub text_primary: Color32,
    pub focus: Color32,
    pub control_radius: f32,
    pub display_size: f32,
    pub body_size: f32,
}

impl Default for NativeTheme {
    fn default() -> Self {
        let tokens = shared_ui_tokens().expect("shared Fluxa tokens must be valid");
        let shared = theme("fluxa-dark").expect("shared Fluxa theme must be valid");
        let color = |token: &str, fallback: Color32| {
            shared
                .color(token)
                .map(|value| {
                    Color32::from_rgba_unmultiplied(
                        (value.r * 255.0).round() as u8,
                        (value.g * 255.0).round() as u8,
                        (value.b * 255.0).round() as u8,
                        (value.a * 255.0).round() as u8,
                    )
                })
                .unwrap_or(fallback)
        };
        let layout_number = |group: &str, token: &str, fallback: f32| {
            tokens
                .layout
                .get(group)
                .and_then(|value| value.get("sp"))
                .and_then(|value| value.get(token))
                .and_then(serde_json::Value::as_f64)
                .map(|value| value as f32)
                .unwrap_or(fallback)
        };
        Self {
            accent: color("accent", Color32::from_rgb(232, 93, 63)),
            muted: color("textMuted", Color32::from_white_alpha(150)),
            text_primary: color("textPrimary", Color32::WHITE),
            focus: color("focus", Color32::WHITE),
            control_radius: shared.shape.control_radius,
            display_size: layout_number("materialTheme", "displayMedium", 34.0),
            body_size: layout_number("materialTheme", "bodyMedium", 15.0),
        }
    }
}

impl NativeTheme {
    pub fn chip(&self, ui: &mut Ui, label: impl Into<RichText>, selected: bool) -> Response {
        let label: RichText = label.into();
        ui.add(
            egui::Button::new(label)
                .fill(if selected {
                    self.accent
                } else {
                    Color32::from_white_alpha(18)
                })
                .stroke(egui::Stroke::new(
                    1.0,
                    if selected {
                        self.focus
                    } else {
                        Color32::from_white_alpha(22)
                    },
                ))
                .corner_radius(self.control_radius),
        )
    }

    pub fn refresh_button(&self, ui: &mut Ui) -> Response {
        ui.add(egui::Button::new("↻  Refresh").corner_radius(self.control_radius))
    }

    pub fn header(&self, ui: &mut Ui, title: &str, subtitle: &str) {
        ui.vertical(|ui| {
            ui.heading(
                RichText::new(title)
                    .size(self.display_size)
                    .strong()
                    .color(self.text_primary),
            );
            ui.label(
                RichText::new(subtitle)
                    .size(self.body_size)
                    .color(self.muted),
            );
        });
    }
}
