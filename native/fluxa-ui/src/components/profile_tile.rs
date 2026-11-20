use super::*;
use crate::IconSize;
use crate::Space;
use crate::dp::Dp;
use crate::flex::{self, Insets};

pub(crate) struct ProfileTile<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub avatar_url: Option<&'a str>,
    pub locked: bool,
    pub primary_label: Option<String>,
    pub cell: Vec2,
    pub radius: f32,
    pub manage: bool,
    pub reveal: f32,
    pub tips: (&'a str, &'a str),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TileAction {
    Edit,
    Delete,
}

struct TileSlots {
    card: Rect,
    avatar: Rect,
    name: Rect,
    badge: Option<Rect>,
    actions: Rect,
    hit: Rect,
}
