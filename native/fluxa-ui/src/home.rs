use super::*;

mod category_bar;
mod hero;
mod layout;
mod metrics;
mod rows;
use category_bar::category_bar_space;
pub use layout::*;
pub use metrics::*;
pub use rows::{ExpandedPoster, expanded_poster, previewed_poster, set_episode_details};

pub trait HomeAssets {
    fn background(&self) -> TextureId;
    fn texture(&mut self, url: Option<&str>) -> Option<TextureId>;
    /// Requests artwork at the raster size this component actually needs.
    /// Hosts may map this to a provider-sized URL and/or a bounded decode;
    /// the fallback keeps small test hosts source-compatible.
    fn texture_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        _priority: ArtworkPriority,
    ) -> Option<TextureId> {
        let _ = target_size;
        self.texture(url)
    }
    /// Starts loading artwork without requiring its row/card to be painted yet.
    /// Hosts can use this to prepare the next shelf while the current shelf is
    /// on screen. The default keeps simple/test hosts compatible.
    fn prefetch(&mut self, url: Option<&str>) {
        let _ = self.texture(url);
    }
    fn prefetch_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) {
        let _ = self.texture_for(url, target_size, priority);
    }
    fn prefetch_animated_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        _priority: ArtworkPriority,
    ) {
        let _ = (url, target_size);
    }
    fn texture_size(&self, _url: Option<&str>) -> Option<[u32; 2]> {
        None
    }
    fn dom_images(&self) -> bool {
        false
    }
    fn artwork_failed(&self, _url: Option<&str>, _target_size: [u32; 2]) -> bool {
        false
    }
    fn has_native_emoji(&self) -> bool {
        false
    }
    fn emoji(&mut self, _cluster: &str) -> Option<TextureId> {
        None
    }
    fn artwork_tones(&self, _url: Option<&str>) -> Option<[[u8; 3]; 2]> {
        None
    }
    fn artwork_focus(&self, _url: Option<&str>) -> Option<std::sync::Arc<[f32]>> {
        None
    }
    fn animated_texture_for(
        &mut self,
        _url: Option<&str>,
        _target_size: [u32; 2],
        _priority: ArtworkPriority,
    ) -> Option<AnimatedTexture> {
        None
    }
    fn media_texture_for(
        &mut self,
        url: Option<&str>,
        target_size: [u32; 2],
        priority: ArtworkPriority,
    ) -> Option<AnimatedTexture> {
        let texture = self.texture_for(url, target_size, priority)?;
        Some(AnimatedTexture {
            texture,
            uv: Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            image_size: self.texture_size(url)?,
        })
    }
    fn cached_texture(&self, _url: Option<&str>) -> Option<TextureId> {
        None
    }
    /// Returns a texture rasterized from the shared native SVG icon set.
    /// Hosts upload the same SVG bytes; the UI never substitutes glyphs or
    /// hand-drawn approximations when an icon is requested.
    fn icon(&self, _name: &str) -> Option<TextureId> {
        None
    }
    fn logo(&self, _name: &str) -> Option<(TextureId, Vec2)> {
        None
    }
    fn active_profile_name(&self) -> Option<&str> {
        None
    }
    fn brand_mark(&self) -> Option<TextureId> {
        None
    }
    fn brand_colors(&self) -> Option<[Color32; 2]> {
        None
    }
    fn app_icon(&self, _id: &str) -> Option<TextureId> {
        None
    }
    fn ambient_glow(&self) -> Option<TextureId> {
        None
    }
    fn background_image_size(&self) -> Option<[u32; 2]> {
        None
    }
    fn custom_background_url(&self) -> Option<&str> {
        None
    }
    fn active_profile_avatar_url(&self) -> Option<&str> {
        None
    }
    fn playback_unavailable(&self) -> Option<&str> {
        None
    }
}

#[derive(Clone)]
struct FocusHeroMemo(HomeHero);

pub fn pinned_focus_hero(context: &crate::gfx::Context) -> Option<HomeHero> {
    let pinned = context
        .data(|data| data.get_temp::<bool>(Id::new("fluxa-hero-pinned")))
        .unwrap_or(false);
    pinned
        .then(|| {
            context.data(|data| data.get_temp::<FocusHeroMemo>(Id::new("fluxa-focus-hero-memo")))
        })
        .flatten()
        .map(|memo| memo.0)
}

pub(crate) fn lane(bounds: Rect, padding: crate::flex::Insets) -> Rect {
    crate::flex::stack(Align2::LEFT_TOP, [crate::flex::fill().id(0)])
        .padding(padding)
        .solve(bounds)
        .find(0)
        .unwrap_or(bounds)
}
