use std::collections::HashMap;

use egui::{Color32, Painter, Pos2, Rect, TextureId, Vec2};
use fluxa_renderer::egui_wgpu_backend::EguiWgpuBackend;
pub use fluxa_renderer::svg_icons::rasterize_svg;
use fluxa_renderer::svg_icons::{ICON_SIZE, ICONS};
use wgpu;

pub struct SvgIconRegistry {
    textures: HashMap<&'static str, (wgpu::Texture, TextureId)>,
}

impl SvgIconRegistry {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, renderer: &mut EguiWgpuBackend) -> Self {
        let mut textures = HashMap::new();
        for (name, source) in ICONS {
            let Some(image) = rasterize(source) else {
                eprintln!("[fluxa-native] failed to rasterize SVG icon {name}");
                continue;
            };
            let size = wgpu::Extent3d {
                width: ICON_SIZE,
                height: ICON_SIZE,
                depth_or_array_layers: 1,
            };
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("fluxa-svg-icon"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[wgpu::TextureFormat::Rgba8Unorm],
            });
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                image.as_raw(),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(4 * ICON_SIZE),
                    rows_per_image: Some(ICON_SIZE),
                },
                size,
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let id = renderer.register_native_texture(device, &view, wgpu::FilterMode::Linear);
            textures.insert(*name, (texture, id));
        }
        Self { textures }
    }

    pub fn texture(&self, name: &str) -> Option<TextureId> {
        self.textures.get(name).map(|(_, id)| *id)
    }

    pub fn paint(&self, painter: &Painter, center: Pos2, label: &str, color: Color32) {
        let Some((_, id)) = self.textures.get(label) else {
            return;
        };
        let rect = Rect::from_center_size(center, Vec2::splat(20.0));
        painter.image(
            *id,
            rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            color,
        );
    }
}

fn rasterize(source: &str) -> Option<image::RgbaImage> {
    let image = rasterize_svg(source.as_bytes(), ICON_SIZE).ok()?;
    Some(image::imageops::resize(
        &image,
        ICON_SIZE,
        ICON_SIZE,
        image::imageops::FilterType::Lanczos3,
    ))
}

#[cfg(test)]
mod tests {
    use super::rasterize;

    #[test]
    fn bundled_icons_rasterize_to_rgba() {
        let image = rasterize(r#"<svg viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg"><circle cx="12" cy="12" r="8" fill="red"/></svg>"#).expect("rasterize SVG");
        assert_eq!(image.dimensions(), (32, 32));
        assert!(image.pixels().any(|pixel| pixel[3] > 0));
    }
}
