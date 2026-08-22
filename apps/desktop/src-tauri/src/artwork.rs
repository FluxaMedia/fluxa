use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

static ARTWORK_CACHE: OnceLock<Mutex<HashMap<String, Vec<u8>>>> = OnceLock::new();
pub fn artwork_cache() -> &'static Mutex<HashMap<String, Vec<u8>>> {
    ARTWORK_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
static ARTWORK_BG_DECODED: OnceLock<Mutex<HashMap<String, (Vec<u8>, i32, i32)>>> = OnceLock::new();
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub fn artwork_bg_decoded() -> &'static Mutex<HashMap<String, (Vec<u8>, i32, i32)>> {
    ARTWORK_BG_DECODED.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
static ARTWORK_LOGO_DECODED: OnceLock<Mutex<HashMap<String, (Vec<u8>, i32, i32)>>> =
    OnceLock::new();
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub fn artwork_logo_decoded() -> &'static Mutex<HashMap<String, (Vec<u8>, i32, i32)>> {
    ARTWORK_LOGO_DECODED.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn normalize_url(url: &str) -> String {
    if let Some(rest) = url.trim().strip_prefix("//") {
        format!("https:{rest}")
    } else {
        url.trim().to_string()
    }
}

pub async fn fetch_player_artwork_bytes(url: Option<&str>) -> Option<Vec<u8>> {
    let url = url?.trim();
    if url.is_empty() {
        return None;
    }
    let normalized = if let Some(protocol_relative) = url.strip_prefix("//") {
        format!("https:{protocol_relative}")
    } else {
        url.to_string()
    };

    if let Ok(cache) = artwork_cache().lock() {
        if let Some(cached) = cache.get(&normalized) {
            return Some(cached.clone());
        }
    }

    let client = crate::net_guard::vetted_client(&normalized, std::time::Duration::from_secs(10))
        .await
        .ok()?;
    let response = client.get(&normalized).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let bytes = response.bytes().await.ok()?;
    if bytes.len() > 8 * 1024 * 1024 {
        return None;
    }
    let vec = bytes.to_vec();

    if let Ok(mut cache) = artwork_cache().lock() {
        if cache.len() >= 40 {
            if let Some(key) = cache.keys().next().cloned() {
                cache.remove(&key);
            }
        }
        cache.insert(normalized, vec.clone());
    }

    Some(vec)
}

pub async fn fetch_player_artwork_bytes_owned(url: Option<String>) -> Option<Vec<u8>> {
    fetch_player_artwork_bytes(url.as_deref()).await
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub fn scale_artwork_cover(
    bytes: Vec<u8>,
    target_w: u32,
    target_h: u32,
) -> Option<(Vec<u8>, i32, i32)> {
    let img = image::load_from_memory(&bytes).ok()?;
    let filled = img.resize_to_fill(target_w, target_h, image::imageops::FilterType::Triangle);
    let rgba = filled.to_rgba8();
    Some((rgba.into_raw(), target_w as i32, target_h as i32))
}

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
pub fn scale_artwork_fit(bytes: Vec<u8>, max_w: u32, max_h: u32) -> Option<(Vec<u8>, i32, i32)> {
    let img = image::load_from_memory(&bytes).ok()?;
    let resized = img.resize(max_w, max_h, image::imageops::FilterType::Triangle);
    let (rw, rh) = (resized.width(), resized.height());
    let rgba = resized.to_rgba8();
    Some((rgba.into_raw(), rw as i32, rh as i32))
}
