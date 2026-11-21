use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ArtworkPriority {
    Hero,
    #[default]
    Visible,
    Prefetch,
}

/// Selects a provider-sized source when the URL is a TMDB image URL. Other
/// providers keep their original URL; their loaders still bound the decoded
/// raster before uploading it to the GPU.
pub fn artwork_request_url(url: &str, target_size: [u32; 2]) -> String {
    if !url.contains("image.tmdb.org") {
        return url.to_owned();
    }
    let marker = "/t/p/";
    let Some(marker_start) = url.find(marker) else {
        return url.to_owned();
    };
    let path_start = marker_start + marker.len();
    let Some((_, path)) = url[path_start..].split_once('/') else {
        return url.to_owned();
    };
    let max_side = target_size[0].max(target_size[1]);
    let variant = if max_side <= 400 {
        "w342"
    } else if max_side <= 650 {
        "w500"
    } else if max_side <= 1000 {
        "w780"
    } else if max_side <= 1280 {
        "w1280"
    } else {
        "original"
    };
    format!("{}{variant}/{path}", &url[..path_start])
}
