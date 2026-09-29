use super::support::{Reply, World};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub const HOST: &str = "api.simkl.com";

pub struct FakeSimkl {
    pub plantowatch: Arc<Mutex<Vec<Value>>>,
}

pub fn serve(world: &World) -> FakeSimkl {
    let plantowatch = Arc::new(Mutex::new(Vec::<Value>::new()));
    world.respond(HOST, "GET", "/sync/activities", 200, json!({"all": "2024-01-01T00:00:00Z"}));
    world.respond(HOST, "GET", "/sync/all-items/shows", 200, json!({"shows": []}));
    world.respond(HOST, "GET", "/sync/all-items/anime", 200, json!({"anime": []}));
    world.respond(HOST, "GET", "/sync/playback", 200, json!([]));
    let list = plantowatch.clone();
    world.on(HOST, "GET", "/sync/all-items/movies", move |_| {
        Reply::json(200, json!({"movies": list.lock().unwrap().clone()}))
    });
    let list = plantowatch.clone();
    world.on(HOST, "POST", "/sync/add-to-list", move |request| {
        for movie in request.json()["movies"].as_array().into_iter().flatten() {
            list.lock().unwrap().push(json!({"status": "plantowatch", "movie": movie}));
        }
        Reply::json(201, json!({"added": {"movies": 1}}))
    });
    world.respond(HOST, "POST", "/sync/history", 201, json!({"added": {"movies": 1}}));
    world.respond(HOST, "POST", "/sync/history/remove", 200, json!({"deleted": {"movies": 1}}));
    world.respond(HOST, "POST", "/scrobble/start", 201, json!({}));
    FakeSimkl { plantowatch }
}
