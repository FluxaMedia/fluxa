#[cfg(target_os = "android")]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[cfg(target_os = "android")]
mod android;

/// The JNI library is only meaningful on Android. Keeping a host-side symbol
/// makes workspace checks and editor tooling work without an Android linker.
#[cfg(not(target_os = "android"))]
pub const ANDROID_HOST_ONLY: bool = true;
