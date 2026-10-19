use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use egui::epaint::ClippedShape;
use fluxa_ui::{ImageHole, VideoHole};
use serde::Serialize;

static ENABLED: AtomicBool = AtomicBool::new(false);
static LATEST: Mutex<Published> = Mutex::new(Published {
    json: String::new(),
    dirty: false,
});

struct Published {
    json: String,
    dirty: bool,
}

#[derive(Serialize)]
struct DomImage<'a> {
    url: &'a str,
    rect: [f32; 4],
    clip: [f32; 4],
    radius: f32,
    contain: bool,
    anchor: [f32; 2],
    video: bool,
}

pub fn set_enabled(enabled: bool) {
    ENABLED.store(enabled, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

pub fn take() -> Option<String> {
    let mut latest = LATEST.lock().ok()?;
    if !std::mem::take(&mut latest.dirty) {
        return None;
    }
    Some(latest.json.clone())
}

pub(crate) fn lift(shapes: &mut [ClippedShape]) {
    let mut images = Vec::new();
    for clipped in shapes.iter_mut() {
        let egui::Shape::Callback(callback) = &clipped.shape else {
            continue;
        };
        let (rect, clip) = (callback.rect, clipped.clip_rect);
        if callback.callback.is::<VideoHole>() {
            let hole = ImageHole {
                url: String::new(),
                radius: 0.0,
                fit: fluxa_ui::ImageFit::Cover,
                anchor: [0.5, 0.5],
            };
            clipped.shape = egui::Shape::Callback(fluxa_renderer::hole::shape(rect, 0.0));
            images.push((String::new(), rect, clip, hole, true));
            continue;
        }
        let Some(hole) = callback.callback.downcast_ref::<ImageHole>() else {
            continue;
        };
        images.push((hole.url.clone(), rect, clip, hole.clone(), false));
        clipped.shape = if hole.fit == fluxa_ui::ImageFit::Contain {
            egui::Shape::Noop
        } else {
            egui::Shape::Callback(fluxa_renderer::hole::shape(rect, hole.radius))
        };
    }
    let list: Vec<DomImage> = images
        .iter()
        .map(|(url, rect, clip, hole, video)| DomImage {
            url,
            rect: [rect.min.x, rect.min.y, rect.width(), rect.height()],
            clip: [clip.min.x, clip.min.y, clip.width(), clip.height()],
            radius: hole.radius,
            contain: hole.fit == fluxa_ui::ImageFit::Contain,
            anchor: hole.anchor,
            video: *video,
        })
        .collect();
    let Ok(json) = serde_json::to_string(&list) else {
        return;
    };
    if let Ok(mut latest) = LATEST.lock()
        && latest.json != json
    {
        latest.json = json;
        latest.dirty = true;
    }
}
