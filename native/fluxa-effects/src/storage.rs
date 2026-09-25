#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(not(target_arch = "wasm32"))]
pub use native::{Storage, data_dir};
#[cfg(target_arch = "wasm32")]
pub use web::Storage;

impl Storage {
    pub fn library_key(profile_id: &str) -> String {
        format!("library_{}", sanitize_key(profile_id))
    }

    pub fn prefs_key(profile_id: &str) -> String {
        format!("prefs_{}", sanitize_key(profile_id))
    }

    pub fn addons_key(owner_id: &str) -> String {
        format!("addons_{}", sanitize_key(owner_id))
    }
}

pub fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '_' || character == '-' {
                character
            } else {
                '_'
            }
        })
        .collect()
}
