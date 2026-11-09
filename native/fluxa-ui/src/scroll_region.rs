use crate::gfx::{Context, Id, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollId {
    Screen,
    Row(usize),
    Chips,
    Strip(&'static str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollAxis {
    Vertical,
    Horizontal,
}

#[derive(Clone, Copy, Debug)]
pub struct ScrollRegion {
    pub id: ScrollId,
    pub axis: ScrollAxis,
    pub rect: Rect,
    pub max: f32,
}

const REGIONS: &str = "fluxa-scroll-regions";

pub fn report_scroll_region(context: &Context, region: ScrollRegion) {
    context.data_mut(|data| {
        let regions = data.get_temp_mut_or_default::<Vec<ScrollRegion>>(Id::new(REGIONS));
        regions.retain(|known| (known.id, known.axis) != (region.id, region.axis));
        regions.push(region);
    });
}

pub fn take_scroll_regions(context: &Context) -> Vec<ScrollRegion> {
    context.data_mut(|data| {
        data.get_temp_mut_or_default::<Vec<ScrollRegion>>(Id::new(REGIONS))
            .drain(..)
            .collect()
    })
}

const STRIP_OFFSETS: &str = "fluxa-scroll-strip-offsets";

pub fn set_strip_offset(context: &Context, name: &'static str, value: f32) {
    context.data_mut(|data| {
        data.get_temp_mut_or_default::<Vec<(&'static str, f32)>>(Id::new(STRIP_OFFSETS))
            .retain(|(known, _)| *known != name);
        data.get_temp_mut_or_default::<Vec<(&'static str, f32)>>(Id::new(STRIP_OFFSETS))
            .push((name, value));
    });
}

pub(crate) fn strip_offset(context: &Context, name: &'static str) -> f32 {
    context
        .data(|data| data.get_temp::<Vec<(&'static str, f32)>>(Id::new(STRIP_OFFSETS)))
        .and_then(|offsets| {
            offsets
                .iter()
                .find(|(known, _)| *known == name)
                .map(|(_, value)| *value)
        })
        .unwrap_or(0.0)
}
