use serde_json::{Value, json};

const BASE_COLOR_WHITE: u32 = 0xFFFFFF;

pub(crate) fn subtitle_style_plan_json(input: &str) -> Option<String> {
    let settings: Value = serde_json::from_str(input).ok()?;
    let text = rgb(&settings, "subtitleColor", BASE_COLOR_WHITE);
    let background = rgb(&settings, "subtitleBackgroundColor", 0);
    let outline = rgb(&settings, "subtitleOutlineColor", 0);
    let text_alpha = unit(&settings, "subtitleTextOpacity", 1.0);
    let background_alpha = unit(&settings, "subtitleBackgroundOpacity", 0.5);
    let outline_alpha = unit(&settings, "subtitleOutlineOpacity", 1.0);
    let size = number(&settings, "subtitleSize", 100.0).clamp(25.0, 300.0);
    let outline_size = number(&settings, "subtitleOutlineSize", 3.0).clamp(0.0, 10.0);
    let flag = |key: &str| settings.get(key).and_then(Value::as_bool).unwrap_or(false);
    let style = if background_alpha > 0.0 {
        "background-box"
    } else {
        "outline-and-shadow"
    };
    let override_mode = if flag("subtitleForceStyle") {
        "force"
    } else {
        "scale"
    };
    let options = json!([
        ["sub-scale", format!("{:.2}", size / 100.0)],
        ["sub-color", argb(text, text_alpha)],
        ["sub-back-color", argb(background, background_alpha)],
        ["sub-border-color", argb(outline, outline_alpha)],
        ["sub-border-size", format!("{outline_size:.1}")],
        ["sub-border-style", style],
        ["sub-bold", yes_no(flag("subtitleBold"))],
        [
            "sub-shadow-offset",
            if flag("subtitleShadow") { "1.5" } else { "0" }
        ],
        ["sub-ass-override", override_mode],
    ]);
    serde_json::to_string(&options).ok()
}

pub(crate) fn subtitle_lift_plan_json(input: &str) -> Option<String> {
    let input: Value = serde_json::from_str(input).ok()?;
    let position = number(&input, "subtitlePosition", 100.0).clamp(0.0, 100.0);
    let height = number(&input, "height", 0.0);
    let lift = number(&input, "lift", 0.0).max(0.0);
    let raised = input
        .get("controlsVisible")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let offset = if raised && height > 0.0 {
        lift / height * 100.0
    } else {
        0.0
    };
    serde_json::to_string(&json!([[
        "sub-pos",
        format!("{:.1}", (position - offset).max(0.0))
    ]]))
    .ok()
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn number(settings: &Value, key: &str, fallback: f64) -> f64 {
    match settings.get(key) {
        Some(Value::Number(value)) => value.as_f64(),
        Some(Value::String(value)) => value.trim().parse().ok(),
        _ => None,
    }
    .unwrap_or(fallback)
}

fn unit(settings: &Value, key: &str, fallback: f64) -> f64 {
    number(settings, key, fallback).clamp(0.0, 1.0)
}

fn rgb(settings: &Value, key: &str, fallback: u32) -> u32 {
    match settings.get(key) {
        Some(Value::String(value)) => u32::from_str_radix(value.trim().trim_start_matches('#'), 16)
            .ok()
            .map(|color| color & 0xFFFFFF),
        Some(Value::Number(value)) => value.as_i64().map(|color| (color as u32) & 0xFFFFFF),
        _ => None,
    }
    .unwrap_or(fallback)
}

fn argb(color: u32, alpha: f64) -> String {
    format!("#{:02X}{color:06X}", (alpha * 255.0).round() as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(settings: Value) -> Vec<(String, String)> {
        let plan = subtitle_style_plan_json(&settings.to_string()).unwrap();
        serde_json::from_str(&plan).unwrap()
    }

    fn get(options: &[(String, String)], name: &str) -> String {
        options
            .iter()
            .find(|(key, _)| key == name)
            .unwrap()
            .1
            .clone()
    }

    #[test]
    fn manifest_defaults_give_a_half_transparent_box() {
        let plan = options(json!({
            "subtitleColor": "#FFFFFF",
            "subtitleBackgroundColor": "#000000",
            "subtitleBackgroundOpacity": "0.5",
            "subtitleSize": "100",
            "subtitlePosition": "100",
        }));
        assert_eq!(get(&plan, "sub-color"), "#FFFFFFFF");
        assert_eq!(get(&plan, "sub-back-color"), "#80000000");
        assert_eq!(get(&plan, "sub-border-style"), "background-box");
        assert_eq!(get(&plan, "sub-scale"), "1.00");
    }

    #[test]
    fn zero_background_opacity_keeps_the_outline_style() {
        let plan = options(json!({"subtitleBackgroundOpacity": "0.0", "subtitleSize": "150"}));
        assert_eq!(get(&plan, "sub-border-style"), "outline-and-shadow");
        assert_eq!(get(&plan, "sub-scale"), "1.50");
    }

    fn lifted(input: Value) -> String {
        let plan = subtitle_lift_plan_json(&input.to_string()).unwrap();
        let options: Vec<(String, String)> = serde_json::from_str(&plan).unwrap();
        options[0].1.clone()
    }

    #[test]
    fn visible_controls_raise_subtitles_by_a_share_of_the_height() {
        let raised = lifted(json!({
            "subtitlePosition": "100",
            "controlsVisible": true,
            "lift": 90.0,
            "height": 900.0,
        }));
        assert_eq!(raised, "90.0");
    }

    #[test]
    fn hidden_controls_keep_the_saved_position() {
        let resting = lifted(json!({
            "subtitlePosition": "95",
            "controlsVisible": false,
            "lift": 90.0,
            "height": 900.0,
        }));
        assert_eq!(resting, "95.0");
    }

    #[test]
    fn lift_never_pushes_below_the_top() {
        let top = lifted(json!({
            "subtitlePosition": "5",
            "controlsVisible": true,
            "lift": 500.0,
            "height": 600.0,
        }));
        assert_eq!(top, "0.0");
    }
}
