use super::*;

pub(super) static LIQUID_GLASS: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);

pub fn set_liquid_glass(enabled: bool) {
    LIQUID_GLASS.store(enabled, std::sync::atomic::Ordering::Relaxed);
}

pub(crate) fn liquid_glass() -> bool {
    LIQUID_GLASS.load(std::sync::atomic::Ordering::Relaxed)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImageFit {
    Cover,
    Contain,
}

#[derive(Clone, Debug)]
pub struct ImageHole {
    pub url: String,
    pub radius: f32,
    pub fit: ImageFit,
    pub anchor: [f32; 2],
}

pub const DOM_TRAILER: egui::TextureId = egui::TextureId::User(u64::MAX);

#[derive(Clone, Debug)]
pub struct VideoHole;

pub(crate) fn dom_video(painter: &Painter, rect: Rect) {
    painter.add(egui::Shape::Callback(egui::PaintCallback {
        rect,
        callback: std::sync::Arc::new(VideoHole),
    }));
}

pub(crate) fn dom_image(painter: &Painter, rect: Rect, radius: f32, url: &str) {
    dom_image_fit(painter, rect, radius, url, ImageFit::Cover, [0.5, 0.5]);
}

pub(crate) fn dom_image_fit(
    painter: &Painter,
    rect: Rect,
    radius: f32,
    url: &str,
    fit: ImageFit,
    anchor: [f32; 2],
) {
    painter.add(egui::Shape::Callback(egui::PaintCallback {
        rect,
        callback: std::sync::Arc::new(ImageHole {
            url: url.to_owned(),
            radius,
            fit,
            anchor,
        }),
    }));
}

pub(crate) fn failed_dom_image(
    painter: &Painter,
    assets: &impl HomeAssets,
    rect: Rect,
    radius: f32,
    url: Option<&str>,
    target_size: [u32; 2],
    fit: ImageFit,
    anchor: [f32; 2],
) -> bool {
    let Some(url) = url.filter(|url| !url.trim().is_empty()) else {
        return false;
    };
    if !assets.dom_images() || !assets.artwork_failed(Some(url), target_size) {
        return false;
    }
    dom_image_fit(painter, rect, radius, url, fit, anchor);
    true
}

pub(crate) fn failed_logo_slot(
    ui: &mut Ui,
    assets: &impl HomeAssets,
    url: Option<&str>,
    bounds: Vec2,
    anchor: [f32; 2],
) -> bool {
    let target = artwork_target_size(bounds, ui.ctx().pixels_per_point());
    let failed = url.is_some_and(|url| {
        !url.trim().is_empty() && assets.dom_images() && assets.artwork_failed(Some(url), target)
    });
    if !failed {
        return false;
    }
    let (rect, _) = ui.allocate_exact_size(bounds, Sense::hover());
    failed_dom_image(
        ui.painter(),
        assets,
        rect,
        0.0,
        url,
        target,
        ImageFit::Contain,
        anchor,
    )
}

pub(crate) fn dom_avatar(
    painter: &Painter,
    assets: &impl HomeAssets,
    center: Pos2,
    radius: f32,
    url: Option<&str>,
) -> bool {
    let Some(url) = url.filter(|url| !url.trim().is_empty() && assets.dom_images()) else {
        return false;
    };
    let rect = Rect::from_center_size(center, Vec2::splat(radius * 2.0));
    dom_image(painter, rect, radius, url);
    true
}

#[derive(Clone, Copy, Debug)]
pub struct Glass {
    pub radius: f32,
    pub tint: Color32,
    pub refraction: f32,
    pub bevel: f32,
    pub rim: f32,
}

pub(crate) fn glass_shape(rect: Rect, radius: f32, fill: Color32) -> egui::Shape {
    if !liquid_glass() {
        return egui::Shape::rect_filled(rect, radius, fill);
    }
    let tint = if fill.a() > 200 {
        fill.gamma_multiply(0.3)
    } else {
        fill
    };
    let radius = radius.min(rect.height() * 0.5).min(rect.width() * 0.5);
    egui::Shape::Callback(egui::PaintCallback {
        rect,
        callback: std::sync::Arc::new(Glass {
            radius,
            tint,
            refraction: crate::metrics::active().glass_refraction.of(rect.height()),
            bevel: crate::metrics::active().glass_bevel.of(radius),
            rim: 1.0,
        }),
    })
}

pub(crate) fn glass(painter: &Painter, rect: Rect, radius: f32, fill: Color32) {
    painter.add(glass_shape(rect, radius, fill));
}
