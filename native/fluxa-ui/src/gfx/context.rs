use super::{
    CursorIcon, FontDefinitions, FontId, Galley, Id, LayerId, Painter, Pos2, RawInput, Rect, Ui,
    Vec2, transparent,
};
use egui::emath::TSTransform;
use egui::epaint::text::LayoutJob;
use egui::layers::ShapeIdx;
use std::any::Any;
use std::sync::Arc;
use std::time::Duration;

pub struct MultiTouch {
    pub zoom_delta: f32,
    pub center_pos: Pos2,
    pub num_touches: usize,
}

#[derive(Clone, Default)]
#[repr(transparent)]
pub struct Context(egui::Context);

transparent!(Context, egui::Context);

#[repr(transparent)]
pub struct Data(egui::util::IdTypeMap);

transparent!(Data, egui::util::IdTypeMap);

impl Data {
    pub fn get_temp<T: 'static + Clone>(&self, id: Id) -> Option<T> {
        self.0.get_temp(id)
    }

    pub fn insert_temp<T: 'static + Any + Clone + Send + Sync>(&mut self, id: Id, value: T) {
        self.0.insert_temp(id, value);
    }

    pub fn remove<T: 'static>(&mut self, id: Id) {
        self.0.remove::<T>(id);
    }

    pub fn remove_temp<T: 'static + Default>(&mut self, id: Id) -> Option<T> {
        self.0.remove_temp(id)
    }

    pub fn get_temp_mut_or_default<T: 'static + Any + Clone + Send + Sync + Default>(
        &mut self,
        id: Id,
    ) -> &mut T {
        self.0.get_temp_mut_or_default(id)
    }
}

#[repr(transparent)]
pub struct Pointer(egui::PointerState);

transparent!(Pointer, egui::PointerState);

impl Pointer {
    pub fn hover_pos(&self) -> Option<Pos2> {
        self.0.hover_pos()
    }

    pub fn latest_pos(&self) -> Option<Pos2> {
        self.0.latest_pos()
    }

    pub fn interact_pos(&self) -> Option<Pos2> {
        self.0.interact_pos()
    }

    pub fn press_origin(&self) -> Option<Pos2> {
        self.0.press_origin()
    }

    pub fn delta(&self) -> Vec2 {
        self.0.delta()
    }

    pub fn press_start_time(&self) -> Option<f64> {
        self.0.press_start_time()
    }

    pub fn primary_down(&self) -> bool {
        self.0.primary_down()
    }

    pub fn primary_pressed(&self) -> bool {
        self.0.primary_pressed()
    }

    pub fn primary_released(&self) -> bool {
        self.0.primary_released()
    }

    pub fn primary_clicked(&self) -> bool {
        self.0.primary_clicked()
    }

    pub fn any_click(&self) -> bool {
        self.0.any_click()
    }
}

#[repr(transparent)]
pub struct InputState(egui::InputState);

transparent!(InputState, egui::InputState);

impl InputState {
    pub fn modifiers(&self) -> super::Modifiers {
        self.0.modifiers
    }

    pub fn time(&self) -> f64 {
        self.0.time
    }

    pub fn stable_dt(&self) -> f32 {
        self.0.stable_dt
    }

    pub fn smooth_scroll_delta(&self) -> Vec2 {
        self.0.smooth_scroll_delta()
    }

    pub fn pointer(&self) -> &Pointer {
        Pointer::wrap(&self.0.pointer)
    }

    pub fn key_pressed(&self, key: super::Key) -> bool {
        self.0.key_pressed(key)
    }
}

#[repr(transparent)]
pub struct Fonts<'a>(egui::epaint::text::FontsView<'a>);

impl<'a> Fonts<'a> {
    pub fn layout_job(&mut self, job: LayoutJob) -> Arc<Galley> {
        self.0.layout_job(job)
    }

    pub fn row_height(&mut self, font: &FontId) -> f32 {
        self.0.row_height(font)
    }

    pub fn layout_no_wrap(
        &mut self,
        text: String,
        font: FontId,
        color: super::Color32,
    ) -> Arc<Galley> {
        self.0.layout_no_wrap(text, font, color)
    }

    pub fn layout(
        &mut self,
        text: String,
        font: FontId,
        color: super::Color32,
        wrap_width: f32,
    ) -> Arc<Galley> {
        self.0.layout(text, font, color, wrap_width)
    }
}

pub struct RunOutput(egui::FullOutput);

impl RunOutput {
    pub fn shape_count(&self) -> usize {
        self.0.shapes.len()
    }

    pub fn drop_without_applying_deltas(self) {
        self.0.drop_without_applying_deltas();
    }
}

impl Context {
    pub fn from_backend(inner: &egui::Context) -> &Self {
        Self::wrap(inner)
    }

    pub fn data<R>(&self, reader: impl FnOnce(&Data) -> R) -> R {
        self.0.data(|data| reader(Data::wrap(data)))
    }

    pub fn data_mut<R>(&self, writer: impl FnOnce(&mut Data) -> R) -> R {
        self.0.data_mut(|data| writer(Data::wrap_mut(data)))
    }

    pub fn input<R>(&self, reader: impl FnOnce(&InputState) -> R) -> R {
        self.0.input(|input| reader(InputState::wrap(input)))
    }

    pub fn fonts_mut<R>(&self, writer: impl FnOnce(&mut Fonts<'_>) -> R) -> R {
        self.0.fonts_mut(|fonts| {
            let fonts: &mut egui::epaint::text::FontsView<'_> = fonts;
            let wrapped = unsafe {
                &mut *(fonts as *mut egui::epaint::text::FontsView<'_> as *mut Fonts<'_>)
            };
            writer(wrapped)
        })
    }

    pub fn pixels_per_point(&self) -> f32 {
        self.0.pixels_per_point()
    }

    pub fn request_repaint(&self) {
        self.0.request_repaint();
    }

    pub fn request_repaint_after(&self, duration: Duration) {
        self.0.request_repaint_after(duration);
    }

    pub fn set_fonts(&self, fonts: FontDefinitions) {
        self.0.set_fonts(fonts);
    }

    pub fn layer_painter(&self, layer: LayerId) -> Painter {
        Painter::from_egui(self.0.layer_painter(layer))
    }

    pub fn cumulative_frame_nr(&self) -> u64 {
        self.0.cumulative_frame_nr()
    }

    pub fn cumulative_pass_nr(&self) -> u64 {
        self.0.cumulative_pass_nr()
    }

    pub fn pointer_hover_pos(&self) -> Option<Pos2> {
        self.0.pointer_hover_pos()
    }

    pub fn pointer_latest_pos(&self) -> Option<Pos2> {
        self.0.pointer_latest_pos()
    }

    pub fn content_rect(&self) -> Rect {
        self.0.content_rect()
    }

    pub fn set_cursor_icon(&self, icon: CursorIcon) {
        self.0.set_cursor_icon(icon);
    }

    pub fn move_to_top(&self, layer: LayerId) {
        self.0.move_to_top(layer);
    }

    pub fn set_transform_layer(&self, layer: LayerId, transform: TSTransform) {
        self.0.set_transform_layer(layer, transform);
    }

    pub fn layer_cursor(&self, layer: LayerId) -> ShapeIdx {
        self.0
            .graphics_mut(|graphics| graphics.entry(layer).next_idx())
    }

    pub fn transform_layer_since(&self, layer: LayerId, from: ShapeIdx, transform: TSTransform) {
        self.0.graphics_mut(|graphics| {
            let list = graphics.entry(layer);
            let end = list.next_idx();
            list.transform_range(from, end, transform);
        });
    }

    pub fn fade_and_shift_layers(&self, keep: &[Id], alpha: f32, shift: TSTransform) {
        let layers: Vec<LayerId> = self.0.memory(|memory| memory.layer_ids().collect());
        self.0.graphics_mut(|graphics| {
            for layer in layers {
                if layer.order == egui::Order::Background || keep.contains(&layer.id) {
                    continue;
                }
                let Some(list) = graphics.get_mut(layer) else {
                    continue;
                };
                let end = list.next_idx();
                list.transform_range(ShapeIdx(0), end, shift);
                for index in 0..end.0 {
                    list.mutate_shape(ShapeIdx(index), |clipped| {
                        egui::epaint::shape_transform::adjust_colors(
                            &mut clipped.shape,
                            move |color| *color = color.gamma_multiply(alpha),
                        );
                    });
                }
            }
        });
    }

    pub fn layer_id_at(&self, pos: Pos2) -> Option<LayerId> {
        self.0.layer_id_at(pos)
    }

    pub fn multi_touch(&self) -> Option<MultiTouch> {
        self.0.multi_touch().map(|touch| MultiTouch {
            zoom_delta: touch.zoom_delta,
            center_pos: touch.center_pos,
            num_touches: touch.num_touches,
        })
    }

    pub fn texture_size(&self, texture: super::TextureId) -> Option<[usize; 2]> {
        self.0
            .tex_manager()
            .read()
            .meta(texture)
            .map(|meta| meta.size)
    }

    pub fn keep_focus_on_click(&self, keep: bool) {
        self.0.options_mut(|options| {
            options.input_options.surrender_focus_on = if keep {
                egui::SurrenderFocusOn::Never
            } else {
                egui::SurrenderFocusOn::Clicks
            }
        });
    }

    pub fn has_focus(&self, id: Id) -> bool {
        self.0.memory(|memory| memory.has_focus(id))
    }

    pub fn request_focus(&self, id: Id) {
        self.0.memory_mut(|memory| memory.request_focus(id));
    }

    pub fn run_ui(&self, input: RawInput, mut run: impl FnMut(&mut Ui)) -> RunOutput {
        RunOutput(self.0.run_ui(input, |ui| run(Ui::wrap_mut(ui))))
    }

    pub fn run_pass(&self, input: RawInput, mut run: impl FnMut(&Context)) -> RunOutput {
        RunOutput(self.0.run_pass(input, |ctx| run(Context::wrap(ctx))))
    }
}
