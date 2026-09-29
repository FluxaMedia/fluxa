use crate::core_error::{CoreError, LogAndDiscard};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
struct Anime4kRequest {
    #[serde(default)]
    tier: String,
    #[serde(default)]
    mode: String,
}

fn chain(tier: &str, mode: &str) -> Vec<String> {
    let (primary, secondary) = match tier {
        "anime4k_s" => ("S", "S"),
        "anime4k_l" => ("VL", "M"),
        _ => ("M", "S"),
    };
    let restore = |size: &str| format!("Anime4K_Restore_CNN_{size}.glsl");
    let soft = |size: &str| format!("Anime4K_Restore_CNN_Soft_{size}.glsl");
    let upscale = |size: &str| format!("Anime4K_Upscale_CNN_x2_{size}.glsl");
    let denoise = |size: &str| format!("Anime4K_Upscale_Denoise_CNN_x2_{size}.glsl");
    let x2 = "Anime4K_AutoDownscalePre_x2.glsl".to_string();
    let x4 = "Anime4K_AutoDownscalePre_x4.glsl".to_string();
    let mut shaders = vec!["Anime4K_Clamp_Highlights.glsl".to_string()];
    shaders.extend(match mode {
        "b" => vec![soft(primary), upscale(primary), x2, x4, upscale(secondary)],
        "bb" => vec![
            soft(primary),
            upscale(primary),
            x2,
            x4,
            soft(secondary),
            upscale(secondary),
        ],
        "c" => vec![denoise(primary), x2, x4, upscale(secondary)],
        "ca" => vec![
            denoise(primary),
            x2,
            x4,
            restore(secondary),
            upscale(secondary),
        ],
        "aa" if tier == "anime4k_s" => vec![
            restore("S"),
            upscale("S"),
            restore("S"),
            x2,
            x4,
            upscale("S"),
        ],
        "aa" => vec![
            restore(primary),
            upscale(primary),
            x2,
            x4,
            restore(secondary),
            upscale(secondary),
        ],
        _ => vec![
            restore(primary),
            upscale(primary),
            x2,
            x4,
            upscale(secondary),
        ],
    });
    shaders.push(
        match tier {
            "anime4k_s" => "Anime4K_Thin_VeryFast.glsl",
            "anime4k_l" => "Anime4K_Thin_HQ.glsl",
            _ => "Anime4K_Thin_Fast.glsl",
        }
        .to_string(),
    );
    shaders
}

pub(crate) fn anime4k_shader_chain_json(request_json: &str) -> Option<String> {
    let request = serde_json::from_str::<Anime4kRequest>(request_json)
        .map_err(|error| CoreError::BadInput {
            context: "anime4k_shader_chain_json",
            detail: error.to_string(),
        })
        .log_discard()?;
    Some(json!(chain(&request.tier, &request.mode)).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_tier_doubles_restore_before_downscale_in_aa() {
        let shaders = chain("anime4k_s", "aa");
        assert_eq!(shaders[3], "Anime4K_Restore_CNN_S.glsl");
        assert_eq!(shaders.last().unwrap(), "Anime4K_Thin_VeryFast.glsl");
    }

    #[test]
    fn large_tier_mode_c_denoises_with_very_large_network() {
        let shaders = chain("anime4k_l", "c");
        assert_eq!(shaders[1], "Anime4K_Upscale_Denoise_CNN_x2_VL.glsl");
        assert_eq!(shaders[4], "Anime4K_Upscale_CNN_x2_M.glsl");
    }
}
