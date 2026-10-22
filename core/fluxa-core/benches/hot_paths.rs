use criterion::{Criterion, criterion_group, criterion_main};
use fluxa_core::ffi::call;
use serde_json::{Value, json};
use std::hint::black_box;

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
