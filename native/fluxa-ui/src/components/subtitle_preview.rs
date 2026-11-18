use super::*;
use serde_json::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct SubtitleLook {
    pub scale: f32,
    pub position: f32,
    pub text: Color32,
    pub background: Color32,
    pub outline: Color32,
    pub outline_size: f32,
    pub bold: bool,
    pub shadow: bool,
}

impl Default for SubtitleLook {
    fn default() -> Self {
        Self::from_values(&Value::Null)
    }
}

impl SubtitleLook {
    pub fn from_values(values: &Value) -> Self {
        let number = |key: &str, fallback: f32| {
            match values.get(key) {
                Some(Value::Number(value)) => value.as_f64().map(|value| value as f32),
                Some(Value::String(value)) => value.trim().parse().ok(),
                _ => None,
            }
            .unwrap_or(fallback)
        };
        let flag = |key: &str| values.get(key).and_then(Value::as_bool).unwrap_or(false);
        let color = |key: &str, fallback: &str, opacity: &str, alpha: f32| {
            let hex = values.get(key).and_then(Value::as_str).unwrap_or(fallback);
            let rgb = u32::from_str_radix(hex.trim().trim_start_matches('#'), 16).unwrap_or(0);
            let alpha = number(opacity, alpha).clamp(0.0, 1.0);
            Color32::from_rgba_unmultiplied(
                (rgb >> 16) as u8,
                (rgb >> 8) as u8,
                rgb as u8,
                (alpha * 255.0).round() as u8,
            )
        };
        Self {
            scale: number("subtitleSize", 100.0).clamp(25.0, 300.0) / 100.0,
            position: number("subtitlePosition", 100.0).clamp(0.0, 100.0) / 100.0,
            text: color("subtitleColor", "#FFFFFF", "subtitleTextOpacity", 1.0),
            background: color(
                "subtitleBackgroundColor",
                "#000000",
                "subtitleBackgroundOpacity",
                0.5,
            ),
            outline: color(
                "subtitleOutlineColor",
                "#000000",
                "subtitleOutlineOpacity",
                1.0,
            ),
            outline_size: number("subtitleOutlineSize", 3.0).clamp(0.0, 10.0),
            bold: flag("subtitleBold"),
            shadow: flag("subtitleShadow"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn opacity_strings_scale_alpha() {
        let look = SubtitleLook::from_values(&json!({
            "subtitleTextOpacity": "0.5",
            "subtitleBackgroundOpacity": "0.0",
            "subtitleColor": "#FFE45C",
        }));
        assert_eq!(
            look.text,
            Color32::from_rgba_unmultiplied(255, 228, 92, 128)
        );
        assert_eq!(look.background.a(), 0);
    }

    #[test]
    fn defaults_match_core_policy() {
        let look = SubtitleLook::default();
        assert_eq!(look.scale, 1.0);
        assert_eq!(look.outline_size, 3.0);
        assert_eq!(look.background.a(), 128);
    }
}
