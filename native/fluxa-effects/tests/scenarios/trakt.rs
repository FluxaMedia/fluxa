use super::support::{Reply, World};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub const HOST: &str = "api.trakt.tv";

pub struct FakeTrakt {
    pub watchlist: Arc<Mutex<Vec<Value>>>,
}

pub fn serve(world: &World) -> FakeTrakt {
    let watchlist = Arc::new(Mutex::new(Vec::<Value>::new()));
    for path in [
        "/sync/favorites/movies",
        "/sync/favorites/shows",
        "/sync/watchlist/shows",
        "/sync/watched/movies",
        "/sync/progress/watched",
        "/sync/history/episodes",
        "/users/hidden/dropped",
        "/sync/playback",
    ] {
        world.respond(HOST, "GET", path, 200, json!([]));
    }
    let list = watchlist.clone();
    world.on(HOST, "GET", "/sync/watchlist/movies", move |_| {
        Reply::json(200, Value::Array(list.lock().unwrap().clone()))
    });
    let list = watchlist.clone();
    world.on(HOST, "POST", "/sync/watchlist", move |request| {
        for movie in request.json()["movies"].as_array().into_iter().flatten() {
            list.lock()
                .unwrap()
                .push(json!({"type": "movie", "movie": movie}));
        }
        Reply::json(201, json!({"added": {"movies": 1}}))
    });
    let list = watchlist.clone();
    world.on(HOST, "POST", "/sync/watchlist/remove", move |request| {
        let ids: Vec<Value> = request.json()["movies"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|movie| movie["ids"]["imdb"].clone())
            .collect();
        list.lock()
            .unwrap()
            .retain(|entry| !ids.contains(&entry["movie"]["ids"]["imdb"]));
        Reply::json(200, json!({"deleted": {"movies": 1}}))
    });
    world.respond(
        HOST,
        "POST",
        "/sync/history",
        201,
        json!({"added": {"movies": 1}}),
    );
    FakeTrakt { watchlist }
}
