use crate::core_error::{CoreError, LogAndDiscard};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceResourceBudgetRequest {
    total_ram_mb: i64,
    #[serde(default)]
    heap_max_mb: i64,
    #[serde(default)]
    is_low_ram_device: bool,
    #[serde(default)]
    is_television: bool,
    #[serde(default)]
    logical_cores: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum DeviceTier {
    Low,
    Mid,
    High,
    Ultra,
}

impl DeviceTier {
    fn classify(total_ram_mb: i64, is_low_ram_device: bool) -> Self {
        if is_low_ram_device || total_ram_mb < 2048 {
            DeviceTier::Low
        } else if total_ram_mb < 4096 {
            DeviceTier::Mid
        } else if total_ram_mb < 8192 {
            DeviceTier::High
        } else {
            DeviceTier::Ultra
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            DeviceTier::Low => "low",
            DeviceTier::Mid => "mid",
            DeviceTier::High => "high",
            DeviceTier::Ultra => "ultra",
        }
    }
}

fn lerp_mb(
    total_ram_mb: i64,
    lo_ram: f64,
    lo_val: f64,
    hi_ram: f64,
    hi_val: f64,
    floor: f64,
    ceil: f64,
) -> i64 {
    let t = ((total_ram_mb as f64 - lo_ram) / (hi_ram - lo_ram)).clamp(0.0, 1.0);
    (lo_val + t * (hi_val - lo_val)).clamp(floor, ceil) as i64
}

pub(crate) fn device_resource_budget_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<DeviceResourceBudgetRequest>(request_json)
        .map_err(|e| CoreError::BadInput {
            context: "device_resource_budget_json",
            detail: e.to_string(),
        })
        .log_discard()?;

    let tier = DeviceTier::classify(request.total_ram_mb, request.is_low_ram_device);
    let tv = request.is_television;

    let image_cache_memory_percent = match (tier, tv) {
        (DeviceTier::Low, true) => 0.10,
        (DeviceTier::Low, false) => 0.12,
        (DeviceTier::Mid, true) => 0.15,
        (DeviceTier::Mid, false) => 0.18,
        (DeviceTier::High, true) => 0.20,
        (DeviceTier::High, false) => 0.25,
        (DeviceTier::Ultra, true) => 0.22,
        (DeviceTier::Ultra, false) => 0.30,
    };

    let image_decode_concurrency: i64 = match (tier, tv) {
        (DeviceTier::Low, _) => 2,
        (DeviceTier::Mid, true) => 2,
        (DeviceTier::Mid, false) => 3,
        (DeviceTier::High, true) => 3,
        (DeviceTier::High, false) => 4,
        (DeviceTier::Ultra, _) => 4,
    };

    let ram = request.total_ram_mb;
    let player_buffer_cache_mb = lerp_mb(ram, 2048.0, 100.0, 8192.0, 300.0, 32.0, 400.0);
    let torrent_cache_mb = lerp_mb(ram, 2048.0, 64.0, 8192.0, 256.0, 32.0, 400.0);
    let subtitle_glyph_cache_mb = lerp_mb(ram, 2048.0, 12.0, 8192.0, 32.0, 6.0, 48.0);

    let heap_max_mb = request.heap_max_mb.max(1);
    let heap_bound_mb = if request.is_low_ram_device {
        (heap_max_mb / 8).clamp(16, 48)
    } else {
        (heap_max_mb / 4).clamp(32, 150)
    };
    let player_target_buffer_bytes = player_buffer_cache_mb.min(heap_bound_mb) * 1_000_000;
    let ui_reserve_mb = ((heap_max_mb as f64) * 0.3).clamp(64.0, 300.0) as i64;
    let cores = request.logical_cores.max(1);
    let torrent_worker_threads = if tier == DeviceTier::Low {
        cores.clamp(2, 3)
    } else {
        cores.clamp(2, 8)
    };
    let stream_reader_buffer_bytes = match tier {
        DeviceTier::Low => 64 * 1024,
        DeviceTier::Mid => 128 * 1024,
        DeviceTier::High | DeviceTier::Ultra => 256 * 1024,
    };
    let torrent_preload_mb = match tier {
        DeviceTier::Low => 4,
        DeviceTier::Mid => 8,
        DeviceTier::High => 16,
        DeviceTier::Ultra => 24,
    };
    let plugin_memory_limit_mb = match tier {
        DeviceTier::Low => 64,
        DeviceTier::Mid => 128,
        DeviceTier::High => 256,
        DeviceTier::Ultra => 384,
    };

    serde_json::to_string(&json!({
        "tier": tier.as_str(),
        "imageCacheMemoryPercent": image_cache_memory_percent,
        "imageCrossfadeEnabled": !tv,
        "imagePrecisionInexact": tv || tier == DeviceTier::Low,
        "imageDecodeConcurrency": image_decode_concurrency,
        "playerBufferCacheMb": player_buffer_cache_mb,
        "playerTargetBufferBytes": player_target_buffer_bytes,
        "torrentCacheMb": torrent_cache_mb,
        "subtitleGlyphCacheBytes": subtitle_glyph_cache_mb * 1_000_000,
        "uiReserveMb": ui_reserve_mb,
        "torrentWorkerThreads": torrent_worker_threads,
        "streamReaderBufferBytes": stream_reader_buffer_bytes,
        "torrentPreloadMb": torrent_preload_mb,
        "pluginMemoryLimitMb": plugin_memory_limit_mb
    }))
    .ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn budget(json: &str) -> Value {
        serde_json::from_str(&device_resource_budget_json(json).unwrap()).unwrap()
    }

    #[test]
    fn low_ram_tv_gets_conservative_image_and_decode_settings() {
        let b = budget(
            r#"{"totalRamMb":1536,"heapMaxMb":256,"isLowRamDevice":true,"isTelevision":true}"#,
        );
        assert_eq!(b["tier"], "low");
        assert_eq!(b["imageCacheMemoryPercent"], 0.10);
        assert_eq!(b["imageCrossfadeEnabled"], false);
        assert_eq!(b["imagePrecisionInexact"], true);
        assert_eq!(b["imageDecodeConcurrency"], 2);
        assert_eq!(b["streamReaderBufferBytes"], 65_536);
        assert_eq!(b["torrentPreloadMb"], 4);
        assert_eq!(b["pluginMemoryLimitMb"], 64);
    }

    #[test]
    fn high_ram_phone_gets_larger_percent_and_crossfade_on() {
        let b = budget(
            r#"{"totalRamMb":6144,"heapMaxMb":512,"isLowRamDevice":false,"isTelevision":false}"#,
        );
        assert_eq!(b["tier"], "high");
        assert_eq!(b["imageCacheMemoryPercent"], 0.25);
        assert_eq!(b["imageCrossfadeEnabled"], true);
        assert_eq!(b["imagePrecisionInexact"], false);
        assert_eq!(b["streamReaderBufferBytes"], 262_144);
        assert_eq!(b["torrentWorkerThreads"], 2);
    }

    #[test]
    fn budgets_scale_between_2gb_and_8gb_anchors() {
        let low = budget(
            r#"{"totalRamMb":2048,"heapMaxMb":256,"isLowRamDevice":false,"isTelevision":false}"#,
        );
        let high = budget(
            r#"{"totalRamMb":8192,"heapMaxMb":512,"isLowRamDevice":false,"isTelevision":false}"#,
        );
        assert_eq!(low["playerBufferCacheMb"], 100);
        assert_eq!(low["torrentCacheMb"], 64);
        assert_eq!(low["subtitleGlyphCacheBytes"], 12_000_000);
        assert_eq!(high["playerBufferCacheMb"], 300);
        assert_eq!(high["torrentCacheMb"], 256);
        assert_eq!(high["subtitleGlyphCacheBytes"], 32_000_000);
    }

    #[test]
    fn player_target_buffer_bytes_is_bounded_by_heap() {
        let b = budget(
            r#"{"totalRamMb":8192,"heapMaxMb":64,"isLowRamDevice":true,"isTelevision":true}"#,
        );
        // heap_bound_mb = (64/8).clamp(16,48) = 16 -> wins over the 300mb tier target
        assert_eq!(b["playerTargetBufferBytes"], 16_000_000);
    }

    #[test]
    fn missing_optional_fields_default_safely() {
        let b = budget(r#"{"totalRamMb":3000}"#);
        assert_eq!(b["tier"], "mid");
        assert_eq!(b["imageCrossfadeEnabled"], true);
    }
}
