use base64::{Engine, engine::general_purpose::STANDARD};
use ed25519_dalek::{Signature, VerifyingKey};
use serde::Deserialize;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    pub version: String,
    pub notes: String,
    pub asset: Asset,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Asset {
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UpdateError {
    BadKey,
    BadSignature,
    Malformed,
    BadVersion(String),
}

impl fmt::Display for UpdateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadKey => f.write_str("update public key is invalid"),
            Self::BadSignature => f.write_str("update manifest signature does not match"),
            Self::Malformed => f.write_str("update manifest is malformed"),
            Self::BadVersion(version) => write!(f, "unsupported version string {version}"),
        }
    }
}

impl std::error::Error for UpdateError {}

#[derive(Deserialize)]
struct Manifest {
    version: String,
    #[serde(default)]
    notes: String,
    assets: HashMap<String, Asset>,
}

pub fn platform_key(os: &str, arch: &str) -> String {
    let arch = match arch {
        "aarch64" => "arm64",
        "amd64" => "x86_64",
        other => other,
    };
    format!("{os}-{arch}")
}

pub fn check(
    manifest: &[u8],
    signature_b64: &str,
    public_key_b64: &str,
    current: &str,
    platform: &str,
) -> Result<Option<Release>, UpdateError> {
    verify_signature(manifest, signature_b64, public_key_b64)?;
    let manifest: Manifest = serde_json::from_slice(manifest).map_err(|_| UpdateError::Malformed)?;
    if compare_versions(&manifest.version, current)? != Ordering::Greater {
        return Ok(None);
    }
    let Some(asset) = manifest.assets.get(platform) else {
        return Ok(None);
    };
    if !asset.url.starts_with("https://") || asset.sha256.len() != 64 {
        return Err(UpdateError::Malformed);
    }
    Ok(Some(Release {
        version: manifest.version.trim_start_matches('v').to_owned(),
        notes: manifest.notes,
        asset: asset.clone(),
    }))
}

pub fn sha256_matches(actual_hex: &str, asset: &Asset) -> bool {
    actual_hex.eq_ignore_ascii_case(&asset.sha256)
}

fn verify_signature(manifest: &[u8], signature_b64: &str, public_key_b64: &str) -> Result<(), UpdateError> {
    let key_bytes: [u8; 32] = STANDARD
        .decode(public_key_b64.trim())
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(UpdateError::BadKey)?;
    let key = VerifyingKey::from_bytes(&key_bytes).map_err(|_| UpdateError::BadKey)?;
    let signature_bytes: [u8; 64] = STANDARD
        .decode(signature_b64.trim())
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or(UpdateError::BadSignature)?;
    key.verify_strict(manifest, &Signature::from_bytes(&signature_bytes))
        .map_err(|_| UpdateError::BadSignature)
}

pub fn compare_versions(left: &str, right: &str) -> Result<Ordering, UpdateError> {
    let (left_core, left_pre) = split_version(left)?;
    let (right_core, right_pre) = split_version(right)?;
    Ok(left_core.cmp(&right_core).then_with(|| match (left_pre, right_pre) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(a), Some(b)) => a.cmp(b),
    }))
}

fn split_version(version: &str) -> Result<([u64; 3], Option<&str>), UpdateError> {
    let bad = || UpdateError::BadVersion(version.to_owned());
    let trimmed = version.trim().trim_start_matches('v');
    let (core, pre) = match trimmed.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (trimmed, None),
    };
    let mut parts = core.split('.');
    let mut numbers = [0u64; 3];
    for slot in &mut numbers {
        *slot = parts.next().ok_or_else(bad)?.parse().map_err(|_| bad())?;
    }
    if parts.next().is_some() {
        return Err(bad());
    }
    Ok((numbers, pre))
}

#[cfg(test)]
mod tests;
