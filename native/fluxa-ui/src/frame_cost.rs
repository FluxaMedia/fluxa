use super::*;
use std::time::{Duration, Instant};

struct NoAssets;

impl HomeAssets for NoAssets {
    fn background(&self) -> TextureId {
        TextureId::Managed(0)
    }

    fn texture(&mut self, _url: Option<&str>) -> Option<TextureId> {
        None
    }
}

struct Cost {
    first: f32,
    mean: f32,
    p95: f32,
    max: f32,
    repainting: usize,
    last_repaints: bool,
    frames: usize,
}








