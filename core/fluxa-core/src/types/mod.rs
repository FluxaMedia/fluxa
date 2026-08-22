pub mod addon;
pub mod plugin;
pub mod profile;
pub mod resource;

pub use addon::{AddonManifest, ResourceRef};
pub use plugin::{PluginManifest, PluginManifestScraper, PluginStreamResult, PluginSubtitleResult};
pub use profile::Profile;
pub use resource::{MetaItem, Stream, Video};
