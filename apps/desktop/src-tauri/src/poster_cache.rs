use crate::DesktopState;
use std::fs;
use std::path::PathBuf;
use tauri::State;

const MAX_CACHED_POSTERS: usize = 800;

fn hash_url(url: &str) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in url.as_bytes() {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn guess_extension(url: &str) -> String {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let ext = path
        .rsplit('.')
        .next()
        .unwrap_or("jpg")
        .to_ascii_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "avif" => ext,
        _ => "jpg".to_string(),
    }
}

fn poster_cache_dir(state: &State<DesktopState>) -> Result<PathBuf, String> {
    let data_dir = state
        .data_dir
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "no writable directory available".to_string())?;
    let dir = data_dir.join("PosterCache");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn evict_oldest(dir: &PathBuf) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut files: Vec<_> = entries
        .flatten()
        .filter_map(|e| {
            let modified = e.metadata().ok()?.modified().ok()?;
            Some((e.path(), modified))
        })
        .collect();
    if files.len() <= MAX_CACHED_POSTERS {
        return;
    }
    files.sort_by_key(|(_, modified)| *modified);
    let excess = files.len() - MAX_CACHED_POSTERS;
    for (path, _) in files.into_iter().take(excess) {
        let _ = fs::remove_file(path);
    }
}

#[tauri::command]
pub async fn cache_poster_image(
    state: State<'_, DesktopState>,
    url: String,
) -> Result<String, String> {
    let normalized = crate::artwork::normalize_url(&url);
    let dir = poster_cache_dir(&state)?;
    let file_name = format!(
        "{:016x}.{}",
        hash_url(&normalized),
        guess_extension(&normalized)
    );
    let dest = dir.join(&file_name);

    if dest.exists() {
        return dest
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "invalid cache path".to_string());
    }

    let client =
        crate::net_guard::vetted_client(&normalized, std::time::Duration::from_secs(10)).await?;
    let response = client
        .get(&normalized)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    if !response.status().is_success() {
        return Err(format!("http {}", response.status()));
    }
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("image too large".to_string());
    }

    let tmp = dir.join(format!("{file_name}.tmp"));
    fs::write(&tmp, &bytes).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &dest).map_err(|e| e.to_string())?;
    evict_oldest(&dir);

    dest.to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "invalid cache path".to_string())
}
