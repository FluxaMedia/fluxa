//! One icon source for every native host.
//!
//! The UI crate only asks for a named texture. Desktop and Android upload the
//! same SVG bytes into their own WGPU/egui renderer, so responsive layout does
//! not create a second, visually different icon set.

use resvg::usvg::{Options, Tree};
use tiny_skia::{Pixmap, Transform};

pub const ICON_SIZE: u32 = 32;

pub const ICONS: &[(&str, &str)] = &[
    // Lucide outline icons are also used by the web client. Keeping their SVG
    // paths here gives native screens the same restrained 1.8px icon language.
    (
        "Account",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M20 21a8 8 0 0 0-16 0" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round"/><circle cx="12" cy="8" r="4" fill="none" stroke="white" stroke-width="1.8"/></svg>"#,
    ),
    (
        "General",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="7" cy="7" r="3" fill="none" stroke="white" stroke-width="1.8"/><circle cx="17" cy="17" r="3" fill="none" stroke="white" stroke-width="1.8"/><path d="M10 7h10M4 17h10" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round"/></svg>"#,
    ),
    (
        "Appearance",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="13.5" cy="6.5" r=".8" fill="white"/><circle cx="17.5" cy="10.5" r=".8" fill="white"/><circle cx="8.5" cy="7.5" r=".8" fill="white"/><circle cx="6.5" cy="12.5" r=".8" fill="white"/><path d="M12 22a10 10 0 1 1 10-10c0 1.7-1.3 3-3 3h-1.8a1.5 1.5 0 0 0-1.1 2.5l.3.3A3 3 0 0 1 14.3 23H12Z" fill="none" stroke="white" stroke-width="1.8" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "Playback",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="m7 4 13 8-13 8V4Z" fill="none" stroke="white" stroke-width="1.8" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "Device",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><rect x="3" y="4" width="18" height="13" rx="2" fill="none" stroke="white" stroke-width="1.8"/><path d="M8 21h8m-4-4v4" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round"/></svg>"#,
    ),
    (
        "Shortcuts",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><rect x="2" y="4" width="20" height="16" rx="2" fill="none" stroke="white" stroke-width="1.8"/><path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M6 12h.01M10 12h.01M14 12h.01M18 12h.01M8 16h8" fill="none" stroke="white" stroke-width="2.2" stroke-linecap="round"/></svg>"#,
    ),
    (
        "Controller",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M6 9h12a4 4 0 0 1 3.8 5.3l-1 3A2.5 2.5 0 0 1 17 18.5l-2.2-1.7h-5.6L7 18.5a2.5 2.5 0 0 1-3.8-1.2l-1-3A4 4 0 0 1 6 9Z" fill="none" stroke="white" stroke-width="1.8" stroke-linejoin="round"/><path d="M7 12v4m-2-2h4m6-1h.01M18 15h.01" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round"/></svg>"#,
    ),
    (
        "Content",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><rect x="3" y="3" width="8" height="8" rx="1.5" fill="none" stroke="white" stroke-width="1.8"/><rect x="13" y="3" width="8" height="5" rx="1.5" fill="none" stroke="white" stroke-width="1.8"/><rect x="13" y="10" width="8" height="11" rx="1.5" fill="none" stroke="white" stroke-width="1.8"/><rect x="3" y="13" width="8" height="8" rx="1.5" fill="none" stroke="white" stroke-width="1.8"/></svg>"#,
    ),
    (
        "Add-ons",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M12 8V5a2 2 0 1 0-2-2H7a2 2 0 0 0-2 2v4H3a2 2 0 1 0 0 4h2v6a2 2 0 0 0 2 2h4v-2a2 2 0 1 1 4 0v2h4a2 2 0 0 0 2-2v-4h-2a2 2 0 1 1 0-4h2V9a2 2 0 0 0-2-2h-5V5a2 2 0 1 0-2 2v1Z" fill="none" stroke="white" stroke-width="1.6" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "Plugins",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M9 6V3m6 3V3M9 21v-3m6 3v-3M3 9h3m-3 6h3m12-6h3m-3 6h3M8 6h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2Z" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "Downloads",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4m4-5 5 5 5-5m-5 5V3" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "ChevronDown",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="m6 9 6 6 6-6" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "Home",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="m3 10 9-7 9 7v10a1 1 0 0 1-1 1h-5v-7H9v7H4a1 1 0 0 1-1-1V10Z" fill="none" stroke="white" stroke-width="1.8" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "Library",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M4 4h11a2 2 0 0 1 2 2v14H6a2 2 0 0 1-2-2V4Zm13 4h3v12a2 2 0 0 1-2 2h-1" fill="none" stroke="white" stroke-width="1.8" stroke-linejoin="round"/><path d="M8 8h5m-5 4h5" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round"/></svg>"#,
    ),
    (
        "Discover",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="10" fill="none" stroke="white" stroke-width="1.8"/><path d="m16.2 7.8-2.4 5.9-6 2.5 2.5-6 5.9-2.4Z" fill="none" stroke="white" stroke-width="1.8" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "Movie",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><rect x="3" y="4" width="18" height="16" rx="2" fill="none" stroke="white" stroke-width="1.8"/><path d="M7 4v4m5-4v4m5-4v4M7 16h10" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round"/></svg>"#,
    ),
    (
        "Calendar",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M6 5.5H18Q20 5.5 20 7.5V18Q20 20 18 20H6Q4 20 4 18V7.5Q4 5.5 6 5.5ZM4 9.5H20M8 3.5V7.5M16 3.5V7.5" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/><path d="M8 14.5H13.5" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round"/></svg>"#,
    ),
    (
        "Settings",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8Z" fill="none" stroke="white" stroke-width="1.8"/><path d="M12 2v2m0 16v2m10-10h-2M4 12H2m17.1-7.1-1.4 1.4M6.3 17.7l-1.4 1.4m14.2 0-1.4-1.4M6.3 6.3 4.9 4.9" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round"/></svg>"#,
    ),
    (
        "Profile",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M20 21a8 8 0 0 0-16 0" fill="none" stroke="white" stroke-width="1.8" stroke-linecap="round"/><circle cx="12" cy="8" r="4" fill="none" stroke="white" stroke-width="1.8"/></svg>"#,
    ),
];

pub fn source(name: &str) -> Option<&'static str> {
    ICONS
        .iter()
        .find(|(key, _)| *key == name)
        .map(|(_, svg)| *svg)
}

pub fn rasterize_svg(source: &[u8], max_side: u32) -> Result<image::RgbaImage, String> {
    let tree = Tree::from_data(source, &Options::default()).map_err(|error| error.to_string())?;
    let size = tree.size();
    let width = size.width().max(1.0);
    let height = size.height().max(1.0);
    let scale = (max_side as f32 / width.max(height)).min(1.0);
    let output_width = (width * scale).round().max(1.0) as u32;
    let output_height = (height * scale).round().max(1.0) as u32;
    let mut pixmap = Pixmap::new(output_width, output_height)
        .ok_or_else(|| "could not allocate SVG pixmap".to_owned())?;
    resvg::render(
        &tree,
        Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    image::RgbaImage::from_raw(output_width, output_height, pixmap.data().to_vec())
        .ok_or_else(|| "SVG pixmap had an invalid RGBA buffer".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{ICONS, rasterize_svg};

    #[test]
    fn all_shared_lucide_svg_icons_rasterize() {
        for (name, source) in ICONS {
            let image = rasterize_svg(source.as_bytes(), 32)
                .unwrap_or_else(|error| panic!("{name} SVG should rasterize: {error}"));
            assert!(
                image.width() > 0 && image.height() > 0,
                "{name} SVG is empty"
            );
            assert!(
                image.pixels().any(|pixel| pixel[3] > 0),
                "{name} SVG is transparent"
            );
        }
    }
}
