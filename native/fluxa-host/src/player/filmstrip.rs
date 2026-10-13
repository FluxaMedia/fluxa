use super::*;

const SLOT_PATIENCE: Duration = Duration::from_secs(3);
const FRAMES_KEPT: usize = 48;

#[derive(Default)]
pub(super) struct Filmstrip {
    wanted: Vec<f64>,
    frames: Vec<(f64, egui::TextureHandle)>,
    pending: Option<(f64, Instant)>,
    failed: Vec<f64>,
}

impl Filmstrip {
    pub(super) fn want(&mut self, slots: Option<Vec<f64>>) {
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
            [first, second, ..] => second - first,
            _ => f64::MAX,
        };
        if (thumbnail.time - slot).abs() > spacing {
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
        Some(slot)
    }
}
