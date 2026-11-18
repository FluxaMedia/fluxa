use super::*;
use crate::player::format_time;

pub(crate) struct FilmstripLayout {
    pub count: usize,
    pub width: f32,
    pub gap: f32,
}

pub(crate) struct FilmstripOutput {
    pub slots: Vec<f64>,
    pub seek: Option<f64>,
}
