use egui::{Context, Id, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollId {
    Screen,
    Row(usize),
    Chips,
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
