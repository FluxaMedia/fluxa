mod chapters;
mod dv_rewrite;
#[cfg(feature = "native")]
pub mod http_proxy;
mod local_stream;
mod torrent_engine;

#[cfg(feature = "native")]
pub mod companion_server;
#[cfg(any(feature = "native", all(feature = "apple", not(fluxa_ffmpeg_bridge))))]
mod ffmpeg_locator;
#[cfg(all(feature = "apple", fluxa_ffmpeg_bridge))]
mod apple_ffmpeg;
#[cfg(feature = "native")]
pub mod oauth_proxy;
#[cfg(feature = "native")]
pub mod transcode;

pub mod bindings;

#[cfg(feature = "native")]
pub use torrent_engine::{start_torrent_server, stop_torrent_server};
