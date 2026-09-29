use serde::Deserialize;
use std::sync::OnceLock;

const APP_ICONS_JSON: &str = include_str!("../../../../shared/brand/app-icons.json");
const MARK_SVG: &str = include_str!("../../../../shared/brand/fluxa-mark.svg");

#[derive(Clone, Debug, Deserialize)]
pub struct AppIcon {
    pub id: String,
    pub from: String,
    pub to: String,
}

#[derive(Deserialize)]
struct Catalog {
    default: String,
    icons: Vec<AppIcon>,
}

fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| serde_json::from_str(APP_ICONS_JSON).expect("valid app-icons.json"))
}

pub fn app_icons() -> &'static [AppIcon] {
    &catalog().icons
}

pub fn default_app_icon() -> &'static AppIcon {
    resolve_app_icon(None)
}

pub fn resolve_app_icon(stored: Option<&str>) -> &'static AppIcon {
    let catalog = catalog();
    stored
        .and_then(|id| catalog.icons.iter().find(|icon| icon.id == id))
        .or_else(|| catalog.icons.iter().find(|icon| icon.id == catalog.default))
        .unwrap_or(&catalog.icons[0])
}

pub fn app_icon_svg(icon: &AppIcon) -> String {
    MARK_SVG
        .trim()
        .replacen(
            "stop-color=\"#FF7A2F\"",
            &format!("stop-color=\"{}\"", icon.from),
            1,
        )
        .replacen(
            "stop-color=\"#FF2E63\"",
            &format!("stop-color=\"{}\"", icon.to),
            1,
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_icon_falls_back_to_default() {
        assert_eq!(resolve_app_icon(Some("missing")).id, "ember");
        assert_eq!(resolve_app_icon(Some("ocean")).id, "ocean");
    }

    #[test]
    fn default_svg_is_the_brand_mark() {
        assert_eq!(app_icon_svg(default_app_icon()), MARK_SVG.trim());
    }
}
