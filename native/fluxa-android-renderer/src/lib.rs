#![cfg_attr(not(target_os = "android"), allow(dead_code))]

#[cfg(target_os = "android")]
mod android;
#[cfg(target_os = "android")]
mod video;

/// The JNI library is only meaningful on Android. Keeping a host-side symbol
/// makes workspace checks and editor tooling work without an Android linker.
#[cfg(not(target_os = "android"))]
pub const ANDROID_HOST_ONLY: bool = true;
