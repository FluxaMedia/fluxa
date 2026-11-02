use super::*;

const SLOT_PATIENCE: Duration = Duration::from_secs(3);
const FRAMES_KEPT: usize = 48;
const DARK_LUMA: u64 = 8;
const DARK_RETRIES: u32 = 2;
const GRID: f64 = 10.0;

#[derive(Default)]
pub(super) struct Filmstrip {
    wanted: Vec<f64>,
    frames: Vec<(f64, egui::TextureHandle)>,
    pending: Option<(f64, Instant)>,
    failed: Vec<f64>,
    chapters: bool,
    dark: Vec<(f64, u32)>,
}

impl Filmstrip {
    pub(super) fn want_chapters(&mut self, starts: Vec<f64>) {
        if !self.chapters {
            self.frames.clear();
            self.failed.clear();
            self.dark.clear();
            self.pending = None;
        }
        self.chapters = true;
        self.wanted = starts;
    }

    pub(super) fn want(&mut self, slots: Option<Vec<f64>>) {
        if self.chapters {
            self.chapters = false;
            self.dark.clear();
            self.frames.clear();
            self.failed.clear();
            self.pending = None;
        }
        self.wanted = slots.unwrap_or_default();
        if self.wanted.is_empty() {
            self.pending = None;
        }
    }

    pub(super) fn active(&self) -> bool {
        !self.wanted.is_empty()
    }

    pub(super) fn textures(&self) -> Vec<(f64, egui::TextureId, egui::Vec2)> {
        self.frames
            .iter()
            .map(|(slot, texture)| {
                (
                    *slot,
                    texture.id(),
                    egui::Vec2::new(texture.size()[0] as f32, texture.size()[1] as f32),
                )
            })
            .collect()
    }

    pub(super) fn accept(
        &mut self,
        thumbnail: Thumbnail,
        context: &egui::Context,
    ) -> Option<Thumbnail> {
        if !self.active() {
            return Some(thumbnail);
        }
        let Some((slot, _)) = self.pending else {
            return None;
        };
        let spacing = match self.wanted.as_slice() {
            [first, second, ..] if !self.chapters => second - first,
            _ => f64::MAX,
        };
        if (thumbnail.time - slot).abs() > spacing {
            return None;
        }
        let luma = thumbnail
            .rgba
            .chunks_exact(4)
            .map(|p| p[0] as u64)
            .sum::<u64>()
            / (thumbnail.rgba.len() as u64 / 4).max(1);
        if self.chapters && luma < DARK_LUMA && self.darkness(slot) < DARK_RETRIES {
            self.darken(slot);
            self.pending = None;
            return None;
        }
        let image = egui::ColorImage::from_rgba_unmultiplied(thumbnail.size, &thumbnail.rgba);
        let texture =
            context.load_texture("fluxa-seek-filmstrip", image, egui::TextureOptions::LINEAR);
        self.frames.retain(|(at, _)| (at - slot).abs() >= 0.01);
        self.frames.push((slot, texture));
        if self.frames.len() > FRAMES_KEPT {
            let _ = self.frames.remove(0);
        }
        self.pending = None;
        None
    }

    fn darkness(&self, slot: f64) -> u32 {
        self.dark
            .iter()
            .find(|(at, _)| (at - slot).abs() < 0.01)
            .map_or(0, |(_, count)| *count)
    }

    fn darken(&mut self, slot: f64) {
        match self
            .dark
            .iter_mut()
            .find(|(at, _)| (*at - slot).abs() < 0.01)
        {
            Some((_, count)) => *count += 1,
            None => self.dark.push((slot, 1)),
        }
    }

    pub(super) fn next(&mut self) -> Option<f64> {
        if let Some((slot, since)) = self.pending {
            if since.elapsed() < SLOT_PATIENCE {
                return None;
            }
            self.failed.push(slot);
            self.pending = None;
        }
        let slot = self.wanted.iter().copied().find(|slot| {
            let known = |at: &f64| (at - slot).abs() < 0.01;
            !self.frames.iter().any(|(at, _)| known(at)) && !self.failed.iter().any(known)
        })?;
        self.pending = Some((slot, Instant::now()));
        Some(slot + self.darkness(slot) as f64 * GRID)
    }
}
