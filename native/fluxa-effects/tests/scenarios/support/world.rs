use serde_json::Value;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
pub struct Recorded {
    pub host: String,
    pub method: String,
    pub path: String,
    pub query: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Recorded {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub fn json(&self) -> Value {
        serde_json::from_str(&self.body).unwrap_or(Value::Null)
    }

    pub fn query_param(&self, name: &str) -> Option<String> {
        url::form_urlencoded::parse(self.query.as_bytes())
            .find(|(key, _)| key == name)
            .map(|(_, value)| value.into_owned())
    }
}

pub struct Reply {
    pub status: u16,
    pub body: String,
}

impl Reply {
    pub fn json(status: u16, body: Value) -> Self {
        Self {
            status,
            body: body.to_string(),
        }
    }

    pub fn status(status: u16) -> Self {
        Self {
            status,
            body: String::new(),
        }
    }
}

type Handler = Arc<dyn Fn(&Recorded) -> Reply + Send + Sync>;

struct Route {
    host: String,
    method: String,
    path: String,
    handler: Handler,
}

#[derive(Default)]
pub struct World {
    routes: Mutex<Vec<Route>>,
    log: Mutex<Vec<Recorded>>,
    missed: Mutex<Vec<Recorded>>,
}

impl World {
    pub fn reset(&self) {
        self.routes.lock().unwrap().clear();
        self.log.lock().unwrap().clear();
        self.missed.lock().unwrap().clear();
    }

    pub fn on(
        &self,
        host: &str,
        method: &str,
        path: &str,
        handler: impl Fn(&Recorded) -> Reply + Send + Sync + 'static,
    ) {
        self.routes.lock().unwrap().push(Route {
            host: host.to_owned(),
            method: method.to_owned(),
            path: path.to_owned(),
            handler: Arc::new(handler),
        });
    }

    pub fn respond(&self, host: &str, method: &str, path: &str, status: u16, body: Value) {
        self.on(host, method, path, move |_| {
            Reply::json(status, body.clone())
        });
    }

    pub fn handle(&self, request: Recorded) -> Reply {
        self.log.lock().unwrap().push(request.clone());
        let handler = self
            .routes
            .lock()
            .unwrap()
            .iter()
            .rev()
            .find(|route| {
                route.host == request.host
                    && route.method.eq_ignore_ascii_case(&request.method)
                    && route.path == request.path
            })
            .map(|route| route.handler.clone());
        match handler {
            Some(handler) => handler(&request),
            None => {
                self.missed.lock().unwrap().push(request);
                Reply::status(404)
            }
        }
    }

    pub fn requests_all(&self) -> Vec<Recorded> {
        self.log.lock().unwrap().clone()
    }

    pub fn requests(&self, host: &str) -> Vec<Recorded> {
        self.log
            .lock()
            .unwrap()
            .iter()
            .filter(|request| request.host == host)
            .cloned()
            .collect()
    }

    pub fn calls(&self, host: &str, method: &str, path: &str) -> Vec<Recorded> {
        self.requests(host)
            .into_iter()
            .filter(|request| request.method.eq_ignore_ascii_case(method) && request.path == path)
            .collect()
    }

    pub fn unmatched(&self) -> Vec<String> {
        self.missed
            .lock()
            .unwrap()
            .iter()
            .map(|request| format!("{} {}{}", request.method, request.host, request.path))
            .collect()
    }
}
