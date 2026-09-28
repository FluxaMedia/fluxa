//! One icon source for every native host.
//!
//! The UI crate only asks for a named texture. Desktop and Android upload the
//! same SVG bytes into their own WGPU/egui renderer, so responsive layout does
//! not create a second, visually different icon set.

use resvg::usvg::{Options, Tree};
use tiny_skia::{Pixmap, Transform};

pub const ICON_SIZE: u32 = 128;

pub const ICONS: &[(&str, &str)] = &[
    // Lucide outline icons are also used by the web client. Keeping their SVG
    // paths here gives native screens the same restrained 1.8px icon language.
    (
        "Account",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="8" r="5"/><path d="M20 21a8 8 0 0 0-16 0"/></g></svg>"#,
    ),
    (
        "General",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 4h-7M10 4H3M21 12h-9M8 12H3M21 20h-5M12 20H3M14 2v4M8 10v4M16 18v4"/></g></svg>"#,
    ),
    (
        "Appearance",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="13.5" cy="6.5" r=".5" fill="white"/><circle cx="17.5" cy="10.5" r=".5" fill="white"/><circle cx="8.5" cy="7.5" r=".5" fill="white"/><circle cx="6.5" cy="12.5" r=".5" fill="white"/><path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c.93 0 1.65-.75 1.65-1.69 0-.44-.18-.84-.44-1.13-.29-.29-.44-.65-.44-1.13a1.64 1.64 0 0 1 1.67-1.67h2c3.05 0 5.55-2.5 5.55-5.55C21.97 6.01 17.46 2 12 2z"/></g></svg>"#,
    ),
    (
        "Posters",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="18" x="3" y="3" rx="2"/><circle cx="9" cy="9" r="2"/><path d="m21 15-3.09-3.09a2 2 0 0 0-2.82 0L6 21"/></g></svg>"#,
    ),
    (
        "Playback",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="m10 8 6 4-6 4Z"/></g></svg>"#,
    ),
    (
        "Device",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="20" height="14" x="2" y="3" rx="2"/><path d="M8 21h8M12 17v4"/></g></svg>"#,
    ),
    (
        "Shortcuts",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="20" height="16" x="2" y="4" rx="2"/><path d="M6 8h.01M10 8h.01M14 8h.01M18 8h.01M8 12h.01M12 12h.01M16 12h.01M7 16h10"/></g></svg>"#,
    ),
    (
        "Controller",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M6 9h12a4 4 0 0 1 3.8 5.3l-1 3A2.5 2.5 0 0 1 17 18.5l-2.2-1.7h-5.6L7 18.5a2.5 2.5 0 0 1-3.8-1.2l-1-3A4 4 0 0 1 6 9Z" fill="none" stroke="white" stroke-width="2" stroke-linejoin="round"/><path d="M7 12v4m-2-2h4m6-1h.01M18 15h.01" fill="none" stroke="white" stroke-width="2" stroke-linecap="round"/></svg>"#,
    ),
    (
        "Content",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="7" height="7" x="3" y="3" rx="1"/><rect width="7" height="7" x="14" y="3" rx="1"/><rect width="7" height="7" x="14" y="14" rx="1"/><rect width="7" height="7" x="3" y="14" rx="1"/></g></svg>"#,
    ),
    (
        "Add-ons",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19.44 7.85c-.05.32.06.65.29.88l1.57 1.57c.47.47.7 1.09.7 1.7s-.23 1.24-.7 1.71l-1.61 1.61a.98.98 0 0 1-.84.28c-.47-.07-.8-.48-.97-.93a2.5 2.5 0 1 0-3.21 3.22c.44.16.85.5.92.97a.98.98 0 0 1-.27.83l-1.61 1.61a2.4 2.4 0 0 1-3.41 0l-1.57-1.57a1.03 1.03 0 0 0-.88-.29c-.49.07-.84.5-1.02.97a2.5 2.5 0 1 1-3.24-3.24c.47-.18.9-.53.97-1.02a1.03 1.03 0 0 0-.29-.88L2.7 13.7a2.4 2.4 0 0 1 0-3.4L4.23 8.77c.24-.24.58-.35.92-.3.51.08.88.53 1.07 1.01a2.5 2.5 0 1 0 3.26-3.26c-.48-.19-.93-.56-1.01-1.07-.05-.34.06-.68.3-.92L10.3 2.7a2.4 2.4 0 0 1 3.4 0l1.57 1.57c.23.23.56.34.88.29.49-.07.84-.5 1.02-.97a2.5 2.5 0 1 1 3.24 3.24c-.47.18-.9.53-.97 1.02Z"/></g></svg>"#,
    ),
    (
        "Plugins",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22v-5M9 8V2M15 8V2M18 8v5a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V8Z"/></g></svg>"#,
    ),
    (
        "Downloads",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3"/></g></svg>"#,
    ),
    (
        "ChevronDown",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="m6 9 6 6 6-6" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "Home",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .71-1.53l7-6a2 2 0 0 1 2.58 0l7 6A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></g></svg>"#,
    ),
    (
        "Library",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="8" height="18" x="3" y="3" rx="1"/><path d="M7 3v18"/><path d="M20.4 18.9c.2.5-.1 1.1-.6 1.3l-1.9.7c-.5.2-1.1-.1-1.3-.6L11.1 5.1c-.2-.5.1-1.1.6-1.3l1.9-.7c.5-.2 1.1.1 1.3.6Z"/></g></svg>"#,
    ),
    (
        "Discover",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="m16.24 7.76-1.8 5.41a2 2 0 0 1-1.27 1.27L7.76 16.24l1.8-5.41a2 2 0 0 1 1.27-1.27z"/></g></svg>"#,
    ),
    (
        "Movie",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><rect x="3" y="4" width="18" height="16" rx="2" fill="none" stroke="white" stroke-width="2"/><path d="M7 4v4m5-4v4m5-4v4M7 16h10" fill="none" stroke="white" stroke-width="2" stroke-linecap="round"/></svg>"#,
    ),
    (
        "Calendar",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M8 2v4M16 2v4"/><rect width="18" height="18" x="3" y="4" rx="2"/><path d="M3 10h18"/></g></svg>"#,
    ),
    (
        "Settings",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></g></svg>"#,
    ),
    (
        "Profile",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="8" r="5"/><path d="M20 21a8 8 0 0 0-16 0"/></g></svg>"#,
    ),
    (
        "Lock",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect width="18" height="11" x="3" y="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></g></svg>"#,
    ),
    (
        "Edit",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/><path d="m15 5 4 4"/></g></svg>"#,
    ),
    (
        "Delete",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"/><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"/><path d="M10 11v6"/><path d="M14 11v6"/></g></svg>"#,
    ),
    (
        "Plus",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M5 12h14"/><path d="M12 5v14"/></g></svg>"#,
    ),
    (
        "ImagePlus",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M16 5h6"/><path d="M19 2v6"/><path d="M21 11.5V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h7.5"/><path d="m21 15-3.086-3.086a2 2 0 0 0-2.828 0L6 21"/><circle cx="9" cy="9" r="2"/></g></svg>"#,
    ),
    (
        "Refresh",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/><path d="M8 16H3v5"/></g></svg>"#,
    ),
    (
        "Close",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></g></svg>"#,
    ),
    (
        "Info",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="9"/><path d="M12 16v-4"/><path d="M12 8h.01"/></g></svg>"#,
    ),
    (
        "EyeOff",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><g fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10.7 5.1A10 10 0 0 1 12 5c7 0 10 7 10 7a13 13 0 0 1-1.7 2.7"/><path d="M6.6 6.6A13.5 13.5 0 0 0 2 12s3 7 10 7a9.7 9.7 0 0 0 5.4-1.6"/><path d="M9.9 9.9a3 3 0 1 0 4.2 4.2"/><path d="m2 2 20 20"/></g></svg>"#,
    ),
    (
        "PlayFilled",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M7 4.5v15a1 1 0 0 0 1.5.86l12.5-7.5a1 1 0 0 0 0-1.72L8.5 3.64A1 1 0 0 0 7 4.5Z" fill="white"/></svg>"#,
    ),
    (
        "Check",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M20 6 9 17l-5-5" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "CircleCheck",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="9" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="m8.5 12 2.5 2.5 4.5-5" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "Shuffle",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="m18 14 4 4-4 4M18 2l4 4-4 4" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="M2 18h1.97c1.3 0 2.51-.64 3.26-1.7l5.54-8.6A4 4 0 0 1 16.03 6H22M2 6h1.97a4 4 0 0 1 3.36 1.83l.46.7M22 18h-5.97a4 4 0 0 1-3.36-1.83l-.46-.7" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "Ban",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="9" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/><path d="m5.7 5.7 12.6 12.6" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "Heart",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M19 14c1.5-1.5 3-3.2 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.8 0-3 .5-4.5 2-1.5-1.5-2.7-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4 3 5.5l7 7Z" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
    ),
    (
        "HeartFilled",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="M19 14c1.5-1.5 3-3.2 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.8 0-3 .5-4.5 2-1.5-1.5-2.7-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4 3 5.5l7 7Z" fill="white"/></svg>"#,
    ),
    (
        "ArrowLeft",
        r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><path d="m12 19-7-7 7-7M19 12H5" fill="none" stroke="white" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/></svg>"#,
    ),
];

pub const LOGO_SIZE: u32 = 96;

pub const LOGOS: &[(&str, &str)] = &[
    ("anilist", include_str!("../assets/logos/anilist.svg")),
    ("imdb", include_str!("../assets/logos/imdb.svg")),
    ("letterboxd", include_str!("../assets/logos/letterboxd.svg")),
    ("mal", include_str!("../assets/logos/mal.svg")),
    ("mdblist", include_str!("../assets/logos/mdblist.svg")),
    ("metacritic", include_str!("../assets/logos/metacritic.svg")),
    ("rt-popcorn-full", include_str!("../assets/logos/rt-popcorn-full.svg")),
    ("rt-popcorn-spilled", include_str!("../assets/logos/rt-popcorn-spilled.svg")),
    ("rt-tomato-empty", include_str!("../assets/logos/rt-tomato-empty.svg")),
    ("rt-tomato-fresh", include_str!("../assets/logos/rt-tomato-fresh.svg")),
    ("rt-tomato-rotten", include_str!("../assets/logos/rt-tomato-rotten.svg")),
    ("simkl", include_str!("../assets/logos/simkl.svg")),
    ("stremio", include_str!("../assets/logos/stremio.svg")),
    ("tmdb", include_str!("../assets/logos/tmdb.svg")),
    ("trakt", include_str!("../assets/logos/trakt.svg")),
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
    let scale = max_side as f32 / width.max(height);
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
    use super::{ICONS, LOGOS, rasterize_svg};

    #[test]
    fn all_shared_lucide_svg_icons_rasterize() {
        for (name, source) in ICONS.iter().chain(LOGOS) {
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
