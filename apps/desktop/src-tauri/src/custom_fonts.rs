use crate::DesktopState;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::State;

const ALLOWED_EXTENSIONS: [&str; 3] = ["ttf", "otf", "ttc"];

#[derive(Serialize, Clone)]
pub struct CustomFont {
    pub file_name: String,
    pub family: String,
}

pub fn fonts_dir(state: &State<DesktopState>) -> Result<PathBuf, String> {
    let data_dir = state
        .data_dir
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "no writable directory available".to_string())?;
    let dir = data_dir.join("Fonts");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn family_name(path: &Path) -> Option<String> {
    let data = fs::read(path).ok()?;
    let face = ttf_parser::Face::parse(&data, 0).ok()?;
    for id in [
        ttf_parser::name_id::TYPOGRAPHIC_FAMILY,
        ttf_parser::name_id::FAMILY,
    ] {
        if let Some(name) = face
            .names()
            .into_iter()
            .find(|n| n.name_id == id)
            .and_then(|n| n.to_string())
        {
            return Some(name);
        }
    }
    None
}

fn fallback_name(path: &Path, file_name: &str) -> String {
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_name.to_string())
}

fn is_font_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            ALLOWED_EXTENSIONS
                .iter()
                .any(|allowed| e.eq_ignore_ascii_case(allowed))
        })
        .unwrap_or(false)
}

#[tauri::command]
pub fn custom_fonts_list(state: State<DesktopState>) -> Result<Vec<CustomFont>, String> {
    let dir = fonts_dir(&state)?;
    let mut fonts = Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !is_font_file(&path) {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().into_owned();
        let family = family_name(&path).unwrap_or_else(|| fallback_name(&path, &file_name));
        fonts.push(CustomFont { file_name, family });
    }
    fonts.sort_by(|a, b| a.family.to_lowercase().cmp(&b.family.to_lowercase()));
    Ok(fonts)
}

#[tauri::command]
pub fn custom_fonts_add(
    state: State<DesktopState>,
    source_path: String,
) -> Result<CustomFont, String> {
    let dir = fonts_dir(&state)?;
    let source = PathBuf::from(&source_path);
    if !is_font_file(&source) {
        return Err("unsupported font file type".to_string());
    }
    let file_name = source
        .file_name()
        .ok_or_else(|| "invalid file name".to_string())?
        .to_string_lossy()
        .into_owned();
    let dest = dir.join(&file_name);
    fs::copy(&source, &dest).map_err(|e| e.to_string())?;
    let family = family_name(&dest).unwrap_or_else(|| fallback_name(&dest, &file_name));
    Ok(CustomFont { file_name, family })
}

#[tauri::command]
pub fn custom_fonts_remove(state: State<DesktopState>, file_name: String) -> Result<(), String> {
    let dir = fonts_dir(&state)?;
    let safe_name = Path::new(&file_name)
        .file_name()
        .ok_or_else(|| "invalid file name".to_string())?;
    fs::remove_file(dir.join(safe_name)).map_err(|e| e.to_string())?;
    Ok(())
}
