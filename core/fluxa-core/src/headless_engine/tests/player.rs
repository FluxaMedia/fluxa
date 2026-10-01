use super::super::*;
use serde_json::Value;






fn dispatch(handle: u64, action: Value) -> Value {
    serde_json::from_str(&headless_engine_dispatch_json(handle, &action.to_string()).unwrap())
        .unwrap()
}




fn srt(times: &[(f64, f64)]) -> String {
    let stamp = |t: f64| {
        let ms = (t * 1000.0).round() as i64;
        format!(
            "{:02}:{:02}:{:02},{:03}",
            ms / 3_600_000,
            ms / 60_000 % 60,
            ms / 1000 % 60,
            ms % 1000
        )
    };
    times
        .iter()
        .enumerate()
        .map(|(i, (s, e))| format!("{}\n{} --> {}\nline {i}\n", i + 1, stamp(*s), stamp(*e)))
        .collect::<Vec<_>>()
        .join("\n")
}



