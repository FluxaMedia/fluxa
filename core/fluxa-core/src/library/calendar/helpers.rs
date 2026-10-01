
pub(super) fn usable_artwork(url: Option<&str>) -> Option<&str> {
    url.filter(|value| {
        let normalized = value.trim().to_ascii_lowercase();
        !normalized.is_empty()
            && normalized != "null"
            && !normalized.contains("default-poster")
            && !normalized.contains("placeholder")
            && !normalized.contains("no-image")
            && !normalized.contains("no_image")
    })
}
