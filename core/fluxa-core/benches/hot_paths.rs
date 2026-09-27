use criterion::{Criterion, black_box, criterion_group, criterion_main};
use fluxa_core::ffi::call;
use serde_json::{Value, json};

fn categories() -> Value {
    let items = |prefix: &str| {
        (0..40)
            .map(|i| {
                json!({
                    "id": format!("{prefix}{i}"),
                    "type": "movie",
                    "name": format!("Title {i}"),
                    "poster": format!("https://img.example/{prefix}{i}.jpg"),
                    "background": format!("https://img.example/{prefix}{i}-bg.jpg"),
                })
            })
            .collect::<Vec<_>>()
    };
    Value::Array(
        (0..12)
            .map(|c| json!({"id": format!("cat{c}"), "type": "movie", "items": items(&format!("c{c}-"))}))
            .collect(),
    )
}

fn hot_paths(c: &mut Criterion) {
    let filenames = [
        "[SubsPlease] Frieren - 17 [1080p].mkv",
        "[Grp] The.Show.2024.S01E02.1080p.mkv",
        "Some.Movie.2019.2160p.UHD.BluRay.x265-GROUP.mkv",
    ];
    c.bench_function("local_media_parse_filename", |b| {
        b.iter(|| {
            for name in filenames {
                let args = json!({"fileName": name, "parentHints": [], "kind": "anime"});
                black_box(call("localMediaParseFilename", &args).ok());
            }
        })
    });

    let filters = (0..60)
        .map(|i| json!({"name": format!("B{i}"), "pattern": format!(r"\btag{i}\b"), "imageURL": format!("https://img.example/{i}.png")}))
        .collect::<Vec<_>>();
    let import = call(
        "parseStreamBadgeImport",
        &json!({"sourceUrl": "https://example.com/badges.json", "payload": json!({"filters": filters}).to_string()}),
    )
    .unwrap();
    let import = import
        .as_str()
        .map(str::to_owned)
        .unwrap_or_else(|| import.to_string());
    let badge_args = json!({
        "streamJson": r#"{"title":"Movie.2024.4K.HDR.DV.Atmos.mkv","description":"tag12 tag40"}"#,
        "rulesJson": format!(r#"{{"imports":[{import}]}}"#),
    });
    c.bench_function("match_stream_badges", |b| {
        b.iter(|| black_box(call("matchStreamBadges", &badge_args).ok()))
    });

    let hero_args = json!({"categories": categories(), "prefs": {}});
    c.bench_function("home_hero_plan", |b| {
        b.iter(|| black_box(call("homeHeroPlan", &hero_args).ok()))
    });
    c.bench_function("home_hero_plan_direct", |b| {
        b.iter(|| black_box(fluxa_core::home_hero_plan(&hero_args)))
    });
}

criterion_group!(benches, hot_paths);
criterion_main!(benches);
