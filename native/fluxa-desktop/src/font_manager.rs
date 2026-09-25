use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
};

const FONT_EXTENSIONS: &[&str] = &["ttf", "otf"];

#[derive(Default)]
pub struct FontManager {
    paths: Vec<PathBuf>,
}

impl FontManager {
    pub fn from_environment() -> Self {
        let mut manager = Self::default();
        if let Some(paths) = env::var_os("FLUXA_NATIVE_FONT_PATH") {
            for path in env::split_paths(&paths) {
                manager.add_path(path);
            }
        }
        if let Some(path) = env::var_os("FLUXA_NATIVE_FONT_DIR") {
            manager.add_path(PathBuf::from(path));
        }
        manager
    }

    fn add_path(&mut self, path: PathBuf) {
        if path.is_dir() {
            let Ok(entries) = fs::read_dir(&path) else {
                eprintln!(
                    "[fluxa-native] font directory could not be read: {}",
                    path.display()
                );
                return;
            };
            let mut files = entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .filter(|path| is_font_file(path))
                .collect::<Vec<_>>();
            files.sort();
            for file in files {
                self.add_path(file);
            }
        } else if is_font_file(&path) && !self.paths.iter().any(|existing| existing == &path) {
            self.paths.push(path);
        } else if !path.exists() {
            eprintln!(
                "[fluxa-native] configured font path does not exist: {}",
                path.display()
            );
        }
    }

    pub fn load_into(&self, fonts: &mut egui::FontDefinitions) -> usize {
        let mut loaded = Vec::new();
        let mut used_keys = HashSet::new();
        for path in &self.paths {
            let Ok(bytes) = fs::read(path) else {
                eprintln!(
                    "[fluxa-native] user font could not be read: {}",
                    path.display()
                );
                continue;
            };
            if ab_glyph::FontArc::try_from_vec(bytes.clone()).is_err() {
                eprintln!(
                    "[fluxa-native] user font is not a valid TTF/OTF: {}",
                    path.display()
                );
                continue;
            }
            let base_key = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(sanitize_key)
                .filter(|key| !key.is_empty())
                .unwrap_or_else(|| "custom".to_owned());
            let mut key = format!("fluxa-user-{base_key}");
            let mut suffix = 2;
            while !used_keys.insert(key.clone()) {
                key = format!("fluxa-user-{base_key}-{suffix}");
                suffix += 1;
            }
            fonts.font_data.insert(
                key.clone(),
                std::sync::Arc::new(egui::FontData::from_owned(bytes)),
            );
            loaded.push((key, base_key));
        }

        if loaded.is_empty() {
            return 0;
        }
        let requested = env::var("FLUXA_NATIVE_FONT_FAMILY").ok();
        let selected = requested.as_deref().and_then(|requested| {
            loaded.iter().position(|(key, base_key)| {
                key == requested || base_key.eq_ignore_ascii_case(requested)
            })
        });
        let family = fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default();
        let mut order = loaded
            .iter()
            .map(|(key, _)| key.clone())
            .collect::<Vec<_>>();
        if let Some(selected) = selected {
            let key = order.remove(selected);
            order.insert(0, key);
        }
        for key in order.into_iter().rev() {
            family.insert(0, key);
        }
        loaded.len()
    }
}

fn is_font_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            FONT_EXTENSIONS
                .iter()
                .any(|candidate| extension.eq_ignore_ascii_case(candidate))
        })
}

fn sanitize_key(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}
