use super::*;
use crate::WatchMemberView;
use crate::flex;

const MAX_SEATS: usize = 6;

pub(crate) fn seat_count(members: usize, capacity: u32) -> usize {
    (capacity as usize).max(members).min(MAX_SEATS)
}
