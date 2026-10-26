use super::helpers::timestamp;
use serde_json::Value;

pub(super) fn air_time(item: &Value) -> i64 {
    let value = timestamp(item, "nextEpisodeAirDate").max(timestamp(item, "newEpisodeReleasedAt"));
    if value > 0 { value } else { i64::MAX }
}
