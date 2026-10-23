use super::super::*;
use serde_json::{Value, json};

fn finish(
    engine: &mut HeadlessEngine,
    effect_id: &str,
    value: Value,
) -> Vec<crate::runtime::EffectEnvelope> {
    let result = serde_json::from_value(json!({
        "effectId": effect_id,
        "status": "ok",
        "value": value,
    }))
    .unwrap();
    engine.complete_effect(result)
}

fn source(id: &str, kind: &str, name: &str) -> Value {
    json!({
        "source": {
            "id": id,
            "name": "Cinemeta - Popular",
            "type": kind,
            "items": [{"id": name, "type": kind, "name": name}],
            "loaded": 1
        },
        "page": {"id": id, "loaded": 1, "hasMore": true}
    })
}

#[test]
fn sections_appear_as_each_catalog_answers_and_keep_addon_order() {
    let mut engine = HeadlessEngine::new(Default::default());
    let action: AppAction =
        serde_json::from_value(json!({"type": "searchRequested", "query": "hotel"})).unwrap();
    let plan = engine.apply(action).effects;
    assert_eq!(plan.len(), 1);

    let fanned = finish(
        &mut engine,
        &plan[0].id,
        json!({"requests": [{"categoryId": "movie"}, {"categoryId": "series"}]}),
    );
    assert_eq!(fanned.len(), 2);
    let id_of = |kind: &str| {
        fanned
            .iter()
            .find(|effect| effect.payload["request"]["categoryId"] == kind)
            .map(|effect| effect.id.clone())
            .unwrap()
    };
    let series_id = id_of("series");
    let movie_id = id_of("movie");

    finish(&mut engine, &series_id, source("series", "series", "Show"));
    let state = serde_json::to_value(&*engine.state.search).unwrap();
    assert_eq!(state["isLoading"], true);
    assert_eq!(state["categories"].as_array().unwrap().len(), 1);

    finish(&mut engine, &movie_id, source("movie", "movie", "Film"));
    let state = serde_json::to_value(&*engine.state.search).unwrap();
    assert_eq!(state["isLoading"], false);
    assert_eq!(state["categories"][0]["id"], "movie");
    assert_eq!(state["categories"][1]["id"], "series");
    assert_eq!(state["results"].as_array().unwrap().len(), 2);
}
