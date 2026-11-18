use super::*;

pub(crate) struct Slider {
    pub id: Id,
    pub track: Rect,
    pub ratio: f32,
    pub fill: Color32,
    pub overdrive: Option<f32>,
    pub amount: f32,
    pub height: f32,
    pub knob: f32,
    pub bubble: Option<String>,
}

pub(crate) struct SliderOutput {
    pub response: Response,
    pub value: Option<f32>,
}
